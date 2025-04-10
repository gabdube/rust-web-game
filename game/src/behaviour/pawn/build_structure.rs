use crate::behaviour::BehaviourState;
use crate::shared::pos;
use crate::world::{BaseAnimated, BaseStatic, StructureData, WorldObject, WorldObjectType};
use crate::world::{MAX_CASTLE_HP, MAX_TOWER_HP, MAX_HOUSE_HP};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_STRUCTURE: u8 = 0;
const BUILD_STRUCTURE: u8 = 1;
const FINALIZE_STRUCTURE: u8 = 2;

struct PawnBuildStructureParams {
    pawn: BaseAnimated,
    structure: BaseStatic,
    structure_data: StructureData,
    last_timestamp: f32,
    structure_id: u32,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

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
        StructureData::GoldMine(_) | StructureData::GoblinHut(_) => false,
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
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_STRUCTURE) => move_to_structure(game, &mut params),
        BehaviourState::Running(BUILD_STRUCTURE) => build_structure(game, &mut params),
        BehaviourState::Running(FINALIZE_STRUCTURE) => finalize_structure(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &params);
}

fn init(game: &DemoGameData, params: &mut PawnBuildStructureParams) {
    params.pawn.animation = game.assets.animations.pawn.walk;
    params.state = BehaviourState::Running(MOVE_TO_STRUCTURE);
    move_to_structure(game, params);
}

fn move_to_structure(game: &DemoGameData, params: &mut PawnBuildStructureParams) {
    use crate::behaviour::behaviour_shared::move_to;

    if early_exit(params.structure_data) {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    // Find the nearest point to the structure base
    let aabb = params.structure.aabb();
    let target_position = pos(f32::max(f32::min(params.pawn.position.x, aabb.right), aabb.left), aabb.bottom + 5.0);
    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.pawn.animation = game.assets.animations.pawn.hammer;
        params.pawn.current_frame = 0;
        params.state = BehaviourState::Running(BUILD_STRUCTURE);
    }

    params.pawn.position = updated_position;
    params.pawn.flipped = params.pawn.position.x > params.structure.position.x;
}

fn build_structure(game: &DemoGameData, params: &mut PawnBuildStructureParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    if early_exit(params.structure_data) {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    // Every 2 hammer strike adds 5 hp to the structure
    let total_animation_time = crate::ANIMATION_INTERVAL * 6.0 * 2.0;
    if !elapsed(game.global.time, params.last_timestamp as f64, total_animation_time) {
        return;
    }

    let finalize = match &mut params.structure_data {
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
        StructureData::GoldMine(_) | StructureData::GoblinHut(_) => unsafe { ::std::hint::unreachable_unchecked() }
    };

    if finalize {
        params.state = BehaviourState::Running(FINALIZE_STRUCTURE);
    }

    params.last_timestamp = game.global.time as f32;
}

fn finalize_structure(game: &DemoGameData, params: &mut PawnBuildStructureParams) {
    match &mut params.structure_data {
        StructureData::Castle(data) => {
            params.structure.sprite = game.assets.structures.knights_castle;
            data.building = false;
        }
        StructureData::Tower(data) => {
            params.structure.sprite = game.assets.structures.knights_tower;
            data.building = false;
        }
        StructureData::House(data) => {
            params.structure.sprite = game.assets.structures.knights_house;
            data.building = false;
        }
        StructureData::GoldMine(..) | StructureData::GoblinHut(_) => {}
    }

    params.new_behaviour = Some(PawnBehaviour::idle());
}

fn early_exit(data: StructureData) -> bool {
    match data {
        StructureData::GoldMine(..) | StructureData::GoblinHut(_) => true,
        StructureData::Castle(data) => data.hp == MAX_CASTLE_HP,
        StructureData::Tower(data) => data.hp == MAX_TOWER_HP,
        StructureData::House(data) => data.hp == MAX_HOUSE_HP,
    }
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnBuildStructureParams {
    let pawn = unsafe { game.world.pawns.get_unchecked(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked(pawn_index) };

    let (structure_index, last_timestamp) = match behaviour.ty {
        PawnBehaviourType::BuildStructure { structure_id, last_timestamp } => (structure_id as usize, last_timestamp),
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    };

    let structure = unsafe { game.world.structures.get_unchecked(structure_index) };
    let structure_data = unsafe { game.world.structures_data.get_unchecked(structure_index) };

    PawnBuildStructureParams {
        pawn: *pawn,
        structure: *structure,
        structure_data: *structure_data,
        last_timestamp,
        structure_id: structure_index as u32,
        new_behaviour: None,
        state: behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnBuildStructureParams) {
    let structure_index = params.structure_id as usize;

    let pawn = unsafe { game.world.pawns.get_unchecked_mut(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked_mut(pawn_index) };
    let structure = unsafe { game.world.structures.get_unchecked_mut(structure_index) };
    let structure_data = unsafe { game.world.structures_data.get_unchecked_mut(structure_index) };

    *pawn = params.pawn;
    *structure = params.structure;
    *structure_data = params.structure_data;

    match params.new_behaviour {
        Some(new_behaviour) => {
            *behaviour = new_behaviour;
        },
        None => {
            behaviour.ty = PawnBehaviourType::BuildStructure { 
                structure_id: params.structure_id,
                last_timestamp: params.last_timestamp
            };

            behaviour.state = params.state;
        }
    }
}
