//! Behaviour are snippets of code that are evaluated each frame. Think "AI", but less fancy.
pub mod behaviour_shared;
pub mod pawn;
pub mod warrior;
pub mod archer;
pub mod sheep;
pub mod spawn_resources;

use crate::DemoGame;

#[derive(Copy, Clone)]
pub enum BehaviourState {
    Initial,
    Running(u8),
}

pub fn update(game: &mut DemoGame) {
    run_pawn_behaviour(game);
    run_warrior_behaviour(game);
    run_archers_behaviour(game);
    run_sheep_behaviour(game);

    if game.data.world.resources_spawn.len() > 0 {
        run_resource_spawn_behaviour(game);
    }
}

fn run_pawn_behaviour(game: &mut DemoGame) {
    use pawn::PawnBehaviourType;

    let data = &mut game.data;
    let mut index = 0;

    while index < data.world.pawns.len() {
        let behaviour_type = data.world.pawns_behaviour[index].ty;
        match behaviour_type {
            PawnBehaviourType::Idle { .. } => pawn::idle(data, index),
            PawnBehaviourType::MoveTo { .. } => pawn::pawn_move::process(data, index),
            PawnBehaviourType::HarvestWood { .. } => pawn::harvest_wood::process(data, index),
            PawnBehaviourType::HarvestGold { .. } => pawn::harvest_gold::process(data, index),
            PawnBehaviourType::GrabResource { .. } => pawn::grab_resource::process(data, index),
            PawnBehaviourType::HuntSheep { .. } => pawn::hunt_sheep::process(data, index),
        }
        index += 1;
    }

}

fn run_warrior_behaviour(game: &mut DemoGame) {
    use warrior::WarriorBehaviourType;

    let data = &mut game.data;
    let mut index = 0;

    while index < data.world.warriors.len() {
        let behaviour_type = data.world.warriors_behaviour[index].ty;
        match behaviour_type {
            WarriorBehaviourType::Idle { .. } => warrior::idle(data, index),
            WarriorBehaviourType::MoveTo { .. } => warrior::warrior_move::process(data, index),
        }
        index += 1;
    }
}

fn run_archers_behaviour(game: &mut DemoGame) {
    use archer::ArcherBehaviourType;

    let data = &mut game.data;
    let mut index = 0;

    while index < data.world.archers.len() {
        let behaviour_type = data.world.archers_behaviour[index].ty;
        match behaviour_type {
            ArcherBehaviourType::Idle { .. } => archer::idle(data, index),
            ArcherBehaviourType::MoveTo { .. } => archer::archer_move::process(data, index),
        }
        index += 1;
    }
}

fn run_sheep_behaviour(game: &mut DemoGame) {
    use sheep::SheepBehaviourType;

    let data = &mut game.data;
    let mut index = 0;

    while index < data.world.sheeps.len() {
        let behaviour_type = data.world.sheep_behaviour[index].ty;
        match behaviour_type {
            SheepBehaviourType::Dead => sheep::dead(data, index),
            SheepBehaviourType::Idle { .. } => sheep::idle::process(data, index),
            SheepBehaviourType::Escaping { .. } => sheep::escaping::process(data, index),
            SheepBehaviourType::Moving { .. } => sheep::sheep_move::process(data, index),
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

impl crate::store::SaveAndLoad for BehaviourState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        match self {
            BehaviourState::Initial => writer.write_u32(0),
            BehaviourState::Running(value) => writer.write_u32(1<<8 + (*value as u32)),
        }
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let value = reader.read_u32();
        match value {
            0 => BehaviourState::Initial,
            _ => BehaviourState::Running((value & !0xFF) as u8)
        }
    }
}
