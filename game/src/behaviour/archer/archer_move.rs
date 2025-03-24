use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{ArcherBehaviour, ArcherBehaviourType};

const MOVING: u8 = 0;

pub fn new(game: &mut DemoGameData, archer: WorldObject, target_position: Position<f32>) {
    let archer_index = archer.id as usize;

    if archer.ty != WorldObjectType::Archer || archer_index >= game.world.archers.len() {
        return;
    }

    game.world.archers_behaviour[archer_index] = ArcherBehaviour {
        ty: ArcherBehaviourType::MoveTo { target_position },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, archer_index: usize) {
    let state = game.world.archers_behaviour[archer_index].state;
    match state {
        BehaviourState::Initial => init(game, archer_index),
        BehaviourState::Running(MOVING) => moving(game, archer_index),
        _ => {},
    }
}

fn init(game: &mut DemoGameData, archer_index: usize) {
    let archer = &mut game.world.archers[archer_index];
    let behaviour = &mut game.world.archers_behaviour[archer_index];
    archer.animation = game.assets.animations.archer .walk;
    behaviour.state = BehaviourState::Running(MOVING);
}

fn moving(game: &mut DemoGameData, archer_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let behaviour = &mut game.world.archers_behaviour[archer_index];
    let archer = &mut game.world.archers[archer_index];
    let target_position = params(behaviour.ty);
   
    let current_position = archer.position;
    let updated_position = move_to(current_position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        *behaviour = ArcherBehaviour::idle();
    } else {
        archer.flipped = current_position.x > target_position.x;
    }

    archer.position = updated_position;
}

#[inline(always)]
fn params(value: ArcherBehaviourType) -> Position<f32> {
    match value {
        ArcherBehaviourType::MoveTo { target_position } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
