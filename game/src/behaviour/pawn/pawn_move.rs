use crate::behaviour::BehaviourState;
use crate::pathfinding::PathFindingData;
use crate::shared::Position;
use crate::world::{BaseAnimated, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVING: u8 = 0;

struct PawnMoveParams {
    pawn: BaseAnimated,
    pawn_grabbed_resource: bool,
    pathfinding_state: PathFindingData,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, pawn: WorldObject, target_position: Position<f32>) {
    let pawn_index = pawn.id as usize;
    if pawn.ty != WorldObjectType::Pawn || pawn_index >= game.world.pawns.len() {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, false);

    let starting_position = game.world.pawns[pawn_index].position;
    let pathfinding_state = game.world.pathfinding.new(starting_position, target_position);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::MoveTo { pathfinding_state },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVING) => moving(game, &mut params),
        _ => {},
    }

    write_params(game, pawn_index, &params);
}

fn init(game: &DemoGameData, params: &mut PawnMoveParams) {
    let completed = game.world.pathfinding.compute_path(&mut params.pathfinding_state);
    if completed {
        params.state = BehaviourState::Running(MOVING);
        params.pawn.animation = match params.pawn_grabbed_resource {
            true => game.assets.animations.pawn.walk_hold,
            false => game.assets.animations.pawn.walk
        };
    }
}

fn moving(game: &DemoGameData, params: &mut PawnMoveParams) {
    use crate::behaviour::behaviour_shared::move_to;
    let updated_position = move_to(params.pawn.position, params.pathfinding_state.final_position, game.global.frame_delta);
    if updated_position == params.pathfinding_state.final_position {
        params.new_behaviour = Some(PawnBehaviour::idle());
    } else {
        params.pawn.flipped = params.pawn.position.x > params.pathfinding_state.final_position.x;
    }

    params.pawn.position = updated_position;
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnMoveParams {
    let pawn = unsafe { game.world.pawns.get_unchecked(pawn_index) };
    let pawn_data = unsafe { game.world.pawns_data.get_unchecked(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked(pawn_index) };

    let pathfinding_state = match behaviour.ty {
        PawnBehaviourType::MoveTo { pathfinding_state } => pathfinding_state,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    };

    PawnMoveParams {
        pawn: *pawn,
        pawn_grabbed_resource: pawn_data.grabbed_resource().is_some(),
        pathfinding_state,
        new_behaviour: None,
        state: behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnMoveParams) {
    let pawn = unsafe { game.world.pawns.get_unchecked_mut(pawn_index) };
    let pawn_data = unsafe { game.world.pawns_data.get_unchecked_mut(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked_mut(pawn_index) };

    if let Some(resource_index) = pawn_data.grabbed_resource() {
        let resource = &mut game.world.resources[resource_index];
        resource.position = params.pawn.position;
        resource.position.y -= 60.0;
    }

    *pawn = params.pawn;

    match params.new_behaviour {
        Some(new_behaviour) => { *behaviour = new_behaviour; },
        None => { behaviour.state = params.state; }
    }
}
