use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::FRAC_PI_2;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
}

pub struct Camera {
    pub eye: Vec3,     // Camera position
    pub target: Vec3,  // Look-at target
    pub up: Vec3,      // Up vector
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: Vec3::new(10.0, 10.0, 10.0), // Position camera at a good viewing angle
            target: Vec3::new(0.0, 0.0, 0.0),  // Look at board center (0,0,0)
            up: Vec3::Y,
            aspect: width as f32 / height as f32,
            fovy: 45.0f32.to_radians(),
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = self.build_view_matrix();
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn get_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.build_view_projection_matrix().to_cols_array_2d(),
            view_pos: Vec4::new(self.eye.x, self.eye.y, self.eye.z, 1.0).to_array(),
        }
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_space_pressed: bool,  // For panning mode
    mouse_dx: f32,
    mouse_dy: f32,
    orbit_distance: f32,
    orbit_angle_x: f32,
    orbit_angle_y: f32,
    pan_offset: Vec3,  // Offset from board center for panning
    board_center: Vec3,  // The center of the board (0,0,0)
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_space_pressed: false,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            orbit_distance: 15.0,  // Good distance for 3x3x3 board
            orbit_angle_x: -FRAC_PI_2 / 3.0,  // Initial angle for good view
            orbit_angle_y: FRAC_PI_2 / 6.0,  // Slight elevation
            pan_offset: Vec3::ZERO,
            board_center: Vec3::ZERO,  // Board center is at origin
        }
    }

    pub fn process_keyboard(&mut self, key: winit::event::VirtualKeyCode, state: winit::event::ElementState) -> bool {
        let is_pressed = state == winit::event::ElementState::Pressed;
        
        match key {
            winit::event::VirtualKeyCode::W | winit::event::VirtualKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                self.is_left_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                self.is_right_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::Space => {
                self.is_space_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::Q => {
                self.is_up_pressed = is_pressed;
                true
            }
            winit::event::VirtualKeyCode::LShift => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.mouse_dx += mouse_dx as f32;
        self.mouse_dy += mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: f32) {
        self.orbit_distance = (self.orbit_distance - delta * 2.0).clamp(5.0, 100.0);
    }

    pub fn zoom_in(&mut self) {
        let zoom_step = 2.0;
        self.orbit_distance = (self.orbit_distance - zoom_step).max(2.0);
    }

    pub fn zoom_out(&mut self) {
        let zoom_step = 2.0;
        self.orbit_distance = (self.orbit_distance + zoom_step).min(50.0);
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        // Handle mouse movement
        if self.mouse_dx.abs() > 0.01 || self.mouse_dy.abs() > 0.01 {
            if self.is_space_pressed {
                // Panning mode - move the pan offset
                let right = (camera.target - camera.eye).cross(camera.up).normalize();
                let up = camera.up;
                
                let pan_speed = self.orbit_distance * 0.002; // Scale panning with distance
                self.pan_offset += right * (-self.mouse_dx * pan_speed);
                self.pan_offset += up * (self.mouse_dy * pan_speed);
            } else {
                // Orbit mode - rotate around board center
                self.orbit_angle_x += self.mouse_dx * self.sensitivity * dt;
                self.orbit_angle_y += self.mouse_dy * self.sensitivity * dt;
                self.orbit_angle_y = self.orbit_angle_y.clamp(-FRAC_PI_2 + 0.1, FRAC_PI_2 - 0.1);
            }
            
            self.mouse_dx = 0.0;
            self.mouse_dy = 0.0;
        }

        // Handle keyboard movement (zoom)
        if self.is_forward_pressed {
            self.orbit_distance = (self.orbit_distance - self.speed * dt).max(5.0);
        }
        if self.is_backward_pressed {
            self.orbit_distance = (self.orbit_distance + self.speed * dt).min(100.0);
        }
        
        // Move pan offset with arrow keys or WASD (when not used for zoom)
        if self.is_left_pressed || self.is_right_pressed || self.is_up_pressed || self.is_down_pressed {
            let right = (camera.target - camera.eye).cross(camera.up).normalize();
            
            if self.is_left_pressed {
                self.pan_offset -= right * self.speed * dt * 0.5;
            }
            if self.is_right_pressed {
                self.pan_offset += right * self.speed * dt * 0.5;
            }
            if self.is_up_pressed {
                self.pan_offset += camera.up * self.speed * dt * 0.5;
            }
            if self.is_down_pressed {
                self.pan_offset -= camera.up * self.speed * dt * 0.5;
            }
        }

        // Calculate camera position based on orbit angles around board center
        let x = self.orbit_distance * self.orbit_angle_y.cos() * self.orbit_angle_x.cos();
        let y = self.orbit_distance * self.orbit_angle_y.sin();
        let z = self.orbit_distance * self.orbit_angle_y.cos() * self.orbit_angle_x.sin();

        // Set camera position: orbit around board center + pan offset
        camera.eye = self.board_center + Vec3::new(x, y, z) + self.pan_offset;
        
        // Camera always looks at board center + pan offset
        camera.target = self.board_center + self.pan_offset;
    }

    pub fn is_panning(&self) -> bool {
        self.is_space_pressed
    }

    pub fn set_orbit_center(&mut self, new_center: Vec3) {
        self.board_center = new_center;
        // Reset pan offset when changing orbit center
        self.pan_offset = Vec3::ZERO;
    }

    pub fn get_orbit_center(&self) -> Vec3 {
        self.board_center + self.pan_offset
    }
}