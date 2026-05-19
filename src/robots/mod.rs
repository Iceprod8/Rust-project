use crate::domain::{Position, RobotId, RobotKind, RobotSnapshot, RobotState};
use crate::knowledge::SharedKnowledge;
use crate::map::Grid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scout {
    id: RobotId,
    position: Position,
    visited: Vec<Position>,
}

impl Scout {
    pub fn new(id: RobotId, position: Position) -> Self {
        Self {
            id,
            position,
            visited: vec![position],
        }
    }

    pub fn id(&self) -> RobotId {
        self.id
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn visited_positions(&self) -> &[Position] {
        &self.visited
    }

    pub fn has_visited(&self, pos: Position) -> bool {
        self.visited.contains(&pos)
    }

    pub fn snapshot(&self) -> RobotSnapshot {
        RobotSnapshot {
            id: self.id,
            kind: RobotKind::Scout,
            position: self.position,
            state: RobotState::Exploring,
            carrying: None,
        }
    }

    pub fn advance(&mut self, grid: &Grid, knowledge: &SharedKnowledge) -> bool {
        let next = self.next_position(grid, knowledge);

        if next.is_none() {
            return false;
        }

        let pos = next.unwrap();
        self.position = pos;

        if !self.visited.contains(&pos) {
            self.visited.push(pos);
        }

        true
    }

    fn next_position(&self, grid: &Grid, knowledge: &SharedKnowledge) -> Option<Position> {
        let neighbors = self.neighbors();
        let mut first_valid = None;

        for pos in neighbors {
            if !self.can_enter(pos, grid, knowledge) {
                continue;
            }

            if first_valid.is_none() {
                first_valid = Some(pos);
            }

            if !self.visited.contains(&pos) {
                return Some(pos);
            }
        }

        first_valid
    }

    fn can_enter(&self, pos: Position, grid: &Grid, knowledge: &SharedKnowledge) -> bool {
        if !grid.in_bounds(pos) {
            return false;
        }

        if !grid.is_walkable(pos) {
            return false;
        }

        !knowledge.is_obstacle_known(pos)
    }

    fn neighbors(&self) -> Vec<Position> {
        vec![
            Position::new(self.position.x + 1, self.position.y),
            Position::new(self.position.x, self.position.y + 1),
            Position::new(self.position.x - 1, self.position.y),
            Position::new(self.position.x, self.position.y - 1),
        ]
    }
}

pub fn register() {}
