use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVING: u8 = 0;
const STOPPING: u8 = 1;

pub fn new(game: &mut DemoGameData, pawn: WorldObject, target_position: Position<f32>) {
    let pawn_index = pawn.id as usize;

    if pawn.ty != WorldObjectType::Pawn || pawn_index >= game.world.pawns.len() {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::MoveTo { target_position },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let state = game.world.pawns_behaviour[pawn_index].state;
    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVING) => moving(game, pawn_index),
        BehaviourState::Running(STOPPING) => stopping(game, pawn_index),
        _ => {},
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    let pawn = &mut game.world.pawns[pawn_index];
    let pawn_data = &game.world.pawns_data[pawn_index];
    let behaviour = &mut game.world.pawns_behaviour[pawn_index];

    pawn.animation = match pawn_data.grabbed_resource() {
        Some(_) => game.assets.animations.pawn.walk_hold,
        None => game.assets.animations.pawn.walk
    };

    behaviour.state = BehaviourState::Running(MOVING);
}

fn moving(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let behaviour = &mut game.world.pawns_behaviour[pawn_index];
    let pawn = &mut game.world.pawns[pawn_index];
    let pawn_data = &mut game.world.pawns_data[pawn_index];
    let target_position = params(behaviour.ty);
   
    let current_position = pawn.position;
    let updated_position = move_to(current_position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        behaviour.state = BehaviourState::Running(STOPPING);
    }

    if let Some(resource_index) = pawn_data.grabbed_resource() {
        let resource = &mut game.world.resources[resource_index];
        resource.position = pawn.position;
        resource.position.y -= 60.0;
    }

    pawn.position = updated_position;
    pawn.flipped = updated_position.x > target_position.x;
}

fn stopping(game: &mut DemoGameData, pawn_index: usize) {
    let pawn = &mut game.world.pawns[pawn_index];
    let pawn_data = &mut game.world.pawns_data[pawn_index];
    let behaviour = &mut game.world.pawns_behaviour[pawn_index];

    pawn.animation = match pawn_data.grabbed_resource() {
        Some(_) => game.assets.animations.pawn.idle_hold,
        None => game.assets.animations.pawn.idle
    };

    *behaviour = PawnBehaviour::idle();
}

#[inline(always)]
fn params(value: PawnBehaviourType) -> Position<f32> {
    match value {
        PawnBehaviourType::MoveTo { target_position } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
