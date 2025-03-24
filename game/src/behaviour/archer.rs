pub mod archer_move;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::DemoGameData;

#[derive(Copy, Clone)]
pub enum ArcherBehaviourType {
    Idle,
    MoveTo { target_position: Position<f32> }
}


#[derive(Copy, Clone)]
pub struct ArcherBehaviour {
    pub ty: ArcherBehaviourType,
    pub state: BehaviourState,
}

impl ArcherBehaviour {

    pub fn idle() -> Self {
        ArcherBehaviour {
            ty: ArcherBehaviourType::Idle,
            state: BehaviourState::Initial
        }
    }

}

pub fn idle(game: &mut DemoGameData, archer_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.archers_behaviour[archer_index];
    if let BehaviourState::Initial = behaviour.state {
        let archer = &mut world.archers[archer_index];
        archer.animation = game.assets.animations.archer.idle;
        behaviour.state = BehaviourState::Running(0);
    }
}
