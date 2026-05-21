use rust_project::domain::{
    Position, ResourceNode, ResourceType, RobotId, RobotKind, RobotSnapshot, RobotState,
    WorldSnapshot,
};
use rust_project::ui::{display_lines, map_lines, render_world};

use ratatui::{Terminal, backend::TestBackend};

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

#[test]
fn display_lines_show_stats_and_map() {
    let snapshot = WorldSnapshot {
        tick: 12,
        base_position: Position::new(0, 0),
        robots: Vec::new(),
        resources: Vec::new(),
        obstacles: Vec::new(),
        collected_energy: 3,
        collected_crystals: 4,
        events: Vec::new(),
    };

    let lines = display_lines(&snapshot);

    assert_eq!(lines[0], "Tick: 12");
    assert_eq!(lines[1], "Energie: 3 | Cristaux: 4");
    assert_eq!(lines[3], "B");
}

#[test]
fn ratatui_backend_can_render_snapshot() {
    let snapshot = WorldSnapshot {
        tick: 2,
        base_position: Position::new(1, 1),
        robots: Vec::new(),
        resources: vec![ResourceNode::new(
            Position::new(0, 0),
            ResourceType::Energy,
            8,
        )],
        obstacles: Vec::new(),
        collected_energy: 1,
        collected_crystals: 0,
        events: Vec::new(),
    };
    let backend = TestBackend::new(20, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    render_world(&mut terminal, &snapshot).unwrap();

    let buffer = terminal.backend().buffer();

    assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), "T");
    assert_eq!(buffer.cell((0, 3)).unwrap().symbol(), "E");
    assert_eq!(buffer.cell((1, 4)).unwrap().symbol(), "B");
}
