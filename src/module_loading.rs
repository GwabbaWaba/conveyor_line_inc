use std::{any::Any, collections::HashMap, fs::{self, DirEntry}, io};
use bimap::BiMap;
use mlua::{Function, Lua, Table, Value};
use serde::Deserialize;

use crate::{dir_entry_is_dir, display::{ColorDisplay, ColorDisplayBuilder, TextDisplay, TextDisplayBuilder}, os_string_to_string, write_to_debug, write_to_debug_pretty, GroundType, ItemType, TileType};

pub type DeserializationDump = HashMap<String, Vec<(String, ModuleDeserialization)>>;
pub type PreMapDump<'a> = Vec<UnmappedData<'a>>;
pub type MappedGameData<T> = HashMap<u16, T>;

pub const MOD_PACK_MAPPINGS_PATH: &str = r"resources/mod_pack_mappings";

type IdTracker = (u16, u16, u16, u16, u16, u16);
pub static mut ID_TRACKER: IdTracker = (0, 0, 0, 0, 0, 0);
pub static mut MAPPED_DUMP: Option<GameDataDump> = None;

pub fn mapped_dump<'a>() -> &'a mut GameDataDump {
    unsafe {
        if let Some(mapped_dump) = MAPPED_DUMP.as_mut() {
            mapped_dump
        } else {
            panic!("call for MAPPED_DUMP while MAPPED_DUMP is None(_)")
        }
    }
}
pub fn game_data_dump<'a>() -> &'a mut MappedGameDatas { &mut mapped_dump().game_data } 
pub fn identifier_dump<'a>() -> &'a mut IdentifierMaps { &mut mapped_dump().identifiers } 

deserializable_module_data!{
    struct VisualDeserData

    identifier: String,

    name: String,

    character_left: String,
    character_right: String,
    
    text_color_left: (u8, u8, u8),
    text_color_right: (u8, u8, u8),
    back_color_left: (u8, u8, u8),
    back_color_right: (u8, u8, u8)
}

deserializable_module_data!{
    [has_visual]

    struct TileDeserialData
    solid: bool,
    world_gen_weight: f64
}

deserializable_module_data!{
    [has_visual]

    struct GroundDeserialData
    solid: bool,
    world_gen_weight: f64
}

deserializable_module_data!{
    [has_visual]

    struct ItemDeserialData
}

deserializable_module_data!{
    [has_visual]

    struct VisibleThingDeserialData

    type_identifier: String,
    bytes: Vec<u8>
}

deserializable_module_data!{
    struct ThingDeserialData

    type_identifier: String,
    bytes: Vec<u8>
}

deserializable_module_data!{
    [has_visual]

    struct ByteStreamDeserialData

    type_identifier: String,
    bytes: Vec<u8>
}

#[derive(Debug)]
pub struct MappedGameDatas {
    pub tile_types: MappedGameData<TileType>, 
    pub ground_types: MappedGameData<GroundType>, 
    pub item_types: MappedGameData<ItemType>, 
    pub visible_thing_types: MappedGameData<VisibleThingType>, 
    pub thing_types: MappedGameData<ThingType>, 
    pub byte_streams: MappedGameData<ByteStream>
}

#[derive(Debug)]
pub struct IdentifierMaps {
    pub tile_types: BiMap<String, u16>,
    pub ground_types: BiMap<String, u16>,
    pub item_types: BiMap<String, u16>,
    pub visible_thing_types: BiMap<String, u16>,
    pub thing_types: BiMap<String, u16>,
    pub byte_streams: BiMap<String, u16>,
}

#[derive(Debug)]
pub struct GameDataDump {
    pub game_data: MappedGameDatas,
    pub identifiers: IdentifierMaps
}

