
use rlua::{Context, Function, Table, Value};

use crate::{lua, write_to_debug_pretty};

pub fn action_from_input(input: &str) {
    lua().lock().unwrap().context(|lua_context| {
        action_from_input_with_context(lua_context, input);
    });
}

pub fn action_from_input_with_context(lua_context: Context, input: &str) {
    let globals = lua_context.globals();

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