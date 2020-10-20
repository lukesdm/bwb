use crate::entity::EntityId;
use crate::game_logic::{GRID_HEIGHT, GRID_WIDTH};
use crate::geometry::{is_collision, Geometry, P};
//use crate::world::ObjectGeometries;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// Bin size for spatial hashmap (square grid).
/// 10000 / 1000 => 10 * 10 grid
const GRID_BIN_SIZE: i32 = 1000;

// TODO: Get rid of this once per-vertex hashing is implemented.
fn get_center(geom: &Geometry) -> P {
    let mut cx = 0;
    let mut cy = 0;
    // Average 4 corner vertices, ignoring 5th (duplicate)
    for p in geom.iter().take(4) {
        let (x, y) = p;
        cx += x;
        cy += y;
    }

    (cx / 4, cy / 4)
}

/// Spatial hash. For now, just use a center point. TODO: use box/circle geometry
fn grid_hash(center: P) -> i32 {
    assert!(GRID_WIDTH == GRID_HEIGHT);
    let (cx, cy) = center;
    let bx = cx / GRID_BIN_SIZE;
    let by = cy / GRID_BIN_SIZE;
    bx + by * GRID_WIDTH as i32 / GRID_BIN_SIZE
}

type ObjectGeometries<'a> = HashMap<EntityId, &'a Geometry>;

/// Just center points for now. TODO: Expand to handle polys + entity IDs
type SpatialMap = HashMap<i32, HashSet<EntityId>>;
type SpatialIndex = HashMap<EntityId, i32>;

/// Build map of bin -> object list, and associated index (currently using center point, as a rough way to ID an object)
fn build_map(geometries: &ObjectGeometries) -> (SpatialMap, SpatialIndex) {
    let mut object_map = SpatialMap::new();
    let mut object_index = SpatialIndex::new();
    for geometry in geometries {
        //let (id, center, _) = *obj;
        let (id, vertices) = geometry;
        let center = get_center(vertices);
        let grid_bin = grid_hash(center); // TODO: use all geometry
        object_map
            .entry(grid_bin)
            .and_modify(|e| {
                e.insert(*id);
            })
            .or_insert(HashSet::from_iter([*id].iter().cloned()));

        let existing = object_index.insert(*id, grid_bin);
        // theoretical problem to watch out for (but not yet a real concern)
        if let Some(p_existing) = existing {
            println!("Duplicate object detected with id:  {}", p_existing);
        }
    }
    (object_map, object_index)
}

type CollisionHandler<'a> = Box<dyn 'a + FnMut(EntityId, EntityId) -> ()>;

/// Detects collisions and runs handlers as appropriate
pub struct CollisionSystem<'a> {
    wall_map: SpatialMap,
    wall_index: SpatialIndex,
    baddie_map: SpatialMap,
    baddie_index: SpatialIndex,
    bullet_map: SpatialMap,
    bullet_index: SpatialIndex,
    baddie_wall_handler: CollisionHandler<'a>,
    bullet_wall_handler: CollisionHandler<'a>,
}

impl<'a> CollisionSystem<'a> {
    pub fn new(
        walls: &ObjectGeometries,
        baddies: &ObjectGeometries,
        bullets: &ObjectGeometries,
        baddie_wall_handler: CollisionHandler<'a>,
        bullet_wall_handler: CollisionHandler<'a>,
    ) -> Self {
        // build hashmaps from object geometries

        let (wall_map, wall_index) = build_map(walls);
        let (baddie_map, baddie_index) = build_map(baddies);
        let (bullet_map, bullet_index) = build_map(bullets);

        Self {
            wall_map,
            wall_index,
            baddie_map,
            baddie_index,
            bullet_map,
            bullet_index,
            baddie_wall_handler,
            bullet_wall_handler,
        }
    }

    /// Check collisions and run appropriate handlers
    // TODO: Handle geometry spanning multiple bins
    pub fn process(
        &mut self, // TODO: Check - needs to be mutable?
        walls: &ObjectGeometries,
        baddies: &ObjectGeometries,
        bullets: &ObjectGeometries,
    ) {
        let bin_count = 100; // TODO: Calculate
        for i in 0..bin_count {
            // Walls-Baddies
            if let Some(wall_ids) = self.wall_map.get(&i) {
                if let Some(baddie_ids) = self.baddie_map.get(&i) {
                    for wall_id in wall_ids {
                        for baddie_id in baddie_ids {
                            let wall_geom = walls.get(wall_id).unwrap();
                            let baddie_geom = baddies.get(baddie_id).unwrap();
                            if is_collision(*wall_geom, *baddie_geom) {
                                (self.baddie_wall_handler)(*baddie_id, *wall_id);
                            }
                        }
                    }
                }
            }

            // Bullets-Walls
            if let Some(bullet_ids) = self.bullet_map.get(&i) {
                for bullet_id in bullet_ids {
                    if let Some(wall_ids) = self.wall_map.get(&i) {
                        for wall_id in wall_ids {
                            let bullet_geom = bullets.get(bullet_id).unwrap();
                            let wall_geom = walls.get(wall_id).unwrap();
                            if is_collision(*bullet_geom, *wall_geom) {
                                (self.bullet_wall_handler)(*bullet_id, *wall_id)
                            }
                        }
                    }
                }
            }
        }
    }

