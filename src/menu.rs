use crate::GameState;
use crate::RootState;
use crate::Point2;
use ggez::{graphics, Context, GameResult, event, timer};
use ggez::event::KeyCode;

pub fn start_menu_input(state: &mut GameState, ctx: &mut Context, keycode: KeyCode) {
    match keycode {
        KeyCode::Return => state.root = RootState::InGame,
        KeyCode::Escape => event::quit(ctx),
        _ => (),
    }
}

pub fn start_menu_update(_state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    const DESIRED_FPS: u32 = 60;
    while timer::check_update_time(ctx, 60) {
        let _seconds = 1.0 / (DESIRED_FPS as f32);
    }
    Ok(())
}

pub fn start_menu_draw(state: &mut GameState, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
    let menu_pos = Point2::new(380.0, 260.0);
    let menu_title = graphics::Text::new(("Tiny Topdown Game", state.font, 32.0));
    graphics::draw(ctx, &menu_title, (menu_pos, 0.0, graphics::WHITE))?;
    graphics::present(ctx)?;
    ggez::timer::yield_now();
    Ok(())
}
