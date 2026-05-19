use crate::domain::{ResourceType, RobotId};

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
}

pub fn register() {}
