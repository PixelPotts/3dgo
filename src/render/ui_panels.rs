use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UIVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl UIVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct UIPanels {
    pub pipeline: wgpu::RenderPipeline,
}

impl UIPanels {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Panel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui_panel.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Panel Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Panel Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[UIVertex::desc()],
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
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, // Match main render pass
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { pipeline }
    }

    pub fn create_panel_border(&self, x: f32, y: f32, width: f32, height: f32, screen_width: f32, screen_height: f32) -> (Vec<UIVertex>, Vec<u16>) {
        // Convert screen coordinates to NDC
        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        let border_color = [1.0, 1.0, 1.0, 1.0]; // White border
        let border_width = 2.0 / screen_width; // 1px converted to NDC
        let border_height = 2.0 / screen_height;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;

        // Create border (4 rectangles around the panel)
        // Top border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - border_height], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - border_height], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Right border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Bottom border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y - ndc_h + border_height], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h + border_height], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Left border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + border_width, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + border_width, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);

        (vertices, indices)
    }

    pub fn create_panel_with_stones(&self, x: f32, y: f32, width: f32, height: f32, screen_width: f32, screen_height: f32, black_count: usize, white_count: usize) -> (Vec<UIVertex>, Vec<u16>) {
        // Convert screen coordinates to NDC
        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;

        // Background (dark gray to show stones better)
        let bg_color = [0.1, 0.1, 0.1, 1.0];
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x + 0.01, ndc_y - 0.01], color: bg_color },
            UIVertex { position: [ndc_x + ndc_w - 0.01, ndc_y - 0.01], color: bg_color },
            UIVertex { position: [ndc_x + ndc_w - 0.01, ndc_y - ndc_h + 0.01], color: bg_color },
            UIVertex { position: [ndc_x + 0.01, ndc_y - ndc_h + 0.01], color: bg_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Simple stone representation - show colored squares for black/white stones
        let stone_size = (ndc_w.min(ndc_h)) * 0.1; // Small squares
        let stones_per_row = 4;
        let total_stones = black_count + white_count;
        
        for i in 0..total_stones.min(12) { // Max 12 stones to fit in panel
            let row = i / stones_per_row;
            let col = i % stones_per_row;
            
            let stone_x = ndc_x + 0.02 + col as f32 * stone_size * 1.2;
            let stone_y = ndc_y - 0.02 - row as f32 * stone_size * 1.2;
            
            let color = if i < black_count {
                [0.1, 0.1, 0.1, 1.0] // Dark for black stones
            } else {
                [0.9, 0.9, 0.9, 1.0] // Light for white stones
            };
            
            vertices.extend_from_slice(&[
                UIVertex { position: [stone_x, stone_y], color },
                UIVertex { position: [stone_x + stone_size, stone_y], color },
                UIVertex { position: [stone_x + stone_size, stone_y - stone_size], color },
                UIVertex { position: [stone_x, stone_y - stone_size], color },
            ]);
            indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
            index_offset += 4;
        }

        (vertices, indices)
    }

    pub fn create_panel_vertices(&self, x: f32, y: f32, width: f32, height: f32, screen_width: f32, screen_height: f32, panel_id: u32) -> (Vec<UIVertex>, Vec<u16>) {
        // Convert screen coordinates to NDC
        let ndc_x = (x / screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / screen_height) * 2.0;
        let ndc_w = (width / screen_width) * 2.0;
        let ndc_h = (height / screen_height) * 2.0;

        // Create colors for visual testing - different color for each panel
        let panel_colors = [
            [1.0, 0.2, 0.2, 0.8], // Red
            [0.2, 1.0, 0.2, 0.8], // Green  
            [0.2, 0.2, 1.0, 0.8], // Blue
            [1.0, 1.0, 0.2, 0.8], // Yellow
            [1.0, 0.2, 1.0, 0.8], // Magenta
            [0.2, 1.0, 1.0, 0.8], // Cyan
        ];
        
        let color = panel_colors[panel_id as usize % 6];
        let border_color = [1.0, 1.0, 1.0, 1.0]; // White border

        let border_width = 2.0 / screen_width; // 1px converted to NDC
        let border_height = 2.0 / screen_height;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u16;

        // Create border (4 rectangles around the panel)
        // Top border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - border_height], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - border_height], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Right border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Bottom border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y - ndc_h + border_height], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h + border_height], color: border_color },
            UIVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Left border
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + border_width, ndc_y], color: border_color },
            UIVertex { position: [ndc_x + border_width, ndc_y - ndc_h], color: border_color },
            UIVertex { position: [ndc_x, ndc_y - ndc_h], color: border_color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);
        index_offset += 4;

        // Create inner panel with test color
        vertices.extend_from_slice(&[
            UIVertex { position: [ndc_x + border_width, ndc_y - border_height], color },
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y - border_height], color },
            UIVertex { position: [ndc_x + ndc_w - border_width, ndc_y - ndc_h + border_height], color },
            UIVertex { position: [ndc_x + border_width, ndc_y - ndc_h + border_height], color },
        ]);
        indices.extend_from_slice(&[index_offset, index_offset + 1, index_offset + 2, index_offset, index_offset + 2, index_offset + 3]);

        (vertices, indices)
    }
}