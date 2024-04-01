use core::num;
use std::{alloc::{alloc, Layout}, borrow::BorrowMut, collections::{hash_set, HashMap, HashSet}, env, error::Error, fmt::{format, Alignment, Display}, fs, hash::Hash, io::{self, Stdout}, process, sync::{Arc, Mutex}, time::{Duration, SystemTime}};

use crossterm::{cursor, event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers}, execute, queue, style::Color, terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen}, QueueableCommand};


use json::{object::Object, JsonValue};
use linked_hash_map::LinkedHashMap;
use rlua::{Context, Function, Lua, Table, ToLua, ToLuaMulti, Value};
use tui::{backend::CrosstermBackend, layout::{self, Rect}, style::{Modifier, Style}, widgets::{self, Block, Borders, ListState, Paragraph, StatefulWidget, TableState, Widget, Wrap}, Frame, Terminal};

use std::io::stdout;

#[macro_use]
mod macros;

#[macro_use]
mod module_loading_macros;

mod ui_characters {
    pub const LEFT_ARROW_WHITE: &str = "\u{001B}[38;2;255;255;255m←\u{001B}[0m";
    pub const RIGHT_ARROW_WHITE: &str = "\u{001B}[38;2;255;255;255m→\u{001B}[0m";
    pub const CURSOR_MIMIC: &str = "\u{001B}[48;2;255;255;255m \u{001B}[0m";
}
use ui_characters::*;
    

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

static mut LUA: once_cell::sync::Lazy<Lua> = once_cell::sync::Lazy::new(|| Lua::new());
static mut LUA_REF: *mut &Lua = std::ptr::null_mut::<&Lua>();
/// fake sense of safety
fn lua() -> &'static Lua {
    unsafe { &(*LUA_REF) }
}

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

    enable_raw_mode().unwrap();

    unsafe {
        {
            let layout = Layout::new::<Stdout>();
            let raw = alloc(layout) as *mut &Stdout;
            *raw = &STDOUT;
            STDOUT_REF = raw;
        }
        {
            let layout = Layout::new::<Lua>();
            let raw = alloc(layout) as *mut &Lua;
            *raw = &LUA;
            LUA_REF = raw;
        }
    }

    execute!(
        terminal().backend_mut(), 
        //EnterAlternateScreen, 
        Clear(ClearType::All),
        EnableMouseCapture,
    )?;

    println!("program started");

    // engine config loading
    unsafe {
        if let JsonValue::Object(config) = get_json_info("resources\\config\\config.json".to_owned()) {
            if let Some(JsonValue::Object(config)) = config.get("conveyor_line_engine_config") {
                if let Some(JsonValue::Object(config)) = config.get("map") {
                    MAP_HEIGHT = config.get("height").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                    MAP_WIDTH = config.get("width").unwrap_or(&JsonValue::Number(0.into())).as_usize();

                    MAP_DISPLAY_HEIGHT = config.get("display_height").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                    MAP_DISPLAY_WIDTH = config.get("display_width").unwrap_or(&JsonValue::Number(0.into())).as_usize();
                }
            }
        }
    }

    lua().context(|lua_context| {
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
    
    let mut type_mode = false;

    unsafe { TIME_BETWEEN_TICKS = Some(Duration::from_millis(50)) };
    unsafe { LAST_TICK = Some(SystemTime::now()) };

    let mut key_events = LinkedHashMap::new();

    execute!(
        terminal().backend_mut(), 
        //EnterAlternateScreen, 
        Clear(ClearType::All)
    )?;
    
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
        
        
        if unsafe { UI_REDRAW_QUEUED } {
            draw_ui();
        }
        
        if unsafe { MAP_REDRAW_QUEUED } {
            let _ = display_map();
            call_lua_events("PostMapDraw", ());
            unsafe { MAP_REDRAW_QUEUED = false; }
        }

        // check input
        if poll(Duration::from_secs(0)).unwrap() {
            write_to_debug(format!("pre: {:?}", key_events));
            // handle input
            match read() {
                Ok(Event::Key(event)) => {
                    let mut almost_key_event = almostify_key_event(&event);
                    almost_key_event.state = determine_real_state(&almost_key_event);
                    match event.kind {
                        KeyEventKind::Press => {
                            key_events.insert(almost_key_event, ReleaseStatus::Press);
                        },
                        KeyEventKind::Repeat => {
                            key_events.insert(almost_key_event, ReleaseStatus::Press);
                        },
                        KeyEventKind::Release => {
                            key_events.insert(almost_key_event, ReleaseStatus::Release);
                        },
                    }
                },
                Ok(Event::Mouse(event)) => {
                    /* for l8r ~*/  
                }
                _ => {}
            }
            write_to_debug(format!("mid: {:?}", key_events));
            do_key_events(&mut key_events, &mut type_mode);
            write_to_debug(format!("post: {:?}\n", key_events));
        }

    }
}

