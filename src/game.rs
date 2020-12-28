use crate::{GameState, Direction, Point2, Player, Position, Renderable, Viewport};
use crate::map::{MapTile, TileSheet, TileType};
use crate::{WIDTH_PX, HEIGHT_PX, TL_PX};
use crate::viewport_system::ViewportSystem;
use crate::movement_system::MovementSystem;
use crate::animation_system::AnimationSystem;
use std::{collections::{BTreeMap}};
use ggez::{graphics, Context, GameResult, event, timer, graphics::Rect};
use ggez::event::KeyCode;
use specs::prelude::*;

const DESIRED_FPS: u32 = 60;
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

        let mut viewport_system = ViewportSystem{};
        let mut movement_system = MovementSystem{};
        viewport_system.run_now(&state.ecs);
        movement_system.run_now(&state.ecs);

        // Something about rebalancing the new / old entities, not exactly sure
        state.ecs.maintain();
    }

    if timer::check_update_time(ctx, 8) {
        let mut animation_system = AnimationSystem{};
        animation_system.run_now(&state.ecs);
    }

    Ok(())
}

pub fn in_game_draw(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, [0.6, 0.6, 0.6, 1.0].into());
    render_tiles(ctx, &state)?;
    render_player(ctx, &state)?;
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
    let mut viewports = ecs.write_storage::<Viewport>();
    for (player, _position, viewport) in (&mut players, &mut positions, &mut viewports).join() {
        player.direction = direction;
        player.velocity.x += delta.0;
        player.velocity.y += delta.1;
        viewport.dirty = true;
    }
}

/// Renders any map tiles visible to the players viewport to the screen
/// Adjusts for map location as well as players perspective (centered).
fn render_tiles(ctx: &mut Context, state: &GameState) -> GameResult<()> {
    let player = state.ecs.read_storage::<Player>();
    let positions = state.ecs.read_storage::<Position>();
    let viewports = state.ecs.read_storage::<Viewport>();
    let (_player, _position, viewport) = (&player, &positions, &viewports).join().next().expect("No player, position, and viewport entities found");

    // Render viewport tiles in place on screen
    // Render each tile using the given pixel positions
    let map = state.ecs.fetch::<BTreeMap<(i32, i32), MapTile>>();
    let tilesheet = state.ecs.fetch::<TileSheet>();

    for (tile_x, tile_y, _view_x, _view_y, screen_x, screen_y) in viewport.tiles.iter() {

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
    Ok(())
}

/// Renders the player sprite onto the screen.
/// Supports animation via a rolling animation index and sprite sheet subrectangles.
fn render_player(ctx: &mut Context, state: &GameState) -> GameResult<()> {
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
    Ok(())
}

/// Unobtrusively renders the rolling average frames per second.
fn render_fps(ctx: &mut Context) -> GameResult<()> {
    let fps = timer::fps(ctx);
    let fps_text = graphics::Text::new(format!("FPS: {:.2}", fps));
    graphics::draw(ctx, &fps_text, (Point2::new(0.0, 0.0), graphics::BLACK))?;

    // let delta = timer::delta(ctx);
    // let delta_text = graphics::Text::new(format!("Delta: {:.2}ms", delta.as_millis()));
    // graphics::draw(ctx, &delta_text, (Point2::new(0.0, 12.0), graphics::BLACK))?;

    // let ticks = timer::ticks(ctx);
    // let ticks_text = graphics::Text::new(format!("Ticks: {}", ticks));
    // graphics::draw(ctx, &ticks_text, (Point2::new(0.0, 24.0), graphics::BLACK))?;

    Ok(())
}
