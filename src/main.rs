//! # Bullets, Walls and Baddies v1  
extern crate sdl2;
extern crate rayon;
extern crate itertools;

mod collision_system;
mod engine;
mod entity;
mod game_logic;
mod geometry;
mod helpers;
mod levels;
mod render;
mod shape;
mod text;
mod world;

pub fn main() {
    // single threaded for debugging
    //rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
    engine::run(3);
}
