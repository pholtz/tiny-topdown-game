use specs::prelude::*;
use crate::component::*;

const PLAYER_ANIMATION_FRAMES: u8 = 4;

pub struct AnimationSystem {}

impl<'a> System<'a> for AnimationSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Player>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (positions, mut players) = data;

        for (_positions, player) in (&positions, &mut players).join() {
            // Alternate between 4 animation frames (0-3)
            if player.velocity.x.abs() > 0.0 || player.velocity.y.abs() > 0.0 {
                player.animation_index = (player.animation_index + 1) % PLAYER_ANIMATION_FRAMES;
            } else {
                player.animation_index = 0;
            }
        }
    }
}
