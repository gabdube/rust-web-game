use crate::world::{BaseAnimated, SheepData};
use crate::{behaviour::BehaviourState, shared::Position};
use crate::DemoGameData;
use super::{SheepBehaviour, SheepBehaviourType};

const MOVE: u8 = 0;

struct SheepMoveParams {
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
        BehaviourState::Running(MOVE) => move_sheep(game, &mut params),
        _ => {}
    }

    write_params(game, sheep_index, &params);
}
 
fn init(game: &DemoGameData, params: &mut SheepMoveParams) {
    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    params.sheep.animation = game.assets.animations.sheep.walk;
    params.sheep.current_frame = 0;
    params.state = BehaviourState::Running(MOVE);

    move_sheep(game, params);
}

fn move_sheep(game: &DemoGameData, params: &mut SheepMoveParams) {
    use crate::behaviour::behaviour_shared::move_to_with_speed;

    if params.sheep_data.life == 0 {
        params.new_behaviour = Some(SheepBehaviour::dead());
        return;
    }

    let updated_position = move_to_with_speed(params.sheep.position, params.target_position, game.global.frame_delta, 0.1);
    if updated_position == params.target_position {
        params.new_behaviour = Some(SheepBehaviour::idle());
    } else {
        params.sheep.flipped = params.sheep.position.x > params.target_position.x;
    }

    params.sheep.position = updated_position;
}

fn read_params(game: &DemoGameData, sheep_index: usize) -> SheepMoveParams {
    let sheep = game.world.sheeps.get(sheep_index);
    let sheep_data = game.world.sheeps_data.get(sheep_index);
    let sheep_behaviour = game.world.sheep_behaviour.get(sheep_index);

    match (sheep, sheep_data, sheep_behaviour) {
        (Some(sheep), Some(sheep_data), Some(sheep_behaviour)) => {
            let target_position = match sheep_behaviour.ty {
                SheepBehaviourType::Moving { target_position } => target_position,
                _ => unsafe { ::std::hint::unreachable_unchecked(); }
            };

            SheepMoveParams {
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

fn write_params(game: &mut DemoGameData, sheep_index: usize, params: &SheepMoveParams) {
    let sheep = game.world.sheeps.get_mut(sheep_index);
    let sheep_behaviour = game.world.sheep_behaviour.get_mut(sheep_index);

    match (sheep, sheep_behaviour) {
        (Some(sheep), Some(sheep_behaviour)) => {
            *sheep = params.sheep;

            match params.new_behaviour {
                Some(new_behaviour) => { *sheep_behaviour = new_behaviour; },
                None => {
                    sheep_behaviour.state = params.state;
                }
            }
        }
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}
