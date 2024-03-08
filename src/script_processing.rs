use std::{collections::HashSet, fs::{self, DirEntry}, sync::{Arc, Mutex}};

use crossterm::{cursor, event::{KeyCode, KeyEvent, KeyEventState, KeyModifiers}};
use json::{object::Object, JsonValue};
use rlua::{Context, Function, Lua, Table, ToLua, ToLuaMulti, Value};

use crate::{dir_entry_is_dir, game_data_dump, identifier_dump, last_tick, lua, player, std_out, tile_map, time_between_ticks, write_to_debug, write_to_debug_pretty, Tile, CURSOR_POS, LAST_TICK, LUA, MAP_HEIGHT, MAP_LENGTH, MODULES_PATH, STATE_CHANGED, TIME_BETWEEN_TICKS};

pub fn run_lua_scripts_from_path(path: &str, lua: Arc<Mutex<Lua>>) {
    let dir = fs::read_dir(path).unwrap();
    
    for data in dir {
        let data = data.unwrap();

        lua.lock().unwrap().context(|lua_context| {
            load_lua_script(Ok(&data), lua_context);
        });
    }

    unsafe { 
        LUA = Some(lua)
    }
}

pub fn load_lua_script(data: Result<&DirEntry, &std::io::Error>, lua_context: Context) {
    if dir_entry_is_dir(data) {
        let data = fs::read_dir(data.unwrap().path()).unwrap();
        for data in data {
            load_lua_script(data.as_ref(), lua_context);
        }

    } else {
        let data = data.unwrap();
        
        if let Some(extension) = data.path().extension() {
            if extension == "lua" {
                
                let script_contents = fs::read_to_string(data.path()).unwrap();

                if let Err(e) = lua_context.load(&script_contents).exec() {
                    write_to_debug_pretty(format!("{}:\n{:?}", data.file_name().to_str().unwrap_or("Invalid utf-8"), e));
                }

            }
        }
        
    }
}

pub fn get_json_info(path: String) -> JsonValue {
    let config_data = fs::read_to_string(path).expect("config should exist");
    let config_data = json::parse(&config_data).expect("config should exist");
    config_data
}

fn json_object_to_lua_table<'a>(lua_context: Context<'a>, object: &Object) -> Table<'a>{
    let table = lua_context.create_table().unwrap();

    for (key, val) in object.iter() {
        table.set(key, json_to_lua(lua_context, val)).unwrap();
    }

    table
}

fn json_array_to_lua_table<'a>(lua_context: Context<'a>, arr: &Vec<JsonValue>) -> Table<'a> {
    let table = lua_context.create_table().unwrap();

    let mut last_index = 1;
    for val in arr {
        table.set(last_index, json_to_lua(lua_context, val)).unwrap();
        last_index += 1;
    }

    table
}

fn json_to_lua<'a>(lua_context: Context<'a>, json_val: &JsonValue) -> rlua::Value<'a> {
    match json_val {
        JsonValue::Null => Value::Nil,
        JsonValue::Short(s) => s.as_str().to_lua(lua_context).unwrap(),
        JsonValue::String(s) => s.as_str().to_lua(lua_context).unwrap(),
        JsonValue::Number(n) => Value::Number((*n).into()),
        JsonValue::Boolean(b) => Value::Boolean(*b),
        JsonValue::Object(o) => Value::Table(json_object_to_lua_table(lua_context, &o)),
        JsonValue::Array(a) => Value::Table(json_array_to_lua_table(lua_context, &a)),
    }
}

fn lua_table_to_json(lua_table: &Table) -> JsonValue {
    let mut keys = Vec::new();
    let mut vals = Vec::new();

    let mut is_arr = true;
    let table_len = lua_table.len().unwrap_or(0) as usize;
    let mut key_checks = HashSet::with_capacity(table_len);

    for pair in lua_table.clone().pairs::<Value, Value>() {
        if let Ok((key, val)) = pair {
            if is_arr {
                if let Value::Integer(i) = key {
                    key_checks.insert(i as usize);
                } else {
                    is_arr = false;
                }
            }

            keys.push(key);
            vals.push(val);
        }
    }

    if is_arr {
        for i in 1..=table_len {
            if !key_checks.contains(&i) {
                is_arr = false;
                break;
            }
        }
    }

    if is_arr {
        let mut json_arr: Vec<JsonValue> = (0..table_len).map(|_| JsonValue::Null).collect();

        for (index, val) in keys.into_iter().zip(vals) {
            if let Value::Integer(index) = index {
                json_arr[index as usize - 1] = lua_to_json(&val);
            }
        }
        return JsonValue::Array(json_arr);
    }

    let mut json_obj = json::object::Object::new();
    for (key, val) in keys.into_iter().zip(vals) {
        if let Value::String(key) = key {
            json_obj.insert(key.to_str().unwrap(), lua_to_json(&val));
        }
    }
    return JsonValue::Object(json_obj);
}


