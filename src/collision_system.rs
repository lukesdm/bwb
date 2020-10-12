use crate::game_logic::{GameObject, Wall, GRID_HEIGHT, GRID_WIDTH};
use crate::geometry::P;
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
type SpatialMap = HashMap<i32, Vec<P>>;
type SpatialIndex = HashMap<P, i32>;

/// Build map of bin -> object list, and associated index (currently using center point, as a rough way to ID an object)
fn build_map(objects: &Vec<&GameObject>) -> (SpatialMap, SpatialIndex) {
    let mut object_map = SpatialMap::new();
    let mut object_index = SpatialIndex::new();
    for obj in objects {
        let grid_bin = grid_hash(obj.get_center());
        object_map
            .entry(grid_bin)
            .and_modify(|e| e.push(obj.get_center()))
            .or_insert(vec![obj.get_center()]);

        let existing = object_index.insert(obj.get_center(), grid_bin);
        // theoretical problem to watch out for (but not yet a real concern)
        if let Some(p_existing) = existing {
            println!("Duplicate object detected with center:  {}", p_existing);
        }
    }
    (object_map, object_index)
}

/// Detects collisions and runs handlers as appropriate
pub struct CollisionSystem {
    bullet_map: SpatialMap,
    bullet_index: SpatialIndex,
    wall_map: SpatialMap,
    wall_index: SpatialIndex,
    baddie_map: SpatialMap,
    baddie_index: SpatialIndex,
}

// TODO: Implement
// impl CollisionSystem {
//     pub fn new(world: &World) -> Self {
//         // build hashmaps from world

//         let bullet_map = 0;
//         let bullet_index = 0;
//         let wall_map = 0;
//         let wall_index = 0;
//         let baddie_map = 0;
//         let baddie_index = 0;

//         Self {
//             bullet_map,
//             bullet_index,
//             wall_map,
//             wall_index,
//             baddie_map,
//             baddie_index,
//         }
//     }

//     pub fn update(world: &mut World) {
//         // update hashmaps and check collisions
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_hash_1_2() {
        let actual = grid_hash((1000, 2000));

        assert_eq!(21, actual);
    }

    /// 2 walls in same bin - build map
    #[test]
    fn build_map_2walls_1bin() {
        // Arrange - 2 walls in bin 11
        let wall1_center = (1200, 1200);
        let wall2_center = (1700, 1700);
        let walls = vec![Wall::new(wall1_center), Wall::new(wall2_center)];

        // Act
        let objects: Vec<&GameObject> = walls.iter().map(|w| &w.0).collect();
        let (wall_map, _) = build_map(&objects);

        // Assert
        assert_eq!(wall_map.get(&11), Some(&vec![wall1_center, wall2_center]));
    }

    /// 2 walls in same bin - index
    #[test]
    fn build_map_index_2walls_1bin() {
        // Arrange - 2 walls in bin 11
        let wall1_center = (1200, 1200);
        let wall2_center = (1700, 1700);
        let walls = vec![Wall::new(wall1_center), Wall::new(wall2_center)];

        // Act
        let objects: Vec<&GameObject> = walls.iter().map(|w| &w.0).collect();
        let (_, wall_index) = build_map(&objects);

        // Assert
        assert_eq!(wall_index.get(&wall1_center), Some(&11));
        assert_eq!(wall_index.get(&wall2_center), Some(&11));
    }


}
