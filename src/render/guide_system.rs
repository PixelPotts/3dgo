use glam::Vec3;
use super::Instance;

pub struct GuideSystem {
    pub plane_x_pos: i32,  // Position along X axis (for YZ plane)
    pub plane_y_pos: i32,  // Position along Y axis (for XZ plane)  
    pub plane_z_pos: i32,  // Position along Z axis (for XY plane)
    board_size: i32,
}

impl GuideSystem {
    pub fn new(board_size: usize) -> Self {
        let size = board_size as i32;
        Self {
            plane_x_pos: size / 2,  // Start in middle
            plane_y_pos: size / 2,
            plane_z_pos: size / 2,
            board_size: size,
        }
    }

    pub fn move_x(&mut self, delta: i32) {
        self.plane_x_pos = (self.plane_x_pos + delta).clamp(0, self.board_size - 1);
    }

    pub fn move_y(&mut self, delta: i32) {
        self.plane_y_pos = (self.plane_y_pos + delta).clamp(0, self.board_size - 1);
    }

    pub fn move_z(&mut self, delta: i32) {
        self.plane_z_pos = (self.plane_z_pos + delta).clamp(0, self.board_size - 1);
    }

    pub fn get_intersection_position(&self) -> (u8, u8, u8) {
        (self.plane_x_pos as u8, self.plane_y_pos as u8, self.plane_z_pos as u8)
    }

    pub fn get_plane_instances(&self) -> (Instance, Instance, Instance) {
        let half_size = self.board_size as f32 * 0.5;
        
        // YZ plane (controlled by X position)
        let mut yz_plane = Instance::new(Vec3::new(
            self.plane_x_pos as f32 - half_size + 0.5,
            0.0,
            0.0
        ));
        yz_plane.scale = Vec3::splat(self.board_size as f32);

        // XZ plane (controlled by Y position)
        let mut xz_plane = Instance::new(Vec3::new(
            0.0,
            self.plane_z_pos as f32 - half_size + 0.5,  // Note: swapped for rendering
            0.0
        ));
        xz_plane.scale = Vec3::splat(self.board_size as f32);

        // XY plane (controlled by Z position)
        let mut xy_plane = Instance::new(Vec3::new(
            0.0,
            0.0,
            self.plane_y_pos as f32 - half_size + 0.5  // Note: swapped for rendering
        ));
        xy_plane.scale = Vec3::splat(self.board_size as f32);

        (yz_plane, xz_plane, xy_plane)
    }

    pub fn get_dot_instance(&self) -> Instance {
        let half_size = self.board_size as f32 * 0.5;
        
        let mut dot = Instance::new(Vec3::new(
            self.plane_x_pos as f32 - half_size + 0.5,
            self.plane_z_pos as f32 - half_size + 0.5,  // Swapped for rendering
            self.plane_y_pos as f32 - half_size + 0.5,  // Swapped for rendering
        ));
        dot.scale = Vec3::splat(0.125);  // 1/8th the size of a stone
        
        dot
    }
}