use crate::behaviour::BehaviourState;
use crate::shared::pos;
use crate::world::{StructureData, WorldObject, WorldObjectType};
use crate::world::{MAX_CASTLE_HP, MAX_TOWER_HP, MAX_HOUSE_HP};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_STRUCTURE: u8 = 0;
const BUILD_STRUCTURE: u8 = 1;
const FINALIZE_STRUCTURE: u8 = 2;

pub fn new(game: &mut DemoGameData, pawn: WorldObject, structure: WorldObject) {
    match (pawn.ty, structure.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Structure) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let structure_index = structure.id as usize;
    if pawn_index >= game.world.pawns.len() || structure_index >= game.world.structures.len() {
        return;
    }

    let ok = match game.world.structures_data[structure_index] {
        StructureData::GoldMine(_) => false,
        StructureData::Castle(data) => data.hp < MAX_CASTLE_HP,
        StructureData::Tower(data) => data.hp < MAX_TOWER_HP,
        StructureData::House(data) => data.hp < MAX_HOUSE_HP,
    };

    if !ok {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::BuildStructure { structure_id: structure.id, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let state = game.world.pawns_behaviour[pawn_index].state;

    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVE_TO_STRUCTURE) => move_to_structure(game, pawn_index),
        BehaviourState::Running(BUILD_STRUCTURE) => build_structure(game, pawn_index),
        BehaviourState::Running(FINALIZE_STRUCTURE) => finalize_structure(game, pawn_index),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;
    game.world.pawns_behaviour[pawn_index].state = BehaviourState::Running(MOVE_TO_STRUCTURE);
    move_to_structure(game, pawn_index);
}

fn move_to_structure(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let structure_index = params(behaviour.ty);
    let structure = &mut world.structures[structure_index];

    if early_exit(world.structures_data[structure_index]) {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    // Find the nearest point to the structure base
    let aabb = structure.aabb();
    let target_position = pos(f32::max(f32::min(pawn.position.x, aabb.right), aabb.left), aabb.bottom + 5.0);
    let updated_position = move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        pawn.animation = game.assets.animations.pawn.hammer;
        pawn.current_frame = 0;
        behaviour.state = BehaviourState::Running(BUILD_STRUCTURE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > structure.position.x;
}

fn build_structure(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];
    let structure_index = params(behaviour.ty);
    let structure_data = &mut world.structures_data[structure_index];

    if early_exit(*structure_data) {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    // Every 2 hammer strike adds 5 hp to the structure
    let total_animation_time = crate::ANIMATION_INTERVAL * 6.0 * 2.0;
    let timestamp = params_last_timestamp(behaviour.ty);
    if !elapsed(game.global.time, timestamp, total_animation_time) {
        return;
    }

    params_set_last_timestamp(&mut behaviour.ty, game.global.time);

    let finalize = match structure_data {
        StructureData::Castle(data) => {
            data.hp = u8::min(data.hp + 5, MAX_CASTLE_HP);
            data.hp == MAX_CASTLE_HP
        },
        StructureData::Tower(data) => {
            data.hp = u8::min(data.hp + 5, MAX_TOWER_HP);
            data.hp == MAX_TOWER_HP
        },
        StructureData::House(data) => {
            data.hp = u8::min(data.hp + 5, MAX_HOUSE_HP);
            data.hp == MAX_HOUSE_HP
        },
        StructureData::GoldMine(_) => unsafe { ::std::hint::unreachable_unchecked() }
    };

    if finalize {
        behaviour.state = BehaviourState::Running(FINALIZE_STRUCTURE);
    }
}

fn finalize_structure(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];
    let structure_index = params(behaviour.ty);
    let structure = &mut world.structures[structure_index];
    let structure_data = &mut world.structures_data[structure_index];

    match structure_data {
        StructureData::Castle(data) => {
            if !data.destroyed {
                structure.sprite = game.assets.structures.knights_castle.aabb;
                data.building = false;
            }
        },
        StructureData::Tower(data) => {
            if !data.destroyed {
                structure.sprite = game.assets.structures.knights_tower.aabb;
                data.building = false;
            }
        },
        StructureData::House(data) => {
            if !data.destroyed {
                structure.sprite = game.assets.structures.knights_house.aabb;
                data.building = false;
            }
        },
        StructureData::GoldMine(..) => {},
    }

    *behaviour = PawnBehaviour::idle();
}

fn early_exit(data: StructureData) -> bool {
    match data {
        StructureData::GoldMine(..) => true,
        StructureData::Castle(data) => data.destroyed || data.hp == MAX_CASTLE_HP,
        StructureData::Tower(data) => data.destroyed || data.hp == MAX_TOWER_HP,
        StructureData::House(data) => data.destroyed || data.hp == MAX_HOUSE_HP,
    }
}

#[inline(always)]
fn params(value: PawnBehaviourType) -> usize {
    match value {
        PawnBehaviourType::BuildStructure { structure_id, .. } => structure_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}

#[inline(always)]
fn params_last_timestamp(value: PawnBehaviourType) -> f64 {
    match value {
        PawnBehaviourType::BuildStructure { last_timestamp, .. } => last_timestamp as f64,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}

#[inline(always)]
fn params_set_last_timestamp(value: &mut PawnBehaviourType, time: f64) {
    match value {
        PawnBehaviourType::BuildStructure { last_timestamp, .. } => *last_timestamp = time as f32,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}
