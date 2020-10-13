use crate::game_logic::{Baddie, EntityId, GameObject, Wall, GRID_HEIGHT, GRID_WIDTH};
use crate::geometry::{is_collision, Geometry, P};
use std::collections::HashMap;

/// Bin size for spatial hashmap (square grid).
/// 10000 / 1000 => 10 * 10 grid
const GRID_BIN_SIZE: i32 = 1000;

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
fn build_map(objects: &Vec<&GameObject>) -> (SpatialMap, SpatialIndex) {
    let mut object_map = SpatialMap::new();
    let mut object_index = SpatialIndex::new();
    for obj in objects {
        //let (id, center, _) = *obj;
        let center = obj.get_center();
        let id = obj.get_id();
        let grid_bin = grid_hash(center);
        object_map
            .entry(grid_bin)
            .and_modify(|e| e.push(id))
            .or_insert(vec![id]);

        let existing = object_index.insert(id, grid_bin);
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
pub struct CollisionSystem<'a, THandler>
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
    walls: &'a Vec<&'a GameObject>,
    baddies: &'a Vec<&'a GameObject>,
    //walls: &'a Vec<CollisionData>,
    //baddies: &'a Vec<CollisionData>,
}

impl<'a, THandler> CollisionSystem<'a, THandler>
where
    THandler: Fn(EntityId, EntityId) -> (),
{
    //where THandler: Fn(Wall, Baddie) -> () {
    //pub fn new(walls: &'a Vec<CollisionData>, baddies: &'a Vec<CollisionData>, handler: THandler) -> Self {
    pub fn new(
        walls: &'a Vec<&GameObject>,
        baddies: &'a Vec<&GameObject>,
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
            walls,
            baddies,
        }
    }

    fn get_wall(&self, id: EntityId) -> &GameObject {
        self.walls.iter().find(|w| w.get_id() == id).unwrap()
    }

    fn get_baddie(&self, id: EntityId) -> &GameObject {
        self.baddies.iter().find(|b| b.get_id() == id).unwrap()
    }

    /// Check collisions and run appropriate handlers
    // TODO: Handle geometry spanning multiple bins
    pub fn process(&self) {
        let bin_count = 100; // TODO: Calculate
        for i in 0..bin_count {
            if let Some(wall_ids) = self.wall_map.get(&i) {
                if let Some(baddie_ids) = self.baddie_map.get(&i) {
                    for wall_id in wall_ids {
                        let wall = self.get_wall(*wall_id);
                        for baddie_id in baddie_ids {
                            let baddie = self.get_baddie(*baddie_id);
                            if is_collision(&wall.geometry, &baddie.geometry) {
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
        let wall1_center = (1200, 1200);
        let wall2_center = (1700, 1700);
        let walls = vec![Wall::new(wall1_center), Wall::new(wall2_center)];

        // Act
        let objects: Vec<&GameObject> = walls.iter().map(|w| &w.0).collect();
        let (wall_map, wall_index) = build_map(&objects);

        // Assert - map
        assert_eq!(
            wall_map.get(&bin_expected),
            Some(&vec![walls[0].0.get_id(), walls[1].0.get_id()])
        );
        // Assert - index
        assert_eq!(wall_index.get(&walls[0].0.get_id()), Some(&bin_expected));
        assert_eq!(wall_index.get(&walls[1].0.get_id()), Some(&bin_expected));
    }

    #[test]
    fn collision_static_simple() {
        // Arrange - 2 walls, 2 baddies, 1 of each colliding, plus associated handler
        let wall1 = Wall::new((1200, 1200));
        let wall1_id = wall1.0.get_id();
        let wall2 = Wall::new((1700, 1700));
        let wall2_id = wall2.0.get_id();
        let walls = vec![wall1, wall2];
        let walls: Vec<&GameObject> = walls.iter().map(|w| &w.0).collect();
        let baddie1 = Baddie::new((1200, 1200), (0, 0), 0.0); // => colliding
        let baddie1_id = baddie1.0.get_id();
        let baddie2 = Baddie::new((0, 0), (0, 0), 0.0); // => not colliding
        let baddie2_id = baddie2.0.get_id();
        let baddies = vec![baddie1, baddie2];
        let baddies: Vec<&GameObject> = baddies.iter().map(|b| &b.0).collect();
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
        let collision_system = CollisionSystem::new(&walls, &baddies, handler);
        // Act
        collision_system.process();

        // Assert - see handler, above
    }
}
