use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{BaseAnimated, SheepData};
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};

const ESCAPING: u8 = 0;

pub struct SheepEscapingParams {
    sheep: BaseAnimated,
    sheep_data: SheepData,
    target_position: Position<f32>,
    new_behaviour: Option<SheepBehaviour>,
    state: BehaviourState,
}

pub fn process(game: &mut DemoGameData, sheep_index: usize) {
    let mut params = read_params(game, sheep_index);

    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(ESCAPING) => escaping(game, &mut params),
        _ => {}
    }

    write_params(game, sheep_index, &params);
}

pub fn init(game: &DemoGameData, params: &mut SheepEscapingParams) {
    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    params.sheep.animation = game.assets.animations.sheep.walk;
    params.sheep.current_frame = 0;
    params.state = BehaviourState::Running(ESCAPING);

    let mut target_position = params.sheep_data.anchor_position;
    target_position.x += fastrand::i16(-100..100) as f32;
    target_position.y += fastrand::i16(-100..100) as f32;
    params.target_position = target_position;

    escaping(game, params);
}

pub fn escaping(game: &DemoGameData, params: &mut SheepEscapingParams) {
    use crate::behaviour::behaviour_shared::move_to_with_speed;

    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    let updated_position = move_to_with_speed(params.sheep.position, params.target_position, game.global.frame_delta, 0.25);
    if updated_position == params.target_position {
        params.new_behaviour = Some(SheepBehaviour::idle());
    } else {
        params.sheep.flipped = params.sheep.position.x > params.target_position.x;
    }

    params.sheep.position = updated_position;
}

fn read_params(game: &DemoGameData, sheep_index: usize) -> SheepEscapingParams {
    let sheep = game.world.sheeps.get(sheep_index);
    let sheep_data = game.world.sheeps_data.get(sheep_index);
    let sheep_behaviour = game.world.sheep_behaviour.get(sheep_index);

    match (sheep, sheep_data, sheep_behaviour) {
        (Some(sheep), Some(sheep_data), Some(sheep_behaviour)) => {
            let target_position = match sheep_behaviour.ty {
                SheepBehaviourType::Escaping { target_position } => target_position,
                _ => unsafe { ::std::hint::unreachable_unchecked(); }
            };

            SheepEscapingParams {
                sheep: *sheep,
                sheep_data: *sheep_data,
                target_position,
                new_behaviour: None,
                state: sheep_behaviour.state
            }
        },
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}

fn write_params(game: &mut DemoGameData, sheep_index: usize, params: &SheepEscapingParams) {
    let sheep = game.world.sheeps.get_mut(sheep_index);
    let sheep_behaviour = game.world.sheep_behaviour.get_mut(sheep_index);

    match (sheep, sheep_behaviour) {
        (Some(sheep), Some(sheep_behaviour)) => {
            *sheep = params.sheep;

            match params.new_behaviour {
                Some(new_behaviour) => { *sheep_behaviour = new_behaviour; },
                None => {
                    sheep_behaviour.ty = SheepBehaviourType::Escaping { target_position: params.target_position };
                    sheep_behaviour.state = params.state;
                }
            }
        }
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}
