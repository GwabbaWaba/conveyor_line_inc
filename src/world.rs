use mlua::{Lua, UserData};

pub struct SaveData {
    world_name: String,
    path: String,
}


pub struct World {
    chunks: Vec<Chunk>
}
impl World {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new()
        }
    }
}
impl UserData for World {
    
}

pub struct Chunk {
    tiles: Vec<Tile>
}
impl UserData for Chunk {
    
}

// f64 due to lua interop
// likely gonna optimize down to u16 and just convert it all the time
// that's the reason for the alias
pub type TileId = u16;

pub struct Tile {
    tile_id: TileId,
}
impl Tile {
    pub fn get_id(&self) -> TileId {
        self.tile_id
    }
}
impl UserData for Tile {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_id", |_lua, this: &Tile, ()| {
            Ok(this.get_id())
        });

        methods.add_method_mut("tick", |_lua, _this: &mut Tile, ()| {
            
            Ok(())
        });
    }
}
