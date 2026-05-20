use std::time::Duration;

use crate::base::BaseStorage;
use crate::comms::{ActorId, Envelope, Message, Recipient, StatusMessage};
use crate::domain::{Event, Position, WorldSnapshot};
use crate::knowledge::SharedKnowledge;
use crate::map::Grid;
use crate::robots::{Collector, Scout};

pub struct AppSkeleton {
    pub module_count: usize,
    pub entrypoint: &'static str,
}

#[derive(Debug)]
pub struct Simulation {
    grid: Grid,
    knowledge: SharedKnowledge,
    base: BaseStorage,
    scouts: Vec<Scout>,
    collectors: Vec<Collector>,
    tick: u64,
    tick_duration: Duration,
    events: Vec<Event>,
    messages: Vec<Envelope>,
}

impl Simulation {
    pub fn new(grid: Grid) -> Self {
        Self {
            grid,
            knowledge: SharedKnowledge::new(),
            base: BaseStorage::new(),
            scouts: Vec::new(),
            collectors: Vec::new(),
            tick: 0,
            tick_duration: Duration::from_millis(200),
            events: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn tick_duration(&self) -> Duration {
        self.tick_duration
    }

    pub fn knowledge(&self) -> SharedKnowledge {
        self.knowledge.clone()
    }

    pub fn add_scout(&mut self, scout: Scout) {
        self.scouts.push(scout);
    }

    pub fn add_collector(&mut self, collector: Collector) {
        self.collectors.push(collector);
    }

    pub fn scouts(&self) -> &[Scout] {
        &self.scouts
    }

    pub fn collectors(&self) -> &[Collector] {
        &self.collectors
    }

    pub fn base(&self) -> &BaseStorage {
        &self.base
    }

    pub fn messages(&self) -> &[Envelope] {
        &self.messages
    }

    pub fn advance_tick(&mut self) -> WorldSnapshot {
        self.tick += 1;
        self.events.clear();
        self.messages.clear();

        self.events.push(Event::TickAdvanced { tick: self.tick });
        self.messages.push(Envelope::new(
            ActorId::Simulation,
            Recipient::Broadcast,
            Message::Status(StatusMessage::TickAdvanced { tick: self.tick }),
        ));

        self.snapshot()
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let mut snapshot = WorldSnapshot {
            tick: self.tick,
            base_position: self.grid.base_position().unwrap_or(Position::new(0, 0)),
            robots: Vec::new(),
            resources: self.grid.resources().to_vec(),
            obstacles: self.knowledge.known_obstacles(),
            collected_energy: 0,
            collected_crystals: 0,
            events: self.events.clone(),
        };

        for scout in &self.scouts {
            snapshot.robots.push(scout.snapshot());
        }

        for collector in &self.collectors {
            snapshot.robots.push(collector.snapshot());
        }

        self.base.apply_to_snapshot(&mut snapshot);
        snapshot
    }
}

pub fn bootstrap() -> AppSkeleton {
    crate::domain::register();
    crate::knowledge::register();
    crate::map::register();
    crate::robots::register();
    crate::base::register();
    crate::comms::register();
    crate::ui::register();

    AppSkeleton {
        module_count: 8,
        entrypoint: "src/main.rs",
    }
}
