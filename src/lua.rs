use std::{collections::HashMap, fs, io::{BufRead, BufReader}, path::Path};

use globset::{Glob, GlobMatcher};
use itertools::Itertools;
use mlua::{Function, Lua, Table, Value, Variadic, LuaSerdeExt};
use ratatui::widgets::{Block, Borders, Paragraph};
use regex::Regex;
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

use crate::{BlockWrapper, CrossTerminal, DynErrResult, GameWrapper, ParagraphWrapper};

use paste::paste;

pub const REGISTRY_NAME: &str = "event_registry";
pub const REGISTRY_PATH: &[&str] = &["core", REGISTRY_NAME];
pub const CLINC: &str = "clinc";

pub fn setup_lua(lua: &Lua, terminal: &CrossTerminal) -> DynErrResult<()> {
    macro_rules! create_ { // create_!(lua, thing, parent, name|key, contents)
        ($lua: ident, $thing: ident, $parent: ident, $name: ident, $stuff: expr) => {
            paste! {
                let [<lua_ $name>] = $lua.[<create_ $thing>]($stuff)?;
                $parent.set(stringify!($name), [<lua_ $name>])?;
            }
        };

        ($lua: ident, $thing: ident, $parent: ident, $name: expr, $stuff: expr) => {
            paste! {
                $parent.set($name, $lua.[<create_ $thing>]($stuff)?)?;
            }
        };
    }
    macro_rules! create_table { // create_table!(lua, parent, name|key)
        ($lua: ident, globals, $name: ident) => {
            let s_name = stringify!($name);
            lua.globals().set(s_name, lua.create_table()?)?;
            let $name = $lua.globals().get::<_, Table>($s_name)?;
        };

        ($lua: ident, $parent: ident, $name: ident) => {
            let s_name = stringify!($name);

            $parent.set(s_name, $lua.create_table()?)?;
            let $name = $parent.get::<_, Table>(s_name)?;
        };

        ($lua: ident, globals, $name: expr) => {
            lua.globals().set($name, $lua.create_table()?)?;
        };

        ($lua: ident, $parent: ident, $name: expr) => {
            paste! {
                $parent.set($name, $lua.create_table()?)?;
            }
        };
    }
    macro_rules! embed_func { // embed_func!(lua, parent, function, args?)
        ($lua: ident, $parent: ident, $func: ident, (lua: &Lua)) => {
            paste!{
                let [<lua_ $func>] = $lua.create_function(|lua, ()| {
                    Ok($func(lua))
                })?;
                $parent.set(stringify!($func), [<lua_ $func>])?;
            }
        };

        ($lua: ident, $parent: ident, $func: ident, (lua: &Lua, $($arg: ident : $ty: ty),+)) => {
            paste! {
                #[allow(unused_parens)]
                let [<lua_ $func>] = $lua.create_function(|lua: &Lua, ($($arg),+): ($($ty),+)| {
                    Ok($func(lua,  $($arg),+ ))
                })?;
                $parent.set(stringify!($func), [<lua_ $func>])?;
            }
        };
        
        ($lua: ident, $parent: ident, $func: ident) => {
            paste!{
                let [<lua_ $func>] = $lua.create_function(|_lua, ()| {
                    Ok($func())
                })?;
                $parent.set(stringify!($func), [<lua_ $func>])?;
            }
        };

        ($lua: ident, $parent: ident, $func: ident, ($($arg: ident : $ty: ty),+)) => {
            paste! {
                #[allow(unused_parens)]
                let [<lua_ $func>] = $lua.create_function(|_lua: &Lua, ($($arg),+): ($($ty),+)| {
                    Ok($func( $($arg),+ ))
                })?;
                $parent.set(stringify!($func), [<lua_ $func>])?;
            }
        };
    }

    {
        let globals = lua.globals();
        let core = globals.get::<_, Table>("core").expect("should have been initialized in lua src");
        let clinc = globals.get::<_, Table>(CLINC).expect("should have been initialized in lua src");
        
        global_lib(&lua, globals)?;

        create_table!(lua, clinc, widget);
        widget_interface(lua, widget)?;
        
        create_table!(lua, clinc, utility);      
        utility_interface(&lua, utility)?;
    }
    
    return Ok(());

    fn world_interface(lua: &Lua, world: Table) -> DynErrResult<()> {
        
        Ok(()) 
    }

    fn widget_interface(lua: &Lua, widget: Table) -> DynErrResult<()> {
        create_!(lua, function, widget, "new", |lua, (r#type, data): (String, Variadic<Value>)| {
            let widget_type = r#type.to_lowercase();
            let widget = match widget_type.as_str() {
                "block" => {
                    let mut block = Block::new();
                    let data = data[0].as_table().cloned().unwrap();
                    for data in data.pairs::<String, Value>() {
                        let (key, val) = data?;
                        block = match key.as_str() {
                            "borders" => block.borders(border_from_str(&val.to_string()?)),
                            _ => {continue;}
                        }
                    }
                    lua.create_userdata(BlockWrapper(block))?
                },
                "paragraph" => {
                    let mut paragraph = Paragraph::new(data[0].as_str().unwrap_or("").to_owned());
                    let data = data[1].as_table().cloned().unwrap();
                    for data in data.pairs::<String, Value>() {
                        let (key, val) = data?;
                        paragraph = match key.as_str() {
                            "block" => {
                                let block = (&val.as_userdata().unwrap().borrow::<BlockWrapper>()?.0).to_owned();
                                paragraph.block(block)
                            },
                            _ => {continue;}
                        }
                    }
                    lua.create_userdata(ParagraphWrapper(paragraph))?
                },
                _ => unimplemented!()
            };
            Ok(widget)
        });
        Ok(())
    }
    
    fn global_lib(lua: &Lua, globals: Table) -> DynErrResult<()> {
        create_!(lua, function, globals, "exit", |_, code: i32| -> mlua::Result<()> /* mlua::Result<!> */ {
            std::process::exit(code)
        });
        create_!(lua, function, globals, "panic!", |_, message: Variadic<String>| -> mlua::Result<()> /* mlua::Result<!> */ {
            if let Some(message) = message.iter().nth(0) {
                panic!("{}", message)
            } else {
                panic!()
            }
        });

        Ok(())
    }

    fn utility_interface(lua: &Lua, utility: Table) -> DynErrResult<()> {
        create_!(lua, function, utility, "table_to_string", |_, table: Table| {
            Ok(format!("{:#?}", table))
        });

        create_!(lua, function, utility, "json_string_to_table", |lua, json: String| {
            let json: serde_json::Value = serde_json::from_str(&json).unwrap();
            let table: mlua::Value = lua.to_value(&json)?;

            Ok(table)
        });

        Ok(())
    }
}

