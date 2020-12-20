use crate::TL_PX;
use crate::HEIGHT_PX;
use crate::WIDTH_PX;

/// Calculate viewport based on player position
/// Viewport is specified as a tuple of pixel top-left coordinates
pub fn calculate_viewport(player_position: (i32, i32)) -> (i32, i32) {
    // println!("Viewport is -> ({}, {})", viewport.0, viewport.1);
    return (player_position.0 - (WIDTH_PX / 2), player_position.1 - (HEIGHT_PX / 2));
}

/// Given viewport start point, decompose into an vector of tile coordinates
/// left-to-right scrolling matrix of tile top-left coordinates
pub fn generate_viewport_tiles(viewport: (i32, i32)) -> Vec<(i32, i32, i32, i32)> {
    let mut viewport_tiles = Vec::new();
    let mut map_tile_x = viewport.0 / TL_PX;
    let mut map_tile_y = viewport.1 / TL_PX;
    for map_pos_y in viewport.1..viewport.1 + HEIGHT_PX {
        if map_pos_y % TL_PX != 0 {
            continue;
        }
        for map_pos_x in viewport.0..viewport.0 + WIDTH_PX {
            if map_pos_x % TL_PX != 0 {
                continue;
            }
            viewport_tiles.push((map_tile_x, map_tile_y, map_pos_x, map_pos_y));
            map_tile_x += 1;
        }
        // Reset the x tile counter in preparation for the next row
        map_tile_x = viewport.0 / TL_PX;
        map_tile_y += 1;
    }
    // println!("Created a viewport matrix with length {}, first {:?}, last {:?}",
    //     viewport_tiles.len(),
    //     viewport_tiles[0],
    //     viewport_tiles.last());
    viewport_tiles
}
