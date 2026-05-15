use rust_project::domain::{Position, ResourceType, Tile};
use rust_project::map::{Grid, ObstacleParams, ResourceParams, ValidationError};

fn tiles(grid: &Grid) -> Vec<Tile> {
    let mut values = Vec::new();

    for y in 0..grid.height() {
        for x in 0..grid.width() {
            values.push(grid.get_tile(Position::new(x as i32, y as i32)).unwrap());
        }
    }

    values
}

fn count_tile(grid: &Grid, tile: Tile) -> usize {
    tiles(grid)
        .into_iter()
        .filter(|current| *current == tile)
        .count()
}

#[test]
fn grid_reads_writes_and_walkability_work() {
    let mut grid = Grid::new(4, 3);
    let pos = Position::new(2, 1);

    assert!(grid.in_bounds(pos));
    assert!(!grid.in_bounds(Position::new(4, 1)));
    assert_eq!(grid.get_tile(pos), Some(Tile::Empty));

    grid.set_tile(pos, Tile::Obstacle).unwrap();

    assert_eq!(grid.get_tile(pos), Some(Tile::Obstacle));
    assert!(!grid.is_walkable(pos));
    assert!(grid.set_tile(Position::new(-1, 0), Tile::Empty).is_err());
}

#[test]
fn obstacles_depend_on_seed_and_keep_a_playable_density() {
    let grid_a = Grid::with_obstacles(30, 20, 10);
    let grid_b = Grid::with_obstacles(30, 20, 11);
    let obstacles = count_tile(&grid_a, Tile::Obstacle);

    assert_ne!(tiles(&grid_a), tiles(&grid_b));
    assert!(obstacles > 0);
    assert!(obstacles <= 180);
}

#[test]
fn custom_obstacle_density_is_limited() {
    let mut grid = Grid::new(20, 20);

    grid.generate_obstacles(ObstacleParams {
        seed: 1,
        scale: 5.0,
        threshold: -1.0,
        max_density: 0.80,
    });

    assert!(count_tile(&grid, Tile::Obstacle) <= 120);
}

#[test]
fn resources_are_placed_on_free_tiles_with_valid_amounts() {
    let mut grid = Grid::with_obstacles(20, 20, 3);

    grid.place_resources(ResourceParams::new(7, 4, 3));

    assert_eq!(grid.resources().len(), 7);

    for resource in grid.resources() {
        assert!((50..=200).contains(&resource.remaining));
        assert_eq!(
            grid.get_tile(resource.position),
            Some(Tile::Resource(resource.resource_type))
        );
    }

    assert_eq!(
        grid.resources()
            .iter()
            .filter(|resource| resource.resource_type == ResourceType::Energy)
            .count(),
        4
    );
    assert_eq!(
        grid.resources()
            .iter()
            .filter(|resource| resource.resource_type == ResourceType::Crystal)
            .count(),
        3
    );
}

#[test]
fn taking_a_resource_removes_it_when_empty() {
    let mut grid = Grid::with_resources(4, 4, ResourceParams::new(2, 1, 0));
    let resource = grid.resources()[0].clone();

    for _ in 0..resource.remaining {
        assert_eq!(
            grid.take_resource(resource.position),
            Some(resource.resource_type)
        );
    }

    assert_eq!(grid.take_resource(resource.position), None);
    assert_eq!(grid.get_tile(resource.position), Some(Tile::Empty));
    assert!(grid.resources().is_empty());
}

#[test]
fn base_is_centered_and_spawns_stay_walkable() {
    let mut grid = Grid::with_obstacles(20, 20, 5);
    let base = grid.place_base().unwrap();

    grid.place_resources(ResourceParams::new(12, 8, 8));

    assert_eq!(base, Position::new(10, 10));
    assert_eq!(grid.base_position(), Some(base));
    assert_eq!(grid.get_tile(base), Some(Tile::Base));
    assert_eq!(grid.spawn_positions().len(), 4);
    assert!(grid.validate_start().is_ok());

    for spawn in grid.spawn_positions() {
        assert!(grid.is_walkable(spawn));
    }
}

#[test]
fn validation_fails_when_a_spawn_is_blocked() {
    let mut grid = Grid::with_base(5, 5).unwrap();
    let spawn = grid.spawn_positions()[0];

    grid.set_tile(spawn, Tile::Obstacle).unwrap();

    assert_eq!(
        grid.validate_start(),
        Err(ValidationError::InvalidSpawn(spawn))
    );
}
