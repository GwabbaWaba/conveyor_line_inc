use std::{cell::Cell, collections::HashMap, error::Error, fs, ops::Deref, process::ExitCode, rc::Rc, sync::RwLock, time::{Duration, SystemTime}};
use bytemuck::from_bytes;
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEvent}, execute, terminal};
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value, Variadic};
use postcard::to_allocvec;
use rand::seq::IteratorRandom;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tap::{Pipe, Tap};
use itertools::Itertools;
use paste::paste;

type DynErrResult<T> = Result<T, Box<dyn Error>>;

mod terminal_settings;
mod game;
use game::*;
mod lua;
use lua::*;
mod world;
use world::*;
mod input;
use input::*;
mod ui;
use ui::*;

enum ThreadLetter {}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Wrapper(HashMap<(u8, u8), String>);

fn main() -> DynErrResult<ExitCode> {
    let game = Game::new()?;
    let sql_db = Rc::new(rusqlite::Connection::open_in_memory()?);
    setup_and_load_lua(&game, sql_db).expect("failed to setup & load");
    let (main_letterboy, main_mailbox) = std::sync::mpsc::channel::<ThreadLetter>();

    // preloading for the engine
    println!("Calling _load");
    call_lua_events_from_game(&game, "_load", Variadic::new())?;

    // primary load events
    println!("Calling pre_load");
    call_lua_events_from_game(&game, "pre_load", Variadic::new())?;
    println!("Calling load");
    call_lua_events_from_game(&game, "load", Variadic::new())?;
    println!("Calling post_load");
    call_lua_events_from_game(&game, "post_load", Variadic::new())?;

    println!("Calling world_gen");
    call_lua_events_from_game(&game, "world_gen", Variadic::new())?;

    terminal_settings::setup(game.terminal().backend_mut())?;

    // panic messages were getting lost to the void of alt-screen
    std::panic::set_hook(Box::new(|info| {
        let msg = "would be pretty funny if this ever failed";
        terminal_settings::cleanup().expect(msg);
        fs::write("panic_out", format!("{}", info)).expect(msg);
    }));

    game_loop(&game)
}

fn game_loop(game: &GameWrapper) -> DynErrResult<ExitCode> {
    let mut exit_code = ExitCode::SUCCESS;
    let mut last_time = SystemTime::now();


    #[allow(unused_labels)]
    'game: loop {
        let lua = game.lua();
        let event_registry: Table = traverse_lua_tables(lua.globals(), REGISTRY_PATH)?;
        
        let current_time = SystemTime::now();
        let delta_time: f64 = current_time.duration_since(last_time)?.as_secs_f64();
        
        last_time = current_time;
        
        call_lua_events(
            event_registry.get("draw")?,
            Variadic::new()
        )?;
        flush_draw_buffer(&lua, game)?;

        let mut inputs = Vec::new();
        // drain input queue
        while let Ok(true) = event::poll(Duration::from_secs(0)) {
            let event = event::read()?;
            inputs.push(event);
        }
        // send out the wrapped input
        lua.globals().get::<_, Table>(CLINC)?
            .set("input", EventWrapper(inputs))?;
        // call the update event
        call_lua_events(
            event_registry.get("update")?,
            Variadic::new().tap_mut(|v| v.push(Value::Number(delta_time)))
        )?;
    }
    #[allow(unreachable_code)]

    Ok(exit_code)
}

