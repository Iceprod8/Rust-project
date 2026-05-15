use std::thread;

use rust_project::domain::{Position, ResourceNode, ResourceType};
use rust_project::knowledge::SharedKnowledge;

#[test]
fn discoveries_are_visible_from_shared_clones() {
    let knowledge = SharedKnowledge::new();
    let robot_view = knowledge.clone();
    let base_view = knowledge.clone();

    robot_view.record_obstacle(Position::new(3, 4));
    robot_view.record_resource(ResourceNode::new(
        Position::new(8, 2),
        ResourceType::Energy,
        90,
    ));

    assert!(base_view.is_obstacle_known(Position::new(3, 4)));
    assert_eq!(
        base_view
            .resource_at(Position::new(8, 2))
            .unwrap()
            .remaining,
        90
    );
}

#[test]
fn depleted_resources_are_not_valid_targets() {
    let knowledge = SharedKnowledge::new();
    let pos = Position::new(5, 6);

    knowledge.record_resource(ResourceNode::new(pos, ResourceType::Crystal, 70));
    knowledge.mark_resource_depleted(pos);

    assert!(knowledge.is_resource_depleted(pos));
    assert!(knowledge.resource_at(pos).is_none());
    assert!(knowledge.valid_resource_targets().is_empty());
    assert!(!knowledge.record_resource(ResourceNode::new(pos, ResourceType::Crystal, 120)));
}

#[test]
fn concurrent_reads_share_the_same_state() {
    let knowledge = SharedKnowledge::new();

    knowledge.record_resource(ResourceNode::new(
        Position::new(1, 1),
        ResourceType::Energy,
        100,
    ));

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let knowledge = knowledge.clone();

            thread::spawn(move || knowledge.valid_resource_targets().len())
        })
        .collect();

    for handle in handles {
        assert_eq!(handle.join().unwrap(), 1);
    }
}
