use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{BaseAnimated, SheepData};
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};

const IDLE: u8 = 0;

struct SheepIdleParams {
    sheep: BaseAnimated,
    sheep_data: SheepData,
    time: f64,
    new_behaviour: Option<SheepBehaviour>,
    state: BehaviourState,
}

pub fn process(game: &mut DemoGameData, sheep_index: usize) {
    let mut params = read_params(game, sheep_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(IDLE) => idle(game, &mut params),
        _ => {}
    }

    write_params(game, sheep_index, &params);
}

fn init(game: &DemoGameData, params: &mut SheepIdleParams) {
    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    if params.sheep_data.anchor_position == Position::default() {
        params.sheep_data.anchor_position = params.sheep.position;
    }

    params.sheep.animation = game.assets.animations.sheep.idle;
    params.sheep.current_frame = 0;

    params.time = game.global.time;
    params.state = BehaviourState::Running(IDLE);
}

fn idle(game: &DemoGameData, params: &mut SheepIdleParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    let timer = 5000.0 + (fastrand::u32(1000..5000) as f64);

    if elapsed(game.global.time, params.time, timer) {
        let mut target_position = params.sheep_data.anchor_position;
        target_position.x += fastrand::i16(-100..100) as f32;
        target_position.y += fastrand::i16(-100..100) as f32;
        params.new_behaviour = Some(SheepBehaviour {
            ty: SheepBehaviourType::MoveTo { target_position },
            state: BehaviourState::Initial
        });
    }
}

fn read_params(game: &DemoGameData, sheep_index: usize) -> SheepIdleParams {
    let sheep = unsafe { game.world.sheeps.get_unchecked(sheep_index) };
    let sheep_data = unsafe { game.world.sheeps_data.get_unchecked(sheep_index) };
    let sheep_behaviour = unsafe { game.world.sheep_behaviour.get_unchecked(sheep_index) };

    let time = match sheep_behaviour.ty {
        SheepBehaviourType::Idle { time } => time,
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    SheepIdleParams {
        sheep: *sheep,
        sheep_data: *sheep_data,
        time,
        new_behaviour: None,
        state: sheep_behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, sheep_index: usize, params: &SheepIdleParams) {
    let sheep = unsafe { game.world.sheeps.get_unchecked_mut(sheep_index) };
    let sheep_data = unsafe { game.world.sheeps_data.get_unchecked_mut(sheep_index) };
    let sheep_behaviour = unsafe { game.world.sheep_behaviour.get_unchecked_mut(sheep_index) };

    *sheep = params.sheep;
    *sheep_data = params.sheep_data;

    match params.new_behaviour {
        Some(new_behaviour) => { *sheep_behaviour = new_behaviour; },
        None => {
            sheep_behaviour.ty = SheepBehaviourType::Idle { time: params.time };
            sheep_behaviour.state = params.state;
        }
    }
}
