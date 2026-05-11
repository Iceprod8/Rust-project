use crate::domain::{Position, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridError {
    OutOfBounds(Position),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Empty; width.saturating_mul(height)],
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
    }

    pub fn get_tile(&self, pos: Position) -> Option<Tile> {
        self.tile_index(pos).map(|idx| self.tiles[idx])
    }

    pub fn set_tile(&mut self, pos: Position, tile: Tile) -> Result<(), GridError> {
        let idx = self.tile_index(pos).ok_or(GridError::OutOfBounds(pos))?;

        self.tiles[idx] = tile;
        Ok(())
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        matches!(
            self.get_tile(pos),
            Some(Tile::Empty | Tile::Base | Tile::Resource(_))
        )
    }

    fn tile_index(&self, pos: Position) -> Option<usize> {
        if !self.in_bounds(pos) {
            return None;
        }

        Some(pos.y as usize * self.width + pos.x as usize)
    }
}

pub fn register() {}
