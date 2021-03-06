use crate::geometry::{direction_vector, scale, Direction, Vector, P};

/// Shape (currently, a square of side `size`) spatial/world-state
pub struct Shape {
    center: P,
    center_prev: P,
    size: u32,
    /// Velocity, in units per second
    vel: Vector,
    /// Current rotation about centre, in radians
    rotation: f32,
    rotation_prev: f32,
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
            center_prev: center,
            size,
            vel,
            rotation,
            rotation_prev: rotation,
            angular_velocity,
        }
    }

    pub fn get_size(&self) -> &u32 {
        &self.size
    }

    pub fn get_center(&self) -> &P {
        &self.center
    }

    pub fn set_center(&mut self, new_center: P) {
        self.center_prev = self.center;
        self.center = new_center;
    }

    pub fn get_rotation(&self) -> &f32 {
        &self.rotation
    }

    pub fn get_vel(&self) -> &Vector {
        &self.vel
    }

    /// Sets the velocity vector according to the given direction
    pub fn set_movement(&mut self, direction: Direction) {
        self.vel = scale(direction_vector(direction), 1000); // COULDDO: const/parameterise
    }

    /// Updates shape rotation, given a time-step (seconds)
    pub fn rotate(&mut self, dt_s: f32) {
        // SHOULDDO: Wrap this to [-2*PI, +2*PI], otherwise there might be jumps on overflow
        self.rotation_prev = self.rotation;
        self.rotation += self.angular_velocity * dt_s;
    }

    /// Reverses the velocity vector
    pub fn reverse(&mut self) {
        let (vx, vy) = self.vel;
        self.vel = (-vx, -vy);
        self.angular_velocity = -self.angular_velocity;
    }

    /// Moves the shape back to its previous position
    pub fn move_back(&mut self) {
        self.center = self.center_prev;
        self.rotation = self.rotation_prev;
    }
}
