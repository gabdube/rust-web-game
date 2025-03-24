pub mod warrior_move;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::DemoGameData;

#[derive(Copy, Clone)]
pub enum WarriorBehaviourType {
    Idle,
    MoveTo { target_position: Position<f32> },
}

#[derive(Copy, Clone)]
pub struct WarriorBehaviour {
    pub ty: WarriorBehaviourType,
    pub state: BehaviourState,
}

impl WarriorBehaviour {
    pub fn idle() -> Self {
        WarriorBehaviour {
            ty: WarriorBehaviourType::Idle,
            state: BehaviourState::Initial
        }
    }
}

pub fn idle(game: &mut DemoGameData, warrior_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.warriors_behaviour[warrior_index];
    if let BehaviourState::Initial = behaviour.state {
        let warrior = &mut world.warriors[warrior_index];
        warrior.animation = game.assets.animations.warrior.idle;
        behaviour.state = BehaviourState::Running(0);
    }
}
