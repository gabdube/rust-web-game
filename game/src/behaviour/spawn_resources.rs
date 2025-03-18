use crate::shared::Position;
use crate::world::{ResourceType, ResourceData};
use crate::DemoGameData;

#[repr(align(4))]
#[derive(Copy, Clone)]
pub struct SpawnResourceBehaviour {
    pub resource_type: ResourceType,
}

impl SpawnResourceBehaviour {
    pub fn spawn(resource_type: ResourceType) -> Self {
        SpawnResourceBehaviour {
            resource_type,
        }
    }
}

pub fn spawn_wood(game: &mut DemoGameData, position: Position<f32>) {
    let spawn = game.assets.resources.wood_spawn;
    game.world.create_resource_spawn(position, &spawn, ResourceType::Wood);
}

pub fn spawn_gold(game: &mut DemoGameData, position: Position<f32>) {
    let spawn = game.assets.resources.gold_spawn;
    game.world.create_resource_spawn(position, &spawn, ResourceType::Gold);
}

pub fn spawn_food(game: &mut DemoGameData, position: Position<f32>) {
    let spawn = game.assets.resources.meat_spawn;
    game.world.create_resource_spawn(position, &spawn, ResourceType::Food);
}

pub fn process(game: &mut DemoGameData, spawn_index: usize) {
    let resource_spawn = game.world.resources_spawn[spawn_index];
    if resource_spawn.current_frame != resource_spawn.animation.last_frame {
        return;
    }

    let behaviour = game.world.resources_spawn_behaviour[spawn_index];
    let data = ResourceData {
        resource_type: behaviour.resource_type,
        grabbed: false,
    };

    match behaviour.resource_type {
        ResourceType::Wood  => {
            game.world.create_resource(resource_spawn.position, game.assets.resources.wood, data);
        },
        ResourceType::Gold  => {
            game.world.create_resource(resource_spawn.position, game.assets.resources.gold, data);
        },
        ResourceType::Food  => {
            game.world.create_resource(resource_spawn.position, game.assets.resources.meat, data);
        },
    }

    game.world.resources_spawn[spawn_index].delete();
}
