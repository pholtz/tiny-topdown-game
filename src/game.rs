use crate::{GameState, Direction, Point2, Player, Position, Renderable};
use crate::map::{MapTile, TileSheet, TileType};
use crate::viewport;
use crate::{WIDTH_PX, HEIGHT_PX, TL_PX};
use std::{collections::{BTreeMap}};
use ggez::{graphics, Context, GameResult, event, timer, graphics::Rect};
use ggez::event::KeyCode;
use specs::prelude::*;

const DESIRED_FPS: u32 = 60;
const PLAYER_ANIMATION_FRAMES: u8 = 4;
const PLAYER_MOVE_SPEED_TPS: f32 = 1.0;

pub fn in_game_input(state: &mut GameState, ctx: &mut Context, keycode: KeyCode) {
    // Here we attempt to convert the Keycode into a Direction using the helper
    // we defined earlier.
    if let Some(dir) = Direction::from_keycode(keycode) {
        try_move_player(dir, &state.ecs);
    }

    match keycode {
        KeyCode::Escape => event::quit(ctx),
        KeyCode::Key0 => state.show_fps = !state.show_fps,
        _ => (), // Do nothing
    }
}

pub fn in_game_update(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    if timer::check_update_time(ctx, DESIRED_FPS) {
        let _seconds = 1.0 / (DESIRED_FPS as f32);

        let mut positions = state.ecs.write_storage::<Position>();
        let mut players = state.ecs.write_storage::<Player>();
        for (pos, player) in (&mut positions, &mut players).join() {
            // Also, burn down velocity using built-in friction rules (for now)
            // This requires clamping to prevent values from going wild
            player.velocity *= 0.1;
            player.velocity.x = unsigned_zeroing_clamp(player.velocity.x, 0.1, 50.0);
            player.velocity.y = unsigned_zeroing_clamp(player.velocity.y, 0.1, 50.0);

            // Move the player according to their velocity in units per second
            pos.x += player.velocity.x;
            pos.y += player.velocity.y;
        }
    }

    if timer::check_update_time(ctx, 8) {
        let mut positions = state.ecs.write_storage::<Position>();
        let mut players = state.ecs.write_storage::<Player>();
        for (_pos, player) in (&mut positions, &mut players).join() {
            // Alternate between 4 animation frames (0-3)
            if player.velocity.x.abs() > 0.0 || player.velocity.y.abs() > 0.0 {
                player.animation_index = (player.animation_index + 1) % PLAYER_ANIMATION_FRAMES;
            } else {
                player.animation_index = 0;
            }
        }
    }
    Ok(())
}

