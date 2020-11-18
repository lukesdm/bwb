use crate::entity::{Entity, EntityId, EntityKind};
use crate::geometry::{rotate, scale, Geometry, Vector, Vertex, P};
use crate::shape::Shape;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

// World coordinate bounds
pub const GRID_WIDTH: u32 = 10000;
pub const GRID_HEIGHT: u32 = 10000;

pub const PLAYER_HEALTH_MAX: i32 = 3;

pub type Health = i32;

/// Aggregate of entity and associated data.
/// Is a tuple so that each component can be borrowed independently
pub type GameObject = (Entity, Shape, Geometry, Option<Health>);

pub type Entities = HashSet<Entity>;
pub type Shapes = HashMap<EntityId, Shape>;
pub type Geometries = HashMap<EntityId, Geometry>;
pub type Healths = HashMap<EntityId, Health>;

/// Map of EntityId to Geometry reference
pub type GeomRefMap<'a> = HashMap<EntityId, &'a Geometry>;

/// Aggregates of world data components.
/// Are tuples so that each component can be borrowed independently.
pub type GameObjects = (Entities, Shapes, Geometries, Healths);
pub type World = GameObjects;

pub fn create_world(level_data: Vec<GameObject>) -> World {
    let mut entities = HashSet::<Entity>::new();
    let mut shapes = HashMap::<EntityId, Shape>::new();
    let mut geometries = Geometries::new();
    let mut healths = Healths::new();

    for (entity, shape, geometry, health) in level_data {
        entities.insert(entity);
        shapes.insert(entity.get_id(), shape);
        geometries.insert(entity.get_id(), geometry);
        if let Some(health) = health {
            healths.insert(entity.get_id(), health);
        }
    }

    (entities, shapes, geometries, healths)
}

/// Adds the provided game object to the world
pub fn add(game_objects: &mut GameObjects, game_obj: GameObject) {
    let (entities, shapes, geometries, healths) = game_objects;
    let (entity, shape, geometry, health) = game_obj;
    entities.insert(entity);
    shapes.insert(entity.get_id(), shape);
    geometries.insert(entity.get_id(), geometry);
    if let Some(health) = health {
        healths.insert(entity.get_id(), health);
    }
}

/// Removes the given entity from the world
pub fn remove(game_objects: &mut GameObjects, id: EntityId) {
    let (entities, shapes, geometries, healths) = game_objects;
    geometries.remove(&id);
    shapes.remove(&id);
    healths.remove(&id); // TODO: check - any effect if item isn't in there?
    entities.remove(&Entity::from_id(id));
}

pub fn get_entity(entities: &Entities, id: EntityId) -> &Entity {
    entities.get(&Entity::from_id(id)).unwrap()
}

/// Gets the cannon
pub fn get_cannon(game_objects: &GameObjects) -> &Entity {
    game_objects
        .0
        .iter()
        .find(|e| *e.get_kind() == EntityKind::Cannon)
        .unwrap()
}

/// Separates geometry collection by entity kind.
/// Note: Allocates separate collections (of references)
pub fn destructure_geom<'a>(
    entities: &'a Entities,
    geometries: &'a Geometries,
) -> (
    GeomRefMap<'a>,
    GeomRefMap<'a>,
    GeomRefMap<'a>,
    GeomRefMap<'a>,
) {
    let mut wall_geoms = HashMap::<EntityId, &Geometry>::new();
    let mut baddie_geoms = HashMap::<EntityId, &Geometry>::new();
    let mut bullet_geoms = HashMap::<EntityId, &Geometry>::new();
    let mut cannon_geoms = HashMap::<EntityId, &Geometry>::new();
    for (entity_id, geom) in geometries.iter() {
        let entity_id = *entity_id;
        let e = get_entity(entities, entity_id);
        match e.get_kind() {
            EntityKind::Wall => {
                wall_geoms.insert(entity_id, geom);
            }
            EntityKind::Baddie => {
                baddie_geoms.insert(entity_id, geom);
            }
            EntityKind::Bullet => {
                bullet_geoms.insert(entity_id, geom);
            }
            EntityKind::Cannon => {
                cannon_geoms.insert(entity_id, geom);
            }
            _ => (),
        }
    }
    (wall_geoms, baddie_geoms, bullet_geoms, cannon_geoms)
}

/// Updates box geometry according to its state
pub fn update_geometry(box_geometry: &mut [Vertex], box_state: &Shape) {
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

/// Builds the box geometry, given its initial state
fn build_box_geometry(box_state: &Shape) -> Geometry {
    let mut vertices = [(0, 0); 5];
    update_geometry(&mut vertices, box_state);
    vertices
}

const BADDIE_SIZE: f32 = 0.75;
const WALL_SIZE: f32 = 1.0;
const BULLET_SIZE: f32 = 0.1;
const CANNON_SIZE: f32 = 0.2;
const BULLET_SPEED: i32 = 1000;

/// Factory for creating the various kinds of game objects
pub struct ObjectFactory {
    base_size: u32,
}

impl ObjectFactory {
    /// Creates a new `ObjectFactory` with the given base size.
    pub fn new(base_size: u32) -> Self {
        Self { base_size }
    }

    /// Creates a cannon
    pub fn make_cannon(&self, center: P) -> GameObject {
        let shape = Shape::new(center, self.calc_size(CANNON_SIZE), (0, 0), PI / 4.0, 0.0);
        let geom = build_box_geometry(&shape);
        (
            Entity::new(EntityKind::Cannon),
            shape,
            geom,
            Some(PLAYER_HEALTH_MAX),
        )
    }

    /// Creates a bullet
    pub fn make_bullet(&self, center: P, direction: Vector) -> GameObject {
        let shape = Shape::new(
            center,
            self.calc_size(BULLET_SIZE),
            scale(direction, BULLET_SPEED),
            0.0,
            0.0,
        );
        let geom = build_box_geometry(&shape);
        (Entity::new(EntityKind::Bullet), shape, geom, None)
    }

    pub fn make_baddie(&self, start: P, vel: Vector, rotation_speed: f32) -> GameObject {
        let shape = Shape::new(start, self.calc_size(BADDIE_SIZE), vel, 0.0, rotation_speed);
        let geom = build_box_geometry(&shape);
        (Entity::new(EntityKind::Baddie), shape, geom, None)
    }

    pub fn make_wall(&self, center: P) -> GameObject {
        let shape = Shape::new(center, self.calc_size(WALL_SIZE), (0, 0), 0.0, 0.0);
        let geom = build_box_geometry(&shape);
        (Entity::new(EntityKind::Wall), shape, geom, None)
    }

    fn calc_size(&self, obj_size: f32) -> u32 {
        (self.base_size as f32 * obj_size) as u32
    }
}
