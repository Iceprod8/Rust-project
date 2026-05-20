use std::time::Duration;

use crate::base::BaseStorage;
use crate::comms::{
    ActorId, BaseMessage, CollectionMessage, DiscoveryMessage, Envelope, Message, Recipient,
    StatusMessage,
};
use crate::domain::{Event, Position, ResourceType, RobotId, RobotSnapshot, WorldSnapshot};
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

        self.record_tick();
        self.update_scouts();
        self.update_collectors();
        self.record_base_status();

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

    fn record_tick(&mut self) {
        self.events.push(Event::TickAdvanced { tick: self.tick });
        self.messages.push(Envelope::new(
            ActorId::Simulation,
            Recipient::Broadcast,
            Message::Status(StatusMessage::TickAdvanced { tick: self.tick }),
        ));
    }

    fn update_scouts(&mut self) {
        for index in 0..self.scouts.len() {
            let report = {
                let scout = &mut self.scouts[index];
                scout.tick(&self.grid, &self.knowledge)
            };

            for message in report.discoveries {
                self.record_discovery(message);
            }

            let snapshot = self.scouts[index].snapshot();
            self.record_robot_status(snapshot);
        }
    }

    fn update_collectors(&mut self) {
        for index in 0..self.collectors.len() {
            let before_carrying = self.collectors[index].carrying();
            let before_energy = self.base.energy();
            let before_crystals = self.base.crystals();

            {
                let collector = &mut self.collectors[index];
                collector.tick(&mut self.grid, &self.knowledge, &mut self.base);
            }

            let after_carrying = self.collectors[index].carrying();
            let position = self.collectors[index].position();
            let robot_id = self.collectors[index].id();

            self.record_collection(robot_id, position, before_carrying, after_carrying);
            self.record_deposit(
                robot_id,
                before_carrying,
                after_carrying,
                before_energy,
                before_crystals,
            );

            let snapshot = self.collectors[index].snapshot();
            self.record_robot_status(snapshot);
        }
    }

    fn record_discovery(&mut self, envelope: Envelope) {
        match &envelope.message {
            Message::Discovery(DiscoveryMessage::ResourceFound { robot_id, resource }) => {
                self.events.push(Event::ResourceDiscovered {
                    robot_id: *robot_id,
                    resource: resource.clone(),
                });
            }
            Message::Discovery(DiscoveryMessage::ObstacleFound { robot_id, position }) => {
                self.events.push(Event::ObstacleDiscovered {
                    robot_id: *robot_id,
                    position: *position,
                });
            }
            _ => {}
        }

        self.messages.push(envelope);
    }

    fn record_collection(
        &mut self,
        robot_id: RobotId,
        position: Position,
        before: Option<ResourceType>,
        after: Option<ResourceType>,
    ) {
        if before.is_some() || after.is_none() {
            return;
        }

        let resource_type = after.unwrap();

        self.events.push(Event::ResourceCollected {
            robot_id,
            resource_type,
            position,
            amount: 1,
        });

        self.messages.push(Envelope::new(
            ActorId::Robot(robot_id),
            Recipient::Broadcast,
            Message::Collection(CollectionMessage::ResourceCollected {
                robot_id,
                resource_type,
                position,
                amount: 1,
            }),
        ));
    }

    fn record_deposit(
        &mut self,
        robot_id: RobotId,
        before: Option<ResourceType>,
        after: Option<ResourceType>,
        before_energy: u32,
        before_crystals: u32,
    ) {
        if before.is_none() || after.is_some() {
            return;
        }

        let resource_type = before.unwrap();
        let amount = self.deposit_amount(resource_type, before_energy, before_crystals);

        if amount == 0 {
            return;
        }

        self.events.push(Event::ResourceDeposited {
            robot_id,
            resource_type,
            amount,
        });

        self.messages.push(Envelope::new(
            ActorId::Base,
            Recipient::Broadcast,
            Message::Base(BaseMessage::DepositConfirmed {
                robot_id,
                resource_type,
                amount,
                total_energy: self.base.energy(),
                total_crystals: self.base.crystals(),
            }),
        ));
    }

    fn deposit_amount(
        &self,
        resource_type: ResourceType,
        before_energy: u32,
        before_crystals: u32,
    ) -> u16 {
        match resource_type {
            ResourceType::Energy => {
                if self.base.energy() > before_energy {
                    (self.base.energy() - before_energy) as u16
                } else {
                    0
                }
            }
            ResourceType::Crystal => {
                if self.base.crystals() > before_crystals {
                    (self.base.crystals() - before_crystals) as u16
                } else {
                    0
                }
            }
        }
    }

    fn record_robot_status(&mut self, snapshot: RobotSnapshot) {
        self.messages.push(Envelope::new(
            ActorId::Robot(snapshot.id),
            Recipient::Broadcast,
            Message::Status(StatusMessage::RobotStateUpdated {
                robot_id: snapshot.id,
                state: snapshot.state,
                position: snapshot.position,
                carrying: snapshot.carrying,
            }),
        ));
    }

    fn record_base_status(&mut self) {
        self.messages.push(Envelope::new(
            ActorId::Base,
            Recipient::Broadcast,
            Message::Status(StatusMessage::BaseStateUpdated {
                total_energy: self.base.energy(),
                total_crystals: self.base.crystals(),
            }),
        ));
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
