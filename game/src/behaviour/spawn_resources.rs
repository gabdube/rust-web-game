use crate::shared::Position;
use crate::world::{ResourceType, ResourceData};
use crate::DemoGameData;

#[repr(align(4))]
#[derive(Copy, Clone)]
pub struct SpawnResourceBehaviour {
    pub resource_type: ResourceType,
    pub init: bool,
}

impl SpawnResourceBehaviour {
    pub fn spawn(resource_type: ResourceType) -> Self {
        SpawnResourceBehaviour {
            resource_type,
            init: true,
        }
    }
}

pub fn spawn_wood(game: &mut DemoGameData, position: Position<f32>) {
    game.world.create_resource_spawn(position, ResourceType::Wood);
}

pub fn spawn_gold(game: &mut DemoGameData, position: Position<f32>) {
    game.world.create_resource_spawn(position, ResourceType::Gold);
}

pub fn spawn_food(game: &mut DemoGameData, position: Position<f32>) {
    game.world.create_resource_spawn(position, ResourceType::Food);
}

pub fn process(game: &mut DemoGameData, spawn_index: usize) {
    let behaviour = &mut game.world.resources_spawn_behaviour[spawn_index];
    if behaviour.init {
        let resource_spawn = &mut game.world.resources_spawn[spawn_index];
        resource_spawn.animation = match behaviour.resource_type {
            ResourceType::Food => game.assets.resources.meat_spawn,
            ResourceType::Gold => game.assets.resources.gold_spawn,
            ResourceType::Wood => game.assets.resources.wood_spawn,
        };
        resource_spawn.current_frame = 0;
        behaviour.init = false;
        return;
    }

    let resource_spawn = game.world.resources_spawn[spawn_index];
    if resource_spawn.current_frame != resource_spawn.animation.last_frame {
        return;
    }

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