fn do_key_event(event: &AlmostKeyEvent, type_mode: &mut bool) {
    match (event.code, &mut *type_mode) {
        (KeyCode::Char('c'), _) if event.modifiers == KeyModifiers::CONTROL => {
            exit_successful().expect("exited, but not successfully");
        },
        (KeyCode::F(1) | KeyCode::Tab, false) => {
            *type_mode = true;
        },
        (KeyCode::F(1) | KeyCode::Tab, true) => {
            *type_mode = false;
        },
        (_, true) => {
            /* action_from_input(_) ~*/
        }
        (_, false) => {
            call_key_events(&actualize_key_event(event));
        }
    }
}

fn determine_real_state(key_event: &AlmostKeyEvent) -> KeyEventState {
    match key_event.state {
        KeyEventState::NONE => {
            if let KeyCode::Char(c) = key_event.code {
                if (c.is_uppercase() && !key_event.modifiers.contains(KeyModifiers::SHIFT)) ||
                   (c.is_lowercase() && key_event.modifiers.contains(KeyModifiers::SHIFT)) {
                    return KeyEventState::CAPS_LOCK;
                }
            }
            return KeyEventState::NONE;
        },
        _ => { key_event.state }
    }
}

fn do_key_events(key_events: &mut LinkedHashMap<AlmostKeyEvent, ReleaseStatus>, type_mode: &mut bool) {
    let mut removals = Vec::new();

    let mut last_modifiers = KeyModifiers::NONE;
    let mut last_state = KeyEventState::NONE;
    for (key_event, release_status) in key_events.iter() {
        let this_modifiers = key_event.modifiers;
        let this_state = key_event.state;

        let go_through = 
            *release_status != ReleaseStatus::Release && ((
                this_modifiers == last_modifiers ||
                !last_modifiers.contains(this_modifiers) ||
                last_modifiers == KeyModifiers::NONE
            ) &&
            (
                this_state == last_state ||
                !last_state.contains(this_state) ||
                last_state == KeyEventState::NONE
            ));

        let mut queue_removal = true;
        if go_through {
            queue_removal = key_event.state.contains(KeyEventState::CAPS_LOCK);
            do_key_event(key_event, type_mode);
        }
        
        if queue_removal {
            removals.push(key_event.clone());
        }

        last_modifiers = this_modifiers;
        last_state = this_state;
    }

    for removal in removals {
        key_events.remove(&removal);
    }
}

#[derive(PartialEq, Debug)]
enum ReleaseStatus {
    Press,
    Release
}

/// Represents almost a key event.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct AlmostKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub state: KeyEventState,
}

fn almostify_key_event(key_event: &KeyEvent) -> AlmostKeyEvent {
    AlmostKeyEvent {
        code: key_event.code,
        modifiers: key_event.modifiers,
        state: key_event.state,
    }
}

fn actualize_key_event(almost_key_event: &AlmostKeyEvent) -> KeyEvent {
    KeyEvent { 
        code: almost_key_event.code, 
        modifiers: almost_key_event.modifiers, 
        kind: KeyEventKind::Press, 
        state: almost_key_event.state 
    }
}

