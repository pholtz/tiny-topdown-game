use crate::TL_PX;
use crate::HEIGHT_PX;
use crate::WIDTH_PX;

/// Calculate viewport based on player position
/// Viewport is specified as a tuple of pixel top-left coordinates
pub fn calculate_viewport(player_position: (f32, f32)) -> (i32, i32) {
    return (
        player_position.0.floor() as i32 - (WIDTH_PX / 2),
        player_position.1.floor() as i32 - (HEIGHT_PX / 2)
    );
}

/// Given viewport start point, decompose into an vector of tile coordinates
/// left-to-right scrolling matrix of tile top-left coordinates
pub fn generate_viewport_tiles(viewport: (i32, i32)) -> Vec<(i32, i32, i32, i32, i32, i32)> {

    // Fit viewport to next lowest tile divisor
    let mut view_px = viewport.0;
    while view_px % TL_PX != 0 {
        view_px -= 1;
    }

    let mut view_py = viewport.1;
    while view_py % TL_PX != 0 {
        view_py -= 1;
    }

    // To calculate the min, we simply reference the tile-clamped value we calculated above.
    // To calculate the max, we add the number of tiles that fit into the width / height
    // However, we also add 1 because in some cases the lower end fitting causes the view
    // to be set too low, which steals some of the render from the bottom and right sides
    // of the screen. Adding a few extra tiles on both the bottom and right accounts for this.
    let mut viewport_tiles = Vec::new();
    let view_tx = view_px / TL_PX;
    let view_ty = view_py / TL_PX;
    let max_view_tx = view_tx + (WIDTH_PX / TL_PX) + 2;
    let max_view_ty = view_ty + (HEIGHT_PX / TL_PX) + 2;

    let mut screen_px = view_px - viewport.0;
    let mut screen_py = view_py - viewport.1;

    for ty in view_ty..max_view_ty {
        for tx in view_tx..max_view_tx {
            viewport_tiles.push((tx, ty, tx * TL_PX, ty * TL_PX, screen_px, screen_py));
            screen_px += TL_PX;
        }
        screen_px = view_px - viewport.0;
        screen_py += TL_PX;
    }
    viewport_tiles
}
