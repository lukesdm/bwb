use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};

use crate::game_logic::{move_cannon, try_fire, update_world};
use crate::geometry::Direction;
use crate::render;
use crate::world::{ObjectFactory, World};

const MAX_FPS: u32 = 60; // Max FPS. Set this low to observe effects.

pub fn run(mut world: World, obj_factory: &ObjectFactory) {
    let sdl_context = sdl2::init().unwrap();
    let mut canvas = render::init(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut current_time = Instant::now();
    // Previous fire time - set such that the player can take their first shot from the start of the game.
    let mut prev_fire_time = current_time - Duration::from_secs(10);

    //let mut world = world;
    'running: loop {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_millis() as i32;
        current_time = new_time;

        world = update_world(world, frame_time);

        render::render(&mut canvas, &world.0, &world.2);
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    prev_fire_time = try_fire(
                        current_time,
                        prev_fire_time,
                        &mut world,
                        Direction::Left,
                        obj_factory,
                    )
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    prev_fire_time = try_fire(
                        current_time,
                        prev_fire_time,
                        &mut world,
                        Direction::Right,
                        obj_factory,
                    )
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => move_cannon(&mut world, Direction::Up),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => move_cannon(&mut world, Direction::Down),
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        // Cap rendering rate. COULDDO: try and calculate more accurately i.e. account for render-time
        let frame_time = Duration::new(0, 1_000_000_000u32 / MAX_FPS);
        ::std::thread::sleep(frame_time);
    }
}
