use glam::{Vec3, Mat4};
use super::{Instance, Vertex, Mesh};

pub struct AxisIndicator {
    pub x_axis_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    pub y_axis_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    pub z_axis_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    pub position: Vec3,
    pub scale: f32,
}

impl AxisIndicator {
    pub fn new(device: &wgpu::Device) -> Self {
        // Create arrow mesh for each axis
        let x_axis_data = Self::create_axis_arrow([1.0, 0.0, 0.0], Vec3::X);  // Red X
        let y_axis_data = Self::create_axis_arrow([0.0, 1.0, 0.0], Vec3::Y);  // Green Y  
        let z_axis_data = Self::create_axis_arrow([0.0, 0.0, 1.0], Vec3::Z);  // Blue Z
        
        let x_axis_mesh = Self::create_mesh_buffers(device, &x_axis_data);
        let y_axis_mesh = Self::create_mesh_buffers(device, &y_axis_data);
        let z_axis_mesh = Self::create_mesh_buffers(device, &z_axis_data);

        Self {
            x_axis_mesh,
            y_axis_mesh,
            z_axis_mesh,
            position: Vec3::new(-0.8, -0.7, 0.0), // Bottom-left of screen
            scale: 0.1,
        }
    }

    fn create_axis_arrow(color: [f32; 3], direction: Vec3) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Create a simple arrow shape
        let length = 1.0;
        let thickness = 0.02;
        let head_length = 0.3;
        let head_width = 0.06;
        
        // Arrow shaft (cylinder)
        let shaft_start = Vec3::ZERO;
        let shaft_end = direction * (length - head_length);
        
        // Simple cylinder approximation with 8 sides
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            // Create perpendicular vectors to direction
            let perp1 = if direction.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
            let perp1 = perp1.cross(direction).normalize() * thickness;
            let perp2 = direction.cross(perp1).normalize() * thickness;
            
            let offset = perp1 * cos_a + perp2 * sin_a;
            
            // Shaft vertices
            vertices.push(Vertex {
                position: (shaft_start + offset).to_array(),
                normal: offset.normalize().to_array(),
                tex_coords: [0.0, 0.0],
                color,
            });
            
            vertices.push(Vertex {
                position: (shaft_end + offset).to_array(),
                normal: offset.normalize().to_array(),
                tex_coords: [1.0, 0.0],
                color,
            });
        }
        
        // Create shaft indices
        for i in 0..8 {
            let next = (i + 1) % 8;
            let base = i * 2;
            let next_base = next * 2;
            
            // Two triangles per face
            indices.extend_from_slice(&[
                base as u32, (base + 1) as u32, next_base as u32,
                next_base as u32, (base + 1) as u32, (next_base + 1) as u32,
            ]);
        }
        
        // Arrow head (cone)
        let head_base_idx = vertices.len();
        let head_start = shaft_end;
        let head_end = direction * length;
        
        // Head center
        vertices.push(Vertex {
            position: head_end.to_array(),
            normal: direction.to_array(),
            tex_coords: [0.5, 1.0],
            color,
        });
        
        // Head base vertices
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            let perp1 = if direction.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
            let perp1 = perp1.cross(direction).normalize() * head_width;
            let perp2 = direction.cross(perp1).normalize() * head_width;
            
            let offset = perp1 * cos_a + perp2 * sin_a;
            
            vertices.push(Vertex {
                position: (head_start + offset).to_array(),
                normal: offset.normalize().to_array(),
                tex_coords: [cos_a * 0.5 + 0.5, sin_a * 0.5 + 0.5],
                color,
            });
        }
        
        // Head triangles
        for i in 0..8 {
            let next = (i + 1) % 8;
            indices.extend_from_slice(&[
                head_base_idx as u32,
                (head_base_idx + 1 + next) as u32,
                (head_base_idx + 1 + i) as u32,
            ]);
        }
        
        Mesh::new(vertices, indices)
    }

    fn create_mesh_buffers(device: &wgpu::Device, mesh: &Mesh) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        use wgpu::util::DeviceExt;
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Axis Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Axis Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer, mesh.indices.len() as u32)
    }

    pub fn get_instances(&self, view_matrix: &Mat4) -> (Instance, Instance, Instance) {
        // Extract rotation from view matrix and apply to axis indicator
        let rotation = Mat4::from_cols(
            view_matrix.x_axis.truncate().extend(0.0),
            view_matrix.y_axis.truncate().extend(0.0),
            view_matrix.z_axis.truncate().extend(0.0),
            Vec3::ZERO.extend(1.0),
        );
        
        let rotation_quat = glam::Quat::from_mat4(&rotation);

        let mut x_instance = Instance::new(self.position);
        x_instance.rotation = rotation_quat;
        x_instance.scale = Vec3::splat(self.scale);

        let mut y_instance = Instance::new(self.position);
        y_instance.rotation = rotation_quat;
        y_instance.scale = Vec3::splat(self.scale);

        let mut z_instance = Instance::new(self.position);
        z_instance.rotation = rotation_quat;
        z_instance.scale = Vec3::splat(self.scale);

        (x_instance, y_instance, z_instance)
    }
}