// mapping time
fn filtered_pre_map_to_map<'a>(filtered_map_dump: &'a mut PreMapDump<'a>, id_tracker: &mut IdTracker) -> GameDataDump {
    // primary types
    let mut tile_type_map: MappedGameData<TileType> = HashMap::new();
    let mut ground_type_map: MappedGameData<GroundType> = HashMap::new();
    let mut item_type_map: MappedGameData<ItemType> = HashMap::new();

    let mut tile_type_identifiers: BiMap<String, u16> = BiMap::new();
    let mut ground_type_identifiers: BiMap<String, u16> = BiMap::new();
    let mut item_type_identifiers: BiMap<String, u16> = BiMap::new();

    // weird arbitrary ones
    let mut visible_thing_type_map: MappedGameData<VisibleThingType> = HashMap::new();
    let mut thing_type_map: MappedGameData<ThingType> = HashMap::new();
    let mut byte_stream_map: MappedGameData<ByteStream> = HashMap::new();

    let mut visible_thing_type_identifiers: BiMap<String, u16> = BiMap::new();
    let mut thing_type_identifiers: BiMap<String, u16> = BiMap::new();
    let mut byte_stream_identifiers: BiMap<String, u16> = BiMap::new();

    for i in (0..filtered_map_dump.len()).rev() {
        let entry = &filtered_map_dump[i];

        let ident = &entry.identifier;

        match ident.data_type.as_str() {
            "tile" => {
                let downcast = entry.data.as_any().downcast_ref::<TileDeserialData>().unwrap();
                let visual_deser = match find_visual_data(filtered_map_dump, &entry.identifier) {
                    Some(vd) => vd,
                    None => { 
                        write_to_debug(format!("Failed to load {}!", entry.identifier.name));
                        continue; 
                    },
                };
                let visual_data = text_and_color_from_deser(visual_deser);

                tile_type_map.insert(
                    id_tracker.0, 
                    TileType {
                        identifier: id_tracker.0,
                        name: visual_data.2,
                        text_display: visual_data.0,
                        color_display: visual_data.1,
                        solid: downcast.solid.unwrap_or(true),
                        world_gen_weight: downcast.world_gen_weight.unwrap_or(0.0),
                    }
                );
                
                tile_type_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.0);
                id_tracker.0 += 1;
            },
            "ground" => {
                let downcast = entry.data.as_any().downcast_ref::<GroundDeserialData>().unwrap();
                let visual_deser = match find_visual_data(filtered_map_dump, &entry.identifier) {
                    Some(vd) => vd,
                    None => { 
                        println!("Failed to load {}!", entry.identifier.name);
                        continue; 
                    },
                };
                let visual_data = text_and_color_from_deser(visual_deser);

                ground_type_map.insert(
                    id_tracker.1, 
                    GroundType {
                        identifier: id_tracker.1,
                        text_display: visual_data.0,
                        color_display: visual_data.1,
                        solid: downcast.solid.unwrap_or(true),
                        world_gen_weight: downcast.world_gen_weight.unwrap_or(0.0),
                    }
                );

                ground_type_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.1);
                id_tracker.1 += 1;
            },
            "item" => {
                let visual_deser = match find_visual_data(filtered_map_dump, &entry.identifier) {
                    Some(vd) => vd,
                    None => { 
                        println!("Failed to load {}!", entry.identifier.name);
                        continue; 
                    },
                };
                let visual_data = text_and_color_from_deser(visual_deser);

                item_type_map.insert(
                    id_tracker.2, 
                    ItemType {
                        identifier: id_tracker.1,
                        text_display: visual_data.0,
                        color_display: visual_data.1,
                    }
                );

                item_type_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.2);
                id_tracker.2 += 1;
            },
            "vis_thing" => {
                let downcast = entry.data.as_any().downcast_ref::<VisibleThingDeserialData>().unwrap();
                let visual_deser = match find_visual_data(filtered_map_dump, &entry.identifier) {
                    Some(vd) => vd,
                    None => { 
                        println!("Failed to load {}!", entry.identifier.name);
                        continue; 
                    },
                };
                let visual_data = text_and_color_from_deser(visual_deser);

                visible_thing_type_map.insert(
                    id_tracker.3, 
                    VisibleThingType {
                        identifier: id_tracker.1,
                        type_identifier: {
                            downcast.type_identifier.as_ref().unwrap().clone()
                        },
                        text_display: visual_data.0,
                        color_display: visual_data.1,
                    }
                );

                visible_thing_type_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.3);
                id_tracker.3 += 1;
            },
            "thing" => {
                let downcast = entry.data.as_any().downcast_ref::<ThingDeserialData>().unwrap();

                thing_type_map.insert(
                    id_tracker.4, 
                    ThingType {
                        identifier: id_tracker.1,
                        type_identifier: {
                            downcast.type_identifier.as_ref().unwrap().clone()
                        },
                    }
                );

                thing_type_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.4);
                id_tracker.4 += 1;
            },
            "byte_stream" => {
                let downcast = entry.data.as_any().downcast_ref::<ByteStreamDeserialData>().unwrap();

                byte_stream_map.insert(
                    id_tracker.5, 
                    ByteStream {
                        identifier: id_tracker.1,
                        bytes: downcast.bytes.as_ref().unwrap().clone(),
                    }
                );

                byte_stream_identifiers.insert(format!("{}:{}", ident.source, ident.name), id_tracker.5);
                id_tracker.5 += 1;
            },
            _ => {}
        }
    }

    GameDataDump {
        game_data: MappedGameDatas {
            tile_types: tile_type_map,
            ground_types: ground_type_map,
            item_types: item_type_map,
            visible_thing_types: visible_thing_type_map,
            thing_types: thing_type_map,
            byte_streams: byte_stream_map,
        },
        identifiers: IdentifierMaps {
            tile_types: tile_type_identifiers,
            ground_types: ground_type_identifiers,
            item_types: item_type_identifiers,
            visible_thing_types: visible_thing_type_identifiers,
            thing_types: thing_type_identifiers,
            byte_streams: byte_stream_identifiers,
        },
    }
}

