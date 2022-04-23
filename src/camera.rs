use glam::{Mat4, Vec3};

pub struct Camera {
    pos: Vec3,
    dir: Vec3,
    up: Vec3,

    pub move_speed: f32,
    pub look_sensitivity: f32,

    current_x: f32,
    current_y: f32,
    azimut: f32,
    zenit: f32,

    changed: bool,

    view_matrix: Mat4,
}

impl Camera {
    pub fn new(pos: Vec3, move_speed: f32, look_sensitivity: f32, width: u32, height: u32) -> Self {
        Self {
            pos,
            dir: Vec3::new(0., 0., -1.),
            up: Vec3::new(0., 1., 0.),
            move_speed,
            look_sensitivity,
            current_x: width as f32 / 2.,
            current_y: height as f32 / 2.,
            azimut: 0.,
            zenit: 0.,
            changed: true,
            view_matrix: Mat4::IDENTITY,
        }
    }

    pub fn get_view_mat(&mut self) -> Mat4 {
        if self.changed {
            self.changed = false;
            self.view_matrix = Mat4::look_at_rh(self.pos, self.pos + self.dir, self.up);
        }

        self.view_matrix
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
        self.changed = true;
    }

    pub fn move_forward(&mut self, d: f32) {
        self.pos += self.dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn move_backward(&mut self, d: f32) {
        self.move_forward(-d);
    }

    pub fn strafe_right(&mut self, d: f32) {
        let dir = self.dir.cross(self.up);
        self.pos += dir * d * self.move_speed;
        self.changed = true;
    }

    pub fn strafe_left(&mut self, d: f32) {
        self.strafe_right(-d);
    }

    pub fn set_x_y(&mut self, new_x: f32, new_y: f32) {
        self.current_x = new_x;
        self.current_y = new_y;
    }

    pub fn adjust_look(&mut self, new_x: f32, new_y: f32) {
        let dx = new_x - self.current_x;
        let dy = self.current_y - new_y;

        self.current_x = new_x;
        self.current_y = new_y;

        let x_offset = dx * self.look_sensitivity;
        let y_offset = dy * self.look_sensitivity;

        self.azimut += x_offset;
        self.zenit += y_offset;

        self.zenit = self.zenit.clamp(-89., 89.);

        self.adjust_dir();
    }

    fn adjust_dir(&mut self) {
        let rad_azimut = (270. + self.azimut).to_radians();
        let rad_zenit = self.zenit.to_radians();

        let x = rad_azimut.cos() * rad_zenit.cos();
        let y = rad_zenit.sin();
        let z = rad_azimut.sin() * rad_zenit.cos();

        self.dir = Vec3::new(x as f32, y as f32, z as f32).normalize();
        self.changed = true;
    }
}
