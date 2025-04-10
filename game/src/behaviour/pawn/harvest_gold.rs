use crate::assets::AnimationBase;
use crate::behaviour::BehaviourState;
use crate::shared::pos;
use crate::world::{BaseAnimated, BaseStatic, StructureData, StructureGoldMineData, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_MINE: u8 = 0;
const ENTER_MINE: u8 = 1;
const MINING: u8 = 2;
const DISABLE_MINE: u8 = 3;

struct PawnHarvestGoldParams {
    pawn: BaseAnimated,
    structure: BaseStatic,
    structure_data: StructureGoldMineData,
    last_timestamp: f32,
    pawn_id: u32,
    structure_id: u32,
    respawn_pawn: bool,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, pawn: WorldObject, mine_structure: WorldObject) {
    match (pawn.ty, mine_structure.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Structure) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let structure_index = mine_structure.id as usize;
    if pawn_index >= game.world.pawns.len() || structure_index >= game.world.structures.len() {
        return;
    }

    let mine_data = match game.world.structures_data[structure_index] {
        StructureData::GoldMine(mine_data) => mine_data,
        _ => { return; }
    };

    if !mine_data.can_be_mined() {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::HarvestGold { structure_id: mine_structure.id, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn cancel(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Running(MINING) => {
            for id in &mut params.structure_data.miners_ids {
                if *id as usize == pawn_index {
                    *id = u32::MAX;
                }
            }

            if params.structure_data.miners_count > 0 {
                params.structure_data.miners_count -= 1;
                params.structure_data.miners_ids.sort_unstable();
            }

            if params.structure_data.miners_count == 0 {
                params.structure.sprite = game.assets.structures.gold_mine_inactive;
            }
        },
        BehaviourState::Running(DISABLE_MINE) => {
            disable_mine(game, &mut params);
        },
        _ => {}
    }

    write_params(game, pawn_index, &mut params);
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_MINE) => move_to_mine(game, &mut params),
        BehaviourState::Running(ENTER_MINE) => enter_mine(game, &mut params),
        BehaviourState::Running(MINING) => mining(game, &mut params),
        BehaviourState::Running(DISABLE_MINE) => disable_mine(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &mut params);
}

fn init(game: &DemoGameData, params: &mut PawnHarvestGoldParams) {
    params.pawn.animation = game.assets.animations.pawn.walk;
    params.state = BehaviourState::Running(MOVE_TO_MINE);
    move_to_mine(game, params);
}

fn move_to_mine(game: &DemoGameData, params: &mut PawnHarvestGoldParams) {
    use crate::behaviour::behaviour_shared::move_to;
    if !params.structure_data.can_be_mined() {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let target_position = params.structure.position + pos(0.0, 10.0);
    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.state = BehaviourState::Running(ENTER_MINE);
    } else {
        params.pawn.flipped = params.pawn.position.x > target_position.x;
    }

    params.pawn.position = updated_position;
}

fn enter_mine(game: &DemoGameData, params: &mut PawnHarvestGoldParams) {
    if !params.structure_data.can_be_mined() {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let index = params.structure_data.miners_count as usize;
    params.structure_data.miners_ids[index] = params.pawn_id;
    params.structure_data.miners_count += 1;
    params.structure.sprite = game.assets.structures.gold_mine;
    params.last_timestamp = game.global.time as f32;

    // Hide the pawn
    params.pawn.animation = AnimationBase::default();
    params.state = BehaviourState::Running(MINING);
}

fn mining(game: &mut DemoGameData, params: &mut PawnHarvestGoldParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    if params.structure_data.remaining_gold == 0 {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let total_animation_time = 5000.0;  // One gold mined every 5 sec
    let mut spawn_gold = false;

    if elapsed(game.global.time, params.last_timestamp as f64, total_animation_time) {
        params.last_timestamp = game.global.time as f32;
        params.structure_data.remaining_gold -= 1;
        spawn_gold = true;
    }

    if params.structure_data.remaining_gold == 0 {
        params.state = BehaviourState::Running(DISABLE_MINE);
    }

    // This needs to be at the end because `spawn_resource::spawn_gold` borrows the game mutably
    if spawn_gold {
        let mut position = params.structure.position;
        position.y += fastrand::u32(30..60) as f32;
        position.x += fastrand::i32(-110..110) as f32;
        crate::behaviour::spawn_resources::spawn_gold(game, position);
    }
}

fn disable_mine(game: &mut DemoGameData, params: &mut PawnHarvestGoldParams) {
    // If mine was already disabled
    if params.structure_data.remaining_gold == 0 && params.structure_data.miners_count == 0 {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    // Disable mine
    params.structure.sprite = game.assets.structures.gold_mine_destroyed;
    params.structure_data.miners_count = 0;
    params.structure_data.miners_ids = [0; 3];
    params.respawn_pawn = true;
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnHarvestGoldParams {
    let pawn = unsafe { game.world.pawns.get_unchecked(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked(pawn_index) };

    let (structure_index, last_timestamp) = match behaviour.ty {
        PawnBehaviourType::HarvestGold { structure_id, last_timestamp } => (structure_id as usize, last_timestamp),
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    };

    let structure = unsafe { game.world.structures.get_unchecked(structure_index) };
    let structure_data = unsafe { game.world.structures_data.get_unchecked(structure_index).gold_mine() };

    PawnHarvestGoldParams {
        pawn: *pawn,
        structure: *structure,
        structure_data,
        last_timestamp,
        pawn_id: pawn_index as u32,
        structure_id: structure_index as u32,
        respawn_pawn: false,
        new_behaviour: None,
        state: behaviour.state
    }
}

fn respawn_pawns(game: &mut DemoGameData, params: &mut PawnHarvestGoldParams) {
    // Respawn pawns
    let mut position = params.structure.position;
    position.y += 10.0;
    position.x -= 70.0;

    for i in 0..(params.structure_data.miners_count as usize) {
        let pawn_index = params.structure_data.miners_ids[i] as usize;
        if pawn_index == params.pawn_id as usize {
            params.pawn.animation = game.assets.animations.pawn.idle;
            params.pawn.position = position;
            params.new_behaviour = Some(PawnBehaviour::idle());
        } else {
            let world = &mut game.world;
            world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
            world.pawns[pawn_index].position = position;
            world.pawns_behaviour[pawn_index] = PawnBehaviour::idle();
        }
        position.x += 70.0;
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &mut PawnHarvestGoldParams) {
    if params.respawn_pawn {
        respawn_pawns(game, params);
    }

    let pawn = unsafe { game.world.pawns.get_unchecked_mut(pawn_index) };
    let behaviour = unsafe { game.world.pawns_behaviour.get_unchecked_mut(pawn_index) };
    let structure = unsafe { game.world.structures.get_unchecked_mut(params.structure_id as usize) };
    let structure_data = unsafe { game.world.structures_data.get_unchecked_mut(params.structure_id as usize) };

    *pawn = params.pawn;
    *structure = params.structure;
    *structure_data = StructureData::GoldMine(params.structure_data);

    match params.new_behaviour {
        Some(new_behaviour) => {
            *behaviour = new_behaviour;
        },
        None => {
            behaviour.ty = PawnBehaviourType::HarvestGold { 
                structure_id: params.structure_id,
                last_timestamp: params.last_timestamp
            };

            behaviour.state = params.state;
        }
    }
}
