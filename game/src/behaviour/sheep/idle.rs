use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};

const IDLE: u8 = 0;

pub fn process(game: &mut DemoGameData, sheep_index: usize) {
    let state = game.world.sheep_behaviour[sheep_index].state;

    match state {
        BehaviourState::Initial => init(game, sheep_index),
        BehaviourState::Running(IDLE) => idle(game, sheep_index),
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

    sheep.animation = game.assets.animations.sheep.idle;
    sheep.current_frame = 0;

    if sheep_data.anchor_position == Position::default() {
        sheep_data.anchor_position = sheep.position;
    }

    match &mut behaviour.ty {
        SheepBehaviourType::Idle { time } => { *time = game.global.time; }
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }

    behaviour.state = BehaviourState::Running(IDLE);
}

fn idle(game: &mut DemoGameData, sheep_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let world = &mut game.world;
    let sheep_data = &mut world.sheeps_data[sheep_index];
    let behaviour = &mut world.sheep_behaviour[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = SheepBehaviour::dead();
        return;
    }

    let timer = 5000.0 + (fastrand::u32(1000..5000) as f64);
    let timestamp_value = timestamp(behaviour.ty);
    let sheep_anchor = sheep_data.anchor_position;

    if elapsed(game.global.time, timestamp_value, timer) {
        let mut target_position = sheep_anchor;
        target_position.x += fastrand::i8(-100..100) as f32;
        target_position.y += fastrand::i8(-100..100) as f32;
        *behaviour = SheepBehaviour {
            ty: SheepBehaviourType::Moving { target_position },
            state: BehaviourState::Initial
        };
    }
}

#[inline(always)]
fn timestamp(value: SheepBehaviourType) -> f64 {
    match value {
        SheepBehaviourType::Idle { time, .. } => time,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

