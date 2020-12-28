use specs::prelude::*;
use specs_derive::Component;
use ggez::nalgebra as na;
use crate::Direction;

type Point2 = na::Point2<f32>;

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Renderable {
}

#[derive(Component, Debug)]
pub struct Player {
    pub direction: Direction,
    pub velocity: Point2,
    pub acceleration: Point2,
    pub animation_index: u8,
}

#[derive(Component, Debug)]
pub struct Viewport {
    pub tiles: Vec<(i32, i32, i32, i32, i32, i32)>,
    pub dirty: bool
}
