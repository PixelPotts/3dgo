use super::{Stone, StoneColor};
use std::collections::{HashMap, HashSet};

pub const BOARD_SIZE: usize = 3;

type Position = (u8, u8, u8);

#[derive(Debug, Clone)]
pub struct Board {
    stones: HashMap<Position, StoneColor>,
    size: usize,
    captured_black: usize,
    captured_white: usize,
}

impl Default for Board {
    fn default() -> Self {
        Self::new_with_dodecahedron(BOARD_SIZE)
    }
}

impl Board {
    pub fn new(size: usize) -> Self {
        Self {
            stones: HashMap::new(),
            size,
            captured_black: 0,
            captured_white: 0,
        }
    }

    pub fn new_with_dodecahedron(size: usize) -> Self {
        let mut board = Self::new(size);
        board.place_dodecahedron();
        board
    }

    fn place_dodecahedron(&mut self) {
        // For a 3x3 board, just place a simple pattern
        if self.size == 3 {
            // Simple pattern for 3x3
            self.place_stone(StoneColor::Black, 0, 0, 0);
            self.place_stone(StoneColor::Black, 2, 0, 2);
            self.place_stone(StoneColor::White, 1, 1, 1);
            self.place_stone(StoneColor::Black, 0, 2, 0);
            self.place_stone(StoneColor::Black, 2, 2, 2);
            return;
        }
        
        let center = self.size as f32 / 2.0;
        let radius = (self.size as f32 * 0.35).min(5.0);
        
        // Golden ratio for dodecahedron vertices
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let inv_phi = 1.0 / phi;
        
        // Dodecahedron vertices (scaled and centered)
        let vertices = vec![
            // Cube vertices
            (1.0, 1.0, 1.0), (1.0, 1.0, -1.0), (1.0, -1.0, 1.0), (1.0, -1.0, -1.0),
            (-1.0, 1.0, 1.0), (-1.0, 1.0, -1.0), (-1.0, -1.0, 1.0), (-1.0, -1.0, -1.0),
            // Rectangle in yz-plane
            (0.0, inv_phi, phi), (0.0, inv_phi, -phi), (0.0, -inv_phi, phi), (0.0, -inv_phi, -phi),
            // Rectangle in xz-plane  
            (inv_phi, phi, 0.0), (inv_phi, -phi, 0.0), (-inv_phi, phi, 0.0), (-inv_phi, -phi, 0.0),
            // Rectangle in xy-plane
            (phi, 0.0, inv_phi), (phi, 0.0, -inv_phi), (-phi, 0.0, inv_phi), (-phi, 0.0, -inv_phi),
        ];

        // Place black stones at scaled dodecahedron vertices
        for (i, &(x, y, z)) in vertices.iter().enumerate() {
            let pos_x = (center + x * radius) as u8;
            let pos_y = (center + y * radius) as u8;  
            let pos_z = (center + z * radius) as u8;
            
            if self.is_valid_position(pos_x, pos_y, pos_z) {
                self.place_stone(StoneColor::Black, pos_x, pos_y, pos_z);
            }
        }

        // Place white stones at face centers (approximate)
        let face_centers = vec![
            // Pentagonal face centers (approximate positions)
            (0.0, 0.7, 1.4), (0.0, 0.7, -1.4), (0.0, -0.7, 1.4), (0.0, -0.7, -1.4),
            (1.4, 0.0, 0.7), (1.4, 0.0, -0.7), (-1.4, 0.0, 0.7), (-1.4, 0.0, -0.7),
            (0.7, 1.4, 0.0), (0.7, -1.4, 0.0), (-0.7, 1.4, 0.0), (-0.7, -1.4, 0.0),
        ];

        for &(x, y, z) in &face_centers {
            let pos_x = (center + x * radius * 0.8) as u8;
            let pos_y = (center + y * radius * 0.8) as u8;
            let pos_z = (center + z * radius * 0.8) as u8;
            
            if self.is_valid_position(pos_x, pos_y, pos_z) {
                self.place_stone(StoneColor::White, pos_x, pos_y, pos_z);
            }
        }
    }

    pub fn clear(&mut self) {
        self.stones.clear();
        self.captured_black = 0;
        self.captured_white = 0;
    }

    pub fn reset_with_dodecahedron(&mut self) {
        self.clear();
        self.place_dodecahedron();
    }