fn find_visual_data<'a>(filtered_map_dump: &'a PreMapDump<'a>, identifier: &PreMapIdentifier) -> Option<&'a VisualDeserData> {
    for i in (0..filtered_map_dump.len()).rev() {
        let ident = &filtered_map_dump[i].identifier;
        if ident.data_type == "vis_data" && ident.source == identifier.source && ident.name == identifier.name  {
            return filtered_map_dump[i].data.as_any().downcast_ref::<VisualDeserData>();
        }
    }
    None
}

fn text_and_color_from_deser(visual_deser: &VisualDeserData) -> (TextDisplay, ColorDisplay, String) {
    let mut text = TextDisplayBuilder::new();

    if let Some(cl) = &visual_deser.character_left {
        text.character_left(cl.chars().nth(0).unwrap());
    }
    if let Some(cr) = &visual_deser.character_right {
        text.character_right(cr.chars().nth(0).unwrap());
    }

    let text = text.finalize();

    let mut color = ColorDisplayBuilder::new();
                            
    if let Some(tl) = &visual_deser.text_color_left {
        color.text_color_left(*tl);
    }
    if let Some(tr) = &visual_deser.text_color_right {
        color.text_color_right(*tr);
    }
    if let Some(bl) = &visual_deser.back_color_left {
        color.back_color_left(*bl);
    }
    if let Some(br) = &visual_deser.back_color_right {
        color.back_color_right(*br);
    }

    let color = color.finalize();

    let name = match &visual_deser.name {
        Some(name) => name.clone(),
        None => String::from(""),
    };

    (text, color, name)
}

// pre map
fn filter_pre_map_dump(pre_map_dump: & mut PreMapDump) {
    for i in (0..pre_map_dump.len()).rev() {
        for j in (0..pre_map_dump.len()).rev() {
            if i == j { continue; }

            let current = &pre_map_dump[i];
            let other = &pre_map_dump[j];

            let cur_ident = &current.identifier;
            let oth_ident = &other.identifier;

            if cur_ident.source == oth_ident.source && cur_ident.data_type == oth_ident.data_type && cur_ident.name == oth_ident.name {
                if cur_ident.priority < oth_ident.priority {
                    pre_map_dump[i] = other.clone();
                }
                pre_map_dump.remove(j);
            }
        }
    }
}

// pre filter
#[derive(Clone, Debug)]
struct PreMapIdentifier {
    source: String,
    priority: u8,
    data_type: String,
    name: String,
}

trait AsAny {
    fn as_any(&self) -> &dyn Any;
}
trait Deserialization: std::fmt::Debug + AsAny {}

#[derive(Clone, Debug)]
pub struct UnmappedData<'a> {
    identifier: PreMapIdentifier,
    data: Box<&'a dyn Deserialization>
}

