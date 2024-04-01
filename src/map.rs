
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{exit_successful, tile::Tile, Ground, WeightedRandom};

pub static mut MAP_WIDTH: Option<usize> = None;
pub static mut MAP_HEIGHT: Option<usize> = None;
pub fn map_width() -> usize {
    unsafe { MAP_WIDTH.unwrap() }
}
pub fn map_height() -> usize {
    unsafe { MAP_HEIGHT.unwrap() }
}

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

    for column in 0..map_height() {
        if poll(Duration::from_secs(0)).unwrap() {
            // handle input
            if let Ok(Event::Key(event)) = read() {
                if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
                    exit_successful().expect("exited, but not successfully");
                }
            }
        }
        
        map.push(Vec::new() as Vec<Y>);
        for _ in 0..map_width() {

            map[column].push( match weighted_random.get_rand() {
                Some(t) => f(t),
                None => unreachable!(),
            });
        }
    }
    map
}