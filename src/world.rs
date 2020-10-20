use crate::entity::{Entity, EntityId, EntityKind};
use crate::geometry::{rotate, scale, Geometry, Vector, Vertex, P};
use crate::shape::Shape;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

/// Aggregate of entity and associated data
pub type GameObject = (Entity, Shape, Geometry);

pub type Entities = HashSet<Entity>;
pub type Shapes = HashMap<EntityId, Shape>;
pub type ObjectGeometries = HashMap<EntityId, Geometry>;

pub type World = (Entities, Shapes, ObjectGeometries);

pub fn create_world(level_data: Vec<GameObject>) -> World {
    let mut entities = HashSet::<Entity>::new();
    let mut shapes = HashMap::<EntityId, Shape>::new();
    let mut geometries = ObjectGeometries::new();

    for (entity, shape, geometry) in level_data {
        entities.insert(entity);
        shapes.insert(entity.get_id(), shape);
        geometries.insert(entity.get_id(), geometry);
    }

    (entities, shapes, geometries)
}

/// Adds the provided game object to the world
pub fn add(world: &mut World, game_obj: GameObject) {
    let (entities, shapes, geometries) = world;
    let (entity, shape, geometry) = game_obj;
    entities.insert(entity);
    shapes.insert(entity.get_id(), shape);
    geometries.insert(entity.get_id(), geometry);
}

/// Removes the given entity from the world
pub fn remove(world: &mut World, id: EntityId) {
    let (entities, shapes, geometries) = world;
    geometries.remove(&id);
    shapes.remove(&id);
    entities.remove(&Entity::from_id(id));
}

pub fn get_entity(entities: &Entities, id: EntityId) -> &Entity {
    entities.get(&Entity::from_id(id)).unwrap()
}

/// Map of EntityId to Geometry reference
pub type GeomRefMap<'a> = HashMap<EntityId, &'a Geometry>;

pub fn destructure_geom<'a>(
    entities: &'a Entities,
    geometries: &'a ObjectGeometries,
) -> (GeomRefMap<'a>, GeomRefMap<'a>, GeomRefMap<'a>) {
    let mut wall_geoms = HashMap::<EntityId, &Geometry>::new();
    let mut baddie_geoms = HashMap::<EntityId, &Geometry>::new();
    let mut bullet_geoms = HashMap::<EntityId, &Geometry>::new();
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
            _ => (),
        }
    }
    (wall_geoms, baddie_geoms, bullet_geoms)
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
    let shape = Shape::new(center, CANON_SIZE, (0, 0), PI / 4.0, 0.0);
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Cannon), shape, geom)
}

/// Creates a bullet
pub fn make_bullet(center: P, direction: Vector) -> GameObject {
    let shape = Shape::new(center, 100, scale(direction, 1000), 0.0, 0.0);
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Bullet), shape, geom)
}

pub fn make_baddie(start: P, vel: Vector, rotation_speed: f32) -> GameObject {
    const BADDIE_SIZE: u32 = 160;
    let shape = Shape::new(start, BADDIE_SIZE, vel, 0.0, rotation_speed);
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Baddie), shape, geom)
}

pub fn make_wall(center: P) -> GameObject {
    const WALL_SIZE: u32 = 200;
    let shape = Shape::new(center, WALL_SIZE, (0, 0), 0.0, 0.0);
    let geom = build_box_geometry(&shape);
    (Entity::new(EntityKind::Wall), shape, geom)
}
