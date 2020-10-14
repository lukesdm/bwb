use crate::game_logic::{GRID_HEIGHT, GRID_WIDTH};
use crate::geometry::{is_collision, Geometry, P};
use std::collections::HashMap;
use crate::entity::EntityId;
use crate::world::ObjectGeometries;

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

/// Just center points for now. TODO: Expand to handle polys + entity IDs
type SpatialMap = HashMap<i32, Vec<EntityId>>;
type SpatialIndex = HashMap<EntityId, i32>;

/// Build map of bin -> object list, and associated index (currently using center point, as a rough way to ID an object)
//fn build_map(objects: &Vec<CollisionData>) -> (SpatialMap, SpatialIndex) {
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
            .and_modify(|e| e.push(*id))
            .or_insert(vec![*id]);

        let existing = object_index.insert(*id, grid_bin);
        // theoretical problem to watch out for (but not yet a real concern)
        if let Some(p_existing) = existing {
            println!("Duplicate object detected with id:  {}", p_existing);
        }
    }
    (object_map, object_index)
}

type CenterPoint = P;

//pub type CollisionData = (EntityId, CenterPoint, Geometry);

//type Wall = CollisionData;
//type Baddie = CollisionData;

//type CollisionHandler = Fn(Wall, Baddie) -> ();

/// Detects collisions and runs handlers as appropriate
pub struct CollisionSystem<THandler>
where
    THandler: Fn(EntityId, EntityId) -> (),
{
    //where THandler: Fn(Wall, Baddie) -> () {
    // bullet_map: SpatialMap,
    // bullet_index: SpatialIndex,
    wall_map: SpatialMap,
    wall_index: SpatialIndex,
    baddie_map: SpatialMap,
    baddie_index: SpatialIndex,
    handler: THandler,
    //walls: &'a Vec<CollisionData>,
    //baddies: &'a Vec<CollisionData>,
}

impl<THandler> CollisionSystem<THandler>
where
    THandler: Fn(EntityId, EntityId) -> (),
{
    //where THandler: Fn(Wall, Baddie) -> () {
    //pub fn new(walls: &'a Vec<CollisionData>, baddies: &'a Vec<CollisionData>, handler: THandler) -> Self {
    pub fn new(
        walls: &ObjectGeometries,
        baddies: &ObjectGeometries,
        handler: THandler,
    ) -> Self {
        // build hashmaps from world

        // let bullet_map = 0;
        // let bullet_index = 0;
        let (wall_map, wall_index) = build_map(walls);
        let (baddie_map, baddie_index) = build_map(baddies);

        Self {
            // bullet_map,
            // bullet_index,
            wall_map,
            wall_index,
            baddie_map,
            baddie_index,
            handler,
        }
    }

    /// Check collisions and run appropriate handlers
    // TODO: Handle geometry spanning multiple bins
    pub fn process(&self, walls: &ObjectGeometries, baddies: &ObjectGeometries) {
        let bin_count = 100; // TODO: Calculate
        for i in 0..bin_count {
            if let Some(wall_ids) = self.wall_map.get(&i) {
                if let Some(baddie_ids) = self.baddie_map.get(&i) {
                    for wall_id in wall_ids {
                        for baddie_id in baddie_ids {
                            let wall_geom = walls.get(wall_id).unwrap();
                            let baddie_geom = baddies.get(baddie_id).unwrap();
                            if is_collision(wall_geom, baddie_geom) {
                                (self.handler)(*wall_id, *baddie_id);
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
    use crate::world::{make_wall, make_baddie};

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
        let (wall1_id, _, wall1_geom) = make_wall((1200, 1200));
        let (wall2_id, _ , wall2_geom) = make_wall((1700, 1700));
        let walls_geoms: ObjectGeometries = [ (wall1_id, wall1_geom), (wall2_id, wall2_geom)  ].iter().cloned().collect();

        // Act        
        let (wall_map, wall_index) = build_map(&walls_geoms);

        // Assert - map
        assert_eq!(
            wall_map.get(&bin_expected),
            Some(&vec![wall2.0.get_id(), wall1.0.get_id()]) // Order here is an implementation detail. COULDDO: make order-independent comparison
        );
        // Assert - index
        assert_eq!(wall_index.get(&wall1.0.get_id()), Some(&bin_expected));
        assert_eq!(wall_index.get(&wall2.0.get_id()), Some(&bin_expected));
    }

    #[test]
    fn collision_static_simple() {
        // Arrange - 2 walls, 2 baddies, 1 of each colliding, plus associated handler
        let (wall1_id, _, wall1_geom) = make_wall((1200, 1200));
        let (wall2_id, _, wall2_geom) = make_wall((1700, 1700));
        let walls_geoms: ObjectGeometries = [ (wall1_id, wall1_geom), (wall2_id, wall2_geometry) ].iter().cloned().collect();

        // colliding baddie:
        let (baddie1_id, baddie1_geom) = make_baddie((1200, 1200), (0, 0), 0.0);
        // not colliding baddie:
        let (baddie2_id, baddie2_geom) = make_baddie((0, 0), (0, 0), 0.0);
        let baddies_geoms: ObjectGeometries = [ (baddie1.0.get_id(), baddie1.0.geometry), (baddie2.0.get_id(), baddie2.0.geometry) ].iter().cloned().collect();
        
        let handler = |wall_id: EntityId, baddie_id: EntityId| {
            // Assert - handler called with correct arguments
            assert!(
                (wall_id == wall1_id && baddie_id == baddie1_id)
                    && !(baddie_id == baddie2_id || wall_id == wall2_id)
            )
        };
        // TODO: Figure out how to get stuff into closure
        // let handler = |wall_id: EntityId, baddie_id: EntityId| {
        //     assert!(
        //         wall_id == wall1.0.get_id()
        //             && baddie_id == baddie1.0.get_id()
        //             && !(baddie_id == baddie2.0.get_id())
        //     )
        // };
        let collision_system = CollisionSystem::new(&walls_geoms, &baddies_geoms, handler);
        
        // Act
        collision_system.process(&walls_geoms, &baddies_geoms);

        // Assert - see handler, above
    }

    #[test]
    fn collision_reverse_baddie() {
        // Arrange - 2 walls, 2 baddies, 1 of each colliding, plus associated handler
        //let wall = make_wall((1200, 1200));
        let (wall_id, _, wall_geom) = make_wall((1200, 1200));
        let walls_geoms: ObjectGeometries = [ (wall_id, wall_geom) ].iter().cloned().collect();
        // colliding baddie:
        let mut baddie = make_baddie((1200, 1200), (0, 0), 0.0);

        let handler = |wall_id: EntityId, baddie_id: EntityId| {
            baddie.1.reverse();
        };
        // TODO: Figure out how to get stuff into closure
        // let handler = |wall_id: EntityId, baddie_id: EntityId| {
        //     assert!(
        //         wall_id == wall1.0.get_id()
        //             && baddie_id == baddie1.0.get_id()
        //             && !(baddie_id == baddie2.0.get_id())
        //     )
        // };
        let collision_system = CollisionSystem::new(&walls, &baddies, handler);
        // Act
        collision_system.process(wall_geoms, baddie_geoms);
    }
}