fn lua_table_to_sql_args<'lua, 'a>(table: Table<'lua>, buf: &'a mut Vec<Box<dyn ToSql>>) -> DynErrResult<()> {
    for pair in table.pairs::<f64, Value>() {
        let (_, val) = pair.unwrap();
        match val {
            Value::Boolean(b) => buf.push(Box::new(b)),
            Value::Integer(i) => buf.push(Box::new(i)),
            Value::Number(n) => buf.push(Box::new(n)),
            Value::String(s) => buf.push(Box::new("".to_string().tap_mut(|buf| buf.push_str(s.to_str().unwrap())))),
            Value::Table(t) => buf.push(Box::new(serde_json::to_value(t)?)),
            _ => return Err(Box::new(mlua::Error::runtime(format!("couldn't convert {:?} to sql", val))))
        }
    }
    Ok(())
}
fn sql_val_ref_to_lua<'lua>(lua: &'lua Lua, val_ref: rusqlite::types::ValueRef) -> Value<'lua> {
    match val_ref {
        rusqlite::types::ValueRef::Null => Value::Nil,
        rusqlite::types::ValueRef::Integer(i) => Value::Integer(i),
        rusqlite::types::ValueRef::Real(f) => Value::Number(f),
        rusqlite::types::ValueRef::Text(c_str) => Value::String(lua.create_string(c_str).unwrap()),
        rusqlite::types::ValueRef::Blob(blob) => Value::Table(lua.create_sequence_from(blob.iter().map(|b| *b)).unwrap()),
    }
}
struct SqlWrapper (
    Rc<rusqlite::Connection>
);
impl UserData for SqlWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("execute", |_lua, this, (sql_instructions, sql_args): (String, Table)| {
            let this = Rc::clone(&this.0);
            let mut buf = Vec::new();
            lua_table_to_sql_args(sql_args, &mut buf).unwrap();
            let args = buf.iter().map(|b_ts: &Box<dyn ToSql>| &**b_ts).collect_vec();

            match this.execute(&sql_instructions, args.as_slice()) {
                Ok(_) => Ok(()),
                Err(e) => return Err(mlua::Error::runtime(format!("{}", e)))
            }
        });

        methods.add_method("query", |lua, this, (sql_instructions, sql_args): (String, Table)| {
            let this = Rc::clone(&this.0);
            let mut buf = Vec::new();
            lua_table_to_sql_args(sql_args, &mut buf).unwrap();
            let args = buf.iter().map(|b_ts: &Box<dyn ToSql>| &**b_ts).collect_vec();
            let mut stmt = this.prepare_cached(&sql_instructions).unwrap();

            let rows = stmt.query_map(args.as_slice(), |row| {
                let table = lua.create_table().unwrap();

                let mut i = 0;
                while let Ok(val) = row.get_ref(i) {
                    table.set(i+1, sql_val_ref_to_lua(lua, val)).expect("failed to set sql value into lua table");
                    i += 1;
                }

                Ok(table)
            }).unwrap();

            let res: Table = lua.create_table()?;
            for (i, row) in rows.enumerate() {
                res.set(i+1, row.unwrap())?;
            }

            Ok(res)
        });
    }
}

fn setup_and_load_lua(game: &GameWrapper, sql_db: Rc<rusqlite::Connection>) -> DynErrResult<()> {
    let src_scripts = register_scripts("src/lua/").expect("srcs failed to register");
    let game_mods = register_mods("mods").expect("mods failed to register");
    
    let lua = game.lua();
    println!("About to load source");
    load_scripts(&lua, &src_scripts).expect("failed to load src scripts");

    let clinc_table = lua.globals().get::<_, Table>(CLINC)?;
    clinc_table.set("world", World::new())?;
    clinc_table.set("sql_db", SqlWrapper(sql_db))?;
    clinc_table.set("terminal", TerminalWrapper(game.terminal_lock()))?;
    clinc_table.set("world", World::new())?;

    println!("About to setup lua environment");
    setup_lua(&lua, &game.terminal()).expect("failed to setup lua");

    for (mod_name, game_mod) in game_mods.iter()
        .sorted_by(|a, b|
            Ord::cmp(&a.1.priority, &b.1.priority)
        ).rev()
    {
        println!("Loading {}", mod_name);
        load_scripts(&lua, &game_mod.mod_scripts).expect("failed to load mod scripts");
    }
    Ok(())
}
