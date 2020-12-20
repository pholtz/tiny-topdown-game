use std::collections::BTreeMap;

use piston_window::PistonWindow;
use specs::prelude::*;

use crate::HEIGHT_TL;
use crate::TL_PX;
use crate::{WIDTH_TL};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub enum TileType {
    Wall,
    Floor,
    Missing
}

pub fn to_px(tl: i32) -> i32 {
    tl * TL_PX
}

/// Creates a basic map with just floor. Doesn't do any objects yet.
pub fn load_basic_map(_window: &mut PistonWindow, _ecs: &mut World) -> BTreeMap<(i32, i32), TileType> {
    let mut map = BTreeMap::new();

    for x in (-1 * WIDTH_TL)..WIDTH_TL {
        for y in (-1 * HEIGHT_TL)..HEIGHT_TL {
            map.insert((x, y), TileType::Floor);
        }
    }
    map.insert((10, 10), TileType::Wall);
    map.insert((9, 9), TileType::Wall);
    map.insert((6, 10), TileType::Wall);
    map.insert((7, 5), TileType::Wall);
    map
}

/// Transforms an xy coordinate into a packed index.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH_TL as usize) + x as usize
}
