//! # Game logic
//! Primary rules:
//! * Bullet meets Enemy => Both destroyed
//! * Bullet meets Wall => Bullet destroyed
//! * Enemy meets Wall => Enemy bounces/reverses
//! * All enemies destroyed => level ends
//! * Enemy meets player => Player health decreases + enemy destroyed
//! * Player health decreases to 0 => Game Over

//! Other rules:
//! * Bullets are destroyed when they reach edge of screen
//! * Enemies wrap to the other side of the screen
//! * Player health reset at start of level

use crate::collision_system::CollisionSystem;
use crate::entity::{EntityId, EntityKind};
use crate::geometry::{direction_vector, Direction, P};
use crate::shape::Shape;
use crate::world;
use crate::world::{
    update_geometry, Entities, GameObjects, Geometries, Healths, Shapes, World, GRID_HEIGHT,
    GRID_WIDTH,
};
use std::collections::HashSet;
use std::time::{Duration, Instant};

fn get_cannon_pos(game_objects: &GameObjects) -> &P {
    let cannon_id = world::get_cannon(game_objects).unwrap().get_id();
    let (_, shapes, _, _) = game_objects;
    shapes.get(&cannon_id).unwrap().get_center()
}

// (ACTION)
/// Try fire the cannon, throttled to the rate of fire.
/// Returns the instant of when the cannon was previously fired successfully.  
/// Note: Rate of fire is hardcoded within.  
pub fn try_fire(
    now: Instant,
    prev: Instant,
    game_objects: &mut GameObjects,
    direction: Direction,
    obj_factory: &world::ObjectFactory,
) -> Instant {
    // 1 / rate of fire
    const RELOAD_TIME: Duration = Duration::from_millis(1000);

    let cannon_pos = *get_cannon_pos(game_objects);

    if now > prev + RELOAD_TIME {
        // Fire!!
        world::add(
            game_objects,
            obj_factory.make_bullet(cannon_pos, direction_vector(direction)),
        );
        return now;
    }
    return prev;
}

