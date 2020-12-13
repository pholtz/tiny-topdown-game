use std::collections::BTreeMap;

use piston_window::PistonWindow;
use specs::prelude::*;

use crate::HEIGHT_TL;
use crate::TL_PX;
use crate::{load_asset, Position, Renderable, Tile, WIDTH_TL};

// pub use super::*;

#[derive(Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub enum TileType {
    Wall,
    Floor,
    Missing
}

pub fn to_px(tl: i32) {
    tl * TL_PX;
}

/// Creates a basic map with just floor. Doesn't do any objects yet.
pub fn load_basic_map(window: &mut PistonWindow, ecs: &mut World) -> BTreeMap<(i32, i32), TileType> {
    let mut map = BTreeMap::new();

    for x in 0..WIDTH_TL {
        for y in 0..HEIGHT_TL {
            map.insert((x as i32, y as i32), TileType::Floor);
        }
    }
    map
}

/// Transforms an xy coordinate into a packed index.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH_TL as usize) + x as usize
}
