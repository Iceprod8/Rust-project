/// Position represente une case de la carte dans l'espace de la simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// ResourceType identifie les deux ressources du sujet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Energy,
    Crystal,
}

/// Tile decrit le contenu logique d'une case de la carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    Obstacle,
    Base,
    Resource(ResourceType),
}

/// ResourceNode represente un gisement pose sur la carte avec sa quantite restante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceNode {
    pub position: Position,
    pub resource_type: ResourceType,
    pub remaining: u16,
}

impl ResourceNode {
    pub const fn new(position: Position, resource_type: ResourceType, remaining: u16) -> Self {
        Self {
            position,
            resource_type,
            remaining,
        }
    }
}

/// RobotId fournit un identifiant stable sans exposer un simple entier brut partout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RobotId(pub u32);

/// RobotKind distingue les scouts des collecteurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RobotKind {
    Scout,
    Collector,
}

/// RobotState capture les grands etats utiles a la simulation et a l'interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobotState {
    Idle,
    Exploring,
    MovingTo(Position),
    Collecting(ResourceType),
    ReturningToBase,
    Unloading,
}

/// RobotSnapshot expose l'etat visible d'un robot sans embarquer sa logique interne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotSnapshot {
    pub id: RobotId,
    pub kind: RobotKind,
    pub position: Position,
    pub state: RobotState,
    pub carrying: Option<ResourceType>,
}

/// Event decrit les messages importants que les modules pourront partager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    ResourceDiscovered {
        robot_id: RobotId,
        resource: ResourceNode,
    },
    ObstacleDiscovered {
        robot_id: RobotId,
        position: Position,
    },
    ResourceCollected {
        robot_id: RobotId,
        resource_type: ResourceType,
        position: Position,
        amount: u16,
    },
    ResourceDeposited {
        robot_id: RobotId,
        resource_type: ResourceType,
        amount: u16,
    },
    TickAdvanced {
        tick: u64,
    },
    SimulationStopped,
}

/// WorldSnapshot rassemble uniquement les donnees partagees entre logique et rendu.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub base_position: Position,
    pub robots: Vec<RobotSnapshot>,
    pub resources: Vec<ResourceNode>,
    pub obstacles: Vec<Position>,
    pub collected_energy: u32,
    pub collected_crystals: u32,
    pub events: Vec<Event>,
}

/// Ce point d'entree suffit pour raccorder le module au reste du crate.
pub fn register() {}
