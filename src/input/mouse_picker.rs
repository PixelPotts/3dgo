use glam::{Vec2, Vec3, Vec4, Mat4, Vec4Swizzles};
use crate::render::Camera;

pub struct MousePicker;

impl MousePicker {
    pub fn screen_to_world_ray(
        mouse_pos: Vec2,
        screen_size: Vec2,
        camera: &Camera,
    ) -> (Vec3, Vec3) {
        let view = Mat4::look_at_rh(camera.eye, camera.target, camera.up);
        let proj = Mat4::perspective_rh(camera.fovy, camera.aspect, camera.znear, camera.zfar);
        let view_proj = proj * view;
        let inv_view_proj = view_proj.inverse();

        let ndc_x = (mouse_pos.x / screen_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_pos.y / screen_size.y) * 2.0;

        let near_point = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_point = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_view_proj * near_point;
        let far_world = inv_view_proj * far_point;

        let near_world = near_world.xyz() / near_world.w;
        let far_world = far_world.xyz() / far_world.w;

        let ray_origin = near_world;
        let ray_direction = (far_world - near_world).normalize();

        (ray_origin, ray_direction)
    }

    pub fn intersect_board_position(
        ray_origin: Vec3,
        ray_direction: Vec3,
        board_size: usize,
    ) -> Option<(u8, u8, u8)> {
        let board_size_f = board_size as f32;
        let half_size = board_size_f * 0.5;

        for z in 0..board_size {
            let z_pos = z as f32 - half_size + 0.5;
            
            if ray_direction.y.abs() < 0.001 {
                continue;
            }

            let t = (z_pos - ray_origin.y) / ray_direction.y;
            
            if t < 0.0 {
                continue;
            }

            let intersection = ray_origin + ray_direction * t;
            let x = intersection.x + half_size - 0.5;
            let y = intersection.z + half_size - 0.5;

            if x >= 0.0 && x < board_size_f && y >= 0.0 && y < board_size_f {
                let board_x = x.round() as u8;
                let board_y = y.round() as u8;
                let board_z = z as u8;

                if board_x < board_size as u8 && board_y < board_size as u8 && board_z < board_size as u8 {
                    return Some((board_x, board_y, board_z));
                }
            }
        }

        None
    }

    pub fn intersect_sphere(
        ray_origin: Vec3,
        ray_direction: Vec3,
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Option<f32> {
        let oc = ray_origin - sphere_center;
        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * oc.dot(ray_direction);
        let c = oc.dot(oc) - sphere_radius * sphere_radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-b - sqrt_discriminant) / (2.0 * a);
            let t2 = (-b + sqrt_discriminant) / (2.0 * a);

            if t1 > 0.0 {
                Some(t1)
            } else if t2 > 0.0 {
                Some(t2)
            } else {
                None
            }
        }
    }

    pub fn find_clicked_stone(
        ray_origin: Vec3,
        ray_direction: Vec3,
        game_rules: &crate::game::GameRules,
    ) -> Option<((u8, u8, u8), f32)> {
        let board_size = game_rules.board().size();
        let half_size = board_size as f32 * 0.5;
        let stone_radius = 0.4; // Same as stone mesh radius
        
        let mut closest_stone: Option<((u8, u8, u8), f32)> = None;
        let mut closest_distance = f32::MAX;

        // Check all stones for intersection
        for ((x, y, z), _color) in game_rules.board().get_all_stones() {
            // Convert board coordinates to world position (same logic as in update_stones)
            let world_pos = Vec3::new(
                *x as f32 - half_size + 0.5,
                *z as f32 - half_size + 0.5, // Note: y/z swap for rendering
                *y as f32 - half_size + 0.5,
            );

            if let Some(distance) = Self::intersect_sphere(ray_origin, ray_direction, world_pos, stone_radius) {
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_stone = Some(((*x, *y, *z), distance));
                }
            }
        }

        closest_stone
    }
}