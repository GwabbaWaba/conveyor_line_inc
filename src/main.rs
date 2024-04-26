use core::num;
use std::{alloc::{alloc, Layout}, borrow::BorrowMut, collections::{hash_set, HashMap, HashSet}, env, error::Error, fmt::{format, Alignment, Display}, fs, hash::Hash, io::{self, Stdout}, process, sync::{mpsc, Arc, Mutex}, thread, time::{Duration, SystemTime}};

use crossterm::{cursor, event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent, MouseEventKind}, execute, queue, style::Color, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, QueueableCommand};

use json::{object::Object, JsonValue};
use linked_hash_map::LinkedHashMap;
use mlua::{Lua, Table};
use tui::{backend::CrosstermBackend, style::{Modifier, Style}, widgets::{self, Block, Borders, ListState, Paragraph, StatefulWidget, TableState, Widget, Wrap}, Frame, Terminal};

use std::io::stdout;

#[macro_use]
mod macros;

#[macro_use]
mod module_loading_macros;

mod ui_characters {
    pub mod arrow {
        pub const WHITE_LEFT: &str = "\u{001B}[38;2;255;255;255m←\u{001B}[0m";
        pub const WHITE_RIGHT: &str = "\u{001B}[38;2;255;255;255m→\u{001B}[0m";
    }
    pub const CURSOR_MIMIC: &str = "\u{001B}[48;2;255;255;255m \u{001B}[0m";
}
use ui_characters::*;

mod debug;
use debug::*;

mod point;
use point::*;

mod display;

mod item;
use item::*;

mod tile;
use tile::*;

mod ground;
use ground::*;

mod weighted_random;
use weighted_random::*;

mod module_universal;
use module_universal::*;

mod script_processing;
use script_processing::*;

mod module_loading;
use module_loading::*;

mod player;
use player::*;

mod map;
use map::*;

mod map_render;
use map_render::*;

mod commands;
use commands::*;

mod ui_render;
use ui_render::*;

mod input;
use input::*;

static mut TIME_BETWEEN_TICKS: Option<Duration> = None;
fn time_between_ticks() -> Duration {
    unsafe { TIME_BETWEEN_TICKS.unwrap() }
}
static mut LAST_TICK: Option<SystemTime> = None;
fn last_tick() -> SystemTime {
    unsafe { LAST_TICK.unwrap() }
}

static mut UI_REDRAW_QUEUED: bool = true;
static mut MAP_REDRAW_QUEUED: bool = true;

static mut STDOUT: once_cell::sync::Lazy<Stdout> = once_cell::sync::Lazy::new(|| stdout());
static mut STDOUT_REF: *mut &Stdout = std::ptr::null_mut::<&Stdout>();

type Writer = &'static Stdout;

static mut TERMINAL: once_cell::sync::Lazy::<Terminal<CrosstermBackend<Writer>>> = once_cell::sync::Lazy::<Terminal<CrosstermBackend<Writer>>>::new(| | Terminal::new(CrosstermBackend::new(unsafe { *STDOUT_REF })).unwrap());
fn terminal() -> &'static mut once_cell::sync::Lazy::<Terminal<CrosstermBackend<Writer>>> {
    unsafe { &mut TERMINAL }
}

