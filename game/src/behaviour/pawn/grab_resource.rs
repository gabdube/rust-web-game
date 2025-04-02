use crate::behaviour::BehaviourState;
use crate::world::{ResourceType, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_RESOURCE: u8 = 0;
const GRAB_RESOURCE: u8 = 1;

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
    let state = game.world.pawns_behaviour[pawn_index].state;

    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVE_TO_RESOURCE) => move_to_resource(game, pawn_index),
        BehaviourState::Running(GRAB_RESOURCE) => grab_resource(game, pawn_index),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;
    game.world.pawns_behaviour[pawn_index].state = BehaviourState::Running(MOVE_TO_RESOURCE);
    move_to_resource(game, pawn_index);
}

fn move_to_resource(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let resource_index = params(behaviour.ty);
    let resource_data = world.resources_data[resource_index];
    if resource_data.grabbed {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let target_position = world.resources[resource_index].position;
    let updated_position = move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        behaviour.state = BehaviourState::Running(GRAB_RESOURCE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn grab_resource(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let pawn_data = &mut world.pawns_data[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let resource_index = params(behaviour.ty);
    let resource = &mut world.resources[resource_index];
    let resource_data = &mut world.resources_data[resource_index];

    if resource_data.grabbed {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    pawn.animation = game.assets.animations.pawn.idle_hold;
    pawn_data.grabbed_resource = resource_index as u32;

    resource_data.grabbed = true;

    resource.position = pawn.position;
    resource.position.y -= 60.0;

    resource.sprite = match resource_data.resource_type {
        ResourceType::Gold => game.assets.resources.gold_shadowless,
        ResourceType::Food => game.assets.resources.meat_shadowless,
        ResourceType::Wood => game.assets.resources.wood_shadowless,
    };

    *behaviour = PawnBehaviour::idle();
}

#[inline(always)]
fn params(value: PawnBehaviourType) -> usize {
    match value {
        PawnBehaviourType::GrabResource { resource_id } => resource_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