// (ACTION)
/// Moves the cannon
pub fn move_cannon(game_objects: &mut GameObjects, direction: Direction) {
    let cannon_id = world::get_cannon(game_objects).unwrap().get_id();
    let shape = game_objects.1.get_mut(&cannon_id).unwrap();
    shape.set_movement(direction);
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

fn is_inside_world(point: P) -> bool {
    let (x, y) = point;
    (x > 0 && x <= GRID_WIDTH as i32) && (y > 0 && y <= GRID_HEIGHT as i32)
}

/// Handle when bullets miss i.e. reach edge of world without hitting anything - remove them.
fn handle_bullet_misses(game_objects: &mut GameObjects) {
    let bullets = game_objects
        .0
        .iter()
        .filter(|e| *e.get_kind() == EntityKind::Bullet);

    let to_remove: Vec<EntityId> = bullets
        .filter(|b| {
            let shape = game_objects.1.get(&b.get_id()).unwrap();
            !is_inside_world(*shape.get_center())
        })
        .map(|b| b.get_id())
        .collect();

    for b in to_remove {
        world::remove(game_objects, b);
    }
}

fn detect_and_handle_collisions(
    entities: &Entities,
    shapes: &mut Shapes,
    geometries: &Geometries,
    healths: &mut Healths,
) -> HashSet<EntityId> {
    // Removal collections. Need a separate one for each closure, but they can be merged at the end.
    let mut to_remove = HashSet::<EntityId>::new();
    let mut to_remove_2 = HashSet::<EntityId>::new();
    let mut to_remove_3 = HashSet::<EntityId>::new();
    {
        let baddie_wall_handler = |baddie_id: EntityId, _wall_id: EntityId| {
            let baddie_shape = shapes.get_mut(&baddie_id).unwrap();
            baddie_shape.move_back();
            baddie_shape.reverse();
        };

        let bullet_wall_handler = |bullet_id: EntityId, _wall_id: EntityId| {
            to_remove.insert(bullet_id);
        };

        let bullet_baddie_handler = |bullet_id: EntityId, baddie_id: EntityId| {
            to_remove_2.insert(bullet_id);
            to_remove_2.insert(baddie_id);
        };

        let baddie_cannon_handler = |baddie_id: EntityId, cannon_id: EntityId| {
            to_remove_3.insert(baddie_id);
            let cannon_health = healths.get_mut(&cannon_id).unwrap();
            let new_health = *cannon_health - 1;
            *cannon_health = new_health;
        };

        let (wall_geoms, baddie_geoms, bullet_geoms, cannon_geoms) =
            world::destructure_geom(&entities, &geometries);
        let mut collision_system = CollisionSystem::new(
            &wall_geoms,
            &baddie_geoms,
            &bullet_geoms,
            &cannon_geoms,
            Box::new(baddie_wall_handler),
            Box::new(bullet_wall_handler),
            Box::new(bullet_baddie_handler),
            Box::new(baddie_cannon_handler),
        );
        collision_system.process(&wall_geoms, &baddie_geoms, &bullet_geoms, &cannon_geoms);
    }
    // Union the removal lists
    for tr in to_remove_2 {
        to_remove.insert(tr);
    }
    for tr in to_remove_3 {
        to_remove.insert(tr);
    }

    to_remove
}

fn update_positions(entities: &Entities, shapes: &mut Shapes, dt: i32) {
    for entity in entities.iter() {
        let shape = shapes.get_mut(&entity.get_id()).unwrap();
        match entity.get_kind() {
            EntityKind::Baddie => update_pos(shape, dt, true),
            EntityKind::Cannon => update_pos(shape, dt, true),
            EntityKind::Bullet => update_pos(shape, dt, false),
            EntityKind::Wall => update_pos(shape, dt, false),
            _ => (),
        }
    }
}

fn update_geometries(shapes: &Shapes, geometries: &mut Geometries) {
    for (id, shape) in shapes.iter() {
        let geometry = geometries.get_mut(&id).unwrap();
        update_geometry(geometry, shape);
    }
}

/// Gets player health.
/// Optional as there may not be a player in the world, as in some test cases.
fn player_health(game_objects: &GameObjects) -> Option<i32> {
    if let Some(cannon) = world::get_cannon(game_objects) {
        let cannon_id = cannon.get_id();
        let (_, _, _, healths) = game_objects;
        Some(*healths.get(&cannon_id).unwrap())
    } else {
        None
    }
}

#[derive(Debug)]
pub enum LevelState {
    InProgress,
    Complete,
    GameOver,
}

pub fn update_world(mut world: World, dt: i32) -> (World, LevelState) {
    // Update shape state
    let (entities, mut shapes, geometries, healths) = world;
    update_positions(&entities, &mut shapes, dt);
    world = (entities, shapes, geometries, healths);

    // Update geometry ready for collision detection
    let (entities, shapes, mut geometries, healths) = world;
    update_geometries(&shapes, &mut geometries);
    world = (entities, shapes, geometries, healths);

    handle_bullet_misses(&mut world);
    // Detect & handle collisions
    let (entities, mut shapes, geometries, mut healths) = world;
    let to_remove = detect_and_handle_collisions(&entities, &mut shapes, &geometries, &mut healths);
    world = (entities, shapes, geometries, healths);
    for e in to_remove {
        world::remove(&mut world, e);
    }

    // 2nd pass of geometry update to reflect destroyed/backed-out objects.
    // Could be more efficient, but so far it's not a bottleneck.
    let (entities, shapes, mut geometries, healths) = world;
    update_geometries(&shapes, &mut geometries);
    world = (entities, shapes, geometries, healths);

    let state = if player_health(&world) == Some(0) {
        LevelState::GameOver
    } else if level_complete(&world) {
        LevelState::Complete
    } else {
        LevelState::InProgress
    };
    (world, state)
}

fn level_complete(world: &World) -> bool {
    let (entities, _, _, _) = world;
    let baddies = entities
        .iter()
        .filter(|e| e.get_kind() == &EntityKind::Baddie);
    baddies.count() == 0
}

/// Game logic tests. Note: These are integration tests, rather than unit tests.
#[cfg(test)]
mod tests {
    use super::{update_world, LevelState, GRID_WIDTH};
    use crate::entity::Entity;
    use crate::world;
    #[test]
    fn bullet_meets_enemy_both_destroyed() {
        // Arrange
        // 2 different bullets, 2 different baddies, and 1 of each about to collide
        let obj_factory = world::ObjectFactory::new(1000);
        let hit_baddie = obj_factory.make_baddie((5500, 5000), (0, 0), 0.0);
        let missed_baddie = obj_factory.make_baddie((5000, 7000), (0, 0), 0.0);
        let expected_id_1 = missed_baddie.0.get_id();

        // Assume baddie size is 750 => left edge at 5500 - 750 / 2 = 5525
        let hitting_bullet = obj_factory.make_bullet((5115, 5000), (1, 0));
        let missing_bullet = obj_factory.make_bullet((4000, 4500), (0, 1));
        let expected_id_2 = missing_bullet.0.get_id();

        // simulate 20ms
        let dt = 20;

        let world = world::create_world(vec![
            hit_baddie,
            missed_baddie,
            hitting_bullet,
            missing_bullet,
        ]);

        // Act
        let ((entities, _, _, _), _) = update_world(world, dt);

        // Assert
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&Entity::from_id(expected_id_1)));
        assert!(entities.contains(&Entity::from_id(expected_id_2)));
    }

    #[test]
    fn bullet_destroyed_at_screen_edge() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let world = world::create_world(vec![
            obj_factory.make_bullet((GRID_WIDTH as i32 - 10, 100), (1, 0))
        ]);
        let dt = 20;

        // Act
        let ((entities, _, _, _), _) = update_world(world, dt);

        // Assert
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn baddies_wrap_at_screen_edge_lr() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let baddie = obj_factory.make_baddie((GRID_WIDTH as i32 - 10, 1000), (1000, 0), 0.0);
        let baddie_id = baddie.0.get_id();
        let world = world::create_world(vec![baddie]);
        let dt = 20;
        let new_center_expected = (10, 1000);

        // Act
        let ((_, shapes, _, _), _) = update_world(world, dt);

        // Assert
        let new_center_actual = shapes.get(&baddie_id).unwrap().get_center();
        assert_eq!(*new_center_actual, new_center_expected);
    }

    #[test]
    fn baddies_wrap_at_screen_edge_rl() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let baddie = obj_factory.make_baddie((10, 1000), (-1000, 0), 0.0);
        let baddie_id = baddie.0.get_id();
        let world = world::create_world(vec![baddie]);
        let dt = 20;
        let new_center_expected = (GRID_WIDTH as i32 - 10, 1000);

        // Act
        let ((_, shapes, _, _), _) = update_world(world, dt);

        // Assert
        let new_center_actual = shapes.get(&baddie_id).unwrap().get_center();
        assert_eq!(*new_center_actual, new_center_expected);
    }

    #[test]
    fn baddies_bounce_off_walls() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let baddie = obj_factory.make_baddie((1000, 1000), (1000, 0), 0.0); // assume size 750 => right edge is at x=1375
        let baddie_id = baddie.0.get_id();
        let wall = obj_factory.make_wall((1900, 1000)); // assume size is 1000 => left edge is at 1400
        let world = world::create_world(vec![baddie, wall]);
        let dt = 100;
        // Expect baddie to travel 25 to the wall, and then be reversed. Doesn't need to be exact so just check the velocity is reversed.

        // Act
        let ((_, shapes, _, _), _) = update_world(world, dt);

        // Assert
        let new_vel = *shapes.get(&baddie_id).unwrap().get_vel();
        assert_eq!(new_vel, (-1000, 0));
    }

    // // COULDDO: Test bounce + wrap

    #[test]
    fn bullet_destroyed_by_wall() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);

        // assume size is 100 => right edge is at 1390. Also, speed is 1000U/sec
        let bullet = obj_factory.make_bullet((1340, 1000), (1, 0));
        let bullet_id = bullet.0.get_id();
        // assume size is 1000 => left edge is at 1400
        let wall = obj_factory.make_wall((1900, 1000));
        let world = world::create_world(vec![bullet, wall]);

        let dt = 20;

        // Act
        let ((entities, _, _, _), _) = update_world(world, dt);

        // Assert
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&Entity::from_id(bullet_id)) == false);
    }

    #[test]
    fn baddie_destroyed_by_cannon() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let cannon = obj_factory.make_cannon((1000, 1000));
        let baddie = obj_factory.make_baddie((1000, 1000), (0, 0), 0.0);
        let baddie_id = baddie.0.get_id();
        let world = world::create_world(vec![cannon, baddie]);

        let dt = 20;

        // Act
        let ((entities, _, _, _), _) = update_world(world, dt);

        // Assert
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&Entity::from_id(baddie_id)) == false);
    }

    #[test]
    fn cannon_damaged_by_baddie() {
        // Arrange
        let obj_factory = world::ObjectFactory::new(1000);
        let cannon = obj_factory.make_cannon((1000, 1000));
        let cannon_id = cannon.0.get_id();
        let baddie = obj_factory.make_baddie((1000, 1000), (0, 0), 0.0);
        let world = world::create_world(vec![cannon, baddie]);
        let expected_health_change = -1;

        let dt = 20;

        // Act
        let health_before = *world.3.get(&cannon_id).unwrap();
        let ((_, _, _, healths), _) = update_world(world, dt);
        let health_after = healths.get(&cannon_id).unwrap();

        // Assert
        assert_eq!(health_after - health_before, expected_health_change);
    }
    #[test]
    fn gameover_at_zero_health() {
        // Arrange - init world with cannon/players health at 0.
        let obj_factory = world::ObjectFactory::new(1000);
        let cannon = obj_factory.make_cannon((1000, 1000));
        let (entity, shape, geometry, _) = cannon;
        let cannon = (entity, shape, geometry, Some(0));
        let world = world::create_world(vec![cannon]);

        // Act
        let (_, level_state) = update_world(world, 10);

        // Assert
        let gameover = match level_state {
            LevelState::GameOver => true,
            _ => false,
        };
        assert!(gameover);
    }
}
