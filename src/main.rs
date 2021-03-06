extern crate tiled;

pub mod map;
pub mod menu;
pub mod game;
pub mod component;
pub mod viewport_system;
pub mod movement_system;
pub mod animation_system;

use component::*;
use std::path;
use std::env;
use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;
use specs::prelude::*;

type Point2 = na::Point2<f32>;

pub const WIDTH_PX: i32 = 960;
pub const HEIGHT_PX: i32 = 540;
pub const TL_PX: i32 = 32;
pub const WIDTH_TL: i32 = WIDTH_PX / TL_PX;
pub const HEIGHT_TL: i32 = HEIGHT_PX / TL_PX;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
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

pub enum RootState {
    StartMenu,
    InGame,
}

pub struct GameState {
    root: RootState,
    ecs: World,
    tilesheet: graphics::Image,
    player_sprite_sheet: graphics::Image,
    font: graphics::Font,
    show_fps: bool,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<Player>();
        world.register::<Viewport>();

        world.create_entity()
            .with(Position { x: 0.0, y: 0.0 })
            .with(Renderable {})
            .with(Player {
                direction: Direction::Down,
                velocity: Point2::new(0.0, 0.0),
                acceleration: Point2::new(0.0, 0.0),
                animation_index: 0,
            })
            .with(Viewport {
                tiles: vec![],
                dirty: true,
            })
            .build();

        let player_sprite_sheet_image = graphics::Image::new(ctx, "/basic_guy/basic_guy_sheet.png").expect("could not load image");
        let font = graphics::Font::new(ctx, "/FiraSans-Regular.ttf").expect("could not load font");
        let tileset_image = graphics::Image::new(ctx, "/grass_tileset.png").expect("could not load image");

        let (map, tilesheet) = map::load_basic_map_tmx();
        world.insert(map);
        world.insert(tilesheet);

        GameState {
            root: RootState::StartMenu,
            ecs: world,
            tilesheet: tileset_image,
            player_sprite_sheet: player_sprite_sheet_image,
            font: font,
            show_fps: true,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.root {
            RootState::StartMenu => menu::start_menu_update(self, ctx),
            RootState::InGame => game::in_game_update(self, ctx),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.root {
            RootState::StartMenu => menu::start_menu_draw(self, ctx),
            RootState::InGame => game::in_game_draw(self, ctx),
        }
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        match self.root {
            RootState::StartMenu => menu::start_menu_input(self, ctx, keycode),
            RootState::InGame => game::in_game_input(self, ctx, keycode),
        }
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
