use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{WarriorBehaviour, WarriorBehaviourType};

const MOVING: u8 = 0;

pub fn new(game: &mut DemoGameData, warrior: WorldObject, target_position: Position<f32>) {
    let warrior_index = warrior.id as usize;

    if warrior.ty != WorldObjectType::Warrior || warrior_index >= game.world.warriors.len() {
        return;
    }

    game.world.warriors_behaviour[warrior_index] = WarriorBehaviour {
        ty: WarriorBehaviourType::MoveTo { target_position },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, warrior_index: usize) {
    let state = game.world.warriors_behaviour[warrior_index].state;
    match state {
        BehaviourState::Initial => init(game, warrior_index),
        BehaviourState::Running(MOVING) => moving(game, warrior_index),
        _ => {},
    }
}

fn init(game: &mut DemoGameData, warrior_index: usize) {
    let warrior = &mut game.world.warriors[warrior_index];
    let behaviour = &mut game.world.warriors_behaviour[warrior_index];
    warrior.animation = game.assets.animations.warrior.walk;
    behaviour.state = BehaviourState::Running(MOVING);
}

fn moving(game: &mut DemoGameData, warrior_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let behaviour = &mut game.world.warriors_behaviour[warrior_index];
    let warrior = &mut game.world.warriors[warrior_index];
    let target_position = params(behaviour.ty);
   
    let current_position = warrior.position;
    let updated_position = move_to(current_position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        *behaviour = WarriorBehaviour::idle();
    } else {
        warrior.flipped = current_position.x > target_position.x;
    }

    warrior.position = updated_position;
}

#[inline(always)]
fn params(value: WarriorBehaviourType) -> Position<f32> {
    match value {
        WarriorBehaviourType::MoveTo { target_position } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
