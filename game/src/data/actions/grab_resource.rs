use crate::data::actions::{Action, ActionType, ActionState};
use crate::world::{ResourceType, WorldObject};
use crate::DemoGameData;

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

    let target_position = game.world.resources[resource_index].position;

    let move_action = Action::from_type(ActionType::MoveActor { actor: pawn, target_position });
    let grab_action = Action::from_type(ActionType::GrabResource { pawn_id: pawn.id, resource_id: resource.id });
    game.actions.push2(move_action, grab_action);
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    let [pawn_index, resource_index] = params(action);
    let mut pawn_position = game.world.pawns[pawn_index].position;
    pawn_position.x += 10.0;

    game.world.resources[resource_index].position = pawn_position;
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Finalized;
        return;
    }

    // Grab resource is an action that must be cancelled in order to be stopped
    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running => run(game, action),
        ActionState::Finalizing => {},
        ActionState::Finalized => {}
    }

    run(game, action)
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let [_, resource_index] = params(action);
    let resource_data = &mut game.world.resources_data[resource_index];
    if resource_data.grabbed {
        action.state = ActionState::Finalized;
        return;
    }

    let resource = &mut game.world.resources[resource_index];
    resource.sprite = match resource_data.resource_type {
        ResourceType::Gold => game.assets.resources.gold_shadowless.aabb,
        ResourceType::Meat => game.assets.resources.meat_shadowless.aabb,
        ResourceType::Wood => game.assets.resources.wood_shadowless.aabb,
    };

    resource_data.grabbed = true;

    action.state = ActionState::Running;

    run(game, action);
}

fn run(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, resource_index] = params(action);

    let mut pawn_position = game.world.pawns[pawn_index].position;
    pawn_position.y -= 60.0;

    game.world.resources[resource_index].position = pawn_position;

    action.state = ActionState::Running;
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