pub fn in_game_draw(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, [0.6, 0.6, 0.6, 1.0].into());

    let player = state.ecs.read_storage::<Player>();
    let positions = state.ecs.read_storage::<Position>();
    let (_player, pos) = (&player, &positions).join().next().expect("No player and position entities found");

    let viewport = viewport::calculate_viewport((pos.x, pos.y));

    let viewport_tiles = viewport::generate_viewport_tiles(viewport);

    // Render viewport tiles in place on screen
    // Render each tile using the given pixel positions
    let map = state.ecs.fetch::<BTreeMap<(i32, i32), MapTile>>();
    let tilesheet = state.ecs.fetch::<TileSheet>();

    for (tile_x, tile_y, _view_x, _view_y, screen_x, screen_y) in viewport_tiles.iter() {

        // Retrieve the appropriate tile type from the map using the tile coordinates
        let map_tile = match map.get(&(*tile_x, *tile_y)) {
            Some(map_tile) => map_tile,
            None => &MapTile {
                tile_id: 10,
                tile_type: TileType::Missing
            }
        };
        
        // Calculate tilesheet subregion containing the sprite matching the desired gid
        // This is the local index of the desired tile on the tilesheet (0 through tiles - 1)
        // This also accounts for the fact that Tiled indexes start at 1, but we use 0 indexed offsets
        const TILES_PER_ROW: u32 = 10;
        let tile_index = map_tile.tile_id - tilesheet.first_tile_id;
        let horizontal_index = tile_index % TILES_PER_ROW;
        let vertical_index = tile_index / TILES_PER_ROW;

        // Presumably, the margin from tiled means the space between the borders and the edge tiles
        // As such, we will always add margin, but width and spacing will be dependent on our index
        let horizontal_offset = tilesheet.margin + (horizontal_index * tilesheet.tile_width) + (horizontal_index * tilesheet.spacing);

        // Vertical offset is a bit trickier, since tiles are only stacked once they exceed horizontal space
        // To calculate this elegantly we can modulo using the number of tiles per row
        let vertical_offset = tilesheet.margin + (vertical_index * tilesheet.tile_height) + (vertical_index * tilesheet.spacing);

        // .src() rectangles represent a fraction of the provided image
        // We can use this to render individual sprites from a spritesheet,
        // similar to piston's .src_rect(). However, ggez expects them to be
        // specified as fractions, not pixels, so we need to adjust the tilesets
        // going into tiled to stop using margin and spacing, and have easily
        // divisible numbers of rows and columns (say, 10x10).
        let tile_rectangle = [
            horizontal_offset as f32 / (TILES_PER_ROW * tilesheet.tile_width) as f32,
            vertical_offset as f32 / (TILES_PER_ROW * tilesheet.tile_height) as f32,
            tilesheet.tile_width as f32 / (TILES_PER_ROW * tilesheet.tile_width) as f32,
            tilesheet.tile_height as f32 / (TILES_PER_ROW * tilesheet.tile_height) as f32
        ];

        let drawparams = graphics::DrawParam::new()
            .src(Rect::new(tile_rectangle[0], tile_rectangle[1], tile_rectangle[2], tile_rectangle[3]))
            .dest(Point2::new(*screen_x as f32, *screen_y as f32))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx, &state.tilesheet, drawparams)?;
    }

    // Render player in place on screen
    // This is easy now since we will always just render the player in the middle of the screen
    let positions = state.ecs.read_storage::<Position>();
    let renderables = state.ecs.read_storage::<Renderable>();
    let players = state.ecs.read_storage::<Player>();
    for (_pos, _render, player) in (&positions, &renderables, &players).join() {

        // TODO: Create subrectangle referencing the part of the sprite sheet containing the desired sprite to render
        let horizontal_index = player.animation_index;
        let vertical_index = match player.direction {
            Direction::Down => 0,
            Direction::Left => 1,
            Direction::Right => 2,
            Direction::Up => 3
        };
        let desired_sprite_subrectangle = [
            horizontal_index as f32 / 10 as f32,
            vertical_index as f32 / 5 as f32,
            0.1,
            0.2,
        ];
        let drawparams = graphics::DrawParam::new()
            .src(Rect::new(desired_sprite_subrectangle[0], desired_sprite_subrectangle[1], desired_sprite_subrectangle[2], desired_sprite_subrectangle[3]))
            .dest(Point2::new((WIDTH_PX / 2) as f32, (HEIGHT_PX / 2) as f32))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx,
            &state.player_sprite_sheet,
            drawparams)?;
    }

    // Render the fps, if desired
    if state.show_fps {
        render_fps(ctx)?;
    }

    // Finally we call graphics::present to cycle the gpu's framebuffer and display
    // the new frame we just drew and then yield the thread until the next update.
    graphics::present(ctx)?;
    ggez::timer::yield_now();
    Ok(())
}

fn try_move_player(direction: Direction, ecs: &World) {
    let delta = match direction {
        Direction::Up => (0.0, -1.0 * (PLAYER_MOVE_SPEED_TPS * TL_PX as f32)),
        Direction::Left => (-1.0 * (PLAYER_MOVE_SPEED_TPS * TL_PX as f32), 0.0),
        Direction::Down => (0.0, PLAYER_MOVE_SPEED_TPS * TL_PX as f32),
        Direction::Right => (PLAYER_MOVE_SPEED_TPS * TL_PX as f32, 0.0),
    };
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    for (player, _pos) in (&mut players, &mut positions).join() {
        player.direction = direction;
        player.velocity.x += delta.0;
        player.velocity.y += delta.1;
    }
}

fn render_fps(ctx: &mut Context) -> GameResult<()> {
    let fps = timer::fps(ctx);
    let fps_text = graphics::Text::new(format!("FPS: {:.2}", fps));
    graphics::draw(ctx, &fps_text, (Point2::new(0.0, 0.0), graphics::BLACK))?;

    let delta = timer::delta(ctx);
    let delta_text = graphics::Text::new(format!("Delta: {:.2}ms", delta.as_millis()));
    graphics::draw(ctx, &delta_text, (Point2::new(0.0, 12.0), graphics::BLACK))?;

    // let ticks = timer::ticks(ctx);
    // let ticks_text = graphics::Text::new(format!("Ticks: {}", ticks));
    // graphics::draw(ctx, &ticks_text, (Point2::new(0.0, 20.0), graphics::BLACK))?;
    Ok(())
}

/// Prevents the given value from going outside of the range.
/// The range is comprised of the `min` and `max`, and is interpreted as both signed and unsigned.
/// This means that a min of 0.1 and a max of 100.0 would keep the given value in the ranges of
/// -100.0 to -0.1 and 0.1 to 100.0. This is useful when clamping a vector which could be positive
/// or negative.
pub fn unsigned_zeroing_clamp(value: f32, min: f32, max: f32) -> f32 {
    let mut clamped_value = value;
    if value.abs() < min {
        clamped_value = 0.0;
    }
    if value.abs() > max {
        clamped_value = max * value.signum();
    }
    clamped_value
}
