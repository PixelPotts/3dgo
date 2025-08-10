use super::{Board, StoneColor};
use std::collections::HashSet;

type Position = (u8, u8, u8);

#[derive(Debug, Clone)]
pub struct GameRules {
    board: Board,
    current_player: StoneColor,
    move_history: Vec<Board>,
    ko_rule_positions: HashSet<Position>,
}

impl GameRules {
    pub fn new(board_size: usize) -> Self {
        Self {
            board: Board::new(board_size),
            current_player: StoneColor::Black,
            move_history: Vec::new(),
            ko_rule_positions: HashSet::new(),
        }
    }

    pub fn new_with_dodecahedron(board_size: usize) -> Self {
        Self {
            board: Board::new_with_dodecahedron(board_size),
            current_player: StoneColor::Black,
            move_history: Vec::new(),
            ko_rule_positions: HashSet::new(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn clear_board(&mut self) {
        self.board.clear();
        self.move_history.clear();
        self.ko_rule_positions.clear();
        self.current_player = StoneColor::Black;
    }

    pub fn reset_with_dodecahedron(&mut self) {
        self.board.reset_with_dodecahedron();
        self.move_history.clear();
        self.ko_rule_positions.clear();
        self.current_player = StoneColor::Black;
    }

    pub fn place_test_pattern(&mut self) {
        self.board.place_test_pattern();
        self.move_history.clear();
        self.ko_rule_positions.clear();
        self.current_player = StoneColor::Black;
    }

    pub fn current_player(&self) -> StoneColor {
        self.current_player
    }

    pub fn is_legal_move(&self, x: u8, y: u8, z: u8) -> bool {
        let pos = (x, y, z);

        if !self.board.is_valid_position(x, y, z) {
            return false;
        }

        if self.board.get_stone(pos).is_some() {
            return false;
        }

        let mut test_board = self.board.clone();
        if !test_board.place_stone(self.current_player, x, y, z) {
            return false;
        }

        let opponent_color = self.current_player.opposite();
        let mut captured_groups = Vec::new();

        for neighbor_pos in test_board.get_neighbors(pos) {
            if let Some(neighbor_color) = test_board.get_stone(neighbor_pos) {
                if neighbor_color == opponent_color {
                    if let Some(group) = test_board.get_group(neighbor_pos) {
                        if test_board.get_liberties(&group).is_empty() {
                            captured_groups.push(group);
                        }
                    }
                }
            }
        }

        for group in captured_groups {
            test_board.capture_group(group);
        }

        if !test_board.has_liberties(pos) {
            return false;
        }

        if self.ko_rule_positions.contains(&pos) {
            return false;
        }

        true
    }

    pub fn make_move(&mut self, x: u8, y: u8, z: u8) -> bool {
        if !self.is_legal_move(x, y, z) {
            return false;
        }

        self.move_history.push(self.board.clone());
        
        let pos = (x, y, z);
        self.board.place_stone(self.current_player, x, y, z);

        let opponent_color = self.current_player.opposite();
        let mut captured_any = false;

        for neighbor_pos in self.board.get_neighbors(pos) {
            if let Some(neighbor_color) = self.board.get_stone(neighbor_pos) {
                if neighbor_color == opponent_color {
                    if let Some(group) = self.board.get_group(neighbor_pos) {
                        if self.board.get_liberties(&group).is_empty() {
                            self.board.capture_group(group);
                            captured_any = true;
                        }
                    }
                }
            }
        }

        self.ko_rule_positions.clear();
        if captured_any && self.move_history.len() >= 2 {
            let prev_board = &self.move_history[self.move_history.len() - 2];
            if self.boards_equal(&self.board, prev_board) {
                self.ko_rule_positions.insert(pos);
            }
        }

        self.current_player = self.current_player.opposite();
        true
    }

    fn boards_equal(&self, board1: &Board, board2: &Board) -> bool {
        if board1.size() != board2.size() {
            return false;
        }

        for x in 0..board1.size() {
            for y in 0..board1.size() {
                for z in 0..board1.size() {
                    let pos = (x as u8, y as u8, z as u8);
                    if board1.get_stone(pos) != board2.get_stone(pos) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn pass(&mut self) {
        self.move_history.push(self.board.clone());
        self.current_player = self.current_player.opposite();
    }

    pub fn can_undo(&self) -> bool {
        !self.move_history.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev_board) = self.move_history.pop() {
            self.board = prev_board;
            self.current_player = self.current_player.opposite();
            self.ko_rule_positions.clear();
            true
        } else {
            false
        }
    }

    pub fn get_territory_score(&self) -> (usize, usize) {
        let mut black_territory = 0;
        let mut white_territory = 0;

        for x in 0..self.board.size() {
            for y in 0..self.board.size() {
                for z in 0..self.board.size() {
                    let pos = (x as u8, y as u8, z as u8);
                    if self.board.get_stone(pos).is_none() {
                        if let Some(controlling_color) = self.get_territory_owner(pos) {
                            match controlling_color {
                                StoneColor::Black => black_territory += 1,
                                StoneColor::White => white_territory += 1,
                            }
                        }
                    }
                }
            }
        }

        (black_territory, white_territory)
    }

    fn get_territory_owner(&self, pos: Position) -> Option<StoneColor> {
        let mut visited = HashSet::new();
        let mut stack = vec![pos];
        let mut territory = HashSet::new();
        let mut bordering_colors = HashSet::new();

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(color) = self.board.get_stone(current) {
                bordering_colors.insert(color);
            } else {
                territory.insert(current);
                
                for neighbor in self.board.get_neighbors(current) {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }

        if bordering_colors.len() == 1 {
            bordering_colors.into_iter().next()
        } else {
            None
        }
    }
}