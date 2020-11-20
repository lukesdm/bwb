//! Geometry and math operations

/// A vector of (x, y)
pub type Vector = (i32, i32);

/// A point of (x, y)
pub type P = Vector;

/// A vertex of (x, y)
pub type Vertex = Vector;

// Might be tempting to combine the above types, but conceptually they are different things, e.g. the centre point of a box is not a vertice.
// P and Vertex can be described by position vectors though, hence just alias the same. // TODO: Use newtype

/// An interval/range of (Min, Max)
pub type MinMax = (i32, i32);

/// Object geometry. All objects are boxes (the first vertex is repeated to close the shape).
pub type Geometry = [Vertex; 5];

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Rotate point `p` around center `c` by `angle` radians (in-place)
/// Based on https://stackoverflow.com/a/2259502
pub fn rotate(p: &mut P, c: &P, angle: f32) {
    let sin = angle.sin();
    let cos = angle.cos();

    let (px, py) = *p;
    let (cx, cy) = c;

    // Move point to origin
    let temp_x = (px - cx) as f32;
    let temp_y = (py - cy) as f32;

    // Calculate rotation
    let rx = temp_x * cos - temp_y * sin;
    let ry = temp_x * sin + temp_y * cos;

    // Move rotated point back
    p.0 = cx + rx as i32;
    p.1 = cy + ry as i32;
}

/// Scale the vector `v` by constant `a`
pub fn scale(v: Vector, a: i32) -> Vector {
    (a * v.0, a * v.1)
}

/// Calculate the edge vector between v1 and v2
fn edge(v1: Vertex, v2: Vertex) -> Vector {
    (v2.0 - v1.0, v2.1 - v1.1)
}

/// Calculate a perpendicular vector to that supplied.
/// Note: doesn't normalize to length 1
fn normal(vector: Vector) -> Vector {
    // "​To calculate a perpendicular vector, swap the x and y components, then negate the x components"
    // - http://programmerart.weebly.com/separating-axis-theorem.html
    (-vector.1, vector.0)
}

/// Calculate the dot product of two vectors.
fn dotprod(vector1: Vector, vector2: Vector) -> i32 {
    vector1.0 * vector2.0 + vector1.1 * vector2.1
}

/// Returns true if the given ranges overlap
/// Note: Inclusive i.e. returns true if range1 max = 3 and range2 min = 3
fn check_overlap(range1: MinMax, range2: MinMax) -> bool {
    let (r1min, r1max) = range1;
    let (r2min, r2max) = range2;

    if (r2min < r1min && r2max < r1min) || (r2min > r1min && r2min > r1max) {
        false
    } else {
        true
    }
}

/// Determines whether `current` contains `input`, and if not then expands to include it.
fn build_range(current: MinMax, input: i32) -> MinMax {
    (
        std::cmp::min(current.0, input),
        std::cmp::max(current.1, input),
    )
}

/// Calculate the min and max values of the projection of the polygon vertices onto the given normal
fn calc_projected_range(poly: &[P], normal: Vector) -> MinMax {
    poly.iter()
        // project onto normal
        .map(|vertex| dotprod(*vertex, normal))
        // fold into (min, max) tuple
        .fold((std::i32::MAX, std::i32::MIN), |acc, projected| {
            build_range(acc, projected)
        })
    // (There's a minmax function in Itertools crate that could've been used for that)
}

/// Check whether there is a collision (i.e. intersection) between the given polygons, using the Separating Axis Theorem.
/// * Only works with **convex** polygons
/// * The polygons should be constructed such that the first vertex is repeated at the end, indicating a closed shape.
/// (Otherwise the edge calc will have to be tweaked)
/// * Doesn't calculate intersection points though
/// Based on http://programmerart.weebly.com/separating-axis-theorem.html
pub fn is_collision(poly1: &[P], poly2: &[P]) -> bool {
    assert_eq!(poly1.first(), poly1.last());
    assert_eq!(poly2.first(), poly2.last());

    // The algorithm is given as:
    // 1. Calculate perpendicular vectors for all edges.
    // 2. Project all vertices from the Polyhedra onto each perpendicular vector, one perpendicular vector at a time.
    // 3. Check if the projections overlap.

    // An edge vector is the difference between a vertex and its predecessor,
    // so no need to store those. So, just iterate over vertices?
    // Start at iv=1 as the first edge is P1 - P0
    for poly in &[poly1, poly2] {
        for iv in 1..poly.len() {
            let edge = edge(poly[iv - 1], poly[iv]);

            let normal = normal(edge);

            // For each polygon, project vertices onto normal and check if they overlap.
            // I.e. whether the min and max for p1 overlaps with min and max for p2.
            // If they don't then the polygons don't intersect.
            let poly1_range = calc_projected_range(poly1, normal);
            let poly2_range = calc_projected_range(poly2, normal);

            if !check_overlap(poly1_range, poly2_range) {
                return false;
            }
        }
    }
    // No breaks therefore polygons intersect
    true
}

