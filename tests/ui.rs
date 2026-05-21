use rust_project::domain::{
    Position, ResourceNode, ResourceType, RobotId, RobotKind, RobotSnapshot, RobotState,
    WorldSnapshot,
};
use rust_project::ui::map_lines;

#[test]
fn map_lines_put_elements_at_their_position() {
    let snapshot = WorldSnapshot {
        tick: 0,
        base_position: Position::new(1, 1),
        robots: vec![RobotSnapshot {
            id: RobotId(1),
            kind: RobotKind::Scout,
            position: Position::new(3, 0),
            state: RobotState::Exploring,
            carrying: None,
        }],
        resources: vec![
            ResourceNode::new(Position::new(2, 1), ResourceType::Energy, 10),
            ResourceNode::new(Position::new(0, 2), ResourceType::Crystal, 5),
        ],
        obstacles: vec![Position::new(0, 0)],
        collected_energy: 0,
        collected_crystals: 0,
        events: Vec::new(),
    };

    let lines = map_lines(&snapshot);

    assert_eq!(lines[0].chars().nth(0), Some('#'));
    assert_eq!(lines[0].chars().nth(3), Some('S'));
    assert_eq!(lines[1].chars().nth(1), Some('B'));
    assert_eq!(lines[1].chars().nth(2), Some('E'));
    assert_eq!(lines[2].chars().nth(0), Some('C'));
}

#[test]
fn robot_is_drawn_above_the_base() {
    let snapshot = WorldSnapshot {
        tick: 0,
        base_position: Position::new(2, 2),
        robots: vec![RobotSnapshot {
            id: RobotId(2),
            kind: RobotKind::Collector,
            position: Position::new(2, 2),
            state: RobotState::Idle,
            carrying: None,
        }],
        resources: Vec::new(),
        obstacles: Vec::new(),
        collected_energy: 0,
        collected_crystals: 0,
        events: Vec::new(),
    };

    let lines = map_lines(&snapshot);

    assert_eq!(lines[2].chars().nth(2), Some('R'));
}
