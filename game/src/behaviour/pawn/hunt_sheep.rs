use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{BaseAnimated, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_SHEEP: u8 = 0;
const ATTACK_SHEEP: u8 = 1;
const PAUSE: u8 = 3;

struct PawnHuntSheepParams {
    pawn: BaseAnimated,
    sheep_position: Position<f32>, 
    sheep_life: u8,
    last_timestamp: f32,
    sheep_id: u32,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, pawn: WorldObject, sheep: WorldObject) {
    match (pawn.ty, sheep.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Sheep) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let sheep_index = sheep.id as usize;
    if pawn_index >= game.world.pawns.len() || sheep_index >= game.world.sheeps.len() {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::HuntSheep { sheep_id: sheep.id, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_SHEEP) => move_to_sheep(game, &mut params),
        BehaviourState::Running(ATTACK_SHEEP) => attack_sheep(game, &mut params),
        BehaviourState::Running(PAUSE) => pause(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &params);
}

fn init(game: &DemoGameData, params: &mut PawnHuntSheepParams) {
    params.pawn.animation = game.assets.animations.pawn.walk;
    params.state = BehaviourState::Running(MOVE_TO_SHEEP);
    move_to_sheep(game, params);
}

fn move_to_sheep(game: &DemoGameData, params: &mut PawnHuntSheepParams) {
    use crate::behaviour::behaviour_shared::move_to;

    if params.sheep_life == 0 {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let mut target_position = params.sheep_position;
    target_position.y += 1.0;
    if params.pawn.position.x > target_position.x {
        target_position.x += 60.0;
    } else {
        target_position.x -= 60.0;
    }

    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.pawn.animation = game.assets.animations.pawn.axe;
        params.pawn.current_frame = 0;
        params.state = BehaviourState::Running(ATTACK_SHEEP);
        params.last_timestamp = game.global.time as f32;
    }

    params.pawn.position = updated_position;
    params.pawn.flipped = params.pawn.position.x > params.sheep_position.x;
}

fn attack_sheep(game: &mut DemoGameData, params: &mut PawnHuntSheepParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    if params.sheep_life == 0 {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    if params.sheep_position.distance(params.pawn.position) > 65.0 {
        params.pawn.animation = game.assets.animations.pawn.idle;
        params.last_timestamp = game.global.time as f32;
        params.state = BehaviourState::Running(PAUSE);
        return;
    }
    else if params.pawn.current_frame == 5 && elapsed(game.global.time, params.last_timestamp as f64, 300.0) {
        params.last_timestamp = game.global.time as f32;
        crate::behaviour::sheep::strike(game, params.sheep_id as usize, 4);
    }
}

fn pause(game: &DemoGameData, params: &mut PawnHuntSheepParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    if elapsed(game.global.time, params.last_timestamp as f64, 500.0) {
        params.pawn.animation = game.assets.animations.pawn.walk;
        params.pawn.current_frame = 0;
        params.state = BehaviourState::Running(MOVE_TO_SHEEP);
    }
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnHuntSheepParams {
    let pawn = game.world.pawns.get(pawn_index);
    let behaviour = game.world.pawns_behaviour.get(pawn_index);

    match (pawn, behaviour) {
        (Some(pawn), Some(behaviour)) => {
            let (sheep_index, last_timestamp) = match behaviour.ty {
                PawnBehaviourType::HuntSheep { sheep_id, last_timestamp } => (sheep_id as usize, last_timestamp),
                _ => unsafe { ::std::hint::unreachable_unchecked() }
            };

            let (sheep_position, sheep_life) = match (game.world.sheeps.get(sheep_index), game.world.sheeps_data.get(sheep_index)) {
                (Some(sheep), Some(sheep_data)) => (sheep.position, sheep_data.life),
                _ => unsafe { ::std::hint::unreachable_unchecked() }
            };

            PawnHuntSheepParams {
                pawn: *pawn,
                sheep_position,
                sheep_life,
                last_timestamp,
                sheep_id: sheep_index as u32,
                new_behaviour: None,
                state: behaviour.state
            }
        },
        _  => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnHuntSheepParams) {
    let pawn = game.world.pawns.get_mut(pawn_index);
    let behaviour = game.world.pawns_behaviour.get_mut(pawn_index);

    match (pawn, behaviour) {
        (Some(pawn), Some(behaviour)) => {
            *pawn = params.pawn;

            match params.new_behaviour {
                Some(new_behaviour) => {
                    *behaviour = new_behaviour;
                },
                None => {
                    behaviour.ty = PawnBehaviourType::HuntSheep { sheep_id: params.sheep_id, last_timestamp: params.last_timestamp };
                    behaviour.state = params.state;
                }
            }
        },
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}
