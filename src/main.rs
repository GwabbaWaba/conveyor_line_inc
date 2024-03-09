use core::num;
use std::{collections::HashMap, env, error::Error, fmt::Display, fs, io::Stdout, process, sync::{Arc, Mutex}, time::{Duration, SystemTime}};

use crossterm::{cursor, event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers}, execute, queue, terminal::{enable_raw_mode, Clear, ClearType}, QueueableCommand};
use json::{object::Object, JsonValue};
use once_cell::sync::Lazy;
use rlua::{Context, Function, Lua, Table, ToLua, ToLuaMulti, Value};
use tui::{backend::CrosstermBackend, Terminal};

use std::io::stdout;

#[macro_use]
mod macros;

#[macro_use]
mod module_loading_macros;

static mut CURSOR_POS: (u16, u16) = (0, 0);

const CURSOR_UPDATE_TIMER: Duration = Duration::from_millis(500);

const LEFT_ARROW_WHITE: &str = "\u{001B}[38;2;255;255;255m←\u{001B}[0m";
const RIGHT_ARROW_WHITE: &str = "\u{001B}[38;2;255;255;255m→\u{001B}[0m";

const TEXT_INPUT_WIPER: &str = "                                     ";

// ui dimensions
const TOP_BOX_HEIGHT: u16 = MAP_HEIGHT as u16;
const BOTTOM_BOX_HEIGHT: u16 = 3;

const LEFT_TOP_LEFT: (u16, u16) = (25, 2);
const LEFT_BOX_WIDTH: u16 = 39;

const RIGHT_TOP_LEFT: (u16, u16) = (LEFT_TOP_LEFT.0 + LEFT_BOX_WIDTH + 1, LEFT_TOP_LEFT.1); 
const RIGHT_BOX_WIDTH: u16 = MAP_LENGTH as u16 * 2 + 2;

const LEFT_BOTTOM_TOP_LEFT: (u16, u16) = (LEFT_TOP_LEFT.0, TOP_BOX_HEIGHT + 4);

const RIGHT_BOTTOM_TOP_LEFT: (u16, u16) = (65, TOP_BOX_HEIGHT + 4);

const MAP_TOP_LEFT: (u16, u16) = (RIGHT_TOP_LEFT.0 + 1, RIGHT_TOP_LEFT.1 + 1);
const PLAYER_COORD_DISPLAY: (u16, u16) = (RIGHT_TOP_LEFT.0 + 2, RIGHT_TOP_LEFT.1 + TOP_BOX_HEIGHT + 1);

const START_OF_INPUT_LINE: (u16, u16) = (27, LEFT_BOTTOM_TOP_LEFT.1 + 2);

mod debug;
use debug::*;

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

// summoning the demons
static mut LUA: Option<Arc<Mutex<Lua>>> = None;

fn lua() -> Arc<Mutex<Lua>> {
    unsafe { LUA.clone().expect("Shouldn't be None") }
}

// flag for if map needs to be redrawn
static mut STATE_CHANGED: bool = true;

static mut TIME_BETWEEN_TICKS: Option<Duration> = None;
fn time_between_ticks() -> Duration {
    unsafe { TIME_BETWEEN_TICKS.unwrap() }
}
static mut LAST_TICK: Option<SystemTime> = None;
fn last_tick() -> SystemTime {
    unsafe { LAST_TICK.unwrap() }
}

static mut TERMINAL: Option<Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>> = None;
fn terminal() -> Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>> {
    unsafe { TERMINAL.clone().expect("Shouldn't be None") }
}

