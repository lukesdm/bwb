use crate::game_logic::{Wall, GRID_HEIGHT, GRID_WIDTH};
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

//type SpatialMap = HashMap<u32, Vec<Wall>>;
/// Just center points for now. TODO: Expand to polys
type SpatialMap = HashMap<i32, Vec<P>>;

/// Build map of bin -> object list
fn build_map(walls: &Vec<Wall>) -> SpatialMap {
    let mut wall_map = SpatialMap::new();
    //let mut wall_map = [vec![]; 100];
    for wall in walls {
        wall_map
            .entry(grid_hash(wall.get_center()))
            .and_modify(|e| e.push(wall.get_center()))
            .or_insert(vec![wall.get_center()]);
    }
    wall_map
}

// TODO: Implement type
type SpatialIndex = i32;

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

    /// 2 walls in same bin
    #[test]
    fn build_map_2walls_1bin() {
        // Arrange - 2 walls in bin 11
        let wall1_center = (1200, 1200);
        let wall2_center = (1700, 1700);
        let walls = vec![Wall::new(wall1_center), Wall::new(wall2_center)];

        // Act
        let wall_map = build_map(&walls);

        // Assert
        assert_eq!(wall_map.get(&11), Some(&vec![wall1_center, wall2_center]));
    }
}
