use std::{cell::Cell, collections::HashMap, error::Error, fs, ops::Deref, process::ExitCode, rc::Rc, sync::RwLock, time::{Duration, SystemTime}};
use bytemuck::from_bytes;
use crossterm::{event::{self, Event, KeyCode, KeyEvent}, execute, terminal};
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value, Variadic};
use postcard::to_allocvec;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};
use tap::Tap;
use itertools::Itertools;
use paste::paste;

type DynErrResult<T> = Result<T, Box<dyn Error>>;

mod terminal_settings;
use terminal_settings::*;
mod game;
use game::*;
mod lua;
use lua::*;
mod input;
use input::*;
mod ui;
use ui::*;

enum ThreadLetter {}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Wrapper(HashMap<(u8, u8), String>);

fn main() -> DynErrResult<ExitCode> {
    let mut test = Wrapper(HashMap::new());
    for i in u8::MIN..=u8::MAX {
        for j in u8::MIN..=u8::MAX {
            test.0.insert((i, j), String::from("test data"));
        }
    }
    let mut buf = String::new();
    let timer = SystemTime::now();
    let output: Vec<u8> = to_allocvec(&test).unwrap();
    buf.push_str(&format!("before write: {}\n", timer.elapsed()?.as_micros()));

    fs::write("output.test", output)?;
    buf.push_str(&format!("after write: {}\n", timer.elapsed()?.as_micros()));

    let out: Wrapper = postcard::from_bytes(&fs::read("output.test")?).unwrap();
    buf.push_str(&format!("after read: {}\n", timer.elapsed()?.as_micros()));

    fs::write("out.test", format!("{:#?}", &out.0))?;
    buf.push_str(&format!("after write 2: {}", timer.elapsed()?.as_micros()));

    fs::write("timers", buf)?;
    
    
    let game = Game::new()?;
    setup_and_load_lua(&game).expect("failed to setup & load");

    let (main_letterboy, main_mailbox) = std::sync::mpsc::channel::<ThreadLetter>();

    let _terminal_settings = TerminalSettings::setup_terminal(game.terminal().backend_mut())?; // cleans the terminal when dropped, I think it's pretty clever

    call_lua_events_from_game(&game, "load", Variadic::new())?;
    {
        let lua = game.lua();
        println!("{:#?}", lua.globals().get::<_, Table>("core")?.get::<_, Table>("items")?);
        println!("{:#?}", lua.globals().get::<_, Table>("core")?.get::<_, Table>("recipes")?);
    }

    return game_loop(&game);
}

fn game_loop(game: &GameWrapper) -> DynErrResult<ExitCode> {
    let mut exit_code = ExitCode::SUCCESS;
    let mut last_time = SystemTime::now();

    #[allow(unused_labels)]
    'game: loop {
        let lua = game.lua();
        let event_registry = traverse_lua_tables(lua.globals(), REGISTRY_PATH)?;
        
        let current_time = SystemTime::now();
        let delta_time = current_time.duration_since(last_time)?.as_secs_f64();
        
        last_time = current_time;
        
        call_lua_events(
            event_registry.get("draw")?,
            Variadic::new()
        )?;
        flush_draw_buffer(&lua, game)?;

        let mut inputs = Vec::new();
        while let Ok(true) = event::poll(Duration::from_secs(0)) {
            let event = event::read()?;
            #[cfg(debug_assertions)] {
                match event {
                    // escape hatch
                    Event::Key(KeyEvent { code: KeyCode::End, .. }) => {
                        break 'game;
                    }
                    // poor man's hot reload
                    Event::Key(KeyEvent { code: KeyCode::Home, .. }) => {
                        exit_code = ExitCode::FAILURE;
                        break 'game;
                    }
                    _ => {}
                }
            }

            inputs.push(event);
        }
        lua.globals().get::<_, Table>(CLINC)?.set("input", EventWrapper(inputs))?;

        call_lua_events(
            event_registry.get("update")?,
            Variadic::new().tap_mut(|v| v.push(Value::Number(delta_time)))
        )?;
    }

    #[forbid(unreachable_code)]
    Ok(exit_code)
}

fn setup_and_load_lua(game: &GameWrapper) -> DynErrResult<()> {
    let src_scripts = register_scripts("src/lua/").expect("srcs failed to register");
    let game_mods = register_mods("mods").expect("mods failed to register");
    
    let lua = game.lua();
    load_scripts(&lua, &src_scripts).expect("failed to load src scripts");
    lua.globals().get::<_, Table>(CLINC)?.set("terminal", TerminalWrapper(game.terminal_lock()))?;
    
    setup_lua(&lua, &game.terminal()).expect("failed to setup lua");

    for (_mod_name, game_mod) in game_mods.iter()
        .sorted_by(|a, b|
            Ord::cmp(&a.1.priority, &b.1.priority)
        ).rev()
    {
        load_scripts(&lua, &game_mod.mod_scripts).expect("failed to load mod scripts");
    }
    Ok(())
}
