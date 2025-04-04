use crate::behaviour::BehaviourState;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_SHEEP: u8 = 0;
const ATTACK_SHEEP: u8 = 1;
const PAUSE: u8 = 3;

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
    let state = game.world.pawns_behaviour[pawn_index].state;

    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVE_TO_SHEEP) => move_to_sheep(game, pawn_index),
        BehaviourState::Running(ATTACK_SHEEP) => attack_sheep(game, pawn_index),
        BehaviourState::Running(PAUSE) => pause(game, pawn_index),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;
    game.world.pawns_behaviour[pawn_index].state = BehaviourState::Running(MOVE_TO_SHEEP);
    move_to_sheep(game, pawn_index);
}

fn move_to_sheep(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let sheep_index = params(behaviour.ty);
    let sheep = &mut world.sheeps[sheep_index];
    let sheep_data = &mut world.sheeps_data[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let mut target_position = sheep.position;
    target_position.y += 1.0;
    if pawn.position.x > target_position.x {
        target_position.x += 60.0;
    } else {
        target_position.x -= 60.0;
    }

    let updated_position = move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        pawn.animation = game.assets.animations.pawn.axe;
        pawn.current_frame = 0;
        behaviour.state = BehaviourState::Running(ATTACK_SHEEP);
        params_set_last_timestamp(&mut behaviour.ty, game.global.time);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > sheep.position.x;
}

fn attack_sheep(data: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let world = &mut data.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let sheep_index = params(behaviour.ty);
    let sheep = &mut world.sheeps[sheep_index];
    let sheep_data = &mut world.sheeps_data[sheep_index];

    if sheep_data.life == 0 {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    if sheep.position.distance(pawn.position) > 65.0 {
        pawn.animation = data.assets.animations.pawn.idle;
        params_set_last_timestamp(&mut behaviour.ty, data.global.time);
        behaviour.state = BehaviourState::Running(PAUSE);
        return;
    }

    let last_timestamp = params_timestamp(behaviour.ty);

    if pawn.current_frame == 5 && elapsed(data.global.time, last_timestamp, 300.0) {
        params_set_last_timestamp(&mut behaviour.ty, data.global.time);
        crate::behaviour::sheep::strike(data, sheep_index, 4);
    }
}

fn pause(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let pawn = &mut game.world.pawns[pawn_index];
    let behaviour = &mut game.world.pawns_behaviour[pawn_index];
    let timestamp = params_timestamp(behaviour.ty);
    if elapsed(game.global.time, timestamp, 500.0) {
        pawn.animation = game.assets.animations.pawn.walk;
        pawn.current_frame = 0;
        behaviour.state = BehaviourState::Running(MOVE_TO_SHEEP);
    }
}

#[inline(always)]
fn params(value: PawnBehaviourType) -> usize {
    match value {
        PawnBehaviourType::HuntSheep { sheep_id, .. } => sheep_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

#[inline(always)]
fn params_timestamp(value: PawnBehaviourType) -> f64 {
    match value {
        PawnBehaviourType::HuntSheep { last_timestamp, .. } => last_timestamp as f64,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

#[inline(always)]
fn params_set_last_timestamp(value: &mut PawnBehaviourType, time: f64) {
    match value {
        PawnBehaviourType::HuntSheep { last_timestamp, .. } => *last_timestamp = time as f32,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}
