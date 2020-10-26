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

mod entity;
mod shape;

mod geometry;
use geometry::*;

mod world;

mod game_logic;
use crate::entity::EntityKind;
use crate::world::{create_world, Entities, GameObject, ObjectFactory, ObjectGeometries, World};
use game_logic::*;
use std::collections::HashMap;

mod collision_system;

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

fn render(canvas: &mut render::WindowCanvas, entities: &Entities, geometries: &ObjectGeometries) {
    let colors: HashMap<EntityKind, Color> = [
        (EntityKind::Bullet, Color::RGB(74, 143, 255)),
        (EntityKind::Wall, Color::RGB(232, 225, 81)),
        (EntityKind::Baddie, Color::RGB(235, 33, 35)),
        (EntityKind::Cannon, Color::RGB(69, 247, 105)),
    ]
    .iter()
    .cloned()
    .collect();
    for entity in entities {
        render_box(
            canvas,
            geometries.get(&entity.get_id()).unwrap(),
            *colors.get(entity.get_kind()).unwrap(),
        );
    }
}

fn engine_run(mut world: World, obj_factory: &ObjectFactory) {
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

    //let mut world = world;
    'running: loop {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_millis() as i32;
        current_time = new_time;

        world = update_world(world, frame_time);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        render(&mut canvas, &world.0, &world.2);
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

/// Procedurally generates level data.
fn init_level(obj_factory: &ObjectFactory, level_params: &LevelParams) -> World {
    const MAX_SPIN: i32 = 120;
    let base_size = level_params.base_size as i32;
    let sparsity = level_params.sparsity as i32;
    let wall_pc = level_params.wall_pc as i32;
    let baddie_speed = level_params.baddie_speed as i32;

    let mut level_data = Vec::<GameObject>::new();
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let mut next_random = |lower, upper| rng.gen_range(lower, upper + 1);
    level_data.push(obj_factory.make_cannon((GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2)));

    let mut curr_y = 0;
    while curr_y < GRID_HEIGHT {
        let y_inc = base_size as u32;
        curr_y += y_inc;
        let mut curr_x = 0;
        while curr_x < GRID_WIDTH {
            let x_inc = next_random(base_size / 2, base_size * sparsity);
            curr_x += x_inc as u32;
            if next_random(0, 100) < wall_pc {
                level_data.push(obj_factory.make_wall((curr_x as i32, curr_y as i32)));
            } else {
                level_data.push(obj_factory.make_baddie(
                    (curr_x as i32, curr_y as i32),
                    (
                        next_random(-baddie_speed, baddie_speed),
                        next_random(-baddie_speed, baddie_speed),
                    ),
                    next_random(-MAX_SPIN, MAX_SPIN) as f32 / 100.0,
                ));
            }
        }
    }
    create_world(level_data)
}

/// Hardcoded alternative first level
fn init_level0(obj_factory: &ObjectFactory) -> World {
    let level_data: Vec<GameObject> = vec![
        obj_factory.make_cannon((GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2)),
        obj_factory.make_wall((2500, 2500)),
        obj_factory.make_wall((7500, 2500)),
        obj_factory.make_wall((7500, 7500)),
        obj_factory.make_wall((2500, 7500)),
        obj_factory.make_baddie((1000, 1000), (100, 200), 0.5),
        obj_factory.make_baddie((4000, 2000), (-200, 100), 0.5),
        obj_factory.make_baddie((6000, 500), (200, 75), 0.5),
        obj_factory.make_baddie((2000, 6000), (100, -200), 0.5),
        obj_factory.make_baddie((1500, 9000), (200, 0), 0.5),
        obj_factory.make_baddie((6500, 7500), (50, -200), 0.5),
    ];

    create_world(level_data)
}

struct LevelParams {
    /// Base size for the level's objects. 1000 is a good amount
    base_size: u32,
    /// Sparsity of generated objects. Valid range from 1 (most dense) to 10 (least dense)  
    sparsity: u32,
    /// % of generated entities that are walls (the rest will be baddies).  
    wall_pc: u32,

    /// Max baddie speed, in units per second. 1000 is a good amount.
    baddie_speed: u32,
}

pub fn main() {
    let level1_params = LevelParams {
        base_size: 1000,
        sparsity: 10,
        wall_pc: 25,
        baddie_speed: 600,
    };

    let level99_params = LevelParams {
        base_size: 100,
        sparsity: 5,
        wall_pc: 20,
        baddie_speed: 600,
    };

    let levelxxx_params = LevelParams {
        base_size: 20,
        sparsity: 5,
        wall_pc: 20,
        baddie_speed: 600,
    };
    // TODO: Parameterize
    let level = 1;

    let level_params = match level {
        1 => level1_params,
        99 => level99_params,
        -1 => levelxxx_params,
        _ => level1_params,
    };
    let obj_factory = ObjectFactory::new(level_params.base_size);
    let world = match level {
        0 => init_level0(&obj_factory),
        _ => init_level(&obj_factory, &level_params),
    };
    engine_run(world, &obj_factory);
}
