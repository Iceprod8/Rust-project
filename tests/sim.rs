use std::time::Duration;

use rust_project::domain::{Event, Position, RobotId};
use rust_project::map::Grid;
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
    assert_eq!(sim.messages().len(), 1);
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
