use rust_project::domain::{Position, RobotId, RobotKind, RobotState};
use rust_project::knowledge::SharedKnowledge;
use rust_project::map::Grid;
use rust_project::robots::Scout;

#[test]
fn scout_moves_and_remembers_visited_tiles() {
    let grid = Grid::new(4, 4);
    let knowledge = SharedKnowledge::new();
    let mut scout = Scout::new(RobotId(1), Position::new(1, 1));

    let moved = scout.advance(&grid, &knowledge);

    assert_eq!(moved, true);
    assert_ne!(scout.position(), Position::new(1, 1));
    assert_eq!(scout.has_visited(Position::new(1, 1)), true);
    assert_eq!(scout.has_visited(scout.position()), true);
    assert_eq!(scout.visited_positions().len(), 2);
}

#[test]
fn scout_avoids_obstacles_known_by_the_shared_store() {
    let grid = Grid::new(5, 5);
    let knowledge = SharedKnowledge::new();
    let blocked = Position::new(2, 1);
    let mut scout = Scout::new(RobotId(2), Position::new(1, 1));

    knowledge.record_obstacle(blocked);

    let moved = scout.advance(&grid, &knowledge);

    assert_eq!(moved, true);
    assert_ne!(scout.position(), blocked);
}

#[test]
fn scout_snapshot_never_carries_resource() {
    let scout = Scout::new(RobotId(5), Position::new(3, 3));
    let snapshot = scout.snapshot();

    assert_eq!(snapshot.id, RobotId(5));
    assert_eq!(snapshot.kind, RobotKind::Scout);
    assert_eq!(snapshot.state, RobotState::Exploring);
    assert_eq!(snapshot.carrying, None);
}
