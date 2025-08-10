pub mod game;
pub mod render;
pub mod input;

use game::{GameRules, StoneColor};
use render::{Graphics, Camera, CameraController, Instance, GuideSystem};
use input::MousePicker;
use glam::Vec3;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use instant::Instant;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;

#[cfg(target_arch = "wasm32")]
use web_sys::{console, HtmlCanvasElement};

struct GameState {
    rules: GameRules,
    black_stone_instances: Vec<Instance>,
    white_stone_instances: Vec<Instance>,
    selected_position: Option<(u8, u8, u8)>,
    mouse_position: glam::Vec2,
    animation_paused: bool,
    guide_system: GuideSystem,
    pending_ai_move: bool,
}

impl GameState {
    fn new() -> Self {
        let rules = GameRules::new_with_dodecahedron(3);
        let black_stone_instances = Vec::new();
        let white_stone_instances = Vec::new();
        let guide_system = GuideSystem::new(3);

        Self {
            rules,
            black_stone_instances,
            white_stone_instances,
            selected_position: None,
            mouse_position: glam::Vec2::ZERO,
            animation_paused: false,
            guide_system,
            pending_ai_move: false,
        }
    }

    fn update_stones(&mut self) {
        self.black_stone_instances.clear();
        self.white_stone_instances.clear();
        let board_size = self.rules.board().size();
        let half_size = board_size as f32 * 0.5;

        for ((x, y, z), color) in self.rules.board().get_all_stones() {
            let pos = Vec3::new(
                *x as f32 - half_size + 0.5,
                *z as f32 - half_size + 0.5,
                *y as f32 - half_size + 0.5,
            );
            
            let mut instance = Instance::new(pos);
            instance.scale = Vec3::splat(1.2);
            
            match color {
                StoneColor::Black => {
                    self.black_stone_instances.push(instance);
                }
                StoneColor::White => {
                    self.white_stone_instances.push(instance);
                }
            }
        }
    }

    fn handle_mouse_click(&mut self, camera: &Camera, screen_size: glam::Vec2) -> bool {
        let (ray_origin, ray_direction) = MousePicker::screen_to_world_ray(
            self.mouse_position,
            screen_size,
            camera,
        );

        if let Some((x, y, z)) = MousePicker::intersect_board_position(
            ray_origin,
            ray_direction,
            self.rules.board().size(),
        ) {
            if self.rules.make_move(x, y, z) {
                self.update_stones();
                return true;
            }
        }

        false
    }

    fn place_stone_at_guide(&mut self) -> bool {
        let (x, y, z) = self.guide_system.get_intersection_position();
        if self.rules.make_move(x, y, z) {
            self.update_stones();
            return true;
        }
        false
    }

    fn make_ai_move(&mut self) -> bool {
        // Simple AI: find all empty positions and choose randomly
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let board_size = self.rules.board().size();
        let mut empty_positions = Vec::new();

        for x in 0..board_size {
            for y in 0..board_size {
                for z in 0..board_size {
                    if self.rules.board().get_stone((x as u8, y as u8, z as u8)).is_none() {
                        empty_positions.push((x as u8, y as u8, z as u8));
                    }
                }
            }
        }

        if !empty_positions.is_empty() {
            let random_pos = empty_positions[rng.gen_range(0..empty_positions.len())];
            if self.rules.make_move(random_pos.0, random_pos.1, random_pos.2) {
                self.update_stones();
                return true;
            }
        }
        false
    }
}

