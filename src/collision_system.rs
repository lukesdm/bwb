use crate::entity::EntityId;
use crate::geometry::{is_collision, Geometry, Vertex};
use crate::world::{GRID_HEIGHT, GRID_WIDTH, GeomRefMap};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// Bin size for spatial hashmap (square grid).
/// 10000 / 1000 => 10 * 10 grid
const GRID_BIN_SIZE: i32 = 1000;

/// Represents the different pairs of entity-kinds' collisions we're interested in observing
#[derive(PartialEq, Hash, Eq)]
pub enum CollisionKind {
    BaddieWall,
    BulletWall,
    BulletBaddie,
}

/// Collision handler - called when the collision of the supplied entity kinds is detected.
pub type CollisionHandler<'a> = Box<dyn 'a + FnMut(EntityId, EntityId) -> ()>;

/// Collision handlers for each entity-kind pair
pub type CollisionHandlers<'a> = HashMap<CollisionKind, CollisionHandler<'a>>;

/// Colliding object pairs
type CollisionPairs = HashSet<(EntityId, EntityId)>;

// Detected collisions for each entity-kind pair
type Collisions = HashMap<CollisionKind, CollisionPairs>;
type Bins = HashSet<i32>;
type SpatialMap = HashMap<i32, HashSet<EntityId>>;
type SpatialIndex = HashMap<EntityId, Bins>;

fn calc_bin_count() -> i32 {
    (GRID_WIDTH as i32 / GRID_BIN_SIZE) * (GRID_HEIGHT as i32 / GRID_BIN_SIZE)
}

fn calc_bin(vertex: &Vertex) -> i32 {
    // Assume grid is square, or this calc won't work
    assert!(GRID_WIDTH == GRID_HEIGHT);
    let (vx, vy) = vertex;
    let bx = vx / GRID_BIN_SIZE;
    let by = vy / GRID_BIN_SIZE;
    bx + by * GRID_WIDTH as i32 / GRID_BIN_SIZE
}

/// Spatial hash. Calculates the indices of a regular grid that the given geometry occupies.
/// Important - this implementation only works if shape size < bin size.
fn grid_hash(vertices: &Geometry) -> Bins {
    let mut bins = Bins::new();
    for v in vertices {
        bins.insert(calc_bin(v));
    }
    bins
}

/// Build map of bin -> object list, and associated index
fn build_map(geometries: &GeomRefMap) -> (SpatialMap, SpatialIndex) {
    let mut object_map = SpatialMap::new();
    let mut object_index = SpatialIndex::new();
    for geometry in geometries {
        let (id, vertices) = geometry;
        let grid_bins = grid_hash(vertices);
        for bin in grid_bins.iter() {
            object_map
                .entry(*bin)
                .and_modify(|e| {
                    e.insert(*id);
                })
                .or_insert(HashSet::from_iter([*id].iter().cloned()));
        }

        let existing = object_index.insert(*id, grid_bins);
        if let Some(_) = existing {
            panic!("Unexpected duplicate object.");
        }
    }
    (object_map, object_index)
}

/// Detects collisions and runs handlers as appropriate
pub struct CollisionSystem<'a> {
    wall_map: SpatialMap,
    #[allow(unused)]
    wall_index: SpatialIndex,
    baddie_map: SpatialMap,
    #[allow(unused)]
    baddie_index: SpatialIndex,
    bullet_map: SpatialMap,
    #[allow(unused)]
    bullet_index: SpatialIndex,
    handlers: CollisionHandlers<'a>,
}

impl<'a> CollisionSystem<'a> {
    pub fn new(
        walls: &GeomRefMap,
        baddies: &GeomRefMap,
        bullets: &GeomRefMap,
        baddie_wall_handler: CollisionHandler<'a>,
        bullet_wall_handler: CollisionHandler<'a>,
        bullet_baddie_handler: CollisionHandler<'a>,
    ) -> Self {
        // build hashmaps from object geometries
        let (wall_map, wall_index) = build_map(walls);
        let (baddie_map, baddie_index) = build_map(baddies);
        let (bullet_map, bullet_index) = build_map(bullets);

        let mut handlers = CollisionHandlers::new();
        handlers.insert(CollisionKind::BaddieWall, baddie_wall_handler);
        handlers.insert(CollisionKind::BulletBaddie, bullet_baddie_handler);
        handlers.insert(CollisionKind::BulletWall, bullet_wall_handler);

        Self {
            wall_map,
            wall_index,
            baddie_map,
            baddie_index,
            bullet_map,
            bullet_index,
            handlers,
        }
    }

