use crate::entity::EntityId;
use crate::geometry::{box_side_len_sqr, is_collision, Geometry, Vertex};
use crate::world::{GeomRefMap, GRID_HEIGHT, GRID_WIDTH};
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// Represents the different pairs of entity-kinds' collisions we're interested in observing
#[derive(PartialEq, Hash, Eq)]
pub enum CollisionKind {
    BaddieWall,
    BulletWall,
    BulletBaddie,
    BaddieCannon,
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

fn calc_bin_count(grid_bin_size: i32) -> i32 {
    ((GRID_WIDTH as i32 / grid_bin_size) + 1) * ((GRID_HEIGHT as i32 / grid_bin_size) + 1)
}

fn calc_bin(vertex: &Vertex, grid_bin_size: i32) -> i32 {
    // Assume grid is square, or this calc won't work
    assert!(GRID_WIDTH == GRID_HEIGHT);
    let (vx, vy) = vertex;
    let bx = vx / grid_bin_size;
    let by = vy / grid_bin_size;
    bx + by * GRID_WIDTH as i32 / grid_bin_size
}

/// Spatial hash. Calculates the indices of a regular grid that the given geometry occupies.
/// Important - this implementation only works if shape size < bin size.
fn grid_hash(vertices: &Geometry, grid_bin_size: i32) -> Bins {
    let mut bins = Bins::new();
    for v in vertices {
        bins.insert(calc_bin(v, grid_bin_size));
    }
    bins
}

/// Build map of bin -> object list, and associated index
fn build_map(geometries: &GeomRefMap, grid_bin_size: i32) -> (SpatialMap, SpatialIndex) {
    let mut object_map = SpatialMap::new();
    let mut object_index = SpatialIndex::new();
    for geometry in geometries {
        let (id, vertices) = geometry;
        let grid_bins = grid_hash(vertices, grid_bin_size);
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

// const empty_set: &HashSet::<EntityId> = &HashSet::<EntityId>::default();
// fn empty() -> &'static HashSet<EntityId> {
//     &empty_set
// }

// Accumulate collision pairs
fn add_collisions(
    collisions_acc: &mut Collisions,
    kind: &CollisionKind,
    left: &(&SpatialMap, &GeomRefMap),
    right: &(&SpatialMap, &GeomRefMap),
    bin: &i32,
) {
    // TODO: can extract these?
    let empty_set = &HashSet::<EntityId>::default();
    let empty = || Some(empty_set);
    let (left_map, left_geoms) = left;
    let (right_map, right_geoms) = right;
    let left_ids = left_map.get(bin).or_else(empty).unwrap();
    let right_ids = right_map.get(bin).or_else(empty).unwrap();
    let collision_pairs =
        left_ids
            .iter()
            .cartesian_product(right_ids)
            .filter(|(left_id, right_id)| {
                let left_geom = left_geoms.get(*left_id).unwrap();
                let right_geom = right_geoms.get(*right_id).unwrap();
                is_collision(*left_geom, *right_geom)
            });

    for (left_id, right_id) in collision_pairs {
        collisions_acc
            .get_mut(kind)
            .unwrap()
            .insert((*left_id, *right_id));
    }
}

fn detect_collisions(
    walls: (&SpatialMap, &GeomRefMap),
    baddies: (&SpatialMap, &GeomRefMap),
    bullets: (&SpatialMap, &GeomRefMap),
    cannons: (&SpatialMap, &GeomRefMap),
    grid_bin_size: i32,
) -> Collisions {
    let bin_count = calc_bin_count(grid_bin_size);
    let init = || {
        let mut collisions_init = Collisions::new();
        collisions_init.insert(CollisionKind::BaddieWall, CollisionPairs::new());
        collisions_init.insert(CollisionKind::BulletBaddie, CollisionPairs::new());
        collisions_init.insert(CollisionKind::BulletWall, CollisionPairs::new());
        collisions_init.insert(CollisionKind::BaddieCannon, CollisionPairs::new());
        collisions_init
    };

    let collisions = (0..bin_count)
        .into_par_iter()
        .fold(init, |mut collisions_acc, bin| {
            add_collisions(
                &mut collisions_acc,
                &CollisionKind::BaddieWall,
                &baddies,
                &walls,
                &bin,
            );
            add_collisions(
                &mut collisions_acc,
                &CollisionKind::BulletWall,
                &bullets,
                &walls,
                &bin,
            );
            add_collisions(
                &mut collisions_acc,
                &CollisionKind::BulletBaddie,
                &bullets,
                &baddies,
                &bin,
            );
            add_collisions(
                &mut collisions_acc,
                &CollisionKind::BaddieCannon,
                &baddies,
                &cannons,
                &bin,
            );
            collisions_acc
        })
        // Stitch together sub-collections
        .reduce(
            || {
                let mut collisions_init = Collisions::new();
                collisions_init.insert(CollisionKind::BaddieWall, CollisionPairs::new());
                collisions_init.insert(CollisionKind::BulletBaddie, CollisionPairs::new());
                collisions_init.insert(CollisionKind::BulletWall, CollisionPairs::new());
                collisions_init.insert(CollisionKind::BaddieCannon, CollisionPairs::new());
                collisions_init
            },
            |mut acc, c_sub| {
                for (collision_kind, entries) in c_sub {
                    let acc_collisionpairs = acc.get_mut(&collision_kind).unwrap();
                    for collision in entries {
                        acc_collisionpairs.insert(collision);
                    }
                }
                acc
            },
        );
    collisions
}

/// Calculates grid bin size by taking the biggest object's horiz/vert span (assumes uniform size by kind)
/// According to radius circumscribed by rotation
/// Down to a minimum size (to avoid diminishing perf)
fn calc_bin_size(
    walls: &GeomRefMap,
    baddies: &GeomRefMap,
    bullets: &GeomRefMap,
    cannons: &GeomRefMap,
) -> i32 {
    let default = 250;
    let x = (walls
        .iter()
        .take(1)
        .chain(baddies.iter().take(1))
        .chain(bullets.iter().take(1))
        .chain(cannons.iter().take(1))
        .map(|(_, geom)| box_side_len_sqr(&geom))
        .max()
        .unwrap_or(default) as f32)
        .sqrt();
    let res = (x * std::f32::consts::SQRT_2) as i32;
    std::cmp::max(res, default)
    // Uses a small optimization there - compares squares and only computes a single sqrt at the end
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
    cannon_map: SpatialMap,
    #[allow(unused)]
    cannon_index: SpatialIndex,
    handlers: CollisionHandlers<'a>,
    /// Bin size for spatial hashmap (square grid).
    /// 10000 / 1000 => 10 * 10 grid
    grid_bin_size: i32,
}

impl<'a> CollisionSystem<'a> {
    pub fn new(
        walls: &GeomRefMap,
        baddies: &GeomRefMap,
        bullets: &GeomRefMap,
        cannons: &GeomRefMap,
        baddie_wall_handler: CollisionHandler<'a>,
        bullet_wall_handler: CollisionHandler<'a>,
        bullet_baddie_handler: CollisionHandler<'a>,
        baddie_cannon_handler: CollisionHandler<'a>,
    ) -> Self {
        // build hashmaps from object geometries
        let grid_bin_size = calc_bin_size(walls, baddies, bullets, cannons);
        let (wall_map, wall_index) = build_map(walls, grid_bin_size);
        let (baddie_map, baddie_index) = build_map(baddies, grid_bin_size);
        let (bullet_map, bullet_index) = build_map(bullets, grid_bin_size);
        let (cannon_map, cannon_index) = build_map(cannons, grid_bin_size);

        let mut handlers = CollisionHandlers::new();
        handlers.insert(CollisionKind::BaddieWall, baddie_wall_handler);
        handlers.insert(CollisionKind::BulletBaddie, bullet_baddie_handler);
        handlers.insert(CollisionKind::BulletWall, bullet_wall_handler);
        handlers.insert(CollisionKind::BaddieCannon, baddie_cannon_handler);

        Self {
            wall_map,
            wall_index,
            baddie_map,
            baddie_index,
            bullet_map,
            bullet_index,
            cannon_map,
            cannon_index,
            handlers,
            grid_bin_size,
        }
    }

    /// Check collisions and run appropriate handlers
    pub fn process(
        &mut self,
        wall_geoms: &GeomRefMap,
        baddie_geoms: &GeomRefMap,
        bullet_geoms: &GeomRefMap,
        cannon_geoms: &GeomRefMap,
    ) {
        let collisions = detect_collisions(
            (&self.wall_map, wall_geoms),
            (&self.baddie_map, baddie_geoms),
            (&self.bullet_map, bullet_geoms),
            (&self.cannon_map, cannon_geoms),
            self.grid_bin_size,
        );

        // Can't parallelize this because the closures close over mutable data.
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
        let actual = grid_hash(&[vertex, vertex, vertex, vertex, vertex], 1000);

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
        let actual = grid_hash(&vertices, 1000);

        assert_eq!(expected, actual);
    }

    /// 2 walls with some occupying some bins in common - build map and index
    #[test]
    fn build_map_2walls_some_common_bins() {
        // Arrange - 2 walls in bin 11
        let obj_factory = ObjectFactory::new(1000);
        let (wall1, _, wall1_geom, _) = obj_factory.make_wall((1200, 1200));
        let w1_bins_expected = Bins::from_iter([0, 1, 10, 11].iter().cloned());
        let (wall2, _, wall2_geom, _) = obj_factory.make_wall((1700, 1700));
        let w2_bins_expected = Bins::from_iter([11, 12, 21, 22].iter().cloned());
        let walls_geoms: GeomRefMap =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();
        let expected = HashSet::from_iter([wall1.get_id(), wall2.get_id()].iter().cloned());
        // Act
        let (wall_map, wall_index) = build_map(&walls_geoms, 1000);

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
        let (wall1, _, wall1_geom, _) = obj_factory.make_wall((1200, 1200));
        let (wall2, _, wall2_geom, _) = obj_factory.make_wall((1700, 1700));
        let walls_geoms: GeomRefMap =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();

        // colliding baddie:
        let (baddie1, _, baddie1_geom, _) = obj_factory.make_baddie((1200, 1200), (0, 0), 0.0);
        // not colliding baddie:
        let (baddie2, _, baddie2_geom, _) = obj_factory.make_baddie((0, 0), (0, 0), 0.0);
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
            &dummy_geoms,
            Box::new(baddie_wall_handler),
            Box::new(dummy_handler),
            Box::new(dummy_handler),
            Box::new(dummy_handler),
        );
        // Act
        collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms, &dummy_geoms);

        // Assert - see handler, above
    }

    #[test]
    fn collision_can_mutate_baddie() {
        // Arrange - 1 wall, 1 baddies, colliding, plus associated baddie_wall_handler
        let obj_factory = ObjectFactory::new(1000);
        let (wall, _, wall_geom, _) = obj_factory.make_wall((1200, 1200));
        let walls_geoms: GeomRefMap = [(wall.get_id(), &wall_geom)].iter().cloned().collect();

        let (baddie, mut baddie_shape, baddie_geom, _) =
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
                &dummy_geoms,
                Box::new(baddie_wall_handler),
                Box::new(dummy_handler),
                Box::new(dummy_handler),
                Box::new(dummy_handler),
            );
            // Act
            collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms, &dummy_geoms);
        }

        // Assert
        assert_eq!(*baddie_shape.get_vel(), (-1000, 0));
    }

    #[test]
    fn calc_bin_count() {
        let bin_count_expected = 121;
        let bin_count_actual = super::calc_bin_count(1000);
        assert_eq!(bin_count_actual, bin_count_expected);
    }
}