pub fn direction_vector(direction: Direction) -> Vector {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}

/// Calculates the square of a box's side length. Assumes square box.
pub fn box_side_len_sqr(geom: &Geometry) -> i32 {
    let (x0, y0) = geom[0];
    let (x1, y1) = geom[1];
    let dx = x1 - x0;
    let dy = y1 - y0;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    #[test]
    fn geom_edge_simple() {
        // Arrange
        let v1 = (3, 2);
        let v2 = (7, 7);
        let edge_expected = (4, 5);

        // Act
        let edge_actual = super::edge(v1, v2);

        // Assert
        assert_eq!(edge_actual, edge_expected);
    }

    #[test]
    fn geom_normal_simple() {
        let vector = (4, 5);
        let normal_expected = (-5, 4);

        let normal_actual = super::normal(vector);

        assert_eq!(normal_actual, normal_expected);
    }

    #[test]
    fn geom_dotprod_simple() {
        let vector1 = (7, 0);
        let vector2 = (13, 10);
        let dotprod_expected = 91;

        let dotprod_actual = super::dotprod(vector1, vector2);

        assert_eq!(dotprod_actual, dotprod_expected);
    }

    #[test]
    fn calc_projected_range_simple() {
        let input = [5, 1, 2];
        let expected = (1, 5);

        let actual = input
            .iter()
            .fold((std::i32::MAX, std::i32::MIN), |acc, projected| {
                super::build_range(acc, *projected)
            });

        assert_eq!(actual, expected);
    }

    #[test]
    fn check_overlap_1() {
        // ---
        //     ---
        let r1 = (1, 3);
        let r2 = (4, 6);
        let overlap_expected = false;

        let overlap_actual = super::check_overlap(r1, r2);

        assert_eq!(overlap_actual, overlap_expected);
    }

    #[test]
    fn check_overlap_2() {
        //     ---
        // ---
        let r1 = (4, 6);
        let r2 = (1, 3);
        let overlap_expected = false;

        let overlap_actual = super::check_overlap(r1, r2);

        assert_eq!(overlap_actual, overlap_expected);
    }

    #[test]
    fn check_overlap_3() {
        // ---
        //  -
        let r1 = (1, 3);
        let r2 = (2, 2);
        let overlap_expected = true;

        let overlap_actual = super::check_overlap(r1, r2);

        assert_eq!(overlap_actual, overlap_expected);
    }

    #[test]
    fn check_overlap_4() {
        // ---
        //    ---
        let r1 = (1, 3);
        let r2 = (3, 5);
        let overlap_expected = true;

        let overlap_actual = super::check_overlap(r1, r2);

        assert_eq!(overlap_actual, overlap_expected);
    }

    #[test]
    fn colliding_simple1() {
        // Arrange
        let poly1 = [(1, 1), (3, 1), (3, 3), (1, 3), (1, 1)];
        let poly2 = [(2, 2), (4, 2), (4, 4), (2, 4), (2, 2)];
        let expected = true;

        // Act
        let result = super::is_collision(&poly1, &poly2);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn colliding_simple2() {
        // Arrange
        let poly1 = [(1, 1), (3, 1), (3, 3), (1, 3), (1, 1)];
        let poly2 = [(4, 4), (6, 4), (6, 6), (4, 6), (4, 4)];
        let expected = false;

        // Act
        let result = super::is_collision(&poly1, &poly2);

        // Assert
        assert_eq!(result, expected);
    }

    /// Regression test for bug resulting in false positive
    #[test]
    fn colliding_nearmiss() {
        // Arrange
        let poly1 = [
            (2260, 2628),
            (3232, 2400),
            (3460, 3372),
            (2488, 3600),
            (2260, 2628),
        ];
        let poly2 = [
            (3098, 3654),
            (4006, 3238),
            (4422, 4146),
            (3514, 4562),
            (3098, 3654),
        ];
        let expected = false;

        // Act
        let result = super::is_collision(&poly1, &poly2);

        // Assert
        assert_eq!(result, expected);
    }
}
