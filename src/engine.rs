use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};

use crate::game_logic::{move_cannon, try_fire, update_world, LevelState};
use crate::geometry::Direction;
use crate::levels;
use crate::render::Renderer;

const MAX_FPS: u32 = 60; // Max FPS. Set this low to observe effects.

pub fn run(mut curr_level: i32) {
    let (mut world, mut obj_factory) = levels::init(curr_level);
    let sdl_context = sdl2::init().unwrap();
    let mut renderer = Renderer::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut current_time = Instant::now();
    // Previous fire time - set such that the player can take their first shot from the start of the game.
    let mut prev_fire_time = current_time - Duration::from_secs(10);

    'running: loop {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_millis() as i32;
        current_time = new_time;

        let (world_temp, level_state) = update_world(world, frame_time);
        world = world_temp;
        match level_state {
            LevelState::Complete => {
                curr_level += 1;
                let (world_temp, obj_factory_temp) = levels::init(curr_level);
                world = world_temp;
                obj_factory = obj_factory_temp;
            }
            _ => (),
        };

        renderer.render(&world.0, &world.2);
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
                        &obj_factory,
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
                        &obj_factory,
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

        renderer.present();
        // Cap rendering rate. COULDDO: try and calculate more accurately i.e. account for render-time
        let frame_time = Duration::new(0, 1_000_000_000u32 / MAX_FPS);
        ::std::thread::sleep(frame_time);
    }
}
