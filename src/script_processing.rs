use std::{fs::{self, DirEntry}, sync::{Arc, Mutex}};

use crossterm::cursor;
use json::{object::Object, JsonValue};
use rlua::{Context, Lua, Table, ToLua, Value};
use terminal_size::terminal_size;

use crate::{dir_entry_is_dir, game_data_dump, identifier_dump, last_tick, player, std_out, tile_map, time_between_ticks, write_to_debug, write_to_debug_pretty, Tile, CURSOR_POS, LAST_TICK, LUA, MAP_HEIGHT, MAP_LENGTH, MODULES_PATH, STATE_CHANGED, TIME_BETWEEN_TICKS};

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

pub fn get_config_info() -> JsonValue {
    let config_data = fs::read_to_string(r#"resources\config\config.json"#).expect("config should exist");
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

        let get_config_lua = lua_context.create_function(|lua_context, ()| {
            Ok(json_to_lua(lua_context, &get_config_info()))
        }).unwrap();
        core.set("getConfig", get_config_lua).unwrap();
    }
    // terminal management
    {
        let terminal_table = lua_context.create_table().unwrap();
        let get_terminal_size = lua_context.create_function(|lua_context, ()| {
            let terminal_size = terminal_size();

            let (terminal_width, terminal_height) = match terminal_size {
                Some(size) => (size.0.0, size.1.0),
                None => (50, 50),
            };

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