    /// Check collisions and run appropriate handlers
    pub fn process(&mut self, walls: &GeomRefMap, baddies: &GeomRefMap, bullets: &GeomRefMap) {
        let mut collisions = Collisions::new();
        collisions.insert(CollisionKind::BaddieWall, CollisionPairs::new());
        collisions.insert(CollisionKind::BulletBaddie, CollisionPairs::new());
        collisions.insert(CollisionKind::BulletWall, CollisionPairs::new());

        let bin_count = calc_bin_count();
        for i in 0..bin_count {
            // Walls vs Baddies
            if let Some(wall_ids) = self.wall_map.get(&i) {
                if let Some(baddie_ids) = self.baddie_map.get(&i) {
                    for wall_id in wall_ids {
                        for baddie_id in baddie_ids {
                            let wall_geom = walls.get(wall_id).unwrap();
                            let baddie_geom = baddies.get(baddie_id).unwrap();
                            if is_collision(*wall_geom, *baddie_geom) {
                                collisions
                                    .get_mut(&CollisionKind::BaddieWall)
                                    .unwrap()
                                    .insert((*baddie_id, *wall_id));
                            }
                        }
                    }
                }
            }

            // Bullets...
            if let Some(bullet_ids) = self.bullet_map.get(&i) {
                for bullet_id in bullet_ids {
                    // ... vs Walls
                    if let Some(wall_ids) = self.wall_map.get(&i) {
                        for wall_id in wall_ids {
                            let bullet_geom = bullets.get(bullet_id).unwrap();
                            let wall_geom = walls.get(wall_id).unwrap();
                            if is_collision(*bullet_geom, *wall_geom) {
                                collisions
                                    .get_mut(&CollisionKind::BulletWall)
                                    .unwrap()
                                    .insert((*bullet_id, *wall_id));
                            }
                        }
                    }

                    // ... vs Baddies
                    if let Some(baddie_ids) = self.baddie_map.get(&i) {
                        for baddie_id in baddie_ids {
                            let bullet_geom = bullets.get(bullet_id).unwrap();
                            let baddie_geom = baddies.get(baddie_id).unwrap();
                            if is_collision(*bullet_geom, *baddie_geom) {
                                collisions
                                    .get_mut(&CollisionKind::BulletBaddie)
                                    .unwrap()
                                    .insert((*bullet_id, *baddie_id));
                            }
                        }
                    }
                }
            }
        }

        for (collision_kind, collision_pairs) in collisions {
            for collision_pair in collision_pairs {
                let handler = self.handlers.get_mut(&collision_kind).unwrap();
                handler(collision_pair.0, collision_pair.1);
            }
        }
    }

    // pub fn update(world: &mut World) {
    //     // update hashmaps and check collisions
    // }
}

