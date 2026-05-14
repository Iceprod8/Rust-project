use noise::{NoiseFn, Perlin};

use crate::domain::{Position, Tile};

const DEFAULT_SCALE: f64 = 14.0;
const DEFAULT_THRESHOLD: f64 = 0.32;
const DEFAULT_MAX_DENSITY: f32 = 0.30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridError {
    OutOfBounds(Position),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleParams {
    pub seed: u32,
    pub scale: f64,
    pub threshold: f64,
    pub max_density: f32,
}

impl ObstacleParams {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            scale: DEFAULT_SCALE,
            threshold: DEFAULT_THRESHOLD,
            max_density: DEFAULT_MAX_DENSITY,
        }
    }

    fn scale(self) -> f64 {
        if self.scale.is_finite() && self.scale > 0.0 {
            self.scale
        } else {
            DEFAULT_SCALE
        }
    }

    fn max_tiles(self, total: usize) -> usize {
        let density = self.max_density.clamp(0.0, DEFAULT_MAX_DENSITY);

        (total as f32 * density).round() as usize
    }
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

    pub fn with_obstacles(width: usize, height: usize, seed: u32) -> Self {
        let mut grid = Self::new(width, height);

        grid.generate_obstacles(ObstacleParams::new(seed));
        grid
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < self.width && (pos.y as usize) < self.height
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

    pub fn generate_obstacles(&mut self, params: ObstacleParams) {
        let noise = Perlin::new(params.seed);
        let scale = params.scale();
        let mut candidates = Vec::new();

        for idx in 0..self.tiles.len() {
            if matches!(self.tiles[idx], Tile::Base | Tile::Resource(_)) {
                continue;
            }

            let x = (idx % self.width) as f64 / scale;
            let y = (idx / self.width) as f64 / scale;
            let score = noise.get([x, y]);

            self.tiles[idx] = Tile::Empty;

            if score >= params.threshold {
                candidates.push((idx, score));
            }
        }

        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

        for (idx, _) in candidates
            .into_iter()
            .take(params.max_tiles(self.tiles.len()))
        {
            self.tiles[idx] = Tile::Obstacle;
        }
    }

    fn tile_index(&self, pos: Position) -> Option<usize> {
        if !self.in_bounds(pos) {
            return None;
        }

        Some(pos.y as usize * self.width + pos.x as usize)
    }
}

pub fn register() {}