    pub fn place_test_pattern(&mut self) {
        self.clear();
        println!("=== PLACING TEST PATTERN FOR 3x3x3 BOARD ===");
        
        // For a 3x3x3 board, positions are 0, 1, 2
        
        // Layer 0 (bottom) - Corner markers
        self.place_stone(StoneColor::Black, 0, 0, 0);
        println!("  Layer Y=0: Black at (0,0,0) - origin corner");
        self.place_stone(StoneColor::White, 2, 0, 0);
        println!("  Layer Y=0: White at (2,0,0) - X-axis corner");
        self.place_stone(StoneColor::Black, 0, 0, 2);
        println!("  Layer Y=0: Black at (0,0,2) - Z-axis corner");
        
        // Layer 1 (middle) - Center and edges
        self.place_stone(StoneColor::White, 1, 1, 1);
        println!("  Layer Y=1: White at (1,1,1) - center");
        self.place_stone(StoneColor::Black, 0, 1, 1);
        println!("  Layer Y=1: Black at (0,1,1) - left middle");
        self.place_stone(StoneColor::White, 2, 1, 1);
        println!("  Layer Y=1: White at (2,1,1) - right middle");
        
        // Layer 2 (top) - Corner markers
        self.place_stone(StoneColor::White, 2, 2, 2);
        println!("  Layer Y=2: White at (2,2,2) - far corner");
        self.place_stone(StoneColor::Black, 0, 2, 2);
        println!("  Layer Y=2: Black at (0,2,2) - Y-Z corner");
        self.place_stone(StoneColor::White, 1, 2, 0);
        println!("  Layer Y=2: White at (1,2,0) - top middle front");
        
        println!("=== TEST PATTERN COMPLETE ===");
        println!("Total stones placed: {}", self.stones.len());
        
        // Report layer occupancy for verification
        let mut layers_x = std::collections::HashSet::new();
        let mut layers_y = std::collections::HashSet::new();
        let mut layers_z = std::collections::HashSet::new();
        
        for ((x, y, z), _) in &self.stones {
            layers_x.insert(*x);
            layers_y.insert(*y);
            layers_z.insert(*z);
        }
        
        println!("Occupied X layers: {:?}", layers_x);
        println!("Occupied Y layers: {:?}", layers_y);
        println!("Occupied Z layers: {:?}", layers_z);
        
        // Print a visual representation for debugging
        println!("\n=== VISUAL REPRESENTATION ===");
        for y in (0..3).rev() {
            println!("Layer Y={}:", y);
            for z in 0..3 {
                print!("  ");
                for x in 0..3 {
                    if let Some(color) = self.get_stone((x, y, z)) {
                        match color {
                            StoneColor::Black => print!("B "),
                            StoneColor::White => print!("W "),
                        }
                    } else {
                        print!(". ");
                    }
                }
                println!(" (z={})", z);
            }
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_valid_position(&self, x: u8, y: u8, z: u8) -> bool {
        (x as usize) < self.size && (y as usize) < self.size && (z as usize) < self.size
    }

    pub fn get_stone(&self, pos: Position) -> Option<StoneColor> {
        self.stones.get(&pos).copied()
    }

    pub fn place_stone(&mut self, color: StoneColor, x: u8, y: u8, z: u8) -> bool {
        let pos = (x, y, z);
        
        if !self.is_valid_position(x, y, z) || self.stones.contains_key(&pos) {
            return false;
        }

        self.stones.insert(pos, color);
        true
    }

    pub fn remove_stone(&mut self, pos: Position) -> Option<StoneColor> {
        self.stones.remove(&pos)
    }

    pub fn get_neighbors(&self, pos: Position) -> Vec<Position> {
        let (x, y, z) = pos;
        let mut neighbors = Vec::new();
        
        let directions = [
            (-1, 0, 0), (1, 0, 0),   // x-axis
            (0, -1, 0), (0, 1, 0),   // y-axis  
            (0, 0, -1), (0, 0, 1),   // z-axis
        ];

        for (dx, dy, dz) in directions {
            let nx = x as i8 + dx;
            let ny = y as i8 + dy;
            let nz = z as i8 + dz;

            if nx >= 0 && ny >= 0 && nz >= 0 {
                let (nx, ny, nz) = (nx as u8, ny as u8, nz as u8);
                if self.is_valid_position(nx, ny, nz) {
                    neighbors.push((nx, ny, nz));
                }
            }
        }

        neighbors
    }

    pub fn get_group(&self, pos: Position) -> Option<HashSet<Position>> {
        let color = self.get_stone(pos)?;
        let mut group = HashSet::new();
        let mut stack = vec![pos];
        
        while let Some(current) = stack.pop() {
            if group.contains(&current) {
                continue;
            }
            
            if let Some(stone_color) = self.get_stone(current) {
                if stone_color == color {
                    group.insert(current);
                    
                    for neighbor in self.get_neighbors(current) {
                        if !group.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        Some(group)
    }

    pub fn get_liberties(&self, group: &HashSet<Position>) -> HashSet<Position> {
        let mut liberties = HashSet::new();
        
        for &pos in group {
            for neighbor in self.get_neighbors(pos) {
                if !self.stones.contains_key(&neighbor) {
                    liberties.insert(neighbor);
                }
            }
        }

        liberties
    }

    pub fn has_liberties(&self, pos: Position) -> bool {
        if let Some(group) = self.get_group(pos) {
            !self.get_liberties(&group).is_empty()
        } else {
            false
        }
    }

    pub fn capture_group(&mut self, group: HashSet<Position>) -> usize {
        let mut captured = 0;
        
        for pos in group {
            if let Some(color) = self.remove_stone(pos) {
                captured += 1;
                match color {
                    StoneColor::Black => self.captured_black += 1,
                    StoneColor::White => self.captured_white += 1,
                }
            }
        }

        captured
    }

    pub fn get_captured(&self, color: StoneColor) -> usize {
        match color {
            StoneColor::Black => self.captured_black,
            StoneColor::White => self.captured_white,
        }
    }

    pub fn get_all_stones(&self) -> impl Iterator<Item = (&Position, &StoneColor)> {
        self.stones.iter()
    }
}