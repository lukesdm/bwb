use crate::world::{create_world, GameObject, ObjectFactory, World, GRID_HEIGHT, GRID_WIDTH};
use rand::{Rng, SeedableRng, StdRng};
use std::collections::HashMap;

struct LevelParams {
    /// Base size for the level's objects. 1000 is a good amount
    base_size: u32,
    /// Sparsity of generated objects. Valid range from 1 (most dense) to 10 (least dense)  
    sparsity: u32,
    /// % of generated entities that are walls (the rest will be baddies).  
    wall_pc: u32,

    /// Max baddie speed, in units per second. 1000 is a good amount.
    baddie_speed: u32,

    /// Whether this is a test level (see usages for what effects this has)
    test: bool
}

/// Procedurally generates level data.
fn build_level(obj_factory: &ObjectFactory, level_params: &LevelParams) -> World {
    const MAX_SPIN: i32 = 120;
    let base_size = level_params.base_size as i32;
    let sparsity = level_params.sparsity as i32;
    let wall_pc = level_params.wall_pc as i32;
    let baddie_speed = level_params.baddie_speed as i32;

    let mut level_data = Vec::<GameObject>::new();
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    let mut next_random = |lower, upper| rng.gen_range(lower, upper + 1);
    let mut cannon = obj_factory.make_cannon((GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2));
    if level_params.test {
        cannon.3 = Some(1000);
    }
    level_data.push(cannon);

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
fn build_level0(obj_factory: &ObjectFactory) -> World {
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

pub fn init(level: i32) -> (World, ObjectFactory) {
    let level_params: HashMap<i32, LevelParams> = vec![
        (
            1,
            LevelParams {
                base_size: 1500,
                sparsity: 25,
                wall_pc: 90,
                baddie_speed: 600,
                test: false
            },
        ),
        (
            2,
            LevelParams {
                base_size: 1500,
                sparsity: 20,
                wall_pc: 80,
                baddie_speed: 600,
                test: false,
            },
        ),
        (
            3,
            LevelParams {
                base_size: 1200,
                sparsity: 20,
                wall_pc: 80,
                baddie_speed: 600,
                test: false,
            },
        ),
        (
            4,
            LevelParams {
                base_size: 800,
                sparsity: 8,
                wall_pc: 25,
                baddie_speed: 600,
                test: false,
            },
        ),
        (
            99,
            LevelParams {
                base_size: 100,
                sparsity: 5,
                wall_pc: 20,
                baddie_speed: 600,
                test: true,
            },
        ),
        (
            -1,
            LevelParams {
                base_size: 20,
                sparsity: 5,
                wall_pc: 20,
                baddie_speed: 600,
                test: true,
            },
        ),
    ]
    .into_iter()
    .collect();

    let default_params = level_params.get(&1).unwrap();
    let level_params = level_params.get(&level).unwrap_or(default_params);
    let obj_factory = ObjectFactory::new(level_params.base_size);
    let world = match level {
        0 => build_level0(&obj_factory),
        _ => build_level(&obj_factory, &level_params),
    };
    (world, obj_factory)
}
