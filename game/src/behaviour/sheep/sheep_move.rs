use crate::{behaviour::BehaviourState, shared::Position};
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};


const MOVE: u8 = 0;

pub fn process(game: &mut DemoGameData, sheep_index: usize) {
    let state = game.world.sheep_behaviour[sheep_index].state;

    match state {
        BehaviourState::Initial => init(game, sheep_index),
        BehaviourState::Running(MOVE) => move_sheep(game, sheep_index),
        _ => {}
    }
}
 
fn init(game: &mut DemoGameData, sheep_index: usize) {
    let world = &mut game.world;
    let sheep = &mut world.sheeps[sheep_index];
    let sheep_data = &mut world.sheeps_data[sheep_index];
    let behaviour = &mut world.sheep_behaviour[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = SheepBehaviour::dead();
        return;
    }

    sheep.animation = game.assets.animations.sheep.walk;
    sheep.current_frame = 0;
    behaviour.state = BehaviourState::Running(MOVE);

    move_sheep(game, sheep_index);
}

fn move_sheep(game: &mut DemoGameData, sheep_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;

    let world = &mut game.world;
    let sheep = &mut world.sheeps[sheep_index];
    let sheep_data = &mut world.sheeps_data[sheep_index];
    let behaviour = &mut world.sheep_behaviour[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = SheepBehaviour::dead();
        return;
    }

    let current_position = sheep.position;
    let target_position = params(behaviour.ty);
    let updated_position = move_to(current_position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        *behaviour = SheepBehaviour::idle(); 
    }

    sheep.position = updated_position;
    sheep.flipped = updated_position.x > updated_position.x;
}

#[inline(always)]
fn params(value: SheepBehaviourType) -> Position<f32> {
    match value {
        SheepBehaviourType::Moving { target_position, .. } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
