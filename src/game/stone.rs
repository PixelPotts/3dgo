#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoneColor {
    Black,
    White,
}

impl StoneColor {
    pub fn opposite(&self) -> Self {
        match self {
            StoneColor::Black => StoneColor::White,
            StoneColor::White => StoneColor::Black,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stone {
    pub color: StoneColor,
    pub position: (u8, u8, u8),
}

impl Stone {
    pub fn new(color: StoneColor, x: u8, y: u8, z: u8) -> Self {
        Self {
            color,
            position: (x, y, z),
        }
    }
}