fn border_from_str(s: &str) -> Borders {
    match s.to_lowercase().as_str() {
        "all" => Borders::ALL,
        "top" => Borders::TOP,
        "bottom" => Borders::BOTTOM,
        "left" => Borders::LEFT,
        "right" => Borders::RIGHT,
        _ => Borders::NONE,
    }
}

pub fn call_lua_events_from_game(game: &GameWrapper, event_name: &'static str, args: Variadic<Value>) -> DynErrResult<()> {
    let lua = game.lua();
    let hidden_registry = traverse_lua_tables(lua.globals(), REGISTRY_PATH)?;
    
    call_lua_events(hidden_registry.get(event_name)?, args)?;
    Ok(())
}

pub fn traverse_lua_tables<'lua>(start: Table<'lua>, path: &'static [&'static str]) -> DynErrResult<Table<'lua>> {
    let mut table = start.clone();
    for &step in path {
        table = table.get::<_, Table>(step)?;
    }
    Ok(table)
}

pub fn call_lua_events(events: Table, args: Variadic<Value>) -> DynErrResult<()> {
    for res in events.pairs::<Value, Function>() {
        let (_, event) = res?;
        event.call(args.clone())?;
    }

    Ok(())
}

pub struct ModScript {
    priority: u16,
    file: DirEntry
}
pub fn register_scripts<P: AsRef<Path>>(path: P) -> DynErrResult<Vec<ModScript>> {
    let lua_glob = Glob::new("*.lua")?.compile_matcher();
    let disabled_glob = Glob::new("--*disable?")?.compile_matcher();
    let priority_reg = Regex::new(r"-- *priority: *(?<priority>[0-9]+)")?;

    let files = files_from_glob(&lua_glob, path);

    let mut scripts = Vec::new();
    for entry in files {
        let file = fs::File::open(entry.path())?;
        let file_buf_reader = BufReader::new(file);

        let first_line = file_buf_reader.lines().next();
        if let Some(Ok(first_line)) = first_line {
            if disabled_glob.is_match(&first_line) { continue; }
            let priority = if let Some(caps) = priority_reg.captures(&first_line) {
                caps["priority"].parse::<u16>().unwrap_or(0)
            } else {
                0
            };

            println!("registering {} with priority {}", 
                entry.path().display(),
                priority,
            );

            scripts.push(ModScript{
                priority,
                file: entry
            });
        }
    }

    Ok(scripts)
}

pub struct Mod {
    pub priority: u16,
    pub mod_scripts: Vec<ModScript>
}
#[derive(Deserialize)]
struct DependencyInfo {
   name: String,
   version: String,
}
#[derive(Deserialize)]
struct ModConfig {
   name: String,
   version: String,
   priority: Option<u16>,
   dependencies: Option<Vec<DependencyInfo>>
}
pub fn register_mods<P: AsRef<Path>>(path: P) -> DynErrResult<HashMap<String, Mod>> {
    let mod_glob = Glob::new("mod.toml")?.compile_matcher();
    let mut mods = HashMap::new();

    let files = files_from_glob(&mod_glob, path);

    for entry in files {
        let config = fs::read_to_string(entry.path())?;
        let config: ModConfig = toml::from_str(&config).expect("improper mod config");
        let priority = config.priority.unwrap_or(0);
        
        let mod_path = entry.path().parent().unwrap();
        let mod_scripts = register_scripts(mod_path)?;
        
        mods.insert(config.name, Mod{
            priority,
            mod_scripts,
        });
    }
    
    Ok(mods)
}

pub fn load_scripts(lua: &Lua, scripts: &Vec<ModScript>) -> DynErrResult<()> {
    for script in scripts.into_iter()
        .sorted_by(|a, b| 
            Ord::cmp(&a.priority, &b.priority)
        ).rev()
    {
        lua.load(fs::read(script.file.path())?).exec()?;
    }

    Ok(())
}

fn files_from_glob<P: AsRef<Path>>(glob: &GlobMatcher, path: P) -> Vec<DirEntry> {
    WalkDir::new(path).into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            glob.is_match(e.file_name())
        }).collect()
}
