mod game;
mod render;
mod input;

use game::{GameRules, StoneColor};
use render::{Graphics, Camera, CameraController, Instance};
use input::MousePicker;
use glam::Vec3;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use web_time::Instant;
    } else {
        use std::time::Instant;
    }
}

struct GameState {
    rules: GameRules,
    black_stone_instances: Vec<Instance>,
    white_stone_instances: Vec<Instance>,
    selected_position: Option<(u8, u8, u8)>,
    mouse_position: glam::Vec2,
    animation_paused: bool,
}

impl GameState {
    fn new() -> Self {
        let rules = GameRules::new_with_dodecahedron(3); // Use 3x3x3 board
        let black_stone_instances = Vec::new();
        let white_stone_instances = Vec::new();

        Self {
            rules,
            black_stone_instances,
            white_stone_instances,
            selected_position: None,
            mouse_position: glam::Vec2::ZERO,
            animation_paused: false,
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
            // Scale stones to be more visible
            instance.scale = Vec3::splat(1.2);
            
            // Add to appropriate instance list based on color
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
}

async fn run() {
    env_logger::init();
    
    // Check for command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let use_test_pattern = args.iter().any(|arg| arg == "--test" || arg == "-t");
    let debug_mode = args.iter().any(|arg| arg == "--debug" || arg == "-d");
    
    if debug_mode || use_test_pattern {
        println!("\n========================================");
        println!("3D GO DEBUG MODE ACTIVATED");
        println!("========================================");
        println!("Keyboard shortcuts:");
        println!("  T - Load test pattern");
        println!("  P - Pause/resume animation");
        println!("  R - Reset board");
        println!("  D - Load dodecahedron");
        println!("  Space + Mouse - Pan camera");
        println!("========================================\n");
    }
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("3D Go")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let mut graphics = Graphics::new(&window).await;
    let mut camera = Camera::new(graphics.size.width, graphics.size.height);
    let mut camera_controller = CameraController::new(10.0, 1.0);
    let mut game_state = GameState::new();
    
    // Load test pattern if requested
    if use_test_pattern {
        println!("Loading test pattern...");
        game_state.rules.place_test_pattern();
    }
    
    let mut last_frame_time = Instant::now();
    let mut mouse_pressed = false;
    let target_fps = 90.0;
    let target_frame_time = std::time::Duration::from_secs_f32(1.0 / target_fps);

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
                                    VirtualKeyCode::R => {
                                        // Reset - clear the board
                                        game_state.rules.clear_board();
                                        game_state.update_stones();
                                    }
                                    VirtualKeyCode::T => {
                                        // Test pattern for debugging side views
                                        println!("\n==== ACTIVATING TEST PATTERN MODE ====");
                                        game_state.rules.place_test_pattern();
                                        game_state.update_stones();
                                        println!("Press P to pause/resume animation");
                                        println!("================================\n");
                                    }
                                    VirtualKeyCode::P => {
                                        // Toggle animation pause
                                        game_state.animation_paused = !game_state.animation_paused;
                                        println!("Animation: {}", if game_state.animation_paused { "PAUSED" } else { "RUNNING" });
                                    }
                                    // Guide plane controls
                                    VirtualKeyCode::A => {
                                        graphics.guide_system_mut().move_x(-1);
                                    }
                                    VirtualKeyCode::D => {
                                        graphics.guide_system_mut().move_x(1);
                                    }
                                    VirtualKeyCode::W => {
                                        graphics.guide_system_mut().move_y(1);
                                    }
                                    VirtualKeyCode::S => {
                                        graphics.guide_system_mut().move_y(-1);
                                    }
                                    VirtualKeyCode::Space => {
                                        // Place stone at guide intersection
                                        let (x, y, z) = graphics.guide_system_mut().get_intersection_position();
                                        if game_state.rules.make_move(x, y, z) {
                                            game_state.update_stones();
                                        }
                                    }
                                    // Zoom controls
                                    VirtualKeyCode::Q | VirtualKeyCode::Left => {
                                        camera_controller.zoom_in();
                                    }
                                    VirtualKeyCode::E | VirtualKeyCode::Right => {
                                        camera_controller.zoom_out();
                                    }
                                    VirtualKeyCode::Up | VirtualKeyCode::Down => {
                                        // Arrow keys up/down don't do anything now (used to be W/S for camera)
                                        // Ignore these since W/S now control guide planes
                                    }
                                    _ => {
                                        // Pass remaining keys to camera controller (but not Q/E/arrows)
                                        match key {
                                            VirtualKeyCode::Q | VirtualKeyCode::E | 
                                            VirtualKeyCode::Left | VirtualKeyCode::Right |
                                            VirtualKeyCode::Up | VirtualKeyCode::Down => {
                                                // These are handled above, don't pass to camera controller
                                            }
                                            _ => {
                                                camera_controller.process_keyboard(key, input.state);
                                            }
                                        }
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
                        
                        // Mouse wheel = Z-axis guide plane movement only
                        if scroll_amount > 0.0 {
                            graphics.guide_system_mut().move_z(1);
                        } else if scroll_amount < 0.0 {
                            graphics.guide_system_mut().move_z(-1);
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

                camera_controller.update_camera(&mut camera, dt);
                graphics.update_camera(&camera);

                match graphics.render(&[], &game_state.black_stone_instances, &game_state.white_stone_instances, &game_state.rules, &camera) {
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
                let now = Instant::now();
                let elapsed = now.duration_since(last_frame_time);
                if elapsed >= target_frame_time {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    });
}

fn main() {
    pollster::block_on(run());
}
