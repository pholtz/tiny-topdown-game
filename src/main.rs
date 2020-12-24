extern crate tiled;

use std::{collections::{BTreeMap, HashMap}};
use std::path;
use std::env;

use specs::prelude::*;
use specs_derive::Component;

use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context, ContextBuilder, GameResult, graphics::Rect};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use ggez::timer;

mod map;
pub use map::*;

mod viewport;
pub use viewport::*;

enum RootState {
    StartMenu,
    InGame,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode(key: KeyCode) -> Option<Direction> {
        match key {
            KeyCode::Up | KeyCode::W => Some(Direction::Up),
            KeyCode::Down | KeyCode::S => Some(Direction::Down),
            KeyCode::Left | KeyCode::A => Some(Direction::Left),
            KeyCode::Right | KeyCode::D => Some(Direction::Right),
            _ => None,
        }
    }
}

type Point2 = na::Point2<f32>;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
}

#[derive(Component, Debug)]
struct Player {
    direction: Direction
}

pub const WIDTH_PX: i32 = 960;
pub const HEIGHT_PX: i32 = 540;
pub const TL_PX: i32 = 32;
pub const WIDTH_TL: i32 = WIDTH_PX / TL_PX;
pub const HEIGHT_TL: i32 = HEIGHT_PX / TL_PX;

struct GameState {
    root: RootState,
    ecs: World,
    tilesheet: graphics::Image,
    font: graphics::Font
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<Player>();

        world.create_entity()
            .with(Position { x: 0, y: 0 })
            .with(Renderable {})
            .with(Player {
                direction: Direction::Down
            })
            .build();

        let mut textures_by_player_direction = HashMap::new();
        textures_by_player_direction.insert(Direction::Down, graphics::Image::new(ctx, "/basic_guy.png").expect("could not load image"));
        textures_by_player_direction.insert(Direction::Up, graphics::Image::new(ctx, "/basic_guy_up.png").expect("could not load image"));
        textures_by_player_direction.insert(Direction::Left, graphics::Image::new(ctx, "/basic_guy_left.png").expect("could not load image"));
        textures_by_player_direction.insert(Direction::Right, graphics::Image::new(ctx, "/basic_guy_right.png").expect("could not load image"));
        world.insert(textures_by_player_direction);

        let font = graphics::Font::new(ctx, "/FiraSans-Regular.ttf").expect("could not load font");
        let tileset_image = graphics::Image::new(ctx, "/grass_tileset.png").expect("could not load image");

        let (map, tilesheet) = load_basic_map_tmx();
        world.insert(map);
        world.insert(tilesheet);

        GameState {
            root: RootState::StartMenu,
            ecs: world,
            tilesheet: tileset_image,
            font: font,
        }
    }
}

fn start_menu_input(state: &mut GameState, ctx: &mut Context, keycode: KeyCode) {
    match keycode {
        KeyCode::Return => state.root = RootState::InGame,
        KeyCode::Escape => event::quit(ctx),
        _ => (),
    }
}

fn start_menu_update(_state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    const DESIRED_FPS: u32 = 60;
    while timer::check_update_time(ctx, 60) {
        let _seconds = 1.0 / (DESIRED_FPS as f32);
    }
    Ok(())
}

fn start_menu_draw(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
    let menu_pos = Point2::new(380.0, 260.0);
    let menu_title = graphics::Text::new(("Tiny Topdown Game", state.font, 32.0));
    graphics::draw(ctx, &menu_title, (menu_pos, 0.0, graphics::WHITE))?;
    graphics::present(ctx)?;
    ggez::timer::yield_now();
    Ok(())
}

fn in_game_input(state: &mut GameState, ctx: &mut Context, keycode: KeyCode) {
    // Here we attempt to convert the Keycode into a Direction using the helper
    // we defined earlier.
    if let Some(dir) = Direction::from_keycode(keycode) {
        try_move_player(dir, &state.ecs);
    }

    match keycode {
        KeyCode::Escape => event::quit(ctx),
        _ => (), // Do nothing
    }
}

fn in_game_update(_state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    const DESIRED_FPS: u32 = 60;
    while timer::check_update_time(ctx, 60) {
        let _seconds = 1.0 / (DESIRED_FPS as f32);
    }
    Ok(())
}

fn in_game_draw(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, [0.6, 0.6, 0.6, 1.0].into());

    let player = state.ecs.read_storage::<Player>();
    let positions = state.ecs.read_storage::<Position>();
    let (_player, pos) = (&player, &positions).join().next().expect("No player and position entities found");

    let viewport = calculate_viewport((pos.x, pos.y));

    let viewport_tiles = generate_viewport_tiles(viewport);

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
    let textures_by_player_direction = state.ecs.fetch::<HashMap<Direction, graphics::Image>>();
    for (_pos, _render, player) in (&positions, &renderables, &players).join() {
        let drawparams = graphics::DrawParam::new()
            .dest(Point2::new((WIDTH_PX / 2) as f32, (HEIGHT_PX / 2) as f32))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx,
            textures_by_player_direction.get(&player.direction).expect("could not source player texture"),
            drawparams)?;
    }

    // Finally we call graphics::present to cycle the gpu's framebuffer and display
    // the new frame we just drew and then yield the thread until the next update.
    graphics::present(ctx)?;
    ggez::timer::yield_now();
    Ok(())
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.root {
            RootState::StartMenu => start_menu_update(self, ctx),
            RootState::InGame => in_game_update(self, ctx),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.root {
            RootState::StartMenu => start_menu_draw(self, ctx),
            RootState::InGame => in_game_draw(self, ctx),
        }
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        match self.root {
            RootState::StartMenu => start_menu_input(self, ctx, keycode),
            RootState::InGame => in_game_input(self, ctx, keycode),
        }
    }
}

fn try_move_player(direction: Direction, ecs: &World) {
    let delta = match direction {
        Direction::Up => (0, -1 * TL_PX),
        Direction::Left => (-1 * TL_PX, 0),
        Direction::Down => (0, TL_PX),
        Direction::Right => (TL_PX, 0),
    };
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    for (player, pos) in (&mut players, &mut positions).join() {
        player.direction = direction;
        pos.x += delta.0;
        pos.y += delta.1;
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    let (ctx, event_loop) = &mut ContextBuilder::new("Tiny Topdown Game", "Paul Holtz")
        .window_setup(ggez::conf::WindowSetup::default().title("Tiny Topdown Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WIDTH_PX as f32, HEIGHT_PX as f32))
        .add_resource_path(resource_dir)
        .build()
		.expect("could not create ggez context");

    let state = &mut GameState::new(ctx);
    event::run(ctx, event_loop, state)
}
