extern crate find_folder;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate tiled;

use std::{collections::{BTreeMap, HashMap}};

use piston::window::WindowSettings;
use piston_window::*;
use specs::prelude::*;
use specs_derive::Component;

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

fn main() {
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