fn lua_to_json(lua_val: &rlua::Value<'_>) -> JsonValue {
    match lua_val {
        Value::Nil => JsonValue::Null,
        Value::Boolean(b) => JsonValue::Boolean(*b),
        Value::LightUserData(_) => todo!(),
        Value::Integer(i) => JsonValue::Number((*i).into()),
        Value::Number(n) => JsonValue::Number((*n).into()),
        Value::String(s) => JsonValue::String(s.to_str().unwrap().to_owned()),
        Value::Table(t) => lua_table_to_json(t),
        Value::Function(_) => todo!(),
        Value::Thread(_) => todo!(),
        Value::UserData(_) => todo!(),
        Value::Error(e) => panic!("{}", e.to_string()),
    }
}

pub fn load_default_lua_data(lua_context: Context) {
    let globals = lua_context.globals();
    let core = lua_context.create_table().unwrap();

    globals.set("print", Value::Nil).unwrap();

    // preset global variables
    // core
    {
        let print_to_debug = lua_context.create_function(|_, text: Value| {
            write_to_debug(match text {
                Value::Nil => String::from("Nil"),
                Value::Boolean(b) => format!("{}", b),
                Value::LightUserData(l) => format!("{:?}", l),
                Value::Integer(i) => format!("{}", i),
                Value::Number(n) => format!("{}", n),
                Value::String(s) => String::from(s.to_str().unwrap_or("invalid utf-8")),
                Value::Table(t) => format!("{:?}", t),
                Value::Function(f) => format!("{:?}", f),
                Value::Thread(t) => format!("{:?}", t),
                Value::UserData(u) => format!("{:?}", u),
                Value::Error(e) => format!("{}", e),
            });
            Ok(())
        }).unwrap();
        core.set("print", print_to_debug).unwrap();

        let reload = lua_context.create_function(|_, ()|{
            let new_lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new( Lua::new() ));

            new_lua.lock().unwrap().context(|lua_context|{
                load_default_lua_data(lua_context);
            });
            run_lua_scripts_from_path(MODULES_PATH, new_lua);

            Ok(())
        }).unwrap();
        core.set("reload", reload).unwrap();

        let get_json_lua = lua_context.create_function(|lua_context, path: String| {
            Ok(json_to_lua(lua_context, &get_json_info(path)))
        }).unwrap();
        core.set("getJSON", get_json_lua).unwrap();

        let set_json_lua = lua_context.create_function(|lua_context, (path, table): (String, Table)| {
            let to_write = lua_table_to_json(&table);
            let to_write = json::stringify(to_write);
            let res = fs::write(path, to_write);
            if let Err(e) = res {
                write_to_debug(e);
            }
            Ok(())
        }).unwrap();
        core.set("setJSON", set_json_lua).unwrap();
    }
    // terminal management
    {
        let terminal_table = lua_context.create_table().unwrap();
        let get_terminal_size = lua_context.create_function(|lua_context, ()| {
            let (terminal_width, terminal_height) = crossterm::terminal::size().unwrap();

            let luafied_terminal_size = lua_context.create_table().unwrap();
            luafied_terminal_size.set("width", terminal_width).unwrap();
            luafied_terminal_size.set("height", terminal_height).unwrap();

            Ok(luafied_terminal_size)
        }).unwrap();
        terminal_table.set("getSize", get_terminal_size).unwrap();

        let cursor_pos_table = lua_context.create_table().unwrap();
        cursor_pos_table.set("x", unsafe { CURSOR_POS.0 }).unwrap();
        cursor_pos_table.set("y", unsafe { CURSOR_POS.1 }).unwrap();
        terminal_table.set("cursorPos", cursor_pos_table).unwrap();

        let move_cursor = lua_context.create_function(|_, (x, y): (u16, u16)| {
            ignorant_execute!(std_out, cursor::MoveToColumn(x));
            ignorant_execute!(std_out, cursor::MoveToRow(y));
            Ok(())
        }).unwrap();
        terminal_table.set("moveCursor", move_cursor).unwrap();

        let print_to_terminal = lua_context.create_function(|_, text: String| {
            println!("{}", text);
            Ok(())
        }).unwrap();
        terminal_table.set("print", print_to_terminal).unwrap();
        
        core.set("Terminal", terminal_table).unwrap();
    }
    // events
    {
        let events_table = lua_context.create_table().unwrap();
        events_table.set("PostDeserializationEvents", lua_context.create_table().unwrap()).unwrap();
        events_table.set("TickEvents", lua_context.create_table().unwrap()).unwrap();
        events_table.set("KeyEvents", lua_context.create_table().unwrap()).unwrap();
        events_table.set("CommandEvents", lua_context.create_table().unwrap()).unwrap();

        core.set("Events", events_table).unwrap();
    }
    // initialization info
    {
        let initialization_info = lua_context.create_table().unwrap();
        initialization_info.set("GameData", lua_context.create_table().unwrap()).unwrap();

        core.set("InitializationInfo", initialization_info).unwrap();
    }
    // game info
    {
        let game_info_table = lua_context.create_table().unwrap();
        // player
        let lua_player = lua_context.create_table().unwrap();

        let player_get_x = lua_context.create_function(|_, ()| {
            Ok(player().position.0)
        }).unwrap();
        lua_player.set("getX", player_get_x).unwrap();

        let player_get_y = lua_context.create_function(|_, ()| {
            Ok(player().position.1)
        }).unwrap();
        lua_player.set("getY", player_get_y).unwrap();

        let player_set_pos = lua_context.create_function(|_, (x, y): (usize, usize)| {
            player().position = (x, y);
            Ok(())
        }).unwrap();
        lua_player.set("setPosition", player_set_pos).unwrap();

        game_info_table.set("Player", lua_player).unwrap();

        // map
        let lua_map = lua_context.create_table().unwrap();
        let tile_map_table = lua_context.create_table().unwrap();

        let tile_map_get = lua_context.create_function(|lua_context, (x, y): (usize, usize)|{
            let requested = tile_map()[y][x];

            let luafied_tile = lua_context.create_table().unwrap();
            luafied_tile.set("type", requested.tile_type).unwrap();

            let luafied_text_display = lua_context.create_table().unwrap();
            luafied_text_display.set("characterLeft", String::from(requested.text_display.character_left.unwrap_or(' '))).unwrap();
            luafied_text_display.set("characterRight", String::from(requested.text_display.character_right.unwrap_or(' '))).unwrap();
            luafied_tile.set("textDisplay", luafied_text_display).unwrap();
            //luafied_tile.set("colorDisplay", requested.color_display);

            Ok(luafied_tile)
        }).unwrap();
        tile_map_table.set("get", tile_map_get).unwrap();

        let tile_map_set_from_id = lua_context.create_function(|_, (x, y, tile_id): (usize, usize, u16)| {
            let tile = Tile::new(tile_id);
            match tile {
                Some(tile) => {
                    tile_map()[y][x] = tile;
                    return Ok(true)
                },
                None => return Ok(false),
            }
        }).unwrap();
        tile_map_table.set("setFromId", tile_map_set_from_id).unwrap();
        lua_map.set("TileMap", tile_map_table).unwrap();

        lua_map.set("width", MAP_LENGTH - 1).unwrap();
        lua_map.set("height", MAP_HEIGHT - 1).unwrap();

        game_info_table.set("Map", lua_map).unwrap();

        // tile
        let tile_table = lua_context.create_table().unwrap();
        let tile_types_table = lua_context.create_table().unwrap();
        let tile_idents_table = lua_context.create_table().unwrap();

        // tile type
        let tile_type_get = lua_context.create_function(|lua_context, tile_id: u16| {
            let luafied_tile_type = lua_context.create_table().unwrap();
            if let Some(tile_type) = game_data_dump().tile_types.get(&tile_id) {
                luafied_tile_type.set("solid", tile_type.solid).unwrap();
            }
            
            Ok(luafied_tile_type)
        }).unwrap();
        tile_types_table.set("get", tile_type_get).unwrap();
        tile_table.set("Types", tile_types_table).unwrap();
        
        // tile ident
        let tile_ident_get = lua_context.create_function(|_, internal_name: String| {
            let ret = identifier_dump().tile_types.get_by_left(&internal_name);
            let ret = match ret {
                Some(id) => Value::Number(*id as f64),
                None => Value::Nil,
            };

            Ok(ret)
        }).unwrap();
        tile_idents_table.set("get", tile_ident_get).unwrap();
        tile_table.set("Identifiers", tile_idents_table).unwrap();

        game_info_table.set("Tile", tile_table).unwrap();
        

        core.set("GameInfo", game_info_table).unwrap();
    }
    // ui render
    {
        let buffer_map_redraw = lua_context.create_function(|_, ()| {
            unsafe { STATE_CHANGED = true; }
            Ok(())
        }).unwrap();

        core.set("bufferMapRedraw", buffer_map_redraw).unwrap();
    }
    // tick
    {
        let tick_table = lua_context.create_table().unwrap();
        let add_time = lua_context.create_function(|_, amount: u32| {
            unsafe {
                LAST_TICK = Some(last_tick() + (time_between_ticks() * amount));
            }
            
            Ok(())
        }).unwrap();
        tick_table.set("addTime", add_time).unwrap();

        core.set("Tick", tick_table).unwrap();
    }

    globals.set("Core", core).unwrap();
}

// hellish conversion to String
pub fn keycode_to_string(keycode: KeyCode) -> String{
    match keycode {
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
            format!("{:?}", keycode).to_lowercase()
        }
    }
}

pub fn call_key_events(event: &KeyEvent) {
    lua().lock().unwrap().context(|lua_context| {
        let globals = lua_context.globals();

        if let Ok(core) = globals.get::<_, Table>("Core") {
            if let Ok(events) = core.get::<_, Table>("Events") {
                if let Ok(key_events) = events.get::<_, Table>("KeyEvents") {
                    let luafied_event = lua_context.create_table().unwrap();
                    let (code, modifiers, kind, state) = (event.code, event.modifiers, event.kind, event.state);

                    let luafied_code = keycode_to_string(code);
                
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