    // pub fn update(world: &mut World) {
    //     // update hashmaps and check collisions
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{make_baddie, make_wall};

    #[test]
    fn grid_hash_1_2() {
        let actual = grid_hash((1000, 2000));

        assert_eq!(21, actual);
    }

    /// 2 walls in same bin - build map and index
    #[test]
    fn build_map_2walls_1bin() {
        // Arrange - 2 walls in bin 11
        let bin_expected = 11;
        let (wall1, _, wall1_geom) = make_wall((1200, 1200));
        let (wall2, _, wall2_geom) = make_wall((1700, 1700));
        let walls_geoms: ObjectGeometries =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();
        let expected = HashSet::from_iter([wall1.get_id(), wall2.get_id()].iter().cloned());
        // Act
        let (wall_map, wall_index) = build_map(&walls_geoms);

        // Assert - map
        assert_eq!(wall_map.get(&bin_expected).unwrap(), &expected);

        // Assert - index
        assert_eq!(wall_index.get(&wall1.get_id()), Some(&bin_expected));
        assert_eq!(wall_index.get(&wall2.get_id()), Some(&bin_expected));
    }

    #[test]
    fn collision_static_simple() {
        // Arrange - 2 walls, 2 baddies, 1 of each colliding, plus associated handler
        let (wall1, _, wall1_geom) = make_wall((1200, 1200));
        let (wall2, _, wall2_geom) = make_wall((1700, 1700));
        let walls_geoms: ObjectGeometries =
            [(wall1.get_id(), &wall1_geom), (wall2.get_id(), &wall2_geom)]
                .iter()
                .cloned()
                .collect();

        // colliding baddie:
        let (baddie1, _, baddie1_geom) = make_baddie((1200, 1200), (0, 0), 0.0);
        // not colliding baddie:
        let (baddie2, _, baddie2_geom) = make_baddie((0, 0), (0, 0), 0.0);
        let baddies_geoms: ObjectGeometries = [
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
        let dummy_geoms = &ObjectGeometries::new();
        let dummy_handler = |_: EntityId, _: EntityId| ();
        let mut collision_system = CollisionSystem::new(
            &walls_geoms,
            &baddies_geoms,
            &dummy_geoms,
            Box::new(baddie_wall_handler),
            Box::new(dummy_handler),
        );
        // Act
        collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms);

        // Assert - see handler, above
    }

    #[test]
    fn collision_can_mutate_baddie() {
        // Arrange - 1 wall, 1 baddies, colliding, plus associated baddie_wall_handler
        let (wall, _, wall_geom) = make_wall((1200, 1200));
        let walls_geoms: ObjectGeometries = [(wall.get_id(), &wall_geom)].iter().cloned().collect();

        let (baddie, mut baddie_shape, baddie_geom) = make_baddie((1200, 1200), (1000, 0), 0.0);
        let baddies_geoms: ObjectGeometries =
            [(baddie.get_id(), &baddie_geom)].iter().cloned().collect();

        let baddie_wall_handler = |baddie_id: EntityId, wall_id: EntityId| {
            assert_eq!(wall_id, wall.get_id());
            assert_eq!(baddie_id, baddie.get_id());
            baddie_shape.reverse();
        };
        let dummy_geoms = &ObjectGeometries::new();
        let dummy_handler = |_: EntityId, _: EntityId| ();
        // Scope needed here for collision system - need to return borrowed references before assert
        {
            let mut collision_system = CollisionSystem::new(
                &walls_geoms,
                &baddies_geoms,
                &dummy_geoms,
                Box::new(baddie_wall_handler),
                Box::new(dummy_handler),
            );
            // Act
            collision_system.process(&walls_geoms, &baddies_geoms, &dummy_geoms);
        }

        // Assert
        assert_eq!(*baddie_shape.get_vel(), (-1000, 0));
    }
}
