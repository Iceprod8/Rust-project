use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng, rngs::StdRng};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    MissingBase,
    InvalidBase(Position),
    NoSpawn,
    InvalidSpawn(Position),
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
    base: Option<Position>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Empty; width.saturating_mul(height)],
            resources: Vec::new(),
            base: None,
        }
    }

    pub fn with_obstacles(width: usize, height: usize, seed: u32) -> Self {
        let mut grid = Self::new(width, height);

        grid.generate_obstacles(ObstacleParams::new(seed));
        grid
    }

    pub fn with_base(width: usize, height: usize) -> Result<Self, GridError> {
        let mut grid = Self::new(width, height);

        grid.place_base()?;
        Ok(grid)
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

    pub const fn base_position(&self) -> Option<Position> {
        self.base
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
        self.update_base(pos, tile);
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

            if self.is_spawn_idx(idx) {
                self.tiles[idx] = Tile::Empty;
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

    pub fn place_base(&mut self) -> Result<Position, GridError> {
        let pos = self.default_base_position();

        if !self.in_bounds(pos) {
            return Err(GridError::OutOfBounds(pos));
        }

        self.clear_base();

        for spawn in self.base_neighbors(pos) {
            self.set_tile(spawn, Tile::Empty)?;
        }

        self.set_tile(pos, Tile::Base)?;
        Ok(pos)
    }

    pub fn spawn_positions(&self) -> Vec<Position> {
        self.base
            .map(|base| {
                self.base_neighbors(base)
                    .into_iter()
                    .filter(|pos| self.is_walkable(*pos))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn validate_start(&self) -> Result<(), ValidationError> {
        let base = self.base.ok_or(ValidationError::MissingBase)?;

        if self.get_tile(base) != Some(Tile::Base) {
            return Err(ValidationError::InvalidBase(base));
        }

        let spawns = self.base_neighbors(base);

        if spawns.is_empty() {
            return Err(ValidationError::NoSpawn);
        }

        for spawn in spawns {
            if !self.is_walkable(spawn) {
                return Err(ValidationError::InvalidSpawn(spawn));
            }
        }

        Ok(())
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
            .filter_map(|(idx, tile)| {
                (matches!(tile, Tile::Empty) && !self.is_spawn_idx(idx)).then_some(idx)
            })
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

    fn clear_base(&mut self) {
        if let Some(pos) = self.base.take() {
            if let Some(idx) = self.tile_index(pos) {
                if matches!(self.tiles[idx], Tile::Base) {
                    self.tiles[idx] = Tile::Empty;
                }
            }
        }
    }

    fn update_base(&mut self, pos: Position, tile: Tile) {
        if matches!(tile, Tile::Base) {
            self.clear_base();
            self.base = Some(pos);
        } else if self.base == Some(pos) {
            self.base = None;
        }
    }

    fn default_base_position(&self) -> Position {
        Position::new((self.width / 2) as i32, (self.height / 2) as i32)
    }

    fn base_neighbors(&self, base: Position) -> Vec<Position> {
        [
            Position::new(base.x, base.y - 1),
            Position::new(base.x + 1, base.y),
            Position::new(base.x, base.y + 1),
            Position::new(base.x - 1, base.y),
        ]
        .into_iter()
        .filter(|pos| self.in_bounds(*pos))
        .collect()
    }

    fn is_spawn_idx(&self, idx: usize) -> bool {
        let Some(base) = self.base else {
            return false;
        };

        let pos = self.pos_from_idx(idx);
        (pos.x - base.x).abs() + (pos.y - base.y).abs() == 1
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
