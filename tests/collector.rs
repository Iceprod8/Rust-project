use rust_project::domain::{Position, RobotId, Tile};
use rust_project::map::Grid;
use rust_project::robots::{Collector, find_path};

#[test]
fn collector_calculates_path_to_accessible_target() {
    let mut grid = Grid::new(5, 5);
    let target = Position::new(4, 0);
    let mut collector = Collector::new(RobotId(10), Position::new(0, 0));

    grid.set_tile(Position::new(1, 0), Tile::Obstacle).unwrap();
    grid.set_tile(Position::new(1, 1), Tile::Obstacle).unwrap();

    let found = collector.plan_path(&grid, target);

    assert_eq!(found, true);
    assert_eq!(collector.path().last(), Some(&target));
    assert_eq!(collector.path().contains(&Position::new(1, 0)), false);
    assert_eq!(collector.path().contains(&Position::new(1, 1)), false);
}

#[test]
fn collector_can_follow_the_path_until_target() {
    let grid = Grid::new(4, 4);
    let target = Position::new(3, 0);
    let mut collector = Collector::new(RobotId(11), Position::new(0, 0));

    assert_eq!(collector.plan_path(&grid, target), true);

    while collector.position() != target {
        assert_eq!(collector.move_one_step(&grid), true);
    }

    assert_eq!(collector.position(), target);
    assert_eq!(collector.path().is_empty(), true);
}

#[test]
fn path_is_empty_when_target_is_blocked() {
    let mut grid = Grid::new(3, 3);

    grid.set_tile(Position::new(1, 0), Tile::Obstacle).unwrap();
    grid.set_tile(Position::new(0, 1), Tile::Obstacle).unwrap();

    let path = find_path(&grid, Position::new(0, 0), Position::new(2, 2));

    assert_eq!(path.is_empty(), true);
}
