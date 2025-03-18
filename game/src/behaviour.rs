//! Behaviour are snippets of code that are evaluated each frame. Think "AI", but less fancy.
pub mod behaviour_shared;
pub mod pawn;
pub mod spawn_resources;

use crate::DemoGame;

#[derive(Copy, Clone)]
pub enum BehaviourState {
    Initial,
    Done,
    Running(u8),
}

pub fn update(game: &mut DemoGame) {
    run_pawn_behaviour(game);

    if game.data.world.resources_spawn.len() > 0 {
        run_resource_spawn_behaviour(game);
    }
}

fn run_pawn_behaviour(game: &mut DemoGame) {
    use pawn::PawnBehaviourType;

    let data = &mut game.data;
    let pawn_count = data.world.pawns.len();
    let mut index = 0;

    while index < pawn_count {
        let behaviour_type = data.world.pawns_behaviour[index].ty;
        match behaviour_type {
            PawnBehaviourType::Idle { .. } => pawn::idle(data, index),
            PawnBehaviourType::MoveTo { .. } => pawn::pawn_move::process(data, index),
            PawnBehaviourType::HarvestWood { .. } => pawn::harvest_wood::process(data, index),
            PawnBehaviourType::HarvestGold { .. } => pawn::harvest_gold::process(data, index),
            PawnBehaviourType::GrabResource { .. } => pawn::grab_resource::process(data, index),
        }
        index += 1;
    }

}

fn run_resource_spawn_behaviour(game: &mut DemoGame) {
    let data = &mut game.data;
    let spawn_count = data.world.resources_spawn.len();
    let mut index = 0;

    while index < spawn_count {
        spawn_resources::process(data, index);
        index += 1;
    }

    // Clear spawned resources
    let mut iter = data.world.resources_spawn.iter().map(|spawn| !spawn.deleted );
    data.world.resources_spawn_behaviour.retain(|_| iter.next().unwrap_or(false) );
    data.world.resources_spawn.retain(|spawn| !spawn.deleted );
}