fn main() -> Result<(), io::Error> {
    clear_debug();

    unsafe {
        {
            let layout = Layout::new::<Stdout>();
            let raw = alloc(layout) as *mut &Stdout;
            *raw = &STDOUT;
            STDOUT_REF = raw;
        }
    }
    
    execute!(
        terminal().backend_mut(), 
        //EnterAlternateScreen, 
        Clear(ClearType::All),
        EnableMouseCapture,
    )?;
    println!("program started");
    
    let (key_sender, key_reciever) = mpsc::channel();
    let key_events = Arc::new(Mutex::new(LinkedHashMap::new()));

    let (mouse_sender, mouse_reciever) = mpsc::channel();
    let mouse_events = Arc::new(Mutex::new(LinkedHashMap::new()));

    let (focus_sender, focus_reciever) = mpsc::channel();

    let input_thread_handle = thread::Builder::new().name(String::from("input")).spawn(move || {
        loop {
            match read() {
                Ok(Event::Key(event)) => {
                    let _ = key_sender.send(event);
                },
                Ok(Event::Mouse(event)) => {
                    let _ = mouse_sender.send(event);
                },
                Ok(Event::FocusLost) => {
                    let _ = focus_sender.send(false);
                },
                Ok(Event::FocusGained) => {
                    let _ = focus_sender.send(true);
                },

                _ => {}
            }
        }
    })?;

    // engine config loading
    let mut player_start_pos = None;
    unsafe {
        if let JsonValue::Object(config) = get_json_info("resources\\config\\config.json".to_owned()) {
            if let Some(JsonValue::Object(config)) = config.get("conveyor_line_engine_config") {
                if let Some(JsonValue::Object(config)) = config.get("map") {
                    MAP_HEIGHT = config.get("height").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                    MAP_WIDTH = config.get("width").unwrap_or(&JsonValue::Number(0.into())).as_usize();

                    MAP_DISPLAY_HEIGHT = config.get("display_height").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                    MAP_DISPLAY_WIDTH = config.get("display_width").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                }
                
                if let Some(JsonValue::Object(config)) = config.get("player") {
                    player_start_pos = Some(Point {
                        x: config.get("x").unwrap_or(&JsonValue::Number(0.into())).as_usize().unwrap(),
                        y: config.get("y").unwrap_or(&JsonValue::Number(0.into())).as_usize().unwrap()
                    });
                }
            }
        }
    }

    let mut lua = Arc::new(Mutex::new(Lua::new()));

    load_default_lua_data(&mut lua);

    // game data init
    unsafe {
        MAPPED_DUMP = {
            let mapped_dump;
            
            match load_module_data_from_persistent_mapping(MOD_PACK_MAPPINGS_PATH) {
                Ok(_) => todo!(),
                Err(_) => {
                    let mut deserial_dump: DeserializationDump = HashMap::new();
                    deserialize_modules_from_path(&lua.lock().unwrap(), &mut deserial_dump, MODULES_PATH);

                    let mut pre_map_dump: PreMapDump = Vec::new();
                    mapped_dump = map_deserialized_dump(&mut pre_map_dump, &deserial_dump, &mut ID_TRACKER );
                },
            }

    
            write_to_debug_pretty(format!("\nfinal dump:\n{:?}\n", mapped_dump));
            Some(mapped_dump)
        }
    };

    // game world init
    let time_before_map_gen = SystemTime::now();
    let tile_gen_thread_handle = thread::Builder::new().name(String::from("tile_gen")).spawn(|| {
        let mut map_rand_tile_gen = WeightedRandomBuilder::new();
        for (id, tile_type) in &game_data_dump().tile_types {
            if tile_type.world_gen_weight > 0.0 {
                map_rand_tile_gen.add_entry(id, tile_type.world_gen_weight);
            }
        }
        unsafe {
            TILE_MAP = Some(gen_map(map_rand_tile_gen.finalize(), |x| Tile::new_unchecked(*x)));
        }
    })?;

    let ground_gen_thread_handle = thread::Builder::new().name(String::from("ground_gen")).spawn(|| {
        let mut map_rand_ground_gen = WeightedRandomBuilder::new();
        for (id, ground_type) in &game_data_dump().ground_types {
            if ground_type.world_gen_weight > 0.0 {
                map_rand_ground_gen.add_entry(id, ground_type.world_gen_weight);
            }
        }
        unsafe {
            GROUND_MAP = Some(gen_map(map_rand_ground_gen.finalize(), |x| Ground::new(*x)));
        }
    })?;

    tile_gen_thread_handle.join().unwrap();
    ground_gen_thread_handle.join().unwrap();
    write_to_debug(format!("Map generated in: {:#?}", SystemTime::now().duration_since(time_before_map_gen).unwrap()));
    
    /* delete when you make world gen good ~*/
    player().position = player_start_pos.unwrap();
    tile_map()[player().position.y][player().position.x] = Tile::new_unchecked({
        identifier_dump().tile_types.get_by_left("conveyor_line_core:air").cloned().unwrap_or(0)
    });

    /* TODO - make all the ui boxes defined through lua script 
     * also make them work good universally (text wrap and nav controls)
     * lol
    ~*/
    
    
    unsafe { TIME_BETWEEN_TICKS = Some(Duration::from_millis(50)) };
    unsafe { LAST_TICK = Some(SystemTime::now()) };
    
    execute!(
        terminal().backend_mut(), 
        //EnterAlternateScreen, 
        Clear(ClearType::All)
    )?;
    
    let mut type_mode = false;

    let key_events_clone = Arc::clone(&key_events);
    let mouse_events_clone = Arc::clone(&mouse_events);
    let lua_clone = Arc::clone(&lua);
    let process_thread_handle = thread::Builder::new().name(String::from("process")).spawn(move || {
        loop {
            do_key_events(&lua_clone.lock().unwrap(), &mut key_events_clone.lock().unwrap(), &mut type_mode);
            do_mouse_events(&lua_clone.lock().unwrap(), &mut mouse_events_clone.lock().unwrap());

            let focus_status = focus_reciever.try_recv();
            // block when focus lost, until focus regained
            if let Ok(false) = focus_status {
                'focus_check: loop {
                    if let Ok(true) = focus_reciever.recv() {
                        break 'focus_check;
                    }
                }
            }
            
            let tick_time_diff = match last_tick().elapsed() {
                Ok(elapsed) => elapsed,
                Err(future_elapsed) => future_elapsed.duration(),
            };
            if tick_time_diff >= time_between_ticks() {
                let amount_of_ticks_to_run = tick_time_diff.as_millis() / time_between_ticks().as_millis();
    
                for _ in 0..amount_of_ticks_to_run {
                    call_lua_events(&lua_clone.lock().unwrap(), "TickEvents", ());
                    unsafe { LAST_TICK = Some(SystemTime::now()) };
                }
                
                unsafe { MAP_REDRAW_QUEUED = true; };
            }
            if unsafe { UI_REDRAW_QUEUED } {
                draw_ui(&lua_clone.lock().unwrap());
            }
            
            if unsafe { MAP_REDRAW_QUEUED } {
                let _ = display_map();
                call_lua_events(&lua_clone.lock().unwrap(), "PostMapDraw", ());
                unsafe { MAP_REDRAW_QUEUED = false; }
            }
        }
    })?;

    // messenger boy for process and input threads
    let key_events_clone = Arc::clone(&key_events);
    let mouse_events_clone = Arc::clone(&mouse_events);
    loop {
        let key_event = key_reciever.try_recv();
        match key_event {
            Ok(key_event) => {
                let mut almost_key_event = almostify_key_event(&key_event);
                almost_key_event.state = determine_real_state(&almost_key_event);

                key_events_clone.lock().unwrap().insert(almost_key_event, key_event.kind);
            },
            _ => {}
        }

        let mouse_event = mouse_reciever.try_recv();
        match mouse_event {
            Ok(mouse_event) => {
                let almost_mouse_event = almostify_mouse_event(&mouse_event);
        
                mouse_events_clone.lock().unwrap().insert(almost_mouse_event, mouse_event.kind);
            },
            _ => {}
        }
    }
}

fn exit_successful() -> Result<(), io::Error>{
    disable_raw_mode()?;
    terminal().show_cursor()?;
    execute!(
        terminal().backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    std::process::exit(0);
}