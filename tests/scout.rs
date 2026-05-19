use rust_project::comms::{DiscoveryMessage, Message};
use rust_project::domain::{Position, ResourceType, RobotId, RobotKind, RobotState, Tile};
use rust_project::knowledge::SharedKnowledge;
use rust_project::map::{Grid, ResourceParams};
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
fn scout_detects_obstacles_and_publishes_discovery() {
    let mut grid = Grid::new(5, 5);
    let knowledge = SharedKnowledge::new();
    let obstacle = Position::new(2, 1);
    let mut scout = Scout::new(RobotId(3), Position::new(1, 1));

    grid.set_tile(obstacle, Tile::Obstacle).unwrap();

    let messages = scout.scan(&grid, &knowledge);
    let mut found_message = false;

    for envelope in messages {
        if let Message::Discovery(DiscoveryMessage::ObstacleFound { robot_id, position }) =
            envelope.message
        {
            if robot_id == RobotId(3) && position == obstacle {
                found_message = true;
            }
        }
    }

    assert_eq!(knowledge.is_obstacle_known(obstacle), true);
    assert_eq!(found_message, true);
}

#[test]
fn scout_detects_resources_without_collecting_them() {
    let mut grid = Grid::new(5, 5);
    let knowledge = SharedKnowledge::new();

    grid.place_resources(ResourceParams::new(8, 1, 0));

    let resource = grid.resources()[0].clone();
    let start = Position::new(resource.position.x - 1, resource.position.y);
    let mut scout = Scout::new(RobotId(4), start);

    let messages = scout.scan(&grid, &knowledge);
    let mut found_message = false;

    for envelope in messages {
        if let Message::Discovery(DiscoveryMessage::ResourceFound {
            robot_id,
            resource: found,
        }) = envelope.message
        {
            if robot_id == RobotId(4) && found.position == resource.position {
                found_message = true;
            }
        }
    }

    assert_eq!(
        grid.get_tile(resource.position),
        Some(Tile::Resource(ResourceType::Energy))
    );
    assert_eq!(grid.resources()[0].remaining, resource.remaining);
    assert_eq!(knowledge.resource_at(resource.position).is_some(), true);
    assert_eq!(found_message, true);
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
