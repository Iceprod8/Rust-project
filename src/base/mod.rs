use crate::comms::BaseMessage;
use crate::domain::{Event, ResourceType, RobotId, WorldSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BaseStats {
    pub energy: u32,
    pub crystals: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositReceipt {
    pub robot_id: RobotId,
    pub resource_type: ResourceType,
    pub amount: u16,
    pub total_energy: u32,
    pub total_crystals: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositError {
    EmptyAmount,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BaseStorage {
    energy: u32,
    crystals: u32,
}

impl BaseStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn energy(&self) -> u32 {
        self.energy
    }

    pub const fn crystals(&self) -> u32 {
        self.crystals
    }

    pub const fn stats(&self) -> BaseStats {
        BaseStats {
            energy: self.energy,
            crystals: self.crystals,
        }
    }

    pub fn deposit(
        &mut self,
        robot_id: RobotId,
        resource_type: ResourceType,
        amount: u16,
    ) -> Result<DepositReceipt, DepositError> {
        if amount == 0 {
            return Err(DepositError::EmptyAmount);
        }

        match resource_type {
            ResourceType::Energy => self.energy += u32::from(amount),
            ResourceType::Crystal => self.crystals += u32::from(amount),
        }

        Ok(DepositReceipt {
            robot_id,
            resource_type,
            amount,
            total_energy: self.energy,
            total_crystals: self.crystals,
        })
    }

    pub fn apply_to_snapshot(&self, snapshot: &mut WorldSnapshot) {
        snapshot.collected_energy = self.energy;
        snapshot.collected_crystals = self.crystals;
    }

    pub fn handle_deposit(&mut self, message: BaseMessage) -> Result<BaseMessage, DepositError> {
        match message {
            BaseMessage::DepositRequested {
                robot_id,
                resource_type,
                amount,
            } => {
                let receipt = self.deposit(robot_id, resource_type, amount)?;

                Ok(BaseMessage::DepositConfirmed {
                    robot_id: receipt.robot_id,
                    resource_type: receipt.resource_type,
                    amount: receipt.amount,
                    total_energy: receipt.total_energy,
                    total_crystals: receipt.total_crystals,
                })
            }
            BaseMessage::DepositConfirmed { .. } => Ok(message),
        }
    }

    pub fn event_from_receipt(receipt: DepositReceipt) -> Event {
        Event::ResourceDeposited {
            robot_id: receipt.robot_id,
            resource_type: receipt.resource_type,
            amount: receipt.amount,
        }
    }
}

pub fn register() {}