pub fn map_deserialized_dump<'a>(pre_map_dump: &'a mut PreMapDump<'a>, deserial_dump: &'a DeserializationDump, id_tracker: &mut IdTracker) -> GameDataDump {
    for (module_name, module_contents) in deserial_dump {
        for (file_name, file_data) in module_contents {
            let priority = file_data.priority.unwrap_or(0);

            let file_contents = get_in_file(file_data);

            for file_content in file_contents {
                if let Some((data_type, data)) = file_content {
                    let pre_map_ident = PreMapIdentifier {
                        source: {
                            match &file_data.source {
                                Some(explicit_source) => explicit_source.clone(),
                                None => module_name.clone(),
                            }
                        },
                        priority,
                        data_type: data_type.to_owned(),
                        name: file_name.clone(),
                    };
        
                    let pre_map_data = UnmappedData {
                        identifier: pre_map_ident,
                        data,
                    };

                    pre_map_dump.push(pre_map_data);
                }
            }
        }
    }

    filter_pre_map_dump(pre_map_dump);
    filtered_pre_map_to_map(pre_map_dump, id_tracker)
}

fn get_in_file(file_data: &ModuleDeserialization) -> [Option<(String, Box<&dyn Deserialization>)>; 7] {
    let mut file_contents: [Option<(String, Box<&dyn Deserialization>)>; 7] = [None, None, None, None, None, None, None];
    // prefixed cause the read in macro wasn't counting as a read
    let mut _last = 0;

    macro_rules! add_to_file_contents {
        (
            $thing:expr; $type:expr
        ) => {
            if let Some(x) = &$thing {
                file_contents[_last] = Some((String::from($type), Box::new(x)));
            }
            _last += 1;
        };
    }

    
    add_to_file_contents!(file_data.tile; "tile");
    add_to_file_contents!(file_data.ground; "ground");
    add_to_file_contents!(file_data.item; "item");
    add_to_file_contents!(file_data.visible_thing; "vis_thing");
    add_to_file_contents!(file_data.thing; "thing");
    add_to_file_contents!(file_data.byte_stream; "byte_stream");
    add_to_file_contents!(file_data.visual_data; "vis_data");

    file_contents
}



// deserialization
pub fn deserialize_modules_from_path(lua: &Lua, game_data: &mut DeserializationDump, path: &'static str) {
    let dir = fs::read_dir(path).unwrap();
    
    for data in dir {
        let data = data.unwrap();

        let mut data_stack = Vec::new();
        load_module_data(&mut data_stack, Ok(&data));

        let data_key = os_string_to_string(data.file_name());
        game_data.insert(data_key, data_stack);
    }

    post_deserialization_events(lua, game_data);
}

/// run post deserialization lua events
fn post_deserialization_events(lua: &Lua, game_data: &mut DeserializationDump) {
    let globals = lua.globals();

    if let Ok(core) = globals.get::<_, Table>("Core") {
        if let Ok(events) = core.get::<_, Table>("Events") {
            if let Ok(post_deserialization_events) = events.get::<_, Table>("PostDeserializationEvents") {
                
                for pair in post_deserialization_events.pairs::<Value, Function>() {
                    let pair = pair.unwrap();
                    if let Err(e) = pair.1.call::<_, Value>(()) {
                        write_to_debug_pretty(format!("{:?}:\n{:?}", pair.0, e));
                    }
                }
            }
        }

        if let Ok(lua_init_info) = core.get::<_, Table>("InitializationInfo") {
            if let Ok(lua_game_data) = lua_init_info.get::<_, Table>("GameData") {
                for pair in lua_game_data.pairs::<String, Table>() {
                    if let Ok((source, data_table)) = pair {
                        match game_data.get_mut(&source) {
                            Some(existing_data_stack) => {
                                data_table_to_data_stack(data_table, existing_data_stack);
                            },
                            None => {
                                let data_stack = Vec::new();
                                data_table_to_data_stack(data_table, &mut Vec::new());
                                game_data.insert(source, data_stack);
                            },
                        }
                    }
                }
            }
        }
    }

}

