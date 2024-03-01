
use crate::{tile::Tile, Ground, WeightedRandom, MAP_HEIGHT, MAP_LENGTH};

pub static mut TILE_MAP: Option<Vec<Vec<Tile>>> = None;
/// safe unsafe action lolz
/// ONLY CALL IF TILE_MAP IS SOME(_)
pub fn tile_map() -> &'static mut Vec<Vec<Tile>> { 
    unsafe { 
        if let Some(tile_map) = TILE_MAP.as_mut() {
            tile_map
        } else {
            panic!("call for TILE_MAP while TILE_MAP is None(_)")
        }
    } 
}

pub static mut GROUND_MAP: Option<Vec<Vec<Ground>>> = None;
/// safe unsafe action lolz
/// ONLY CALL IF GROUND_MAP IS SOME(_)
pub fn ground_map() -> &'static mut Vec<Vec<Ground>> { 
    unsafe { 
        if let Some(ground_map) = GROUND_MAP.as_mut() {
            ground_map
        } else {
            panic!("call for GROUND_MAP while GROUND_MAP is None(_)")
        }
    } 
}

/// Generates a random 2d vec of Y from f(T)
pub fn gen_map<T: Clone, Y>(mut weighted_random: WeightedRandom<T>, f: fn(T) -> Y) -> Vec<Vec<Y>> {
    let mut map: Vec<Vec<Y>> = Vec::new();

    for column in 0..MAP_HEIGHT {
        map.push(Vec::new() as Vec<Y>);
        for _ in 0..MAP_LENGTH {
            map[column].push( match weighted_random.get_rand() {
                Some(t) => f(t),
                None => unreachable!(),
            });
        }
    }
    map
}