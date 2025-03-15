use crate::data::actions::{Action, ActionType, ActionState};
use crate::shared::Position;
use crate::world::{ResourceData, ResourceType};
use crate::DemoGameData;


pub fn spawn_wood(game: &mut DemoGameData, position: Position<f32>) {
    let wood_spawn = game.assets.resources.wood_spawn;
    let spawn_id = game.world.create_resource_spawn(position, &wood_spawn);
    let spawn_action = Action::from_type(ActionType::SpawnResource { spawn_id: spawn_id as u32, resource_type: ResourceType::Wood });
    game.actions.push(spawn_action);
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    game.world.resources_spawn[spawn_index(action)].delete();
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Finalized;
        return;
    }

    match action.state {
        ActionState::Initial => run(game, action),
        ActionState::Running => run(game, action),
        ActionState::Finalizing => done(game, action),
        ActionState::Finalized => {}
    }
}

fn run(game: &mut DemoGameData, action: &mut Action) {
    let spawn = game.world.resources_spawn[spawn_index(action)];
    if spawn.current_frame == spawn.animation.last_frame {
        action.state = ActionState::Finalizing
    } else {
        action.state = ActionState::Running
    }
}

fn done(game: &mut DemoGameData, action: &mut Action) {
    let (spawn_index, resource_type) = match action.ty {
        ActionType::SpawnResource { spawn_id, resource_type } => (spawn_id as usize, resource_type),
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    };

    let data = ResourceData {
        resource_type,
        grabbed: false,
    };

    let spawn_position = game.world.resources_spawn[spawn_index].position;
    game.world.resources_spawn[spawn_index].delete();

    match resource_type {
        ResourceType::Wood  => {
            game.world.create_resource(spawn_position, game.assets.resources.wood, data);
        },
        _ => {}
    }

    action.state = ActionState::Finalized;
}

fn validate(game: &mut DemoGameData, action: &mut Action) -> bool {
    game.world.resources_spawn.len() > spawn_index(action)
}

#[inline]
fn spawn_index(action: &mut Action) -> usize {
    match action.ty {
        ActionType::SpawnResource { spawn_id, .. } => spawn_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
