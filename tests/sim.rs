use std::time::Duration;

use rust_project::comms::{DiscoveryMessage, Message};
use rust_project::domain::{Event, Position, ResourceNode, ResourceType, RobotId, Tile};
use rust_project::map::{Grid, ResourceParams};
use rust_project::robots::{Collector, Scout};
use rust_project::sim::Simulation;

#[test]
fn simulation_uses_a_fixed_tick() {
    let grid = Grid::new(5, 5);
    let mut sim = Simulation::new(grid);

    assert_eq!(sim.tick(), 0);
    assert_eq!(sim.tick_duration(), Duration::from_millis(200));

    let snapshot = sim.advance_tick();

    assert_eq!(sim.tick(), 1);
    assert_eq!(snapshot.tick, 1);
    assert_eq!(snapshot.events, vec![Event::TickAdvanced { tick: 1 }]);
    assert_eq!(sim.messages().len(), 2);
}

#[test]
fn snapshot_contains_robots_and_base_state() {
    let mut grid = Grid::new(7, 7);
    let base = grid.place_base().unwrap();
    let mut sim = Simulation::new(grid);

    sim.add_scout(Scout::new(RobotId(1), Position::new(3, 2)));
    sim.add_collector(Collector::new(RobotId(2), Position::new(3, 4)));

    let snapshot = sim.snapshot();

    assert_eq!(snapshot.tick, 0);
    assert_eq!(snapshot.base_position, base);
    assert_eq!(snapshot.robots.len(), 2);
    assert_eq!(snapshot.collected_energy, 0);
    assert_eq!(snapshot.collected_crystals, 0);
}

#[test]
fn simulation_records_scout_discoveries() {
    let mut grid = Grid::new(5, 5);
    let resource_pos = Position::new(2, 1);

    grid.set_tile(resource_pos, Tile::Resource(ResourceType::Energy))
        .unwrap();

    let mut sim = Simulation::new(grid);

    sim.add_scout(Scout::new(RobotId(3), Position::new(1, 1)));

    let snapshot = sim.advance_tick();
    let mut found_event = false;
    let mut found_message = false;

    for event in snapshot.events {
        if let Event::ResourceDiscovered { robot_id, resource } = event {
            if robot_id == RobotId(3) && resource.position == resource_pos {
                found_event = true;
            }
        }
    }

    for envelope in sim.messages() {
        if let Message::Discovery(DiscoveryMessage::ResourceFound { robot_id, resource }) =
            &envelope.message
        {
            if *robot_id == RobotId(3) && resource.position == resource_pos {
                found_message = true;
            }
        }
    }

    assert_eq!(found_event, true);
    assert_eq!(found_message, true);
}

#[test]
fn simulation_updates_collectors_and_base() {
    let mut grid = Grid::with_base(8, 8).unwrap();

    grid.place_resources(ResourceParams::new(42, 1, 0));

    let resource = grid.resources()[0].clone();
    let base_pos = grid.base_position().unwrap();
    let mut sim = Simulation::new(grid);
    let knowledge = sim.knowledge();

    knowledge.record_resource(ResourceNode::new(
        resource.position,
        resource.resource_type,
        resource.remaining,
    ));
    sim.add_collector(Collector::new(RobotId(4), base_pos));

    let mut snapshot = sim.snapshot();

    for _ in 0..80 {
        snapshot = sim.advance_tick();

        if snapshot.collected_energy == 1 {
            break;
        }
    }

    let mut found_deposit = false;

    for event in &snapshot.events {
        if let Event::ResourceDeposited {
            robot_id,
            resource_type,
            amount,
        } = event
        {
            if *robot_id == RobotId(4) && *resource_type == ResourceType::Energy && *amount == 1 {
                found_deposit = true;
            }
        }
    }

    assert_eq!(snapshot.collected_energy, 1);
    assert_eq!(snapshot.robots.len(), 1);
    assert_eq!(found_deposit, true);
}
