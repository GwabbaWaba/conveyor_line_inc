use std::{collections::HashMap, env, error::Error, fmt::Display, fs, io::Stdout, process, sync::{Arc, Mutex}, time::{Duration, SystemTime}};

use crossterm::{cursor, event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers}, execute, queue, terminal::{Clear, ClearType}, QueueableCommand};
use json::{object::Object, JsonValue};
use once_cell::sync::Lazy;
use rlua::{Context, Function, Lua, Table, ToLua, ToLuaMulti, Value};

use std::io::stdout;


#[macro_use]
mod macros;

#[macro_use]
mod module_loading_macros;

static mut STDOUT:once_cell::sync::Lazy<Stdout> =  Lazy::<Stdout>::new(|| stdout());
fn std_out() -> &'static mut Stdout {
    unsafe { &mut STDOUT }
}

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

mod ui_render;
use ui_render::*;

mod map_render;
use map_render::*;

mod input;
use input::*;

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

fn main() {
    clear_debug();

    println!("program started");

    // lua init
    unsafe {
        LUA = Some(Arc::new(Mutex::new(Lua::new())))
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
    
    // ui setup
    ignorant_execute!(std_out, cursor::Hide);
    ignorant_queue!(std_out, cursor::MoveTo(0, 0));
    ignorant_queue!(std_out, Clear(ClearType::FromCursorDown)); 

    /* delete when you make world gen good ~*/
    tile_map()[player().position.0][player().position.1] = Tile::new_unchecked({
        identifier_dump().tile_types.get_by_left("conveyor_line_core:air").cloned().unwrap_or(0)
    });

    /* TODO - make all the ui boxes defined through lua script 
     * also make them work good universally (text wrap and nav controls)
     * lol
    ~*/
    let mut map_box = TextBoxBuilder::new(RIGHT_TOP_LEFT, RIGHT_BOX_WIDTH, TOP_BOX_HEIGHT)
        .finalize();
    
    let mut input_box = TextBoxBuilder::new(LEFT_BOTTOM_TOP_LEFT, LEFT_BOX_WIDTH, BOTTOM_BOX_HEIGHT)
        .finalize();
    
    let mut output_box = TextBoxBuilder::new(LEFT_TOP_LEFT, LEFT_BOX_WIDTH, TOP_BOX_HEIGHT)
        .flair_top(format!("╔═|h|b|{:═<w$}|?|*|═╗", "", w = LEFT_BOX_WIDTH as usize - 14))
        .finalize();

    let mut hotbar_box = TextBoxBuilder::new(RIGHT_BOTTOM_TOP_LEFT, RIGHT_BOX_WIDTH, BOTTOM_BOX_HEIGHT)
        .finalize();
    
    /* TODO - have this set through config, for fun ~*/
    let cursor_mimic = "\u{001B}[48;2;255;255;255m \u{001B}[0m";
    
    let mut cursor_state = true;
    let mut last_cursor_update = SystemTime::now();
    
    let mut chars_behind_cursor: Vec<char> = Vec::new();
    let mut chars_ahead_cursor: Vec<char> = Vec::new();
    
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

        if unsafe {STATE_CHANGED} { 
            display_play_info(&chars_behind_cursor);
            unsafe {STATE_CHANGED = false;}
        }

        ignorant_queue!(std_out, cursor::RestorePosition);

        // render the fake cursor
        match cursor_state {
            true if type_mode => {
                print!("{}", cursor_mimic);

                ignorant_execute!(std_out, cursor::MoveLeft(cursor_mimic.len() as u16));
            },
            false => {
                if chars_ahead_cursor.is_empty() {
                    print!(" ");
                } else {
                    if behind_print_amount() >= MAX_INPUT_CHARS {
                        print!("{}", RIGHT_ARROW_WHITE);
                    } else {
                        print!("{}", chars_ahead_cursor[chars_ahead_cursor.len() - 1]);
                    }
                }
                ignorant_execute!(std_out, cursor::MoveLeft(1));
            },
            _ => {}
        }
        
        if last_cursor_update.elapsed().unwrap() >= CURSOR_UPDATE_TIMER {
            cursor_state = !cursor_state;
            last_cursor_update = SystemTime::now();
        }

        // check input
        if poll(Duration::from_secs(0)).unwrap() {
            // handle input
            match read() {
                Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => {
                    match (event.code, type_mode) {
                        (KeyCode::F(1) | KeyCode::Tab, false) => {
                            type_mode = true;
                        },
                        (KeyCode::F(1) | KeyCode::Tab, true) => {
                            type_mode = false;
                        },
                        (_, true) => {
                            // key_output returns the input command when enter is pressed
                            let output = key_output(event, &mut chars_behind_cursor, &mut chars_ahead_cursor);
                            match output {
                                Some(current_input) => {
                                    action_from_input( &current_input);
                                },
                                None => {},
                            }
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

fn call_key_events(event: &KeyEvent) {
    lua().lock().unwrap().context(|lua_context| {
        let globals = lua_context.globals();

        if let Ok(core) = globals.get::<_, Table>("Core") {
            if let Ok(events) = core.get::<_, Table>("Events") {
                if let Ok(key_events) = events.get::<_, Table>("KeyEvents") {
                    let luafied_event = lua_context.create_table().unwrap();
                    let (code, modifiers, kind, state) = (event.code, event.modifiers, event.kind, event.state);

                    // hellish conversion to String
                    let luafied_code = match code {
                        KeyCode::Char(c) => {
                            String::from(c)
                        },
                        KeyCode::F(n) => {
                            format!("F{}", n)
                        },
                        KeyCode::Media(m) => {
                            format!("{:?}", m).to_lowercase()
                        },
                        KeyCode::Modifier(m) => {
                            format!("{:?}", m).to_lowercase()
                        },
                        _ => {
                            format!("{:?}", code).to_lowercase()
                        }
                    };
                
                    let luafied_modifiers = lua_context.create_table().unwrap();
                    let possible_modifiers = [
                        KeyModifiers::CONTROL, KeyModifiers::ALT, KeyModifiers::SHIFT, 
                        KeyModifiers::HYPER, KeyModifiers::SUPER, KeyModifiers::META,
                        KeyModifiers::NONE
                    ];
                    for possible in possible_modifiers { 
                        let debugged_modifier = format!("{:?}", possible).to_lowercase();
                        luafied_modifiers.set(&debugged_modifier[13..(debugged_modifier.len()-1)], modifiers.contains(possible)).unwrap();
                    }

                    let luafied_kind = format!("{:?}", kind).to_lowercase();
                
                    let luafied_state = lua_context.create_table().unwrap();
                    let possible_states = [
                        KeyEventState::CAPS_LOCK, KeyEventState::KEYPAD, KeyEventState::NUM_LOCK,
                        KeyEventState::NONE
                    ];
                    for possible in possible_states { 
                        let debugged_state = format!("{:?}", possible).to_lowercase();
                        luafied_state.set(&debugged_state[14..(debugged_state.len()-1)], state.contains(possible)).unwrap();
                    }
                
                    luafied_event.set("code", luafied_code).unwrap();
                    luafied_event.set("modifiers", luafied_modifiers).unwrap();
                    luafied_event.set("kind", luafied_kind).unwrap();
                    luafied_event.set("state", luafied_state).unwrap();
    
                    for pair in key_events.pairs::<Value, Function>() {
                        let pair = pair.unwrap();
                        if let Err(e) = pair.1.call::<Table, ()>(luafied_event.clone()) {
                            write_to_debug_pretty(format!("{:?}:\n{:?}", pair.0, e));
                        }
                    }
                }
            }
        }
    });
}

pub fn call_lua_events<T: for<'a> ToLuaMulti<'a> + Clone>(event_key: &str, args: T) {

    lua().lock().unwrap().context(|lua_context| {
        let globals = lua_context.globals();

        if let Ok(core) = globals.get::<_, Table>("Core") {
            if let Ok(events) = core.get::<_, Table>("Events") {
                if let Ok(events_table) = events.get::<_, Table>(event_key) {
    
                    for pair in events_table.pairs::<Value, Function>() {
                        pair.unwrap().1.call::<T, ()>(args.clone()).unwrap();
                    }
                }
            }
        }
    });
}

fn display_play_info(chars_behind_cursor: &Vec<char>) {
    ignorant_queue!(std_out, cursor::MoveTo(MAP_TOP_LEFT.0, MAP_TOP_LEFT.1));
    display_map();
    ignorant_queue!(std_out, cursor::MoveTo(PLAYER_COORD_DISPLAY.0, PLAYER_COORD_DISPLAY.1));
    println!("({}, {})", &player().position.0 , &player().position.1);
    
    ignorant_queue!(std_out, cursor::MoveTo(27, MAP_HEIGHT as u16 + 6 + behind_print_amount().min(chars_behind_cursor.len()) as u16));
    ignorant_queue!(std_out, cursor::SavePosition);
}

/// visually clears the input feed
fn wipe_input() {
    ignorant_execute!(std_out, cursor::MoveTo(START_OF_INPUT_LINE.0 - 1, START_OF_INPUT_LINE.1));
    print!("{}", TEXT_INPUT_WIPER);
}

/// displays the input feed as scrolling text
fn print_input_text(chars_behind_cursor: &Vec<char>, chars_ahead_cursor: &Vec<char>) {
    wipe_input();

    ignorant_execute!(std_out, cursor::MoveTo(START_OF_INPUT_LINE.0, START_OF_INPUT_LINE.1));

    
    let behind_start_index = if chars_behind_cursor.len() > behind_print_amount() {
        chars_behind_cursor.len() - behind_print_amount()
    } else { 0 };
    for i in behind_start_index..chars_behind_cursor.len() {
        print!("{}", chars_behind_cursor[i]);
    }
    
    let show_left_arrow = behind_start_index > 0;
    
    ignorant_execute!(std_out, cursor::SavePosition);

    let gap_ahead = MAX_INPUT_CHARS - behind_print_amount();
    let mut show_right_arrow = gap_ahead < chars_ahead_cursor.len();
    for (i, c) in chars_ahead_cursor.iter().rev().enumerate() {
        if i < gap_ahead {
            print!("{}", c);
        } else {
            show_right_arrow = show_right_arrow || i < chars_ahead_cursor.len() - 1;
            break;
        }
    }

    // show arrows which indicate that text is being wrapped
    if show_left_arrow {
        ignorant_execute!(std_out, cursor::MoveTo(START_OF_INPUT_LINE.0 - 1, START_OF_INPUT_LINE.1));
        print!("{}", LEFT_ARROW_WHITE);
    }
    if show_right_arrow {
        ignorant_execute!(std_out, cursor::MoveTo(START_OF_INPUT_LINE.0 + MAX_INPUT_CHARS as u16, START_OF_INPUT_LINE.1));
        print!("{}", RIGHT_ARROW_WHITE);
    }
}