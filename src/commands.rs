
use mlua::{Function, Lua, Table, Value};

use crate::write_to_debug_pretty;

pub fn action_from_input(lua: &Lua, input: &str) {
    let globals = lua.globals();

    if let Ok(core) = globals.get::<_, Table>("Core") {
        if let Ok(events) = core.get::<_, Table>("Events") {
            if let Ok(events_table) = events.get::<_, Table>("CommandEvents") {

                for pair in events_table.pairs::<Value, Function>() {
                    let pair = pair.unwrap();
                    if let Err(e) = pair.1.call::<&str, ()>(input) {
                        write_to_debug_pretty(format!("{:?}:\n{:?}", pair.0, e));
                    }
                }
            }
        }
    }
}