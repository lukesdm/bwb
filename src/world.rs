use crate::geometry::{Geometry, P, Vector, scale, Vertex, rotate};
use std::collections::{HashSet, HashMap};
use crate::entity::{Entity, EntityId, EntityKind};
use crate::shape::Shape;
use std::f32::consts::PI;

/// Aggregate of entity and associated data
pub type GameObject = (Entity, Shape, Geometry);

pub type Entities = HashSet<Entity>;
pub type Shapes = HashMap<EntityId, Shape>;
pub type ObjectGeometries = HashMap<EntityId, Geometry>;

pub struct World {
    entities: Entities,
    shapes: Shapes,
    geometries: ObjectGeometries,
}

impl World {
    pub fn new(level_data: Vec<GameObject>) -> Self {
        let mut entities = HashSet::<Entity>::new();
        let mut shapes = HashMap::<EntityId, Shape>::new();
        let mut geometries = ObjectGeometries::new();

        for (entity, shape, geometry) in level_data {
            entities.insert(entity);
            shapes.insert(entity.get_id(), shape);
            geometries.insert(entity.get_id(), geometry);
        }

        Self {
            entities,
            shapes,
            geometries,
        }
    }

    // TODO: may need to rethink - narrow inputs down & generate rest of data in here?
    /// Adds the provided game object to the world
    pub fn add(&mut self, game_obj: GameObject) {
        let (entity, shape, geometry) = game_obj;
        self.entities.insert(entity);
        self.shapes.insert(entity.get_id(), shape);
        self.geometries.insert(entity.get_id(), geometry);
    }

    /// Removes the given entity from the world
    pub fn remove(&mut self, id: EntityId) {
        self.geometries.remove(&id);
        self.shapes.remove(&id);
        self.entities.remove(&Entity::from_id(id));
    }

    pub fn get_entities(&self) -> &Entities {
        &self.entities
    }

    pub fn get_shapes(&self) -> &Shapes {
        &self.shapes
    }
    
    pub fn get_geometries(&self) -> &ObjectGeometries {
        &self.geometries
    }

    pub fn get_geometries_mut(&mut self) -> &mut ObjectGeometries {
        &mut self.geometries
    }

    // pub fn get_entities_by_kind(&self, kind: EntityKind) -> SetIter<Entity> {
    //     &self.entities.iter().filter(|e| e.get_kind() == kind)
    // }

    pub fn get_shape(&self, id: EntityId) -> &Shape {
        &self.shapes.get(&id).unwrap()
    }

    pub fn get_shape_mut(&mut self, id: EntityId) -> &mut Shape {
        self.shapes.get_mut(&id).unwrap()
    }
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
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Cannon), shape, geom)
}

/// Creates a bullet
pub fn make_bullet(center: P, direction: Vector) -> GameObject {
    let shape = Shape::new(
        center,
        100,
        scale(direction, 1000),
        0.0,
        0.0,
    );
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Bullet), shape, geom)
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
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Baddie), shape, geom)
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
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Wall), shape, geom)
}