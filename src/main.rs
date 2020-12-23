extern crate find_folder;
extern crate tiled;

use std::{collections::{BTreeMap, HashMap}};
use std::time::{Duration, Instant};
use std::path;
use std::env;

use specs::prelude::*;
use specs_derive::Component;

use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context, ContextBuilder, GameResult, graphics::Rect};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;

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
    /// We create a helper function that will allow us to easily get the inverse
    /// of a `Direction` which we can use later to check if the player should be
    /// able to move the snake in a certain direction.
    pub fn inverse(&self) -> Self {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

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
type Vector2 = na::Vector2<f32>;

struct State {
    ecs: World,
}

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


const UPDATES_PER_SECOND: f32 = 60.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;


fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    // Make a Context.
    let (ctx, event_loop) = &mut ContextBuilder::new("Tiny Topdown Game", "Paul Holtz")
        .window_setup(ggez::conf::WindowSetup::default().title("Tiny Topdown Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WIDTH_PX as f32, HEIGHT_PX as f32))
        .add_resource_path(resource_dir)
        .build()
		.expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let state = &mut GameState::new(ctx);

    // Run!
    event::run(ctx, event_loop, state)
}

struct GameState {
    last_update: Instant,
    ecs: World,
    tilesheet: graphics::Image
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        // Load/create resources such as images here.
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets")
            .unwrap();

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

        let font = graphics::Font::new(ctx, "/FiraSans-Regular.ttf");
        let tileset_image = graphics::Image::new(ctx, "/grass_tileset.png").expect("could not load image");

        let (map, tilesheet) = load_basic_map_tmx();
        world.insert(map);
        world.insert(tilesheet);
        // world.insert(tileset_image);

        GameState {
            last_update: Instant::now(),
            ecs: world,
            tilesheet: tileset_image,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Ensure that at least a full tick has elapsed before updating
        if Instant::now() - self.last_update < Duration::from_millis(MILLIS_PER_UPDATE) {
            return Ok(());
        }


        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // First we clear the screen to a nice (well, maybe pretty glaring ;)) green
        graphics::clear(ctx, [0.0, 1.0, 0.0, 1.0].into());
        
        // Then we tell the snake and the food to draw themselves
        // self.snake.draw(ctx)?;
        // self.food.draw(ctx)?;

        let player = self.ecs.read_storage::<Player>();
        let positions = self.ecs.read_storage::<Position>();
        let (_player, pos) = (&player, &positions).join().next().expect("No player and position entities found");

        let viewport = calculate_viewport((pos.x, pos.y));

        let viewport_tiles = generate_viewport_tiles(viewport);

        // Render viewport tiles in place on screen
        // Render each tile using the given pixel positions
        let map = self.ecs.fetch::<BTreeMap<(i32, i32), MapTile>>();
        let tilesheet = self.ecs.fetch::<TileSheet>();
        // let tilesheet_image = self.ecs.fetch::<graphics::Image>();

        for (tile_x, tile_y, _view_x, _view_y, screen_x, screen_y) in viewport_tiles.iter() {

            // let tile_transform = c.transform.trans(*screen_x as f64, *screen_y as f64);

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
            let tile_index = map_tile.tile_id - tilesheet.first_tile_id;
            let horizontal_index = tile_index % 10;
            let vertical_index = tile_index / 10;

            // Presumably, the margin from tiled means the space between the borders and the edge tiles
            // As such, we will always add margin, but width and spacing will be dependent on our index
            let horizontal_offset = tilesheet.margin + (horizontal_index * tilesheet.tile_width) + (horizontal_index * tilesheet.spacing);

            // Vertical offset is a bit trickier, since tiles are only stacked once they exceed horizontal space
            // To calculate this elegantly we can modulo using the number of tiles per row
            let vertical_offset = tilesheet.margin + (vertical_index * tilesheet.tile_height) + (vertical_index * tilesheet.spacing);

            let tile_rectangle = [
                horizontal_offset as f32,
                vertical_offset as f32,
                tilesheet.tile_width as f32,
                tilesheet.tile_height as f32
            ];

            // Image::new().src_rect(tile_rectangle).draw(&*tilesheet_image, &c.draw_state, tile_transform, g);
            let drawparams = graphics::DrawParam::new()
                // .src(Rect::new(tile_rectangle[0], tile_rectangle[1], tile_rectangle[2], tile_rectangle[3]))
                .src(Rect::new(2 as f32, 2 as f32, 34 as f32, 34 as f32))
                .dest(Point2::new(*screen_x as f32, *screen_y as f32))
                .offset(Point2::new(0.5, 0.5));
            // graphics::draw(ctx, &self.tilesheet, drawparams)?;
        }

        // TODO: .src() rectangles represent a fraction of the provided image
        // We can use this to render individual sprites from a spritesheet,
        // similar to piston's .src_rect(). However, ggez expects them to be
        // specified as fractions, not pixels, so we should adjust the tilesets
        // going into tiled to stop using margin and spacing, and have easily
        // divisible numbers of rows and columns (say, 10x10).
        graphics::draw(ctx, &self.tilesheet, graphics::DrawParam::new()
            .src(Rect::new(0 as f32, 0 as f32, 0.1 as f32, 0.1333 as f32))
            .dest(Point2::new(0 as f32, 0 as f32)))?;

        // Render player in place on screen
        // This is easy now since we will always just render the player in the middle of the screen
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let players = self.ecs.read_storage::<Player>();
        let textures_by_player_direction = self.ecs.fetch::<HashMap<Direction, graphics::Image>>();
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

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        // Here we attempt to convert the Keycode into a Direction using the helper
        // we defined earlier.
        if let Some(dir) = Direction::from_keycode(keycode) {
            try_move_player(dir, &self.ecs);
        }

        match keycode {
            KeyCode::Escape => event::quit(ctx),
            _ => (), // Do nothing
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






/*
fn main2() {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow = WindowSettings::new("Tiny Topdown Game", [WIDTH_PX as u32, HEIGHT_PX as u32])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    println!("{:?}", assets);
    let mut glyphs = window
        .load_font(assets.join("FiraSans-Regular.ttf"))
        .unwrap();

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs
        .create_entity()
        .with(Position { x: 0, y: 0 })
        .with(Renderable {})
        .with(Player {
            direction: Direction::Down
        })
        .build();

    // let map = load_basic_map(&mut window, &mut gs.ecs);
    let (map, tilesheet) = load_basic_map_tmx();
    gs.ecs.insert(map);
    gs.ecs.insert(tilesheet);
    gs.ecs.insert(load_asset(&mut window, "grass_tileset.png"));

    let mut textures_by_player_direction = HashMap::new();
    textures_by_player_direction.insert(Direction::Down, load_asset(&mut window, "basic_guy.png"));
    textures_by_player_direction.insert(Direction::Up, load_asset(&mut window, "basic_guy_up.png"));
    textures_by_player_direction.insert(Direction::Left, load_asset(&mut window, "basic_guy_left.png"));
    textures_by_player_direction.insert(Direction::Right, load_asset(&mut window, "basic_guy_right.png"));
    gs.ecs.insert(textures_by_player_direction);

    let mut textures_by_tile_type = HashMap::new();
    textures_by_tile_type.insert(TileType::Missing, load_asset(&mut window, "missing.png"));
    textures_by_tile_type.insert(TileType::Wall, load_asset(&mut window, "missing.png"));
    textures_by_tile_type.insert(TileType::Floor, load_asset(&mut window, "basic_grass.png"));
    gs.ecs.insert(textures_by_tile_type);

    window.set_lazy(true);
    window.events.set_ups(1);
    let mut root = RootState::StartMenu;
    while let Some(event) = window.next() {
        match root {
            RootState::StartMenu => {
                start_menu_capture(&event);
                start_menu_draw(&mut window, &mut glyphs, &event);
                root = start_transition(&event);
            }
            RootState::InGame => {
                in_game_capture(&event, &gs.ecs);
                in_game_draw(&mut window, &event, &gs.ecs);
            }
        }
    }
}

fn load_asset(window: &mut PistonWindow, filename: &str) -> G2dTexture {
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    Texture::from_path(
        &mut window.create_texture_context(),
        assets.join(filename),
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap()
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

fn print_player_position(ecs: &World) {
    let positions = ecs.read_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    for (_player, pos) in (&players, &positions).join() {
        println!("Current Player Location -> ({}, {})", pos.x, pos.y);
    }
}

fn in_game_capture(event: &Event, ecs: &World) {
    match event.press_args() {
        Some(Button::Keyboard(Key::W)) => try_move_player(Direction::Up, ecs),
        Some(Button::Keyboard(Key::A)) => try_move_player(Direction::Left, ecs),
        Some(Button::Keyboard(Key::S)) => try_move_player(Direction::Down, ecs),
        Some(Button::Keyboard(Key::D)) => try_move_player(Direction::Right, ecs),
        Some(Button::Keyboard(Key::Return)) => print_player_position(ecs),
        _ => (),
    }
}

fn in_game_draw(window: &mut PistonWindow, event: &Event, ecs: &World) {
    window.draw_2d(event, |c, g, _device| {

        clear([0.8, 0.8, 0.8, 1.0], g);
        g.clear_stencil(0);

        let player = ecs.read_storage::<Player>();
        let positions = ecs.read_storage::<Position>();
        let (_player, pos) = (&player, &positions).join().next().expect("No player and position entities found");

        let viewport = calculate_viewport((pos.x, pos.y));
        // println!("Player position is ({}, {}), viewport calculated as ({}, {})", pos.x, pos.y, viewport.0, viewport.1);

        let viewport_tiles = generate_viewport_tiles(viewport);

        // Render viewport tiles in place on screen
        // Render each tile using the given pixel positions
        let map = ecs.fetch::<BTreeMap<(i32, i32), MapTile>>();
        let tilesheet = ecs.fetch::<TileSheet>();
        let tilesheet_image = ecs.fetch::<G2dTexture>();

        for (tile_x, tile_y, _view_x, _view_y, screen_x, screen_y) in viewport_tiles.iter() {

            let tile_transform = c.transform.trans(*screen_x as f64, *screen_y as f64);

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
            let tile_index = map_tile.tile_id - tilesheet.first_tile_id;
            let horizontal_index = tile_index % 10;
            let vertical_index = tile_index / 10;

            // Presumably, the margin from tiled means the space between the borders and the edge tiles
            // As such, we will always add margin, but width and spacing will be dependent on our index
            let horizontal_offset = tilesheet.margin + (horizontal_index * tilesheet.tile_width) + (horizontal_index * tilesheet.spacing);

            // Vertical offset is a bit trickier, since tiles are only stacked once they exceed horizontal space
            // To calculate this elegantly we can modulo using the number of tiles per row
            let vertical_offset = tilesheet.margin + (vertical_index * tilesheet.tile_height) + (vertical_index * tilesheet.spacing);

            let tile_rectangle = [
                horizontal_offset as f64,
                vertical_offset as f64,
                tilesheet.tile_width as f64,
                tilesheet.tile_height as f64
            ];

            Image::new().src_rect(tile_rectangle).draw(&*tilesheet_image, &c.draw_state, tile_transform, g);
        }

        // Render player in place on screen
        // This is easy now since we will always just render the player in the middle of the screen
        let positions = ecs.read_storage::<Position>();
        let renderables = ecs.read_storage::<Renderable>();
        let players = ecs.read_storage::<Player>();
        let textures_by_player_direction = ecs.fetch::<HashMap<Direction, G2dTexture>>();
        for (_pos, _render, player) in (&positions, &renderables, &players).join() {
            Image::new().draw(textures_by_player_direction.get(&player.direction).expect("could not source player texture"),
                &c.draw_state,
                c.transform.trans((WIDTH_PX / 2) as f64, (HEIGHT_PX / 2) as f64),
                g);
        }
    });
}

fn start_menu_capture(_event: &Event) {
    
}

fn start_menu_draw(window: &mut PistonWindow, glyphs: &mut Glyphs, event: &Event) {
    window.draw_2d(event, |c, g, device| {
        let transform = c.transform.trans(320.0, 280.0);

        clear([0.0, 0.0, 0.0, 1.0], g);
        text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32)
            .draw("Tiny Dropdown Game", glyphs, &c.draw_state, transform, g)
            .unwrap();
        glyphs.factory.encoder.flush(device);
    });
}

fn start_transition(event: &Event) -> RootState {
    match event.press_args() {
        Some(Button::Keyboard(Key::Return)) => {
            println!("User pressed return");
            RootState::InGame
        }
        _ => RootState::StartMenu,
    }
}
*/
