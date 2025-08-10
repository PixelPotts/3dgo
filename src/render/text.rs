use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl TextVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct TextRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub font_texture: wgpu::Texture,
    pub font_view: wgpu::TextureView,
    pub font_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // Create a simple bitmap font texture (8x8 characters)
        let font_data = Self::create_simple_font();
        
        let font_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Font Texture"),
            size: wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &font_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &font_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(128),
                rows_per_image: Some(128),
            },
            wgpu::Extent3d {
                width: 128,
                height: 128,
                depth_or_array_layers: 1,
            },
        );

        let font_view = font_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let font_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Text Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&font_sampler),
                },
            ],
            label: Some("Text Bind Group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[TextVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            font_texture,
            font_view,
            font_sampler,
            bind_group,
        }
    }

    fn create_simple_font() -> Vec<u8> {
        // Create a simple bitmap font with basic ASCII characters
        let mut font_data = vec![0u8; 128 * 128];
        
        // Define simple character patterns (8x8 each)
        let chars = [
            // Space (32)
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            // A (65)
            [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
            // B (66) 
            [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
            // C (67)
            [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
            // E (69)
            [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00],
            // F (70)
            [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00],
            // G (71)
            [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00],
            // H (72)
            [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
            // I (73)
            [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
            // K (75)
            [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
            // L (76)
            [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
            // O (79)
            [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
            // P (80)
            [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
            // R (82)
            [0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x00],
            // T (84)
            [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        ];
        
        // Map characters to positions
        let char_positions = [
            (32, 0), // Space
            (65, 1), // A
            (66, 2), // B  
            (67, 3), // C
            (69, 4), // E
            (70, 5), // F
            (71, 6), // G
            (72, 7), // H
            (73, 8), // I
            (75, 9), // K
            (76, 10), // L
            (79, 11), // O
            (80, 12), // P
            (82, 13), // R
            (84, 14), // T
        ];

        for (ascii_code, pattern_idx) in char_positions {
            if pattern_idx < chars.len() {
                let pattern = chars[pattern_idx];
                let char_x = (ascii_code % 16) * 8;
                let char_y = (ascii_code / 16) * 8;
                
                for row in 0..8 {
                    let byte = pattern[row];
                    for col in 0..8 {
                        if (byte >> (7 - col)) & 1 != 0 {
                            let x = char_x + col;
                            let y = char_y + row;
                            if x < 128 && y < 128 {
                                font_data[y * 128 + x] = 255;
                            }
                        }
                    }
                }
            }
        }
        
        font_data
    }

    pub fn create_text_quad(&self, text: &str, x: f32, y: f32, size: f32, screen_width: f32, screen_height: f32) -> (Vec<TextVertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let char_width = size;
        let char_height = size;
        
        for (i, ch) in text.chars().enumerate() {
            let char_x = x + i as f32 * char_width;
            let char_y = y;
            
            // Convert screen coordinates to NDC
            let ndc_x = (char_x / screen_width) * 2.0 - 1.0;
            let ndc_y = 1.0 - (char_y / screen_height) * 2.0;
            let ndc_w = (char_width / screen_width) * 2.0;
            let ndc_h = (char_height / screen_height) * 2.0;
            
            // Calculate texture coordinates
            let ascii_code = ch as u32;
            let tex_x = ((ascii_code % 16) as f32 * 8.0) / 128.0;
            let tex_y = ((ascii_code / 16) as f32 * 8.0) / 128.0;
            let tex_w = 8.0 / 128.0;
            let tex_h = 8.0 / 128.0;
            
            let base_index = vertices.len() as u16;
            
            // Add vertices for character quad
            vertices.extend_from_slice(&[
                TextVertex { position: [ndc_x, ndc_y], tex_coords: [tex_x, tex_y] },
                TextVertex { position: [ndc_x + ndc_w, ndc_y], tex_coords: [tex_x + tex_w, tex_y] },
                TextVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], tex_coords: [tex_x + tex_w, tex_y + tex_h] },
                TextVertex { position: [ndc_x, ndc_y - ndc_h], tex_coords: [tex_x, tex_y + tex_h] },
            ]);
            
            // Add indices for two triangles
            indices.extend_from_slice(&[
                base_index, base_index + 1, base_index + 2,
                base_index, base_index + 2, base_index + 3,
            ]);
        }
        
        (vertices, indices)
    }
}