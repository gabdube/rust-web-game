use crate::behaviour::BehaviourState;
use crate::world::{BaseAnimated, BaseStatic, PawnData, ResourceData, ResourceType, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_RESOURCE: u8 = 0;
const GRAB_RESOURCE: u8 = 1;

struct PawnGrabResourceParams {
    pawn: BaseAnimated,
    pawn_data: PawnData,
    resource: BaseStatic,
    resource_data: ResourceData,
    resource_index: u32,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, pawn: WorldObject, resource: WorldObject) {
    match (pawn.ty, resource.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Resource) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let resource_index = resource.id as usize;
    if resource_index >= game.world.resources.len() || pawn_index >= game.world.pawns.len() {
        return;
    }

    let resource_data = game.world.resources_data[resource_index];
    if resource_data.grabbed {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::GrabResource { resource_id: resource.id },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_RESOURCE) => move_to_resource(game, &mut params),
        BehaviourState::Running(GRAB_RESOURCE) => grab_resource(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &params)
}

fn init(game: &DemoGameData, params: &mut PawnGrabResourceParams) {
    params.state = BehaviourState::Running(MOVE_TO_RESOURCE);
    params.pawn.animation = game.assets.animations.pawn.walk;
    move_to_resource(game, params);
}

fn move_to_resource(game: &DemoGameData, params: &mut PawnGrabResourceParams) {
    use crate::behaviour::behaviour_shared::move_to;

    if params.resource_data.grabbed {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let target_position = params.resource.position;
    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.state = BehaviourState::Running(GRAB_RESOURCE);
    }

    params.pawn.position = updated_position;
    params.pawn.flipped = params.pawn.position.x > target_position.x;
}

fn grab_resource(game: &DemoGameData, params: &mut PawnGrabResourceParams) {
    if params.resource_data.grabbed {
        // Targeted resource was grabbed by another pawn
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    params.pawn.animation = game.assets.animations.pawn.idle_hold;
    params.pawn_data.grabbed_resource = params.resource_index as u32;

    params.resource_data.grabbed = true;

    params.resource.position = params.pawn.position;
    params.resource.position.y -= 60.0;

    params.resource.sprite = match params.resource_data.resource_type {
        ResourceType::Gold => game.assets.resources.gold_shadowless,
        ResourceType::Food => game.assets.resources.meat_shadowless,
        ResourceType::Wood => game.assets.resources.wood_shadowless,
    };

    params.new_behaviour = Some(PawnBehaviour::idle());
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnGrabResourceParams {
    let pawn = unsafe { game.world.pawns.get_unchecked(pawn_index) };
    let pawn_data = unsafe { game.world.pawns_data.get_unchecked(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked(pawn_index) };

    let resource_index = match behaviour.ty {
        PawnBehaviourType::GrabResource { resource_id } => resource_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    };

    let resource = unsafe { game.world.resources.get_unchecked(resource_index) };
    let resource_data = unsafe { game.world.resources_data.get_unchecked(resource_index) };

    PawnGrabResourceParams {
        pawn: *pawn,
        pawn_data: *pawn_data,
        resource: *resource,
        resource_data: *resource_data,
        resource_index: resource_index as u32,
        new_behaviour: None,
        state: behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnGrabResourceParams) {
    let pawn = unsafe { game.world.pawns.get_unchecked_mut(pawn_index) };
    let pawn_data = unsafe { game.world.pawns_data.get_unchecked_mut(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked_mut(pawn_index) };
    let resource = unsafe { game.world.resources.get_unchecked_mut(params.resource_index as usize) };
    let resource_data = unsafe { game.world.resources_data.get_unchecked_mut(params.resource_index as usize) };

    *pawn = params.pawn;
    *pawn_data = params.pawn_data;
    *resource = params.resource;
    *resource_data = params.resource_data;

    match params.new_behaviour {
        Some(new_behaviour) => {
            *behaviour = new_behaviour;
        },
        None => {
            behaviour.state = params.state;
        }
    }
}
