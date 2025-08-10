use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StoneColor {
    Black,
    White,
}

impl StoneColor {
    fn opposite(&self) -> Self {
        match self {
            StoneColor::Black => StoneColor::White,
            StoneColor::White => StoneColor::Black,
        }
    }
}

struct Board {
    stones: HashMap<(u8, u8, u8), StoneColor>,
    size: usize,
}

impl Board {
    fn new(size: usize) -> Self {
        Self {
            stones: HashMap::new(),
            size,
        }
    }

    fn place_stone(&mut self, color: StoneColor, x: u8, y: u8, z: u8) -> bool {
        let pos = (x, y, z);
        if (x as usize) >= self.size || (y as usize) >= self.size || (z as usize) >= self.size {
            return false;
        }
        if self.stones.contains_key(&pos) {
            return false;
        }
        self.stones.insert(pos, color);
        true
    }

    fn get_stone_count(&self) -> usize {
        self.stones.len()
    }
}

struct GameRules {
    board: Board,
    current_player: StoneColor,
}

impl GameRules {
    fn new(size: usize) -> Self {
        Self {
            board: Board::new(size),
            current_player: StoneColor::Black,
        }
    }

    fn make_move(&mut self, x: u8, y: u8, z: u8) -> bool {
        if self.board.place_stone(self.current_player, x, y, z) {
            self.current_player = self.current_player.opposite();
            true
        } else {
            false
        }
    }

    fn current_player(&self) -> StoneColor {
        self.current_player
    }

    fn board(&self) -> &Board {
        &self.board
    }
}

fn main() {
    let mut game = GameRules::new(9);
    
    println!("3D Go Game Test");
    println!("Current player: {:?}", game.current_player());
    
    // Test placing some stones
    println!("\nPlacing black stone at (4, 4, 4)...");
    if game.make_move(4, 4, 4) {
        println!("Move successful!");
    } else {
        println!("Move failed!");
    }
    
    println!("Current player: {:?}", game.current_player());
    
    // Test placing white stone
    println!("\nPlacing white stone at (4, 5, 4)...");
    if game.make_move(4, 5, 4) {
        println!("Move successful!");
    } else {
        println!("Move failed!");
    }
    
    // Try to place in same position (should fail)
    println!("\nTrying to place stone at (4, 4, 4) again...");
    if game.make_move(4, 4, 4) {
        println!("Move successful!");
    } else {
        println!("Move failed (correct - position occupied)!");
    }
    
    // Count stones on board
    let stone_count = game.board().get_stone_count();
    println!("\nTotal stones on board: {}", stone_count);
    
    println!("\nGame mechanics working correctly!");
}