//! # Game logic
//! Primary rules:
//! * Bullet meets Enemy => Both destroyed
//! * Bullet meets Wall => Bullet destroyed
//! * Enemy meets Wall => Enemy bounces/reverses

//! Other rules:
//! * Bullets are destroyed when they reach edge of screen
//! * Enemies wrap to the other side of the screen

// TODO: Refactor - some stuff should live elsewhere
use crate::geometry::{is_collision, rotate, scale, Geometry, Vector, Vertex, P, Direction, direction_vector};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use crate::shape::Shape;
use crate::entity::{EntityKind, Entity};
use crate::world::{World, ObjectGeometries, make_bullet};

// World coordinate bounds
pub const GRID_WIDTH: u32 = 10000;
pub const GRID_HEIGHT: u32 = 10000;



/// Updates box geometry according to its state
fn update_geometry(box_geometry: &mut [Vertex], box_state: &Shape) {
    let (cx, cy) = box_state.get_center();
    let delta = (box_state.get_size() / 2) as i32;
    let vs = box_geometry;
    vs[0] = (cx - delta, cy - delta);
    vs[1] = (cx + delta, cy - delta);
    vs[2] = (cx + delta, cy + delta);
    vs[3] = (cx - delta, cy + delta);

    // Repeat first to close shape - just an implementation detail, could be reworked.
    vs[4] = (cx - delta, cy - delta);

    for v in vs.iter_mut() {
        rotate(v, &box_state.get_center(), *box_state.get_rotation())
    }
}

fn get_cannon(world: &World) -> &Entity {
    world.get_entities()
        .iter()
        .find(|e| e.get_kind() == EntityKind::Cannon)
        .unwrap()
}

fn get_cannon_pos(world: &World) -> &P {
    let cannon_id = get_cannon(world).get_id();
    world.get_shape(cannon_id)
        .get_center()
}

// (ACTION)
/// Try fire the cannon, throttled to the rate of fire.
/// Returns the instant of when the cannon was previously fired successfully.  
/// Note: Rate of fire is hardcoded within.  
pub fn try_fire(
    now: Instant,
    prev: Instant,
    world: &mut World,
    direction: Direction,
) -> Instant {
    // 1 / rate of fire
    const RELOAD_TIME: Duration = Duration::from_millis(1000);

    let cannon_pos = get_cannon_pos(world);

    if now > prev + RELOAD_TIME {
        world.add(make_bullet(*cannon_pos, direction_vector(direction)));
        //fire(cannon, bullets, direction);
        return now;
    }
    return prev;
}

// (ACTION)
pub fn move_cannon(world: &mut World, direction: Direction) {
    let cannon_id = get_cannon(world).get_id();
    world.get_shape(cannon_id).set_movement(direction);
}

fn move_with_wrap(start: i32, amt: i32, bound: i32) -> i32 {
    if start + amt < 0 {
        // assume amt is negative
        bound + amt + start
    } else if start + amt < bound {
        start + amt
    } else {
        start + amt - bound
    }
}

/// Calculate new box spatial state
/// `dt`: frame time, in ms
fn update_pos(box_state: &mut Shape, dt: i32, wrap: bool) {
    let (cx, cy) = *box_state.get_center();
    let (vx, vy) = *box_state.get_vel();

    // Watch for rounding errors here - grid has to be suitably large otherwise slow moving objects will misbehave
    let step_x = vx * dt / 1000;
    let step_y = vy * dt / 1000;

    let new_center = if wrap {
        (
            move_with_wrap(cx, step_x, GRID_WIDTH as i32),
            move_with_wrap(cy, step_y, GRID_HEIGHT as i32),
        )
    } else {
        (cx + step_x, cy + step_y)
    };
    box_state.set_center(new_center);

    box_state.rotate(dt as f32 / 1000.0);
}

fn update_geometry_all(shapes: &Shapes, geometries: &mut ObjectGeometries) {
    for (id, shape) in shapes {
        update_geometry(geometries.get_mut(id).unwrap(), shape);
    }
}