fn main() {
    clear_debug();

    println!("program started");
    enable_raw_mode().unwrap();

    // lua init
    unsafe {
        LUA = Some(Arc::new(Mutex::new(Lua::new())))
    }
    unsafe {
        TERMINAL = Some(Arc::new(Mutex::new(Terminal::new(CrosstermBackend::new(stdout())).expect("Shouldn't"))))
    }

    lua().lock().unwrap().context(|lua_context| {
        load_default_lua_data(lua_context);
    });

    run_lua_scripts_from_path(MODULES_PATH, lua());

    // game data init
    unsafe { 
        MAPPED_DUMP = {
            let mapped_dump;
            
            match load_module_data_from_persistent_mapping(MOD_PACK_MAPPINGS_PATH) {
                Ok(_) => todo!(),
                Err(_) => {
                    let mut deserial_dump: DeserializationDump = HashMap::new();
                    deserialize_modules_from_path(&mut deserial_dump, MODULES_PATH);

                    let mut pre_map_dump: PreMapDump = Vec::new();
                    mapped_dump = map_deserialized_dump(&mut pre_map_dump, &deserial_dump, &mut ID_TRACKER );
                },
            }

            write_to_debug_pretty(format!("\nfinal dump:\n{:?}\n", mapped_dump));
            Some(mapped_dump)
        }
    };

    // game world init
    unsafe {
        TILE_MAP = {
            let mut map_rand_tile_gen = WeightedRandomBuilder::new();
        
            for (id, tile_type) in &game_data_dump().tile_types {
                if tile_type.world_gen_weight > 0.0 {
                    map_rand_tile_gen.add_entry(id, tile_type.world_gen_weight);
                }
            }
            
            Some(gen_map(map_rand_tile_gen.finalize(), |x| Tile::new_unchecked(*x)))
        }
    }

    unsafe {
        GROUND_MAP = {
            let mut map_rand_ground_gen = WeightedRandomBuilder::new();
        
            for (id, ground_type) in &game_data_dump().ground_types {
                if ground_type.world_gen_weight > 0.0 {
                    map_rand_ground_gen.add_entry(id, ground_type.world_gen_weight);
                }
            }
        
            Some(gen_map(map_rand_ground_gen.finalize(), |x| Ground::new(*x)))
        }
    }
    
    /* delete when you make world gen good ~*/
    tile_map()[player().position.0][player().position.1] = Tile::new_unchecked({
        identifier_dump().tile_types.get_by_left("conveyor_line_core:air").cloned().unwrap_or(0)
    });

    /* TODO - make all the ui boxes defined through lua script 
     * also make them work good universally (text wrap and nav controls)
     * lol
    ~*/
    
    
    /* TODO - have this set through config, for fun ~*/
    let cursor_mimic = "\u{001B}[48;2;255;255;255m \u{001B}[0m";
    
    let mut type_mode = false;

    unsafe { TIME_BETWEEN_TICKS = Some(Duration::from_millis(50)) };
    unsafe { LAST_TICK = Some(SystemTime::now()) };
    
    loop {
        let tick_time_diff = match last_tick().elapsed() {
            Ok(elapsed) => elapsed,
            Err(future_elapsed) => future_elapsed.duration(),
        };
        if tick_time_diff >= time_between_ticks() {
            let amount_of_ticks_to_run = tick_time_diff.as_millis() / time_between_ticks().as_millis();

            for _ in 0..amount_of_ticks_to_run {
                call_lua_events("TickEvents", ());
                unsafe { LAST_TICK = Some(SystemTime::now()) };
            }
        }

        // check input
        if poll(Duration::from_secs(0)).unwrap() {
            // handle input
            match read() {
                Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => {
                    match (event.code, type_mode) {
                        (KeyCode::Char('c'), _) if event.modifiers == KeyModifiers::CONTROL => {
                            std::process::exit(0);
                        },
                        (KeyCode::F(1) | KeyCode::Tab, false) => {
                            type_mode = true;
                        },
                        (KeyCode::F(1) | KeyCode::Tab, true) => {
                            type_mode = false;
                        },
                        (_, true) => {
                            /* TODO - actionFromInput must run here ~*/
                        },
                        (_, false) => {
                            call_key_events(&event)
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
}