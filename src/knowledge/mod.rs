use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::domain::{Position, ResourceNode};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeSnapshot {
    pub obstacles: Vec<Position>,
    pub resources: Vec<ResourceNode>,
    pub depleted_resources: Vec<Position>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeState {
    obstacles: HashSet<Position>,
    resources: HashMap<Position, ResourceNode>,
    depleted_resources: HashSet<Position>,
}

#[derive(Debug, Clone, Default)]
pub struct SharedKnowledge {
    inner: Arc<RwLock<KnowledgeState>>,
}

impl SharedKnowledge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_obstacle(&self, pos: Position) {
        self.write().obstacles.insert(pos);
    }

    pub fn record_resource(&self, resource: ResourceNode) -> bool {
        let mut state = self.write();

        if resource.remaining == 0 {
            state.resources.remove(&resource.position);
            state.depleted_resources.insert(resource.position);
            return false;
        }

        if state.depleted_resources.contains(&resource.position) {
            return false;
        }

        state.resources.insert(resource.position, resource);
        true
    }

    pub fn mark_resource_depleted(&self, pos: Position) -> Option<ResourceNode> {
        let mut state = self.write();
        let removed = state.resources.remove(&pos);

        state.depleted_resources.insert(pos);
        removed
    }

    pub fn known_obstacles(&self) -> Vec<Position> {
        self.read().obstacles.iter().copied().collect()
    }

    pub fn known_resources(&self) -> Vec<ResourceNode> {
        self.read().resources.values().cloned().collect()
    }

    pub fn valid_resource_targets(&self) -> Vec<ResourceNode> {
        self.read()
            .resources
            .values()
            .filter(|resource| resource.remaining > 0)
            .cloned()
            .collect()
    }

    pub fn resource_at(&self, pos: Position) -> Option<ResourceNode> {
        self.read().resources.get(&pos).cloned()
    }

    pub fn is_obstacle_known(&self, pos: Position) -> bool {
        self.read().obstacles.contains(&pos)
    }

    pub fn is_resource_depleted(&self, pos: Position) -> bool {
        self.read().depleted_resources.contains(&pos)
    }

    pub fn snapshot(&self) -> KnowledgeSnapshot {
        let state = self.read();

        KnowledgeSnapshot {
            obstacles: state.obstacles.iter().copied().collect(),
            resources: state.resources.values().cloned().collect(),
            depleted_resources: state.depleted_resources.iter().copied().collect(),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, KnowledgeState> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, KnowledgeState> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub fn register() {}