// TODO: Reimplement
// fn handle_bullet_hits(bullets: &mut Vec<Bullet>, baddies: &mut Vec<Baddie>) {
//     // On collision, remove the bullet and the baddie, and resume with the other bullets, i.e. don't allow one bullet to destroy multiple baddies.
//     // Use while loop as we want to re-evaluate len() every loop. (for loop evals once - SHOULDDO: check that)
//     let mut i_bullet = 0;
//     while i_bullet < bullets.len() && !baddies.is_empty() {
//         let mut collided = false;
//         for i_baddie in 0..baddies.len() {
//             // can use a for loop here as we break out once the length has changed.
//             if is_collision(&bullets[i_bullet].0.geometry, &baddies[i_baddie].0.geometry) {
//                 baddies.swap_remove(i_baddie);
//                 bullets.swap_remove(i_bullet);
//                 collided = true;
//                 break;
//             }
//         }
//         if !collided {
//             i_bullet += 1;
//         } else {
//             // NOP - Don't increment, as the item at this index has changed, and we want to check it at the next iteration.
//         }
//     }
// }

fn is_inside_world(point: P) -> bool {
    let (x, y) = point;
    (x > 0 && x <= GRID_WIDTH as i32) && (y > 0 && y <= GRID_HEIGHT as i32)
}

/// Handle when bullets miss i.e. reach edge of world without hitting anything - remove them.
fn handle_bullet_misses(world: &mut World) {
//fn handle_bullet_misses(bullets: &mut Vec<Bullet>) {
    let b_iter = world.get_entities().iter();
    let bullets = world.get_entities().iter()
        .filter(|e| e.get_kind() == EntityKind::Bullet);

    let to_remove: Vec<&Entity> = bullets.filter(|b| {
        let shape = world.get_shape(bullet.get_id());
        !is_inside_world(*shape.get_center())
    }).collect();

    for b in to_remove {
        world.remove(b.get_id());
    }
}

// TODO: Reimplement
/// Reverse baddies when they hit a wall
// fn handle_wall_collisions(walls: &Vec<Wall>, baddies: &mut Vec<Baddie>) {
//     for wall in walls {
//         for baddie in baddies.iter_mut() {
//             if is_collision(&wall.0.geometry, &baddie.0.geometry) {
//                 let (vx, vy) = baddie.0.state.vel;
//                 // Reverse the velocity vector. COULDDO: Reflect rather than reverse
//                 baddie.0.state.vel = (-vx, -vy);
//             }
//         }
//     }
// }

// TODO: Reimplement
// fn handle_bullet_wall_collisions(bullets: &mut Vec<Bullet>, walls: &Vec<Wall>) {
//     let mut i_bullet = 0;
//     while i_bullet < bullets.len() {
//         let mut collided = false;
//         for wall in walls {
//             if is_collision(&bullets[i_bullet].0.geometry, &wall.0.geometry) {
//                 bullets.swap_remove(i_bullet);
//                 collided = true;
//                 break;
//             }
//         }
//         if !collided {
//             i_bullet += 1;
//         } else {
//             // NOP - Don't increment, as the item at this index has changed, and we want to check it at the next iteration.
//         }
//     }
// }

pub fn update_world(world: &mut World, dt: i32) {

    for entity in world.get_entities() {
        //let mut shape = world.get_shape_mut(entity.get_id());
        let shape = world.get_shape_mut(entity.get_id());
        match entity.get_kind() {
            EntityKind::Baddie => update_pos(shape, dt, true),
            EntityKind::Cannon => update_pos(shape, dt, true),
            EntityKind::Bullet => update_pos(shape, dt, false),
            EntityKind:: Wall => update_pos(shape, dt, false),
            _ => ()
        }
    }

    // Update geometry ready for collision detection
    let shapes = world.get_shapes();
    let geometries = world.get_geometries();
    for (id, shape) in shapes {
        update_geometry(geometries.get_mut(id).unwrap(), shape);
    }

    // TODO: Reimplment
    //handle_bullet_hits(&mut game_objects.bullets, &mut game_objects.baddies);
    handle_bullet_misses(world);

    // TODO: Reimplment
    //handle_wall_collisions(&game_objects.walls, &mut game_objects.baddies);

    // TODO: Reimplment
    //handle_bullet_wall_collisions(&mut game_objects.bullets, &game_objects.walls);

    // Could add 2nd pass of geometry update to reflect destroyed objects.
    // Has side effect of showing objects inside one another, as positions aren't backed-out after collision.
    // update_geometry_all(game_objects);
}