pub mod minimal;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // AGGRESSIVE DEBUG MODE - Restore complex renderer
    log::warn!("🔥 STARTING AGGRESSIVE DEBUG MODE 🔥");
    
    // Original complex renderer (UN-COMMENTED FOR DEBUGGING)
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("3D Go")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut graphics = Graphics::new(&window).await;
    let mut camera = Camera::new(graphics.size.width, graphics.size.height);
    let mut camera_controller = CameraController::new(10.0, 1.0);
    let mut game_state = GameState::new();
    
    let mut last_frame_time = Instant::now();
    let mut mouse_pressed = false;

    game_state.update_stones();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    WindowEvent::Resized(physical_size) => {
                        graphics.resize(*physical_size);
                        camera.update_aspect(physical_size.width, physical_size.height);
                    }

                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        graphics.resize(**new_inner_size);
                        camera.update_aspect(new_inner_size.width, new_inner_size.height);
                    }

                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(key) = input.virtual_keycode {
                            // Handle special game commands only on key press
                            if input.state == ElementState::Pressed {
                                match key {
                                    // Guide plane controls
                                    VirtualKeyCode::W => {
                                        game_state.guide_system.move_y(1);  // Y plane forward
                                    }
                                    VirtualKeyCode::S => {
                                        game_state.guide_system.move_y(-1); // Y plane backward  
                                    }
                                    VirtualKeyCode::A => {
                                        game_state.guide_system.move_x(-1); // X plane left
                                    }
                                    VirtualKeyCode::D => {
                                        game_state.guide_system.move_x(1);  // X plane right
                                    }
                                    VirtualKeyCode::Space => {
                                        // Place stone at guide intersection
                                        if game_state.place_stone_at_guide() {
                                            game_state.pending_ai_move = true;
                                        }
                                    }
                                    VirtualKeyCode::R => {
                                        // Reset - clear the board
                                        game_state.rules.clear_board();
                                        game_state.update_stones();
                                        game_state.pending_ai_move = false;
                                    }
                                    // Zoom controls
                                    VirtualKeyCode::Q | VirtualKeyCode::Left => {
                                        camera_controller.zoom_in();
                                    }
                                    VirtualKeyCode::E | VirtualKeyCode::Right => {
                                        camera_controller.zoom_out();
                                    }
                                    _ => {
                                        // Pass all other keys to camera controller
                                        camera_controller.process_keyboard(key, input.state);
                                    }
                                }
                            } else {
                                // Always pass key releases to camera controller
                                camera_controller.process_keyboard(key, input.state);
                            }
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        game_state.mouse_position = glam::Vec2::new(position.x as f32, position.y as f32);
                    }

                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    } => {
                        mouse_pressed = true;
                    }

                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    } => {
                        if mouse_pressed {
                            // Check if we clicked on a stone to set new orbit center
                            let screen_size = glam::Vec2::new(
                                graphics.size.width as f32,
                                graphics.size.height as f32,
                            );
                            
                            let (ray_origin, ray_direction) = MousePicker::screen_to_world_ray(
                                game_state.mouse_position,
                                screen_size,
                                &camera,
                            );

                            if let Some(((x, y, z), _distance)) = MousePicker::find_clicked_stone(
                                ray_origin,
                                ray_direction,
                                &game_state.rules,
                            ) {
                                // Convert board coordinates to world position for orbit center
                                let board_size = game_state.rules.board().size();
                                let half_size = board_size as f32 * 0.5;
                                let new_center = glam::Vec3::new(
                                    x as f32 - half_size + 0.5,
                                    z as f32 - half_size + 0.5, // y/z swap for rendering
                                    y as f32 - half_size + 0.5,
                                );
                                
                                camera_controller.set_orbit_center(new_center);
                                println!("New orbit center: stone at ({}, {}, {}) -> world pos: {:?}", x, y, z, new_center);
                            }
                            
                            mouse_pressed = false;
                        }
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        let scroll_amount = match delta {
                            MouseScrollDelta::LineDelta(_, y) => *y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                        };
                        // Mouse wheel controls Z plane movement
                        if scroll_amount > 0.0 {
                            game_state.guide_system.move_z(1);
                        } else if scroll_amount < 0.0 {
                            game_state.guide_system.move_z(-1);
                        }
                    }

                    _ => {}
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if mouse_pressed {
                    camera_controller.process_mouse(delta.0, delta.1);
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = Instant::now();
                let dt = now.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = now;

                // Handle pending AI move
                if game_state.pending_ai_move {
                    game_state.make_ai_move();
                    game_state.pending_ai_move = false;
                }

                camera_controller.update_camera(&mut camera, dt);
                graphics.update_camera(&camera);

                // Create guide plane instances
                let guide_instances = vec![game_state.guide_system.get_dot_instance()];

                match graphics.render(&guide_instances, &game_state.black_stone_instances, &game_state.white_stone_instances, &game_state.rules, &camera, Some(&game_state.guide_system)) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        graphics.resize(graphics.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout");
                    }
                }
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        }
    });
}