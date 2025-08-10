use super::{Camera, Graphics, Instance};
use crate::game::{GameRules, StoneColor};
use glam::{Vec3, Mat4};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum ViewDirection {
    Top,
    Bottom, 
    Left,
    Right,
    Front,
    Back,
}

impl ViewDirection {
    pub fn all() -> [ViewDirection; 6] {
        [
            ViewDirection::Top,
            ViewDirection::Left,
            ViewDirection::Right,
            ViewDirection::Back,
            ViewDirection::Front,
            ViewDirection::Bottom,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewDirection::Top => "TOP",
            ViewDirection::Bottom => "BOTTOM",
            ViewDirection::Left => "LEFT", 
            ViewDirection::Right => "RIGHT",
            ViewDirection::Front => "FRONT",
            ViewDirection::Back => "BACK",
        }
    }

    pub fn get_camera_position(&self, board_size: f32) -> (Vec3, Vec3, Vec3) {
        let distance = board_size * 1.5;
        let center = Vec3::new(0.0, 0.0, 0.0);
        
        match self {
            ViewDirection::Top => (
                Vec3::new(0.0, distance, 0.0),
                center,
                Vec3::new(0.0, 0.0, 1.0)
            ),
            ViewDirection::Bottom => (
                Vec3::new(0.0, -distance, 0.0),
                center,
                Vec3::new(0.0, 0.0, -1.0)
            ),
            ViewDirection::Left => (
                Vec3::new(-distance, 0.0, 0.0),
                center,
                Vec3::new(0.0, 1.0, 0.0)
            ),
            ViewDirection::Right => (
                Vec3::new(distance, 0.0, 0.0),
                center,
                Vec3::new(0.0, 1.0, 0.0)
            ),
            ViewDirection::Front => (
                Vec3::new(0.0, 0.0, distance),
                center,
                Vec3::new(0.0, 1.0, 0.0)
            ),
            ViewDirection::Back => (
                Vec3::new(0.0, 0.0, -distance),
                center,
                Vec3::new(0.0, 1.0, 0.0)
            ),
        }
    }
}

pub struct SideView {
    pub direction: ViewDirection,
    pub current_layer: usize,
    pub animation_time: f32,
    pub layer_cycle_speed: f32,
}

impl SideView {
    pub fn new(direction: ViewDirection) -> Self {
        Self {
            direction,
            current_layer: 0,
            animation_time: 0.0,
            layer_cycle_speed: 0.3, // layers per second (slower for better visibility)
        }
    }

    pub fn update(&mut self, dt: f32, _board_size: usize) {
        self.animation_time += dt;
        
        // Each view cycles through layers at different offsets for visual variety
        let offset = match self.direction {
            ViewDirection::Top => 0.0,
            ViewDirection::Left => 0.2,
            ViewDirection::Right => 0.4,
            ViewDirection::Back => 0.6,
            ViewDirection::Front => 0.8,
            ViewDirection::Bottom => 1.0,
        };
        
        self.animation_time += offset * dt; // Apply offset to the time increment
    }

    pub fn get_visible_stones(&self, game_rules: &GameRules, max_layers: usize) -> (Vec<Instance>, Vec<Instance>) {
        let mut black_stones = Vec::new();
        let mut white_stones = Vec::new();
        let board_size = game_rules.board().size();
        let half_size = board_size as f32 * 0.5;

        // First, find the min and max occupied layers for this view direction
        let mut min_layer = board_size;
        let mut max_layer = 0;
        let mut has_stones = false;
        
        for ((x, y, z), _color) in game_rules.board().get_all_stones() {
            let stone_layer = match self.direction {
                ViewDirection::Top | ViewDirection::Bottom => *y as usize,
                ViewDirection::Left | ViewDirection::Right => *x as usize,
                ViewDirection::Front | ViewDirection::Back => *z as usize,
            };
            
            if stone_layer < min_layer {
                min_layer = stone_layer;
            }
            if stone_layer > max_layer {
                max_layer = stone_layer;
            }
            has_stones = true;
        }

        // If no stones, return empty
        if !has_stones {
            return (black_stones, white_stones);
        }

        // Calculate the range of layers to cycle through (min to max inclusive)
        let layer_range = max_layer - min_layer + 1;
        
        // Calculate which layer in the range to show based on animation
        let cycle_position = (self.animation_time * self.layer_cycle_speed) % layer_range as f32;
        let current_layer = min_layer + cycle_position as usize;
        
        // Debug output - only print occasionally to avoid spam
        static mut DEBUG_COUNTER: usize = 0;
        unsafe {
            DEBUG_COUNTER += 1;
            if DEBUG_COUNTER % 300 == 0 {  // Print every ~5 seconds at 60fps
                println!("[{:?}] Layer {}/{} (range: {}-{})", 
                    self.direction, current_layer, layer_range, min_layer, max_layer);
            }
        }

        // Show only stones from the current animated layer
        for ((x, y, z), color) in game_rules.board().get_all_stones() {
            let stone_layer = match self.direction {
                ViewDirection::Top | ViewDirection::Bottom => *y as usize,
                ViewDirection::Left | ViewDirection::Right => *x as usize,
                ViewDirection::Front | ViewDirection::Back => *z as usize,
            };

            if stone_layer == current_layer {
                let pos = Vec3::new(
                    *x as f32 - half_size + 0.5,
                    *z as f32 - half_size + 0.5,
                    *y as f32 - half_size + 0.5,
                );
                
                let mut instance = Instance::new(pos);
                instance.scale = Vec3::splat(0.8); // Smaller for side views
                
                match color {
                    StoneColor::Black => black_stones.push(instance),
                    StoneColor::White => white_stones.push(instance),
                }
            }
        }

        (black_stones, white_stones)
    }
}

pub struct UISystem {
    pub side_views: [SideView; 6],
    pub last_update_time: Instant,
}

impl UISystem {
    pub fn new() -> Self {
        let directions = ViewDirection::all();
        let side_views = [
            SideView::new(directions[0]),
            SideView::new(directions[1]),
            SideView::new(directions[2]),
            SideView::new(directions[3]),
            SideView::new(directions[4]),
            SideView::new(directions[5]),
        ];

        Self {
            side_views,
            last_update_time: Instant::now(),
        }
    }

    pub fn update(&mut self, board_size: usize) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update_time).as_secs_f32();
        self.last_update_time = now;

        for view in &mut self.side_views {
            view.update(dt, board_size);
        }
    }
}