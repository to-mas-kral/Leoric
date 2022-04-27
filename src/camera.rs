use glam::{Mat4, Vec3};

/// A component encapsulating the camera transformations
pub struct Camera {
    /// Position of the camera
    pos: Vec3,
    /// The direction vector
    dir: Vec3,
    /// The 'up' vector
    up: Vec3,
    /// Move speed
    pub move_speed: f32,
    /// Look sensitivity
    pub look_sensitivity: f32,
    /// Last x position of the mouse
    current_x: f32,
    /// Last y position of the mouse
    current_y: f32,
    /// Horizontal angle from center
    azimuth: f32,
    /// Vertical angle from center
    zenith: f32,
    /// Signals that the view transformation needs to be recomputed
    changed: bool,
    /// Cache of the view matrix
    view_matrix: Mat4,
}

impl Camera {
    /// Creates the camera
    pub fn new(
        pos: Vec3,
        move_speed: f32,
        look_sensitivity: f32,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        Self {
            pos,
            dir: Vec3::new(0., 0., -1.),
            up: Vec3::new(0., 1., 0.),
            move_speed,
            look_sensitivity,
            current_x: window_width as f32 / 2.,
            current_y: window_height as f32 / 2.,
            azimuth: 0.,
            zenith: 0.,
            changed: true,
            view_matrix: Mat4::IDENTITY,
        }
    }

    /// Returns the view matrix (either cached or recomputed)
    pub fn view_mat(&mut self) -> Mat4 {
        if self.changed {
            self.changed = false;
            self.view_matrix = Mat4::look_at_rh(self.pos, self.pos + self.dir, self.up);
        }

        self.view_matrix
    }

    /// Sets the position of the camera
    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
        self.changed = true;
    }

    /// Moves the camera forward
    pub fn move_forward(&mut self, d: f32) {
        self.pos += self.dir * d * self.move_speed;
        self.changed = true;
    }

    /// Moves the camera backward
    pub fn move_backward(&mut self, d: f32) {
        self.move_forward(-d);
    }

    /// Moves the camera sideways to the right
    pub fn strafe_right(&mut self, d: f32) {
        let dir = self.dir.cross(self.up);
        self.pos += dir * d * self.move_speed;
        self.changed = true;
    }

    /// Moves the camera sideways to the left
    pub fn strafe_left(&mut self, d: f32) {
        self.strafe_right(-d);
    }

    /// Updates the latest (x,y) mouse position
    pub fn set_x_y(&mut self, new_x: f32, new_y: f32) {
        self.current_x = new_x;
        self.current_y = new_y;
    }

    /// Update the (x, y) mouse position and update the azimuth and zenith
    pub fn adjust_look(&mut self, new_x: f32, new_y: f32) {
        let dx = new_x - self.current_x;
        let dy = self.current_y - new_y;

        self.current_x = new_x;
        self.current_y = new_y;

        let x_offset = dx * self.look_sensitivity;
        let y_offset = dy * self.look_sensitivity;

        self.azimuth += x_offset;
        self.zenith += y_offset;

        self.zenith = self.zenith.clamp(-89., 89.);

        self.adjust_dir();
    }

    /// Update the azimuth and zenith
    fn adjust_dir(&mut self) {
        let rad_azimuth = (270. + self.azimuth).to_radians();
        let rad_zenith = self.zenith.to_radians();

        let x = rad_azimuth.cos() * rad_zenith.cos();
        let y = rad_zenith.sin();
        let z = rad_azimuth.sin() * rad_zenith.cos();

        self.dir = Vec3::new(x as f32, y as f32, z as f32).normalize();
        self.changed = true;
    }
}
