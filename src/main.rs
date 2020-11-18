//! # Bullets, Walls and Baddies v1  

extern crate sdl2;

mod collision_system;
mod engine;
mod entity;
mod game_logic;
mod geometry;
mod helpers;
mod levels;
mod render;
mod shape;
mod world;

pub fn main() {
    engine::run(99);
}
