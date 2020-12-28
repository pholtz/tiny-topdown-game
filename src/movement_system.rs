use specs::prelude::*;
use crate::component::*;

pub struct MovementSystem {}

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Player>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut position, mut player) = data;

        for (_entity, position, player) in (&entities, &mut position, &mut player).join() {
            // Burn down velocity using built-in friction rules (for now)
            // This requires clamping to prevent values from going wild
            player.velocity *= 0.1;
            player.velocity.x = unsigned_zeroing_clamp(player.velocity.x, 0.1, 50.0);
            player.velocity.y = unsigned_zeroing_clamp(player.velocity.y, 0.1, 50.0);

            // Move the player according to their velocity in units per second
            position.x += player.velocity.x;
            position.y += player.velocity.y;
        }
    }
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
