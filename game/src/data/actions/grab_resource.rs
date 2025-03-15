use crate::data::actions::{Action, ActionType, ActionState};
use crate::world::{ResourceType, WorldObject};
use crate::DemoGameData;

const MOVE_TO_RESOURCE: u8 = 0;
const GRAB_RESOURCE: u8 = 1;
const MOVE_GRABBED_RESOURCE: u8 = 2;

pub fn new(game: &mut DemoGameData, pawn: WorldObject, resource: WorldObject) {
    let pawn_index = pawn.id as usize;
    let resource_index = resource.id as usize;
    if game.world.pawns.len() <= pawn_index || game.world.resources.len() <= resource_index {
        return;
    }

    let resource_data = game.world.resources_data[resource_index];
    if resource_data.grabbed {
        return;
    }

    let grab_action = Action::from_type(ActionType::GrabResource { pawn_id: pawn.id, resource_id: resource.id });
    game.actions.push(grab_action);
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    if action.running_value() != MOVE_GRABBED_RESOURCE {
        return;
    }

    let [pawn_index, resource_index] = params(action);
    let pawn = &mut game.world.pawns[pawn_index];
    let pawn_data = &mut game.world.pawns_data[pawn_index];
    let resource = &mut game.world.resources[resource_index];
    let resource_data = &mut game.world.resources_data[resource_index];
    
    pawn_data.grabbed_resource = u32::MAX;

    resource_data.grabbed = false;

    resource.position = pawn.position;
    resource.position.y += 5.0;
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Done;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running(MOVE_TO_RESOURCE) => move_to_resource(game, action),
        ActionState::Running(GRAB_RESOURCE) => grab_resource(game, action),
        ActionState::Running(MOVE_GRABBED_RESOURCE) => move_grabbed_resource(game, action),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, _] = params(action);

    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;

    action.state = ActionState::Running(MOVE_TO_RESOURCE);

    move_to_resource(game, action);
}

fn move_to_resource(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, resource_index] = params(action);

    let pawn = &mut game.world.pawns[pawn_index];
    let resource_data = game.world.resources_data[resource_index];
    if resource_data.grabbed {
        pawn.animation = game.assets.animations.pawn.idle;
        action.state = ActionState::Done;
        return;
    }

    let target_position = game.world.resources[resource_index].position;
    let updated_position = super::actions_shared::move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        action.state = ActionState::Running(GRAB_RESOURCE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn grab_resource(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, resource_index] = params(action);

    let pawn = &mut game.world.pawns[pawn_index];
    let pawn_data = &mut game.world.pawns_data[pawn_index];
    let resource = &mut game.world.resources[resource_index];
    let resource_data = &mut game.world.resources_data[resource_index];

    pawn.animation = game.assets.animations.pawn.idle_hold;
    pawn_data.grabbed_resource = resource_index as u32;

    resource_data.grabbed = true;
    resource.sprite = match resource_data.resource_type {
        ResourceType::Gold => game.assets.resources.gold_shadowless.aabb,
        ResourceType::Meat => game.assets.resources.meat_shadowless.aabb,
        ResourceType::Wood => game.assets.resources.wood_shadowless.aabb,
    };

    action.state = ActionState::Running(MOVE_GRABBED_RESOURCE);
}

fn move_grabbed_resource(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, resource_index] = params(action);
    let pawn = &game.world.pawns[pawn_index];
    let resource = &mut game.world.resources[resource_index];

    resource.position = pawn.position;
    resource.position.y -= 60.0;
}

fn validate(game: &mut DemoGameData, action: &mut Action) -> bool {
    let [pawn_index, resource_index] = params(action);
    game.world.pawns.len() > pawn_index && game.world.resources.len() > resource_index
}

fn params(action: &mut Action) -> [usize; 2] {
    match action.ty {
        ActionType::GrabResource { pawn_id, resource_id } => [pawn_id as usize, resource_id as usize],
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
