use crate::geometry::{Geometry, P, Vector, scale, Vertex};
use std::collections::{HashSet, HashMap};
use crate::entity::{Entity, EntityId, EntityKind};
use crate::shape::Shape;
use std::f32::consts::PI;

/// Aggregate of entity and associated data
pub type GameObject = (Entity, Shape, Geometry);

pub type ObjectGeometries = HashMap<EntityId, Geometry>;

pub struct World {
    entities: HashSet<Entity>,
    shapes: HashMap<EntityId, Shape>,
    geometries: ObjectGeometries,
}

impl World {
    // TODO: construct from level data
    pub fn new() -> Self {
        Self {
            entities: HashSet::<Entity>::new(),
            shapes: HashMap::<EntityId, Shape>::new(),
            geometries: ObjectGeometries::new(),
        }
    }

    // TODO: may need to rethink - narrow inputs down & generate rest of data in here?
    /// Adds the provided game object to the world
    pub fn add(&mut self, game_obj: GameObject) {
        let (entity, shape, geometry) = game_obj;
        entities.add(entity);
        shapes.add(shape);
        geometries.add(geometry);
    }

    /// Removes the given entity from the world
    pub fn remove(&mut self, id: EntityId) {
        self.geometries.remove(&id);
        self.shapes.remove(&id);
        self.entities.remove(&Entity::from_id(id));
    }
}

/// Builds the box geometry, given its initial state
fn build_box_geometry(box_state: &Shape) -> [Vertex; 5] {
    let mut vertices = [(0, 0); 5];
    update_geometry(&mut vertices, box_state);
    vertices
}

/// Creates a cannon
pub fn make_cannon(center: P) -> GameObject {
    const CANON_SIZE: u32 = 50;
    let shape = Shape::new(
        center,
        CANON_SIZE,
        (0, 0),
        PI / 4.0,
        0.0,
    );
    (Entity::new(EntityKind::Cannon), shape, make_box_geometry(&shape))
}

/// Creates a bullet
pub fn make_bullet(center: P, direction: Vector) -> GameObject {
    let shape = Shape::new(
        start,
        100,
        scale(direction, 1000),
        0.0,
        0.0,
    );
    (Entity::new(EntityKind::Bullet), shape, make_box_geometry(&shape))
}

pub fn make_baddie(start: P, vel: Vector, rotation_speed: f32) -> GameObject {
    const BADDIE_SIZE: u32 = 160;
    let shape = Shape::new(
        start,
        BADDIE_SIZE,
        vel,
        0.0,
        rotation_speed,
    );
    (Entity::new(kind: EntityKind::Baddie), shape, build_box_geometry(&shape))
}

pub fn make_wall(center: P) -> GameObject {
    const WALL_SIZE: u32 = 200;
    let shape = Shape::new(
        center,
        WALL_SIZE,
        (0, 0),
        0.0,
        0.0,
    );
    (Entity::new(EntityKind::Wall), shape, build_box_geometry(&shape))
}