// TODO: Decouple tests from World functions
#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::ObjectFactory;

    #[test]
    fn grid_hash_single() {
        let vertex = (1000, 2000);
        let expected = Bins::from_iter([21].iter().cloned());
        let actual = grid_hash(&[vertex, vertex, vertex, vertex, vertex]);

        assert_eq!(expected, actual);
    }

    /// Tilted box spanning multiple bins
    #[test]
    fn grid_hash_box() {
        let vertices = [
            (1000, 2000), // bin 21
            (1770, 2200), // bin 21
            (1550, 2900), // bin 21
            (780, 2770),  // bin 20
            (1000, 2000), // bin 21
        ];
        let expected = Bins::from_iter([20, 21].iter().cloned());
        let actual = grid_hash(&vertices);

        assert_eq!(expected, actual);
    }

    /// 2 walls with some occupying some bins in common - build map and index
    #[test]
    fn build_map_2walls_some_common_bins() {
        // Arrange - 2 walls in bin 11
        let obj_factory = ObjectFactory::new(1000);
        let (wall1, _, wall1_geom) = obj_factory.make_wall((1200, 1200));
        let w1_bins_expected = Bins::from_iter([0, 1, 10, 11].iter().cloned());
        let (wall2, _, wall2_geom) = obj_factory.make_wall((1700, 1700));
        let w2_bins_expected = Bins::from_iter([11, 12, 21, 22].iter().cloned());
        let walls_geoms: GeomRefMap =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();
        let expected = HashSet::from_iter([wall1.get_id(), wall2.get_id()].iter().cloned());
        // Act
        let (wall_map, wall_index) = build_map(&walls_geoms);

        // Assert - map
        assert_eq!(wall_map.get(&11).unwrap(), &expected);

        // Assert - index
        assert_eq!(wall_index.get(&wall1.get_id()), Some(&w1_bins_expected));
        assert_eq!(wall_index.get(&wall2.get_id()), Some(&w2_bins_expected));
    }

    #[test]
    fn collision_static_simple() {
        // Arrange - 2 walls, 2 baddies, 1 of each colliding, plus associated handler
        let obj_factory = ObjectFactory::new(400);
        let (wall1, _, wall1_geom) = obj_factory.make_wall((1200, 1200));
        let (wall2, _, wall2_geom) = obj_factory.make_wall((1700, 1700));
        let walls_geoms: GeomRefMap =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();

        // colliding baddie:
        let (baddie1, _, baddie1_geom) = obj_factory.make_baddie((1200, 1200), (0, 0), 0.0);
        // not colliding baddie:
        let (baddie2, _, baddie2_geom) = obj_factory.make_baddie((0, 0), (0, 0), 0.0);
        let baddies_geoms: GeomRefMap = [
            (baddie1.get_id(), &baddie1_geom),
            (baddie2.get_id(), &baddie2_geom),
        ]
        .iter()
        .cloned()
        .collect();
        let baddie_wall_handler = |baddie_id: EntityId, wall_id: EntityId| {
            // Assert - handler called with correct arguments
            assert!(
                (wall_id == wall1.get_id() && baddie_id == baddie1.get_id())
                    && !(baddie_id == baddie2.get_id() || wall_id == wall2.get_id())
            )
        };
        let dummy_geoms = &GeomRefMap::new();
        let dummy_handler = |_: EntityId, _: EntityId| ();
        let mut collision_system = CollisionSystem::new(
            &walls_geoms,
            &baddies_geoms,
            &dummy_geoms,
            Box::new(baddie_wall_handler),
            Box::new(dummy_handler),
            Box::new(dummy_handler),
        );
        // Act
        collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms);

        // Assert - see handler, above
    }

    #[test]
    fn collision_can_mutate_baddie() {
        // Arrange - 1 wall, 1 baddies, colliding, plus associated baddie_wall_handler
        let obj_factory = ObjectFactory::new(1000);
        let (wall, _, wall_geom) = obj_factory.make_wall((1200, 1200));
        let walls_geoms: GeomRefMap = [(wall.get_id(), &wall_geom)].iter().cloned().collect();

        let (baddie, mut baddie_shape, baddie_geom) =
            obj_factory.make_baddie((1200, 1200), (1000, 0), 0.0);
        let baddies_geoms: GeomRefMap = [(baddie.get_id(), &baddie_geom)].iter().cloned().collect();

        let baddie_wall_handler = |baddie_id: EntityId, wall_id: EntityId| {
            assert_eq!(wall_id, wall.get_id());
            assert_eq!(baddie_id, baddie.get_id());
            baddie_shape.reverse();
        };
        let dummy_geoms = &GeomRefMap::new();
        let dummy_handler = |_: EntityId, _: EntityId| ();
        // Scope needed here for collision system - need to return borrowed references before assert
        {
            let mut collision_system = CollisionSystem::new(
                &walls_geoms,
                &baddies_geoms,
                &dummy_geoms,
                Box::new(baddie_wall_handler),
                Box::new(dummy_handler),
                Box::new(dummy_handler),
            );
            // Act
            collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms);
        }

        // Assert
        assert_eq!(*baddie_shape.get_vel(), (-1000, 0));
    }

    #[test]
    fn calc_bin_count() {
        let bin_count_expected = 100;
        let bin_count_actual = super::calc_bin_count();
        assert_eq!(bin_count_actual, bin_count_expected);
    }
}