fn data_table_to_data_stack(data_table: Table<'_>, data_stack: &mut Vec<(String, ModuleDeserialization)>) {
    for pair in data_table.pairs::<String, Table>() {
        if let Ok((name, data)) = pair {
            let mod_deser = ModuleDeserialization {
                _force_deser: None,
                source: match data.get("source") {
                    Ok(source) => Some(source),
                    Err(_) => None,
                },
                priority: match data.get("priority") {
                    Ok(priority) => Some(priority),
                    Err(_) => None,
                },
                tile: match data.get::<_, Table<'_>>("tile") {
                    Ok(tile) => Some(TileDeserialData {
                        _force_deser: None,
                        visual_data: None,
                        solid: match tile.get("solid") {
                            Ok(solid) => Some(solid),
                            Err(_) => Some(true),
                        },
                        world_gen_weight: match tile.get("world_gen_weight") {
                            Ok(weight) => Some(weight),
                            Err(_) => None,
                        },
                    }),
                    Err(_) => None,
                },
                ground: None,
                item: match data.get::<_, Table<'_>>("item") {
                    Ok(item) => Some(ItemDeserialData {
                        _force_deser: None,
                        visual_data: None,
                    }),
                    Err(_) => None,
                },
                visible_thing: None,
                thing: None,
                byte_stream: None,
                visual_data: match data.get::<_, Table<'_>>("visual_data") {
                    Ok(visual_data) => Some(VisualDeserData {
                        _force_deser: None,
                        identifier: match visual_data.get("identifier") {
                            Ok(ident) => Some(ident),
                            Err(_) => None,
                        },
                        name: match visual_data.get("name") {
                            Ok(name) => Some(name),
                            Err(_) => None,
                        },
                        character_left: match visual_data.get("character_left") {
                            Ok(cl) => Some(cl),
                            Err(_) => None,
                        },
                        character_right: match visual_data.get("character_right") {
                            Ok(cr) => Some(cr),
                            Err(_) => None,
                        },
                        text_color_left: match visual_data.get::<_, [u8;3]>("text_color_left") {
                            Ok(tl) => Some((tl[0], tl[1], tl[2])),
                            Err(_) => None,
                        },
                        text_color_right: match visual_data.get::<_, [u8;3]>("text_color_right") {
                            Ok(tr) => Some((tr[0], tr[1], tr[2])),
                            Err(_) => None,
                        },
                        back_color_left: match visual_data.get::<_, [u8;3]>("back_color_left") {
                            Ok(bl) => Some((bl[0], bl[1], bl[2])),
                            Err(_) => None,
                        },
                        back_color_right: match visual_data.get::<_, [u8;3]>("back_color_right") {
                            Ok(br) => Some((br[0], br[1], br[2])),
                            Err(_) => None,
                        },
                    }),
                    Err(_) => None,
                },
            };

            data_stack.push((name, mod_deser));
        }

    }
}


fn load_module_data(data_stack: &mut Vec<(String, ModuleDeserialization)>, data: Result<&DirEntry, &std::io::Error>) {
    if dir_entry_is_dir(data) {
        let data = fs::read_dir(data.unwrap().path()).unwrap();
        for data in data {
            load_module_data(data_stack, data.as_ref());
        }

    } else {
        let data = data.unwrap();
        if let Some(extension) = data.path().extension() {
            if extension == "toml" {
                let toml_data = (
                    os_string_to_string(data.file_name()).trim_end_matches(".toml").to_owned(), 
                    toml::from_str(&fs::read_to_string(data.path()).unwrap()).unwrap()
                );
                data_stack.push(toml_data);
            }
        }
        
    }
}

/* TODO - make this ~*/
pub fn load_module_data_from_persistent_mapping(path: &'static str) -> io::Result<()> {

    fs::read_dir("not_yet_implemented_lol")?;

    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModuleDeserialization {
    /// omit to not force deserialization
    _force_deser: Option<bool>,

    // by default the module name
    source: Option<String>,
    priority: Option<u8>,

    tile: Option<TileDeserialData>,
    ground: Option<GroundDeserialData>,
    item: Option<ItemDeserialData>,

    visible_thing: Option<VisibleThingDeserialData>,
    thing: Option<ThingDeserialData>,
    byte_stream: Option<ByteStreamDeserialData>,

    visual_data: Option<VisualDeserData>,
}

// end point data
#[derive(Debug)]
pub struct VisibleThingType {
    type_identifier: String,
    identifier: u16,
    text_display: TextDisplay,
    color_display: ColorDisplay
}

#[derive(Debug)]
pub struct ThingType {
    type_identifier: String,
    identifier: u16,
}

#[derive(Debug)]
pub struct ByteStream {
    identifier: u16,
    bytes: Vec<u8>
}