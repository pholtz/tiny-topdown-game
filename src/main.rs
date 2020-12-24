extern crate tiled;

pub mod map;
pub mod viewport;
pub mod menu;
pub mod game;

use std::{collections::{HashMap}};
use std::path;
use std::env;
use specs::prelude::*;
use specs_derive::Component;
use ggez::event::{KeyCode, KeyMods};
use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::nalgebra as na;

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
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Renderable {
}

#[derive(Component, Debug)]
struct Player {
    direction: Direction,
    animation_index: u8,
}

pub const WIDTH_PX: i32 = 960;
pub const HEIGHT_PX: i32 = 540;
pub const TL_PX: i32 = 32;
pub const WIDTH_TL: i32 = WIDTH_PX / TL_PX;
pub const HEIGHT_TL: i32 = HEIGHT_PX / TL_PX;

pub struct GameState {
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
            .with(Position { x: 0.0, y: 0.0 })
            .with(Renderable {})
            .with(Player {
                direction: Direction::Down,
                animation_index: 0,
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

        let (map, tilesheet) = map::load_basic_map_tmx();
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
