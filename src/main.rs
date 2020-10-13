//! # Bullets, Walls and Baddies v1  
//! Rules:  
//! * Bullet meets Enemy => Both destroyed, and a new bullet is fired  
//! * Bullet meets Wall => Bullet destroyed, and a new bullet is fired  
//! * Enemy meets Wall => Enemy bounces/reverses  
//! (Baddies can't win here)

extern crate sdl2;

use rand::{Rng, SeedableRng, StdRng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render;
use std::time::{Duration, Instant};

mod helpers;

mod geometry;
use geometry::*;

mod game_logic;
use game_logic::*;

// Screen coordinate bounds
const WIN_WIDTH: u32 = 600;
const WIN_HEIGHT: u32 = 600;

const MAX_FPS: u32 = 60; // Max FPS. Set this low to observe effects.

fn world_to_screen(coords: &(i32, i32)) -> (i32, i32) {
    let sf_x = WIN_WIDTH as f32 / GRID_WIDTH as f32;
    let sf_y = WIN_HEIGHT as f32 / GRID_HEIGHT as f32;

    // Assume common origin, so just need to multiply
    let (wx, wy) = *coords;
    let sx = wx as f32 * sf_x;
    let sy = wy as f32 * sf_y;
    (sx as i32, sy as i32)
}

fn render_box(canvas: &mut render::WindowCanvas, box_geometry: &[Vertex], color: Color) {
    // COULDDO: Way to avoid reallocating here? (E.g. re-use existing render vec)
    let points: Vec<Point> = box_geometry
        .iter()
        .map(|p| world_to_screen(p))
        .map(|p| Point::new(p.0, p.1))
        .collect();

    canvas.set_draw_color(color);

    canvas.draw_lines(&points[..]).unwrap();
}

fn render(canvas: &mut render::WindowCanvas, world: &World) {
    for bullet in &world.bullets {
        render_box(canvas, &bullet.0.geometry, Color::RGB(74, 143, 255));
    }

    for wall in &world.walls {
        render_box(canvas, &wall.0.geometry, Color::RGB(232, 225, 81));
    }

    for baddie in &world.baddies {
        render_box(canvas, &baddie.0.geometry, Color::RGB(235, 33, 35));
    }

    render_box(canvas, &world.cannon.0.geometry, Color::RGB(69, 247, 105));
}

fn engine_run(world: &mut World) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Bullets, Walls and Baddies", WIN_WIDTH, WIN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut current_time = Instant::now();
    // Previous fire time - set such that the player can take their first shot from the start of the game.
    let mut prev_fire_time = current_time - Duration::from_secs(10);

    'running: loop {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_millis() as i32;
        current_time = new_time;

        update_world(world, frame_time);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        render(&mut canvas, world);
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    prev_fire_time = try_fire(
                        current_time,
                        prev_fire_time,
                        &world.cannon,
                        &mut world.bullets,
                        Direction::Left,
                    )
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    prev_fire_time = try_fire(
                        current_time,
                        prev_fire_time,
                        &world.cannon,
                        &mut world.bullets,
                        Direction::Right,
                    )
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => world.cannon.moove(Direction::Up),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => world.cannon.moove(Direction::Down),
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

fn init_level() -> World {
    const WALL_RATIO: u32 = 20; // % of generated entities that are walls.
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let mut next_random = |lower, upper| rng.gen_range(lower, upper + 1);
    let cannon = Cannon::new((GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2));

    let mut walls: Vec<Wall> = vec![];
    let mut baddies: Vec<Baddie> = vec![];
    let mut curr_y = 0;
    while curr_y < GRID_HEIGHT {
        let y_inc = 1000; // TODO: Parameterize
        curr_y += y_inc;
        let mut curr_x = 0;
        while curr_x < GRID_WIDTH {
            let x_inc = next_random(500, 1500); // TODO: Parameterize
            curr_x += x_inc as u32;
            if next_random(0, 100) < WALL_RATIO as i32 {
                walls.push(Wall::new((curr_x as i32, curr_y as i32)));
            } else {
                baddies.push(Baddie::new(
                    (curr_x as i32, curr_y as i32),
                    (next_random(-100, 100), next_random(-100, 100)),
                    next_random(-100, 100) as f32 / 100.0,
                ));
            }
        }
    }
    World::new(cannon, baddies, walls)
}

/// Hardcoded first level - TODO: add back in once level system implemented.
fn init_level0() -> World {
    let cannon = Cannon::new((GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2));
    let walls = vec![
        Wall::new((2500, 2500)),
        Wall::new((7500, 2500)),
        Wall::new((7500, 7500)),
        Wall::new((2500, 7500)),
    ];
    let baddies = vec![
        Baddie::new((1000, 1000), (100, 200), 0.5),
        Baddie::new((4000, 2000), (-200, 100), 0.5),
        Baddie::new((6000, 500), (200, 75), 0.5),
        Baddie::new((2000, 6000), (100, -200), 0.5),
        Baddie::new((1500, 9000), (200, 0), 0.5),
        Baddie::new((6500, 7500), (50, -200), 0.5),
    ];
    World::new(cannon, baddies, walls)
}

pub fn main() {
    let mut world = init_level();
    engine_run(&mut world);
}
