use std::collections::BTreeMap;

use specs::prelude::*;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashMap;

use tiled::parse;
use tiled::Map;
use tiled::LayerData;
use tiled::LayerTile;
use tiled::Chunk;
use tiled::Tileset;

use crate::HEIGHT_TL;
use crate::TL_PX;
use crate::{WIDTH_TL};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub enum TileType {
    Wall,
    Floor,
    Missing
}

pub struct MapTile {
    pub tile_id: u32,
    pub tile_type: TileType
}

pub struct TileSheet {
    pub first_tile_id: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
}

pub fn to_px(tl: i32) -> i32 {
    tl * TL_PX
}

/// Load the map from file using the tiled library.
/// Afterwards, convert both the layer and tileset into our own data format
/// so that we are not using tiled data structures all over the place.
pub fn load_basic_map_tmx() -> (BTreeMap<(i32, i32), MapTile>, TileSheet) {
    let file = File::open(&Path::new("assets/map/basic.tmx")).unwrap();
    let reader = BufReader::new(file);
    let map = parse(reader).unwrap();
    println!("Loaded map with dimensions {}x{} and tile dimensions {}x{}", map.width, map.height, map.tile_width, map.tile_height);
    
    println!("Map has {} layers", map.layers.len());
    for layer in map.layers.iter() {
        println!("Layer {} has some layerdata", layer.name);
    }

    println!("Map has {} tilesets", map.tilesets.len());
    for tileset in map.tilesets.iter() {
        println!("Tileset {} has {} tiles, {:?} tilecount, and {} images", tileset.name, tileset.tiles.len(), tileset.tilecount, tileset.images.len());
    }

    let first_layer = map.layers.first().expect("Map parser can only process exactly one layer");
    let basic_map = match &first_layer.tiles {
        LayerData::Finite(data) => load_basic_map_tmx_finite(&map, &data),
        LayerData::Infinite(data) => load_basic_map_tmx_infinite(&map, &data),
    };

    let first_tileset = map.tilesets.first().expect("Map parser can only process exactly one tileset");
    let first_tilesheet = load_basic_tilesheet(&first_tileset);

    (basic_map, first_tilesheet)
}

/// Transforms a Tileset from tiled into our internal model
/// At this point, they are basically the same, I just don't
/// want to have to import tiled classes all over the place
pub fn load_basic_tilesheet(tileset: &Tileset) -> TileSheet {
    TileSheet {
        first_tile_id: tileset.first_gid,
        tile_width: tileset.tile_width,
        tile_height: tileset.tile_height,
        spacing: tileset.spacing,
        margin: tileset.margin
    }
}

/// Load a finite map from tiled into the internal map structure.
/// Tiles are linked to textures via gids to avoid sharing too many explicit references.
pub fn load_basic_map_tmx_finite(_map: &Map, tiles: &Vec<Vec<LayerTile>>) -> BTreeMap<(i32, i32), MapTile> {
    let mut basic_map = BTreeMap::new();
    let mut x = 0;
    let mut y = 0;
    for row in tiles.iter() {
        for tile in row.iter() {
            basic_map.insert((x, y), MapTile {
                tile_id: tile.gid,
                tile_type: TileType::Floor
            });
            x += 1;
        }
        x = 0;
        y += 1;
    }
    basic_map
}

// TODO: Add support for infinite maps
/// Load an infinite map from tiled into the internal map structure.
pub fn load_basic_map_tmx_infinite(_map: &Map, _tiles: &HashMap<(i32, i32), Chunk>) -> BTreeMap<(i32, i32), MapTile> {
    BTreeMap::new()
}

/// Creates a basic map with just floor. Doesn't do any objects yet.
pub fn load_basic_map(_ecs: &mut World) -> BTreeMap<(i32, i32), MapTile> {
    let mut map = BTreeMap::new();

    for x in (-1 * WIDTH_TL)..WIDTH_TL {
        for y in (-1 * HEIGHT_TL)..HEIGHT_TL {
            map.insert((x, y), MapTile {
                tile_id: 0,
                tile_type: TileType::Floor
            });
        }
    }
    map.insert((10, 10), MapTile { tile_id: 0, tile_type: TileType::Wall });
    map.insert((9, 9), MapTile { tile_id: 0, tile_type: TileType::Wall });
    map.insert((6, 10), MapTile { tile_id: 0, tile_type: TileType::Wall });
    map.insert((7, 5), MapTile { tile_id: 0, tile_type: TileType::Wall });
    map
}

/// Transforms an xy coordinate into a packed index.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH_TL as usize) + x as usize
}
