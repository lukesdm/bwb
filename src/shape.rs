/// Shape (currently, a square of side `size`) spatial/world-state
pub struct Shape {
    center: (i32, i32),
    size: u32,
    /// Velocity, in units per second
    vel: (i32, i32),
    /// Current rotation about centre, in radians
    rotation: f32,
    /// Rotational speed, in radians/sec
    angular_velocity: f32,
}

impl Shape {
    pub fn new(
        center: (i32, i32),
        size: u32,
        vel: (i32, i32),
        rotation: f32,
        angular_velocity: f32,
    ) -> Self {
        assert!(size > 0);
        Self {
            center,
            size,
            vel,
            rotation,
            angular_velocity,
        }
    }

    pub fn set_center(&mut self, new_center: (i32, i32)) {
        self.center = new_center;
    }

    pub fn reverse(&mut self) {
        let (vx, vy) = self.vel;
        self.vel = (-vx, -vy);
    }
}