fn exit_successful() -> Result<(), io::Error>{
    disable_raw_mode()?;
    execute!(
        terminal().backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal().show_cursor()?;
    
    std::process::exit(0);
}

fn draw_ui() {
    lua().context(|lua_context| do_the_lua(lua_context, &["ui"], |_, innards| {
        let ui = &innards[2];

        if let Ok(ui_elements) = ui.get::<_, Table>("UiElements") {             
            terminal().draw(|f| {
                for ui_element in ui_elements.pairs::<Value, Table>() {
                    let (_, ui_element) = ui_element.unwrap();

                    let ui_element = lua_table_to_ui_element(ui_element);
                    match ui_element {
                        Ok((widget_type, rect, widget_state)) => {
                            render_ui_element(f, widget_type, rect, widget_state);
                        },
                        Err(e) => write_to_debug(format!("{:?}", e)),
                    }
                }
            }).unwrap();

            unsafe { UI_REDRAW_QUEUED = false; }
        }

    }));
}

fn render_ui_element(f: &mut Frame<'_, CrosstermBackend<Writer>>, widget_type: WidgetType, rect: Rect, widget_state: Option<WidgetState>) {
    match widget_type {
        WidgetType::Block(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Tabs(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::List(w) => {
            match widget_state {
                Some(WidgetState::List(mut s)) => {
                    f.render_stateful_widget(w, rect, &mut s)
                },
                _ => f.render_widget(w, rect),
            }
        },
        WidgetType::Table(w) => {
            match widget_state {
                Some(WidgetState::Table(mut s)) => {
                    f.render_stateful_widget(w, rect, &mut s)
                },
                _ => f.render_widget(w, rect),
            }
        },
        WidgetType::Paragraph(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Chart(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::BarChart(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Gauge(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Sparkline(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Clear(w) => {
            f.render_widget(w, rect);
        },
    }
}

enum WidgetType<'a> {
    Block(widgets::Block<'a>),
    Tabs(widgets::Tabs<'a>),
    List(widgets::List<'a>),
    Table(widgets::Table<'a>),
    Paragraph(widgets::Paragraph<'a>),
    Chart(widgets::Chart<'a>),
    BarChart(widgets::BarChart<'a>),
    Gauge(widgets::Gauge<'a>),
    Sparkline(widgets::Sparkline<'a>),
    Clear(widgets::Clear)
}

enum WidgetState {
    List(ListState),
    Table(TableState)
}

const LUA_TO_UI_ERR: rlua::Error = rlua::Error::FromLuaConversionError { from: "str", to: "ui", message: None };

fn str_to_ui_alignment(a: &str) -> Result<tui::layout::Alignment, rlua::Error> {
    Ok(match a {
        "center" => {tui::layout::Alignment::Center},
        "right"  => {tui::layout::Alignment::Right},
        "left"   => {tui::layout::Alignment::Left},

        _ => return Err(LUA_TO_UI_ERR)
    })
}

fn lua_table_to_text_modifier(table: Table) -> Result<Modifier, rlua::Error> {
    let mut rusty_modifiers = Modifier::empty();
        
    for pair in table.pairs::<String, bool>() {
        let (modifier, enabled) = pair?;
        let modifier: &str = &modifier.to_lowercase();

        let modifier = match modifier {
            "bold"        => { Modifier::BOLD },
            "dim"         => { Modifier::DIM },
            "italic"      => { Modifier::ITALIC },
            "underlined"  => { Modifier::UNDERLINED },
            "slowBlink"   => { Modifier::SLOW_BLINK },
            "rapidBlink"  => { Modifier::RAPID_BLINK },
            "reversed"    => { Modifier::REVERSED },
            "hidden"      => { Modifier::HIDDEN },
            "crossedOut"  => { Modifier::CROSSED_OUT },

             _ => return Err(LUA_TO_UI_ERR)
        };
        match enabled {
            true => rusty_modifiers.insert(modifier),
            false => rusty_modifiers.remove(modifier),
        }
    }

    Ok(rusty_modifiers)
}

fn lua_table_to_color(table: Table) -> Result<tui::style::Color, rlua::Error> {
    Ok(tui::style::Color::Rgb(
        table.get::<_, u8>(1)?,
        table.get::<_, u8>(2)?,
        table.get::<_, u8>(3)?
    ))
}

fn lua_table_to_ui_style(table: Table) -> Result<Style, rlua::Error> {
    let mut style = Style::default();
    if let Ok(bg) = table.get::<_, Table>("bg") {
        style = style.bg(lua_table_to_color(bg)?);
    }
    if let Ok(fg) = table.get::<_, Table>("fg") {
        style = style.fg(lua_table_to_color(fg)?);
    }
    if let Ok(modifiers) = table.get::<_, Table>("modifiers") {
        style = style.add_modifier(lua_table_to_text_modifier(modifiers)?);
    }
    
    
    Ok(style)
}

fn lua_table_to_ui_element(table: Table) -> Result<(WidgetType, Rect, Option<WidgetState>), rlua::Error> {
    let data = table.get::<_, Table>("data")?;

    let (widget_type, rect, mut widget_state);
    widget_state = None;

    let lua_rect = table.get::<_, Table>("rect")?;
    rect = Rect::new(
        lua_rect.get::<_, u16>("x")?,
        lua_rect.get::<_, u16>("y")?,
        lua_rect.get::<_, u16>("width")?,
        lua_rect.get::<_, u16>("height")?,
    );

    match table.get::<_, String>("type")?.as_str() {
        "block" => {
            let mut block = Block::default();

            if let Ok(title) = data.get::<_, String>("title") { 
                block = block.title(title); 
            }
            if let Ok(title_alignment) = data.get::<_, String>("titleAlignment") {
                block = block.title_alignment(str_to_ui_alignment(title_alignment.to_lowercase().as_str())?);
            }
            if let Ok(style) = data.get::<_, Table>("style") {
                block = block.style(lua_table_to_ui_style(style)?);
            }

            if let Ok(borders) = data.get::<_, Table>("borders") {
                let mut rusty_borders = Borders::empty();
        
                for pair in borders.pairs::<String, bool>() {
                    let (border, enabled) = pair?;
                    let border: &str = &border.to_lowercase();
                
                    let border = match border {
                        "all"    => { Borders::ALL },
                        "bottom" => { Borders::BOTTOM },
                        "left"   => { Borders::LEFT },
                        "top"    => { Borders::TOP },
                        "none"   => { Borders::NONE },

                        _ => return Err(LUA_TO_UI_ERR)
                    };
                    match enabled {
                        true => rusty_borders.insert(border),
                        false => rusty_borders.remove(border),
                    }
                }
            
                block = block.borders(rusty_borders);
            }

            if let Ok(border_type) = data.get::<_, String>("borderType") {
                block = match border_type.as_str() {
                    "plain" => block.border_type(widgets::BorderType::Plain),
                    "double" => block.border_type(widgets::BorderType::Double),
                    "thick" => block.border_type(widgets::BorderType::Thick),
                    "rounded" => block.border_type(widgets::BorderType::Rounded),
                    _ => return Err(LUA_TO_UI_ERR),
                }
            }
            
            widget_type = WidgetType::Block(block);
        },
        "tabs" => {
            todo!()
        },
        "list" => {
            widget_state = Some(todo!());
        },
        "table" => {
            widget_state = Some(todo!());
        },
        "paragraph" => {
            todo!()
        },
        "chart" => {
            todo!()
        },
        "barchart" => {
            todo!()
        },
        "gauge" => {
            todo!()
        },
        "sparkline" => {
            todo!()
        },
        _ => unimplemented!(),
    }
    
    Ok((widget_type, rect, widget_state))
}

fn do_the_lua<'a, F: FnOnce(Context<'a>, Vec<Table<'a>>)>(lua_context: Context<'a>, path: &[&str], f: F) {
    let globals = lua_context.globals();

    if let Ok(core) = globals.get::<_, Table>("Core") {
        let mut v = vec![globals, core];
        let mut i = 1;

        // makeshift goto
        'goto: loop {
            let comp = path.len();
            while let Ok(table) = v[i].get::<_, Table>(path[{
                let i = i - 1;
                if i >= comp { break 'goto; }
                i
            }]) {
                v.push(table);
                i += 1;
            }
            
            break 'goto;
        }

        f(lua_context, v);
    }
}