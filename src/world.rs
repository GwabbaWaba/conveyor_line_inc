use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use mlua::UserData;

/// general data about the current save
pub struct SaveData {
    world_name: String,
    path: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Position {
    x: i32,
    y: i16,
    z: i32
}
impl Position {
    pub fn new(x: i32, y: i16, z: i32) -> Self {
        Self {x, y, z}
    }
}

/// the current session world, for chunks actively in load distance
pub struct World {
    chunks: HashMap<Position, Chunk>
}
impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }

    pub fn get_chunk(& self, chunk_pos: &Position) -> Option<&Chunk> {
        self.chunks.get(chunk_pos)
    }

    pub fn set_chunk(&mut self, chunk_pos: &Position, chunk: Chunk) {
        self.chunks.insert(chunk_pos.clone(), chunk);
    }

    pub fn remove_chunk(&mut self, chunk_pos: &Position) -> Option<Chunk> {
        self.chunks.remove(chunk_pos)
    }

    pub fn get_tile(&self, pos: &Position) -> Option<Tile> {
        let chunk_pos = Position::new(
            pos.x / 32,
            pos.y / 32,
            pos.z / 32
        );
        self.chunks.get(&chunk_pos)?.get_tile(pos)
    }

    pub fn set_tile(&mut self, pos: &Position, tile: Tile) -> Option<Tile> {
        let chunk_pos = Position::new(
            pos.x / 32,
            pos.y / 32,
            pos.z / 32
        );
        self.chunks.get_mut(&chunk_pos)?.set_tile(pos, tile)
    }
}
impl UserData for World {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_tile", |_lua, this, (x, y, z): (i32, i16, i32)| {
            let tile = this.get_tile(&Position::new(x, y, z));
            Ok(tile)
        });

        methods.add_method_mut("set_tile", |_lua, this, (x, y, z, tile): (i32, i16, i32, Tile)| {
            let tile = this.set_tile(&Position::new(x, y, z), tile);
            Ok(tile)
        });

        methods.add_method("get_chunk", |_lua, this, (x, y, z): (i32, i16, i32)| {
            let chunk = this.get_chunk(&Position::new(x, y, z));
            Ok(match chunk {
                Some(chunk) => Some(chunk.tiles),
                _ => None
            })
        });

        methods.add_method_mut("set_chunk", |_lua, this, (x, y, z, chunk): (i32, i16, i32, [[[Tile; 32]; 32]; 32])| {
            let chunk = this.set_chunk(&Position::new(x, y, z), Chunk::new(chunk));
            Ok(chunk)
        });
    }
    
}

/// holds chunk data which reflects what will be saved to disk
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    /// [[[z]y]x]
    tiles: [[[Tile; 32]; 32]; 32]
}
impl Chunk {
    pub fn new(tiles: [[[Tile; 32]; 32]; 32]) -> Self {
        Self {tiles}
    }

    pub fn get_tile(&self, pos: &Position) -> Option<Tile> {
        self.tiles
            .get(pos.x.rem_euclid(32)as usize)?
            .get(pos.y.rem_euclid(32)as usize)?
            .get(pos.z.rem_euclid(32)as usize)
            .copied()
    }

    pub fn set_tile(&mut self, pos: &Position, tile: Tile) -> Option<Tile> {
        let current = self.tiles
            .get_mut(pos.x.rem_euclid(32)as usize)?
            .get_mut(pos.y.rem_euclid(32)as usize)?
            .get_mut(pos.z.rem_euclid(32)as usize)?;
        let current_copy = *current;
        *current = tile;
        Some(current_copy)
    }
}

type Tile = u16;