// TODO: Fix tests
// #[cfg(test)]
// mod tests {
//     use super::{update_world, Baddie, Bullet, Cannon, GameObject, Shape, Wall, World, GRID_WIDTH};
//     #[test]
//     fn bullet_meets_enemy_both_destroyed() {
//         // Arrange
//         // 2 different bullets, 2 different baddies, and 1 of each about to collide
//         let hit_baddie = Baddie(GameObject::new(Shape::new(
//             (5000, 5000),
//             1000,
//             (0, 0),
//             0.0,
//             0.0,
//         )));
//         let missed_baddie = Baddie(GameObject::new(Shape::new(
//             (5000, 7000),
//             1000,
//             (0, 0),
//             0.0,
//             0.0,
//         )));
//
//         let hitting_bullet = Bullet(GameObject::new(Shape::new(
//             (4490, 4500),
//             100,
//             (1000, 0),
//             0.0,
//             0.0,
//         )));
//         let missing_bullet = Bullet(GameObject::new(Shape::new(
//             (4000, 4500),
//             100,
//             (0, 1000),
//             0.0,
//             0.0,
//         )));
//
//         let dt = 20; // simulate 20ms
//
//         let mut world = World {
//             cannon: Cannon::new((0, 0)), // dummy - not used here
//             bullets: vec![hitting_bullet, missing_bullet],
//             baddies: vec![hit_baddie, missed_baddie],
//             walls: vec![],
//         };
//
//         // Act
//         update_world(&mut world, dt);
//
//         // Assert
//         assert_eq!(world.bullets.len(), 1);
//         assert_eq!(world.baddies.len(), 1);
//         // Could add more precise assertion to determine that the expected objects were destroyed but that involves lots of boilerplate.
//     }
//
//     #[test]
//     fn bullet_destroyed_at_screen_edge() {
//         let mut world = World {
//             cannon: Cannon::new((0, 0)), // dummy - not used here
//             bullets: vec![Bullet::new((GRID_WIDTH as i32 - 10, 100), (1, 0))],
//             baddies: vec![],
//             walls: vec![],
//         };
//         let dt = 20;
//
//         assert_eq!(world.bullets.len(), 1);
//
//         update_world(&mut world, dt);
//
//         assert_eq!(world.bullets.len(), 0);
//     }
//
//     #[test]
//     fn baddies_wrap_at_screen_edge_lr() {
//         let mut world = World::new(
//             Cannon::new((0, 0)),
//             vec![Baddie::new((GRID_WIDTH as i32 - 10, 1000), (1000, 0), 0.0)],
//             vec![],
//         );
//         let dt = 20;
//         let new_center_expected = (10, 1000);
//
//         update_world(&mut world, dt);
//
//         let new_center_actual = world.baddies[0].0.state.center;
//         assert_eq!(new_center_actual, new_center_expected);
//     }
//
//     #[test]
//     fn baddies_wrap_at_screen_edge_rl() {
//         let mut world = World::new(
//             Cannon::new((0, 0)), // don't care
//             vec![Baddie::new((10, 1000), (-1000, 0), 0.0)],
//             vec![],
//         );
//         let dt = 20;
//         let new_center_expected = (GRID_WIDTH as i32 - 10, 1000);
//
//         update_world(&mut world, dt);
//
//         let new_center_actual = world.baddies[0].0.state.center;
//         assert_eq!(new_center_actual, new_center_expected);
//     }
//
//     #[test]
//     fn baddies_bounce_off_walls() {
//         let mut world = World::new(
//             Cannon::new((0, 0)),                             // don't care
//             vec![Baddie::new((1000, 1000), (1000, 0), 0.0)], // assume size 750 => right edge is at x=1375
//             vec![Wall::new((1900, 1000))],
//         ); // assume size is 1000 => left edge is at 1400
//         let dt = 100;
//         //// Expect baddie to travel 25 to the wall, and then be reversed. Doesn't need to be exact so just check the velocity is reversed.
//
//         // println!("Wall geometry: {:?}", world.walls[0].0.geometry);
//         // println!("Baddie before: {:?}", world.baddies[0].0.geometry);
//
//         update_world(&mut world, dt);
//
//         //println!("Baddie after: {:?}", world.baddies[0].0.geometry);
//
//         assert_eq!(world.baddies[0].0.state.vel, (-1000, 0));
//     }
//
//     // COULDDO: Test bounce + wrap
//
//     #[test]
//     fn bullet_destroyed_by_wall() {
//         // Arrange
//         let mut world = World {
//             cannon: Cannon::new((0, 0)),                      // dummy - not used here
//             bullets: vec![Bullet::new((1340, 1000), (1, 0))], // assume size is 100 => right edge is at 1390. Also, speed is 1000U/sec
//             baddies: vec![],
//             walls: vec![Wall::new((1900, 1000))], // assume size is 1000 => left edge is at 1400
//         };
//         let dt = 20;
//
//         // Act
//         update_world(&mut world, dt);
//
//         // Assert
//         assert_eq!(world.bullets.len(), 0);
//     }
// }
