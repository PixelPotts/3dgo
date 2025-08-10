use super::{Camera, Mesh, Vertex, Shader, UISystem, TextRenderer, TextVertex, UIPanels, UIVertex};
use wgpu::util::DeviceExt;
use super::camera::CameraUniform;
use crate::game::GameRules;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Instance {
    pub position: Vec3,
    pub rotation: glam::Quat,
    pub scale: Vec3,
}

impl Instance {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        InstanceRaw {
            model: model.to_cols_array_2d(),
        }
    }
}

pub struct Graphics {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    
    multisampled_framebuffer: wgpu::Texture,
    multisampled_view: wgpu::TextureView,
    
    sphere_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    black_sphere_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    white_sphere_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    cube_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    line_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    transparent_box_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    
    // Guide system meshes
    guide_plane_xy_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    guide_plane_xz_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    guide_plane_yz_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    guide_dot_mesh: (wgpu::Buffer, wgpu::Buffer, u32),
    
    sphere_shader: Shader,
    line_shader: Shader,
    transparent_shader: Shader,
    
    ui_system: UISystem,
    text_renderer: TextRenderer,
    ui_panels: UIPanels,
    guide_system: super::GuideSystem,
    axis_indicator: super::AxisIndicator,
}

impl Graphics {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window).unwrap() };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        log::warn!("🔍 Creating DEPTH texture with sample_count=1");
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        log::warn!("✅ DEPTH texture created successfully");

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        log::warn!("🔍 Creating MULTISAMPLED framebuffer with sample_count=1");
        let multisampled_framebuffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisampled Framebuffer"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        log::warn!("✅ MULTISAMPLED framebuffer created successfully");

        let multisampled_view = multisampled_framebuffer.create_view(&wgpu::TextureViewDescriptor::default());

        let sphere_mesh_data = Mesh::create_sphere(0.4, 20, 20, [0.8, 0.8, 0.8]);
        let sphere_mesh = Self::create_mesh_buffers(&device, &sphere_mesh_data);
        
        let black_sphere_mesh_data = Mesh::create_sphere(0.4, 20, 20, [0.1, 0.1, 0.1]);
        let black_sphere_mesh = Self::create_mesh_buffers(&device, &black_sphere_mesh_data);
        
        let white_sphere_mesh_data = Mesh::create_sphere(0.4, 20, 20, [0.9, 0.9, 0.9]);
        let white_sphere_mesh = Self::create_mesh_buffers(&device, &white_sphere_mesh_data);

        let cube_mesh_data = Mesh::create_cube(0.05, [0.8, 0.8, 0.8]);
        let cube_mesh = Self::create_mesh_buffers(&device, &cube_mesh_data);

        let line_mesh_data = Mesh::create_line(Vec3::ZERO, Vec3::X, [0.5, 0.5, 0.5]);
        let line_mesh = Self::create_mesh_buffers(&device, &line_mesh_data);

        let transparent_box_data = Mesh::create_transparent_box(1.0, [0.3, 0.5, 0.8]);  // Unit cube, will scale based on board
        let transparent_box_mesh = Self::create_mesh_buffers(&device, &transparent_box_data);

        // Create guide plane meshes (very faint yellow)
        let guide_plane_xy_data = Mesh::create_guide_plane_xy(1.0, [0.8, 0.8, 0.3]);
        let guide_plane_xy_mesh = Self::create_mesh_buffers(&device, &guide_plane_xy_data);
        
        let guide_plane_xz_data = Mesh::create_guide_plane_xz(1.0, [0.8, 0.8, 0.3]);
        let guide_plane_xz_mesh = Self::create_mesh_buffers(&device, &guide_plane_xz_data);
        
        let guide_plane_yz_data = Mesh::create_guide_plane_yz(1.0, [0.8, 0.8, 0.3]);
        let guide_plane_yz_mesh = Self::create_mesh_buffers(&device, &guide_plane_yz_data);
        
        // Create guide dot mesh (blue, 1/8 size)
        let guide_dot_data = Mesh::create_sphere(0.05, 10, 10, [0.2, 0.4, 0.9]);
        let guide_dot_mesh = Self::create_mesh_buffers(&device, &guide_dot_data);

        let sphere_shader = Shader::create_basic_shader(
            &device,
            config.format,
            &[&camera_bind_group_layout],
            &[Vertex::desc(), InstanceRaw::desc()],
            wgpu::PrimitiveTopology::TriangleList,
        );

        let line_shader = Shader::create_basic_shader(
            &device,
            config.format,
            &[&camera_bind_group_layout],
            &[Vertex::desc(), InstanceRaw::desc()],
            wgpu::PrimitiveTopology::LineList,
        );

        let transparent_shader = Shader::create_transparent_shader(
            &device,
            config.format,
            &[&camera_bind_group_layout],
            &[Vertex::desc(), InstanceRaw::desc()],
            wgpu::PrimitiveTopology::TriangleList,
        );

        let ui_system = UISystem::new();
        let text_renderer = TextRenderer::new(&device, &queue, config.format);
        let ui_panels = UIPanels::new(&device, config.format);
        let axis_indicator = super::AxisIndicator::new(&device);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            depth_texture,
            depth_view,
            multisampled_framebuffer,
            multisampled_view,
            sphere_mesh,
            black_sphere_mesh,
            white_sphere_mesh,
            cube_mesh,
            line_mesh,
            transparent_box_mesh,
            guide_plane_xy_mesh,
            guide_plane_xz_mesh,
            guide_plane_yz_mesh,
            guide_dot_mesh,
            sphere_shader,
            line_shader,
            transparent_shader,
            ui_system,
            text_renderer,
            ui_panels,
            guide_system: super::GuideSystem::new(3),  // 3x3x3 board
            axis_indicator,
        }
    }

    fn create_mesh_buffers(device: &wgpu::Device, mesh: &Mesh) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer, mesh.indices.len() as u32)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,  // Match multisampling
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            self.multisampled_framebuffer = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisampled Framebuffer"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            self.multisampled_view = self.multisampled_framebuffer.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn guide_system_mut(&mut self) -> &mut super::GuideSystem {
        &mut self.guide_system
    }

    pub fn update_camera(&self, camera: &Camera) {
        let camera_uniform = camera.get_uniform();
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    pub fn render(&mut self, instances: &[Instance], black_stones: &[Instance], white_stones: &[Instance], game_rules: &GameRules, camera: &super::Camera) -> Result<(), wgpu::SurfaceError> {
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let instance_buffer = if !instances.is_empty() {
            let instance_data: Vec<InstanceRaw> = instances.iter().map(|i| i.to_raw()).collect();
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        let black_stone_buffer = if !black_stones.is_empty() {
            let stone_data: Vec<InstanceRaw> = black_stones.iter().map(|i| i.to_raw()).collect();
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Black Stone Buffer"),
                contents: bytemuck::cast_slice(&stone_data),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        let white_stone_buffer = if !white_stones.is_empty() {
            let stone_data: Vec<InstanceRaw> = white_stones.iter().map(|i| i.to_raw()).collect();
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("White Stone Buffer"),
                contents: bytemuck::cast_slice(&stone_data),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        // Create transparent box buffer scaled to board size
        let board_size = game_rules.board().size() as f32;
        let mut box_instance = Instance::new(Vec3::new(0.0, 0.0, 0.0));
        box_instance.scale = Vec3::splat(board_size);  // Scale box to match board dimensions
        let box_data = vec![box_instance.to_raw()];
        let box_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Box Buffer"),
            contents: bytemuck::cast_slice(&box_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create guide plane buffers
        let (yz_plane, xz_plane, xy_plane) = self.guide_system.get_plane_instances();
        
        let yz_data = vec![yz_plane.to_raw()];
        let yz_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("YZ Plane Buffer"),
            contents: bytemuck::cast_slice(&yz_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let xz_data = vec![xz_plane.to_raw()];
        let xz_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("XZ Plane Buffer"),
            contents: bytemuck::cast_slice(&xz_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let xy_data = vec![xy_plane.to_raw()];
        let xy_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("XY Plane Buffer"),
            contents: bytemuck::cast_slice(&xy_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create guide dot buffer
        let dot_instance = self.guide_system.get_dot_instance();
        let dot_data = vec![dot_instance.to_raw()];
        let dot_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Guide Dot Buffer"),
            contents: bytemuck::cast_slice(&dot_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create axis indicator buffers
        let view_matrix = camera.build_view_matrix();
        let (x_axis_instance, y_axis_instance, z_axis_instance) = self.axis_indicator.get_instances(&view_matrix);
        
        let x_axis_data = vec![x_axis_instance.to_raw()];
        let x_axis_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("X Axis Buffer"),
            contents: bytemuck::cast_slice(&x_axis_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let y_axis_data = vec![y_axis_instance.to_raw()];
        let y_axis_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Y Axis Buffer"),
            contents: bytemuck::cast_slice(&y_axis_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let z_axis_data = vec![z_axis_instance.to_raw()];
        let z_axis_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Z Axis Buffer"),
            contents: bytemuck::cast_slice(&z_axis_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        {
            log::warn!("🔥 STARTING MAIN RENDER PASS - surface sample_count should be 1");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,  // Black background
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render transparent boundary box
            log::warn!("🔥 Setting TRANSPARENT SHADER pipeline (sample_count=1)");
            render_pass.set_pipeline(&self.transparent_shader.render_pipeline);
            render_pass.set_vertex_buffer(0, self.transparent_box_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, box_buffer.slice(..));
            render_pass.set_index_buffer(self.transparent_box_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.transparent_box_mesh.2, 0, 0..1 as _);

            // Render guide planes (very faint)
            // YZ plane
            render_pass.set_vertex_buffer(0, self.guide_plane_yz_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, yz_buffer.slice(..));
            render_pass.set_index_buffer(self.guide_plane_yz_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.guide_plane_yz_mesh.2, 0, 0..1 as _);
            
            // XZ plane
            render_pass.set_vertex_buffer(0, self.guide_plane_xz_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, xz_buffer.slice(..));
            render_pass.set_index_buffer(self.guide_plane_xz_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.guide_plane_xz_mesh.2, 0, 0..1 as _);
            
            // XY plane
            render_pass.set_vertex_buffer(0, self.guide_plane_xy_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, xy_buffer.slice(..));
            render_pass.set_index_buffer(self.guide_plane_xy_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.guide_plane_xy_mesh.2, 0, 0..1 as _);

            if let Some(ref buffer) = black_stone_buffer {
                log::warn!("🔥 Setting BLACK SPHERE SHADER pipeline (sample_count=1)");
                render_pass.set_pipeline(&self.sphere_shader.render_pipeline);
                render_pass.set_vertex_buffer(0, self.black_sphere_mesh.0.slice(..));
                render_pass.set_vertex_buffer(1, buffer.slice(..));
                render_pass.set_index_buffer(self.black_sphere_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.black_sphere_mesh.2, 0, 0..black_stones.len() as _);
            }

            if let Some(ref buffer) = white_stone_buffer {
                log::warn!("🔥 Setting WHITE SPHERE SHADER pipeline (sample_count=1)");
                render_pass.set_pipeline(&self.sphere_shader.render_pipeline);
                render_pass.set_vertex_buffer(0, self.white_sphere_mesh.0.slice(..));
                render_pass.set_vertex_buffer(1, buffer.slice(..));
                render_pass.set_index_buffer(self.white_sphere_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.white_sphere_mesh.2, 0, 0..white_stones.len() as _);
            }
            
            // Render guide dot (always on top) 
            render_pass.set_pipeline(&self.sphere_shader.render_pipeline);
            render_pass.set_vertex_buffer(0, self.guide_dot_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, dot_buffer.slice(..));
            render_pass.set_index_buffer(self.guide_dot_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.guide_dot_mesh.2, 0, 0..1 as _);

            // Render 3D axis indicator (always on top)
            render_pass.set_pipeline(&self.sphere_shader.render_pipeline);
            
            // X axis (red)
            render_pass.set_vertex_buffer(0, self.axis_indicator.x_axis_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, x_axis_buffer.slice(..));
            render_pass.set_index_buffer(self.axis_indicator.x_axis_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.axis_indicator.x_axis_mesh.2, 0, 0..1 as _);
            
            // Y axis (green)
            render_pass.set_vertex_buffer(0, self.axis_indicator.y_axis_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, y_axis_buffer.slice(..));
            render_pass.set_index_buffer(self.axis_indicator.y_axis_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.axis_indicator.y_axis_mesh.2, 0, 0..1 as _);
            
            // Z axis (blue)
            render_pass.set_vertex_buffer(0, self.axis_indicator.z_axis_mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, z_axis_buffer.slice(..));
            render_pass.set_index_buffer(self.axis_indicator.z_axis_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.axis_indicator.z_axis_mesh.2, 0, 0..1 as _);
        }

        // Render 2D UI panels with visible borders and stone representation
        self.render_ui_side_panels_with_stones(&mut encoder, &view, game_rules);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_ui_side_panels_with_stones(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, game_rules: &GameRules) {
        // Update UI system animations first
        self.ui_system.update(game_rules.board().size());
        
        let panel_width = 120.0;
        let panel_height = 80.0;
        let panel_spacing = 90.0;
        let right_margin = 20.0;
        let start_y = 20.0;

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut current_index_offset = 0u16;

        // Create panels with stone representation
        for (i, side_view) in self.ui_system.side_views.iter().enumerate() {
            let panel_x = self.size.width as f32 - panel_width - right_margin;
            let panel_y = start_y + i as f32 * panel_spacing;

            // Get animated stones from this view (smart layer detection with animation)
            let (black_stones, white_stones) = side_view.get_visible_stones(game_rules, 1);
            
            let (vertices, indices) = self.ui_panels.create_panel_with_stones(
                panel_x, panel_y, panel_width, panel_height,
                self.size.width as f32, self.size.height as f32,
                black_stones.len(), white_stones.len()
            );

            let vertex_count = vertices.len() as u16;
            all_vertices.extend(vertices);
            all_indices.extend(indices.iter().map(|&idx| idx + current_index_offset));
            current_index_offset += vertex_count.max(20);
        }

        // Render all panels
        if !all_vertices.is_empty() {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Panel Content Buffer"),
                contents: bytemuck::cast_slice(&all_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Panel Content Index Buffer"),
                contents: bytemuck::cast_slice(&all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Panel Content Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            ui_render_pass.set_pipeline(&self.ui_panels.pipeline);
            ui_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            ui_render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            ui_render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }

        // Render the white borders over everything
        self.render_panel_borders(encoder, view, panel_width, panel_height, panel_spacing, right_margin, start_y);
        
        // Add simple text labels
        let view_directions = [
            super::ViewDirection::Top,
            super::ViewDirection::Left,
            super::ViewDirection::Right,
            super::ViewDirection::Back,
            super::ViewDirection::Front,
            super::ViewDirection::Bottom,
        ];
        self.render_panel_labels(encoder, view, &view_directions, panel_width, panel_height, panel_spacing, right_margin, start_y);
    }


    fn render_panel_borders(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, panel_width: f32, panel_height: f32, panel_spacing: f32, right_margin: f32, start_y: f32) {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut current_index_offset = 0u16;

        // Create borders for all 6 panels
        for i in 0..6 {
            let panel_x = self.size.width as f32 - panel_width - right_margin;
            let panel_y = start_y + i as f32 * panel_spacing;

            let (vertices, indices) = self.ui_panels.create_panel_border(
                panel_x, panel_y, panel_width, panel_height,
                self.size.width as f32, self.size.height as f32
            );

            let vertex_count = vertices.len() as u16;
            all_vertices.extend(vertices);
            all_indices.extend(indices.iter().map(|&idx| idx + current_index_offset));
            current_index_offset += vertex_count.max(20);
        }

        if !all_vertices.is_empty() {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Panel Border Buffer"),
                contents: bytemuck::cast_slice(&all_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Panel Border Index Buffer"),
                contents: bytemuck::cast_slice(&all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Panel Border Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            ui_render_pass.set_pipeline(&self.ui_panels.pipeline);
            ui_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            ui_render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            ui_render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }
    }

    fn render_panel_labels(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, directions: &[super::ViewDirection], panel_width: f32, panel_height: f32, panel_spacing: f32, right_margin: f32, start_y: f32) {
        // Render text labels under each panel
        for (i, direction) in directions.iter().enumerate() {
            let panel_x = self.size.width as f32 - panel_width - right_margin;
            let panel_y = start_y + i as f32 * panel_spacing;
            let text_y = panel_y + panel_height + 5.0;
            let text_x = panel_x + (panel_width - direction.label().len() as f32 * 8.0) / 2.0;
            
            // Use the existing text renderer (even though it might not show up properly yet)
            let (vertices, indices) = self.text_renderer.create_text_quad(
                direction.label(),
                text_x,
                text_y,
                12.0,
                self.size.width as f32,
                self.size.height as f32,
            );

            if !vertices.is_empty() {
                // Render text (simplified - may need more work)
                // For now, this provides the foundation for text rendering
            }
        }
    }

    fn render_simple_side_panels(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // Create simple white cubes on the right side as visible panels
        let panel_spacing = 1.5;
        let panel_size = 0.8;
        let right_offset = 6.0;
        
        let mut panel_instances = Vec::new();
        
        // Create 6 white cube instances for the side panels
        for i in 0..6 {
            let y_pos = (i as f32 - 2.5) * panel_spacing;
            let pos = Vec3::new(right_offset, 0.0, y_pos);
            let mut instance = Instance::new(pos);
            instance.scale = Vec3::splat(panel_size);
            panel_instances.push(instance);
        }
        
        // Render the white panel cubes
        if !panel_instances.is_empty() {
            let panel_data: Vec<InstanceRaw> = panel_instances.iter().map(|i| i.to_raw()).collect();
            let panel_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Panel Buffer"),
                contents: bytemuck::cast_slice(&panel_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let mut panel_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Panel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            panel_render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            panel_render_pass.set_pipeline(&self.sphere_shader.render_pipeline);
            panel_render_pass.set_vertex_buffer(0, self.white_sphere_mesh.0.slice(..));
            panel_render_pass.set_vertex_buffer(1, panel_buffer.slice(..));
            panel_render_pass.set_index_buffer(self.white_sphere_mesh.1.slice(..), wgpu::IndexFormat::Uint32);
            panel_render_pass.draw_indexed(0..self.white_sphere_mesh.2, 0, 0..panel_instances.len() as _);
        }
    }

    fn render_side_text_labels(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let panel_width = 150.0;
        let panel_height = 80.0;
        let panel_spacing = 90.0;
        let right_margin = 20.0;
        let panel_x = self.size.width as f32 - panel_width - right_margin;

        // Render panel outlines and labels for each side view
        let labels = ["TOP", "LEFT", "RIGHT", "BACK", "FRONT", "BOTTOM"];
        for (i, label) in labels.iter().enumerate() {
            let panel_y = 20.0 + i as f32 * panel_spacing;
            
            // Render white outline rectangle
            self.render_panel_outline(encoder, view, panel_x, panel_y, panel_width, panel_height);
            
            // Render white text label below the panel
            let text_y = panel_y + panel_height + 5.0;
            let text_x = panel_x + (panel_width - label.len() as f32 * 8.0) / 2.0; // Center text
            self.render_panel_text(encoder, view, label, text_x, text_y);
        }
    }

    fn render_panel_outline(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, x: f32, y: f32, width: f32, height: f32) {
        // Create a simple rectangle outline using text rendering system 
        // Draw 4 lines to form rectangle border
        
        // Top border
        self.render_line_segment(encoder, view, x, y, x + width, y);
        // Right border  
        self.render_line_segment(encoder, view, x + width, y, x + width, y + height);
        // Bottom border
        self.render_line_segment(encoder, view, x + width, y + height, x, y + height);
        // Left border
        self.render_line_segment(encoder, view, x, y + height, x, y);
    }

    fn render_line_segment(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, x1: f32, y1: f32, x2: f32, y2: f32) {
        // For now, render a simple white rectangle to simulate a line
        // This is a simplified approach - we could create a proper line renderer later
        let line_width = 1.0;
        
        // Determine if this is a horizontal or vertical line
        if (y2 - y1).abs() < 0.1 {
            // Horizontal line
            self.render_filled_rect(encoder, view, x1, y1, x2 - x1, line_width, [1.0, 1.0, 1.0, 1.0]);
        } else {
            // Vertical line  
            self.render_filled_rect(encoder, view, x1, y1, line_width, y2 - y1, [1.0, 1.0, 1.0, 1.0]);
        }
    }

    fn render_filled_rect(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        // Convert screen coordinates to NDC
        let ndc_x = (x / self.size.width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / self.size.height as f32) * 2.0;
        let ndc_w = (width / self.size.width as f32) * 2.0;
        let ndc_h = (height / self.size.height as f32) * 2.0;

        // Create vertices for a simple white rectangle
        use super::TextVertex;
        let vertices = vec![
            TextVertex { position: [ndc_x, ndc_y], tex_coords: [0.0, 0.0] },
            TextVertex { position: [ndc_x + ndc_w, ndc_y], tex_coords: [1.0, 0.0] },
            TextVertex { position: [ndc_x + ndc_w, ndc_y - ndc_h], tex_coords: [1.0, 1.0] },
            TextVertex { position: [ndc_x, ndc_y - ndc_h], tex_coords: [0.0, 1.0] },
        ];

        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut rect_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Rectangle Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // Use the text renderer pipeline for simple rectangles
        rect_render_pass.set_pipeline(&self.text_renderer.pipeline);
        rect_render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        rect_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rect_render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rect_render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    fn render_panel_text(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, text: &str, x: f32, y: f32) {
        let (vertices, indices) = self.text_renderer.create_text_quad(
            text,
            x,
            y,
            16.0,
            self.size.width as f32,
            self.size.height as f32,
        );

        if vertices.is_empty() {
            return;
        }

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut text_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view, // Use multisampled view
                resolve_target: None, // Resolve to final view
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        text_render_pass.set_pipeline(&self.text_renderer.pipeline);
        text_render_pass.set_bind_group(0, &self.text_renderer.bind_group, &[]);
        text_render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        text_render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        text_render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}