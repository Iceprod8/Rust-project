use noise::{NoiseFn, Perlin};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::domain::{Position, ResourceNode, ResourceType, Tile};

const DEFAULT_SCALE: f64 = 14.0;
const DEFAULT_THRESHOLD: f64 = 0.32;
const DEFAULT_MAX_DENSITY: f32 = 0.30;
const MIN_RESOURCE_AMOUNT: u16 = 50;
const MAX_RESOURCE_AMOUNT: u16 = 200;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceParams {
    pub seed: u64,
    pub energy: usize,
    pub crystals: usize,
}

impl ResourceParams {
    pub const fn new(seed: u64, energy: usize, crystals: usize) -> Self {
        Self {
            seed,
            energy,
            crystals,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    resources: Vec<ResourceNode>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Empty; width.saturating_mul(height)],
            resources: Vec::new(),
        }
    }

    pub fn with_obstacles(width: usize, height: usize, seed: u32) -> Self {
        let mut grid = Self::new(width, height);

        grid.generate_obstacles(ObstacleParams::new(seed));
        grid
    }

    pub fn with_resources(width: usize, height: usize, params: ResourceParams) -> Self {
        let mut grid = Self::new(width, height);

        grid.place_resources(params);
        grid
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn resources(&self) -> &[ResourceNode] {
        &self.resources
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < self.width && (pos.y as usize) < self.height
    }

    pub fn get_tile(&self, pos: Position) -> Option<Tile> {
        self.tile_index(pos).map(|idx| self.tiles[idx])
    }

    pub fn set_tile(&mut self, pos: Position, tile: Tile) -> Result<(), GridError> {
        let idx = self.tile_index(pos).ok_or(GridError::OutOfBounds(pos))?;

        self.resources.retain(|resource| resource.position != pos);
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

    pub fn place_resources(&mut self, params: ResourceParams) {
        let mut rng = StdRng::seed_from_u64(params.seed);
        self.clear_resources();

        let mut free_tiles = self.free_tiles();

        self.place_resource_type(
            ResourceType::Energy,
            params.energy,
            &mut rng,
            &mut free_tiles,
        );
        self.place_resource_type(
            ResourceType::Crystal,
            params.crystals,
            &mut rng,
            &mut free_tiles,
        );
    }

    pub fn take_resource(&mut self, pos: Position) -> Option<ResourceType> {
        let resource = self
            .resources
            .iter_mut()
            .find(|resource| resource.position == pos)?;

        if resource.remaining == 0 {
            return None;
        }

        resource.remaining -= 1;
        let resource_type = resource.resource_type;

        if resource.remaining == 0 {
            self.resources.retain(|resource| resource.position != pos);

            if let Some(idx) = self.tile_index(pos) {
                self.tiles[idx] = Tile::Empty;
            }
        }

        Some(resource_type)
    }

    fn tile_index(&self, pos: Position) -> Option<usize> {
        if !self.in_bounds(pos) {
            return None;
        }

        Some(pos.y as usize * self.width + pos.x as usize)
    }

    fn free_tiles(&self) -> Vec<usize> {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, tile)| matches!(tile, Tile::Empty).then_some(idx))
            .collect()
    }

    fn clear_resources(&mut self) {
        let resources = std::mem::take(&mut self.resources);

        for resource in resources {
            if let Some(idx) = self.tile_index(resource.position) {
                self.tiles[idx] = Tile::Empty;
            }
        }
    }

    fn pos_from_idx(&self, idx: usize) -> Position {
        Position::new((idx % self.width) as i32, (idx / self.width) as i32)
    }

    fn place_resource_type(
        &mut self,
        resource_type: ResourceType,
        count: usize,
        rng: &mut StdRng,
        free_tiles: &mut Vec<usize>,
    ) {
        for _ in 0..count {
            if free_tiles.is_empty() {
                return;
            }

            let choice = rng.random_range(0..free_tiles.len());
            let idx = free_tiles.swap_remove(choice);
            let pos = self.pos_from_idx(idx);
            let remaining = rng.random_range(MIN_RESOURCE_AMOUNT..=MAX_RESOURCE_AMOUNT);

            self.tiles[idx] = Tile::Resource(resource_type);
            self.resources
                .push(ResourceNode::new(pos, resource_type, remaining));
        }
    }
}

pub fn register() {}
