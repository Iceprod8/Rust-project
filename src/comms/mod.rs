use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::domain::{Position, ResourceNode, ResourceType, RobotId, RobotState};

/// Identifie l'emetteur d'un message sans exposer les details internes du module appelant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorId {
    Simulation,
    Base,
    Robot(RobotId),
}

/// Indique a qui le message doit etre livre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Recipient {
    Simulation,
    Base,
    Robot(RobotId),
    Broadcast,
}

/// Transport commun pour tous les messages echanges entre robots, base et moteur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub sender: ActorId,
    pub recipient: Recipient,
    pub message: Message,
}

impl Envelope {
    pub fn new(sender: ActorId, recipient: Recipient, message: Message) -> Self {
        Self {
            sender,
            recipient,
            message,
        }
    }

    /// Helper pratique pour les informations qui doivent etre vues par plusieurs acteurs.
    pub fn broadcast(sender: ActorId, message: Message) -> Self {
        Self::new(sender, Recipient::Broadcast, message)
    }
}

/// Contrat principal de communication. Chaque famille couvre un besoin metier clair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Discovery(DiscoveryMessage),
    Collection(CollectionMessage),
    Base(BaseMessage),
    Status(StatusMessage),
    Shutdown(ShutdownMessage),
}

/// Messages emis quand un robot decouvre quelque chose sur la carte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMessage {
    ResourceFound {
        robot_id: RobotId,
        resource: ResourceNode,
    },
    ObstacleFound {
        robot_id: RobotId,
        position: Position,
    },
}

/// Messages emis pendant la recolte d'une ressource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionMessage {
    ResourceCollected {
        robot_id: RobotId,
        resource_type: ResourceType,
        position: Position,
        amount: u16,
    },
    ResourceDepleted {
        robot_id: RobotId,
        position: Position,
    },
}

/// Messages echanges avec la base pour les depots et la mise a jour du stock global.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseMessage {
    DepositRequested {
        robot_id: RobotId,
        resource_type: ResourceType,
        amount: u16,
    },
    DepositConfirmed {
        robot_id: RobotId,
        resource_type: ResourceType,
        amount: u16,
        total_energy: u32,
        total_crystals: u32,
    },
}

/// Messages d'etat pour suivre l'avancement de la simulation sans coupler les modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusMessage {
    RobotStateUpdated {
        robot_id: RobotId,
        state: RobotState,
        position: Position,
        carrying: Option<ResourceType>,
    },
    BaseStateUpdated {
        total_energy: u32,
        total_crystals: u32,
    },
    TickAdvanced {
        tick: u64,
    },
}

/// Messages de fin de simulation pour un arret coordonne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShutdownMessage {
    Requested,
    Acknowledged { actor: ActorId },
}

/// Alias exposes pour que tous les modules utilisent le meme type de canal.
pub type CommSender = Sender<Envelope>;
pub type CommReceiver = Receiver<Envelope>;

/// Construit un canal standard qui suffit pour le bootstrap du projet.
pub fn channel() -> (CommSender, CommReceiver) {
    mpsc::channel()
}

/// Ce point d'entree suffit pour raccorder le module au reste du crate.
pub fn register() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_build_a_broadcast_message() {
        let resource = ResourceNode::new(Position::new(4, 7), ResourceType::Energy, 120);
        let envelope = Envelope::broadcast(
            ActorId::Robot(RobotId(1)),
            Message::Discovery(DiscoveryMessage::ResourceFound {
                robot_id: RobotId(1),
                resource,
            }),
        );

        assert_eq!(envelope.recipient, Recipient::Broadcast);
    }

    #[test]
    fn channel_transports_the_shared_contract() {
        let (sender, receiver) = channel();
        let message = Envelope::new(
            ActorId::Simulation,
            Recipient::Base,
            Message::Shutdown(ShutdownMessage::Requested),
        );

        sender.send(message.clone()).unwrap();

        assert_eq!(receiver.recv().unwrap(), message);
    }
}
