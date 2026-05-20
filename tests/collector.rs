use rust_project::domain::{Position, ResourceNode, ResourceType, RobotId, RobotState, Tile};
use rust_project::knowledge::SharedKnowledge;
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

#[test]
fn collector_plans_path_to_known_resource() {
    let mut grid = Grid::new(6, 4);
    let knowledge = SharedKnowledge::new();
    let target = Position::new(5, 2);
    let mut collector = Collector::new(RobotId(12), Position::new(0, 0));

    grid.set_tile(target, Tile::Resource(ResourceType::Crystal))
        .unwrap();
    knowledge.record_resource(ResourceNode::new(target, ResourceType::Crystal, 80));

    let selected = collector.plan_to_resource(&grid, &knowledge);

    assert_eq!(selected, Some(target));
    assert_eq!(collector.target(), Some(target));
    assert_eq!(collector.state(), RobotState::MovingTo(target));
    assert_eq!(collector.path().last(), Some(&target));
}

#[test]
fn collector_ignores_known_resource_missing_from_grid() {
    let grid = Grid::new(6, 4);
    let knowledge = SharedKnowledge::new();
    let target = Position::new(5, 2);
    let mut collector = Collector::new(RobotId(15), Position::new(0, 0));

    knowledge.record_resource(ResourceNode::new(target, ResourceType::Crystal, 80));

    let selected = collector.plan_to_resource(&grid, &knowledge);

    assert_eq!(selected, None);
    assert_eq!(collector.target(), None);
    assert_eq!(collector.state(), RobotState::Idle);
    assert_eq!(knowledge.is_resource_depleted(target), true);
}

#[test]
fn collector_can_plan_return_to_base() {
    let mut grid = Grid::new(7, 7);
    let mut collector = Collector::new(RobotId(13), Position::new(6, 6));

    let base = grid.place_base().unwrap();

    assert_eq!(collector.plan_to_base(&grid), true);

    while collector.position() != base {
        assert_eq!(collector.move_one_step(&grid), true);
    }

    assert_eq!(collector.position(), base);
}

#[test]
fn collector_clears_path_when_next_step_is_blocked() {
    let mut grid = Grid::new(4, 1);
    let mut collector = Collector::new(RobotId(14), Position::new(0, 0));

    assert_eq!(collector.plan_path(&grid, Position::new(3, 0)), true);

    grid.set_tile(Position::new(1, 0), Tile::Obstacle).unwrap();

    assert_eq!(collector.path_is_valid(&grid), false);
    assert_eq!(collector.move_one_step(&grid), false);
    assert_eq!(collector.path().is_empty(), true);
    assert_eq!(collector.position(), Position::new(0, 0));
}
