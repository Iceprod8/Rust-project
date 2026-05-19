use crate::comms::{ActorId, DiscoveryMessage, Envelope, Message};
use crate::domain::{Position, ResourceNode, RobotId, RobotKind, RobotSnapshot, RobotState, Tile};
use crate::knowledge::SharedKnowledge;
use crate::map::Grid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scout {
    id: RobotId,
    position: Position,
    visited: Vec<Position>,
    seen: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutReport {
    pub moved: bool,
    pub discoveries: Vec<Envelope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collector {
    id: RobotId,
    position: Position,
    path: Vec<Position>,
}

impl Scout {
    pub fn new(id: RobotId, position: Position) -> Self {
        Self {
            id,
            position,
            visited: vec![position],
            seen: Vec::new(),
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

    pub fn tick(&mut self, grid: &Grid, knowledge: &SharedKnowledge) -> ScoutReport {
        let discoveries = self.scan(grid, knowledge);
        let moved = self.advance(grid, knowledge);

        ScoutReport { moved, discoveries }
    }

    pub fn scan(&mut self, grid: &Grid, knowledge: &SharedKnowledge) -> Vec<Envelope> {
        let mut messages = Vec::new();
        let visible = self.visible_positions(grid);

        for pos in visible {
            if self.seen.contains(&pos) {
                continue;
            }

            self.seen.push(pos);

            let tile = grid.get_tile(pos);

            if tile == Some(Tile::Obstacle) {
                self.scan_obstacle(pos, knowledge, &mut messages);
            }

            if let Some(Tile::Resource(resource_type)) = tile {
                self.scan_resource(pos, resource_type, grid, knowledge, &mut messages);
            }
        }

        messages
    }

    fn scan_obstacle(
        &self,
        pos: Position,
        knowledge: &SharedKnowledge,
        messages: &mut Vec<Envelope>,
    ) {
        if knowledge.is_obstacle_known(pos) {
            return;
        }

        knowledge.record_obstacle(pos);

        let message = Message::Discovery(DiscoveryMessage::ObstacleFound {
            robot_id: self.id,
            position: pos,
        });

        messages.push(Envelope::broadcast(ActorId::Robot(self.id), message));
    }

    fn scan_resource(
        &self,
        pos: Position,
        resource_type: crate::domain::ResourceType,
        grid: &Grid,
        knowledge: &SharedKnowledge,
        messages: &mut Vec<Envelope>,
    ) {
        if knowledge.resource_at(pos).is_some() {
            return;
        }

        let mut found = ResourceNode::new(pos, resource_type, 1);

        for resource in grid.resources() {
            if resource.position == pos {
                found = resource.clone();
                break;
            }
        }

        if !knowledge.record_resource(found.clone()) {
            return;
        }

        let message = Message::Discovery(DiscoveryMessage::ResourceFound {
            robot_id: self.id,
            resource: found,
        });

        messages.push(Envelope::broadcast(ActorId::Robot(self.id), message));
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

    fn visible_positions(&self, grid: &Grid) -> Vec<Position> {
        let mut positions = Vec::new();

        if grid.in_bounds(self.position) {
            positions.push(self.position);
        }

        for pos in self.neighbors() {
            if grid.in_bounds(pos) {
                positions.push(pos);
            }
        }

        positions
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

impl Collector {
    pub fn new(id: RobotId, position: Position) -> Self {
        Self {
            id,
            position,
            path: Vec::new(),
        }
    }

    pub fn id(&self) -> RobotId {
        self.id
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn path(&self) -> &[Position] {
        &self.path
    }

    pub fn snapshot(&self) -> RobotSnapshot {
        let state = if self.path.is_empty() {
            RobotState::Idle
        } else {
            RobotState::MovingTo(self.path[self.path.len() - 1])
        };

        RobotSnapshot {
            id: self.id,
            kind: RobotKind::Collector,
            position: self.position,
            state,
            carrying: None,
        }
    }

    pub fn plan_path(&mut self, grid: &Grid, target: Position) -> bool {
        let path = find_path(grid, self.position, target);

        if path.is_empty() && self.position != target {
            self.path.clear();
            return false;
        }

        self.path = path;
        true
    }

    pub fn move_one_step(&mut self, grid: &Grid) -> bool {
        if self.path.is_empty() {
            return false;
        }

        let next = self.path[0];

        if !grid.is_walkable(next) {
            self.path.clear();
            return false;
        }

        self.position = next;
        self.path.remove(0);
        true
    }
}

pub fn find_path(grid: &Grid, start: Position, target: Position) -> Vec<Position> {
    let mut queue = Vec::new();
    let mut visited = Vec::new();
    let mut parents = Vec::new();

    if !grid.in_bounds(start) || !grid.in_bounds(target) {
        return Vec::new();
    }

    queue.push(start);
    visited.push(start);

    while !queue.is_empty() {
        let current = queue.remove(0);

        if current == target {
            return build_path(start, target, parents);
        }

        for next in neighbors(current) {
            if !grid.in_bounds(next) {
                continue;
            }

            if !grid.is_walkable(next) {
                continue;
            }

            if visited.contains(&next) {
                continue;
            }

            visited.push(next);
            parents.push((next, current));
            queue.push(next);
        }
    }

    Vec::new()
}

fn build_path(
    start: Position,
    target: Position,
    parents: Vec<(Position, Position)>,
) -> Vec<Position> {
    let mut path = Vec::new();
    let mut current = target;

    while current != start {
        path.push(current);

        let mut found_parent = false;

        for (child, parent) in &parents {
            if *child == current {
                current = *parent;
                found_parent = true;
                break;
            }
        }

        if !found_parent {
            return Vec::new();
        }
    }

    path.reverse();
    path
}

fn neighbors(pos: Position) -> Vec<Position> {
    vec![
        Position::new(pos.x + 1, pos.y),
        Position::new(pos.x, pos.y + 1),
        Position::new(pos.x - 1, pos.y),
        Position::new(pos.x, pos.y - 1),
    ]
}

pub fn register() {}
