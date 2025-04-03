use crate::behaviour::BehaviourState;
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};

const ESCAPING: u8 = 0;

pub fn process(game: &mut DemoGameData, sheep_index: usize) {
    let state = game.world.sheep_behaviour[sheep_index].state;

    match state {
        BehaviourState::Initial => init(game, sheep_index),
        BehaviourState::Running(ESCAPING) => escaping(game, sheep_index),
        _ => {}
    }
}

pub fn init(game: &mut DemoGameData, sheep_index: usize) {
    let world = &mut game.world;
    let sheep = &mut world.sheeps[sheep_index];
    let behaviour = &mut world.sheep_behaviour[sheep_index];

    sheep.animation = game.assets.animations.sheep.walk;
    sheep.current_frame = 0;
    behaviour.state = BehaviourState::Running(ESCAPING);

    match &mut behaviour.ty {
        SheepBehaviourType::Escaping { timestamp, angle } => { 
            *angle = f32::to_radians(fastrand::u32(0..360) as f32);
            *timestamp = game.global.time;
        },
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }

    escaping(game, sheep_index);
}

pub fn escaping(game: &mut DemoGameData, sheep_index: usize) {
    use crate::behaviour::behaviour_shared::{move_to_with_speed, elapsed};

    let world = &mut game.world;

    let sheep = &mut world.sheeps[sheep_index];
    let sheep_data = &mut world.sheeps_data[sheep_index];
    let behaviour = &mut world.sheep_behaviour[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = SheepBehaviour::dead();
        return;
    }

    if elapsed(game.global.time, params_timestamp(behaviour.ty), 500.0) {
        *behaviour = SheepBehaviour::idle();
        return;
    }

    let angle = params_angle(behaviour.ty);
    let mut target_position = sheep.position;
    target_position.x = f32::ceil(sheep.position.x + f32::cos(angle) * 10.0);
    target_position.y = f32::ceil(sheep.position.y + f32::sin(angle) * 10.0);

    let updated_position = move_to_with_speed(sheep.position, target_position, game.global.frame_delta, 0.25);
    if updated_position == target_position {
        *behaviour = SheepBehaviour::idle(); 
    } else {
        sheep.flipped = sheep.position.x > target_position.x;
    }

    sheep.position = updated_position;
}

#[inline(always)]
fn params_angle(value: SheepBehaviourType) -> f32 {
    match value {
        SheepBehaviourType::Escaping { angle, .. } => angle,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

#[inline(always)]
fn params_timestamp(value: SheepBehaviourType) -> f64 {
    match value {
        SheepBehaviourType::Escaping { timestamp, .. } => timestamp,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
