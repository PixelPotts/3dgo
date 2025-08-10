use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec2};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn create_cube(size: f32, color: [f32; 3]) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            // Front face
            Vertex { position: [-s, -s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [ s, -s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s,  s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s,  s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color },
            
            // Back face
            Vertex { position: [-s, -s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s,  s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0], color },
            
            // Left face
            Vertex { position: [-s, -s, -s], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [-s, -s,  s], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s,  s,  s], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s,  s, -s], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color },
            
            // Right face
            Vertex { position: [ s, -s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s,  s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color },
            
            // Bottom face
            Vertex { position: [-s, -s, -s], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s, -s], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s, -s,  s], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s, -s,  s], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0], color },
            
            // Top face
            Vertex { position: [-s,  s, -s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [-s,  s,  s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s,  s,  s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s, -s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], color },
        ];

        let indices = vec![
            0,  1,  2,   0,  2,  3,   // front
            4,  5,  6,   4,  6,  7,   // back
            8,  9, 10,   8, 10, 11,   // left
           12, 13, 14,  12, 14, 15,   // right
           16, 17, 18,  16, 18, 19,   // bottom
           20, 21, 22,  20, 22, 23,   // top
        ];

        Self::new(vertices, indices)
    }

    pub fn create_sphere(radius: f32, rings: u32, sectors: u32, color: [f32; 3]) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let ring_step = std::f32::consts::PI / rings as f32;
        let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;

        for r in 0..=rings {
            let ring_angle = std::f32::consts::PI / 2.0 - r as f32 * ring_step;
            let xy = radius * ring_angle.cos();
            let z = radius * ring_angle.sin();

            for s in 0..=sectors {
                let sector_angle = s as f32 * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let normal = Vec3::new(x, y, z).normalize();
                let tex_coords = Vec2::new(s as f32 / sectors as f32, r as f32 / rings as f32);

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: normal.to_array(),
                    tex_coords: tex_coords.to_array(),
                    color,
                });
            }
        }

        for r in 0..rings {
            for s in 0..sectors {
                let current_ring = r * (sectors + 1);
                let next_ring = (r + 1) * (sectors + 1);

                indices.push(current_ring + s);
                indices.push(next_ring + s);
                indices.push(next_ring + s + 1);

                indices.push(current_ring + s);
                indices.push(next_ring + s + 1);
                indices.push(current_ring + s + 1);
            }
        }

        Self::new(vertices, indices)
    }

    pub fn create_line(start: Vec3, end: Vec3, color: [f32; 3]) -> Self {
        let vertices = vec![
            Vertex {
                position: start.to_array(),
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                color,
            },
            Vertex {
                position: end.to_array(),
                normal: [0.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
                color,
            },
        ];

        let indices = vec![0, 1];
        Self::new(vertices, indices)
    }

    pub fn create_transparent_box(size: f32, color: [f32; 3]) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            // Front face
            Vertex { position: [-s, -s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [ s, -s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s,  s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s,  s,  s], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color },
            
            // Back face
            Vertex { position: [-s, -s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s,  s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s, -s], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0], color },
            
            // Left face
            Vertex { position: [-s, -s, -s], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [-s, -s,  s], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s,  s,  s], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s,  s, -s], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color },
            
            // Right face
            Vertex { position: [ s, -s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s,  s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color },
            
            // Bottom face
            Vertex { position: [-s, -s, -s], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s, -s, -s], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s, -s,  s], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [-s, -s,  s], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0], color },
            
            // Top face
            Vertex { position: [-s,  s, -s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [-s,  s,  s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], color },
            Vertex { position: [ s,  s,  s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [ s,  s, -s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], color },
        ];

        let indices = vec![
            0,  1,  2,   0,  2,  3,   // front
            4,  5,  6,   4,  6,  7,   // back
            8,  9, 10,   8, 10, 11,   // left
           12, 13, 14,  12, 14, 15,   // right
           16, 17, 18,  16, 18, 19,   // bottom
           20, 21, 22,  20, 22, 23,   // top
        ];

        Self::new(vertices, indices)
    }

    pub fn create_guide_plane_xy(size: f32, color: [f32; 3]) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            Vertex { position: [-s, -s, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [ s, -s, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s,  s, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s,  s, 0.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color },
        ];
        
        let indices = vec![
            0, 1, 2,  0, 2, 3,  // Front
            0, 3, 2,  0, 2, 1,  // Back (double-sided)
        ];
        
        Self::new(vertices, indices)
    }

    pub fn create_guide_plane_xz(size: f32, color: [f32; 3]) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            Vertex { position: [-s, 0.0, -s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [ s, 0.0, -s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [ s, 0.0,  s], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [-s, 0.0,  s], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], color },
        ];
        
        let indices = vec![
            0, 1, 2,  0, 2, 3,  // Top
            0, 3, 2,  0, 2, 1,  // Bottom (double-sided)
        ];
        
        Self::new(vertices, indices)
    }

    pub fn create_guide_plane_yz(size: f32, color: [f32; 3]) -> Self {
        let s = size / 2.0;
        
        let vertices = vec![
            Vertex { position: [0.0, -s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color },
            Vertex { position: [0.0,  s, -s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color },
            Vertex { position: [0.0,  s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color },
            Vertex { position: [0.0, -s,  s], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color },
        ];
        
        let indices = vec![
            0, 1, 2,  0, 2, 3,  // Right
            0, 3, 2,  0, 2, 1,  // Left (double-sided)
        ];
        
        Self::new(vertices, indices)
    }
}