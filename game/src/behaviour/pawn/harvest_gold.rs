use crate::assets::AnimationBase;
use crate::behaviour::BehaviourState;
use crate::world::{WorldObject, WorldObjectType, StructureData};
use crate::DemoGameData;
use super::{PawnBehaviour,  PawnBehaviourType};

const MOVE_TO_MINE: u8 = 0;
const ENTER_MINE: u8 = 1;
const MINING: u8 = 2;
const DISABLE_MINE: u8 = 3;

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
    };

    if !mine_data.can_be_mined() {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id);

    if game.world.pawns_data[pawn_index].grabbed_resource().is_some() {
        super::drop_resource(game, pawn_index);
    }

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::HarvestGold { structure_id: mine_structure.id },
        state: BehaviourState::Initial,
    };
}

pub fn cancel(game: &mut DemoGameData, pawn_index: usize) {
    let behaviour = game.world.pawns_behaviour[pawn_index];

    match behaviour.state {
        BehaviourState::Running(MINING) => {
            let world = &mut game.world;
            let structure_index = params(behaviour.ty);
            let mine = &mut world.structures[structure_index];
            let mine_data = world.structures_data[structure_index].gold_mine_mut();

            for id in &mut mine_data.miners_ids {
                if *id as usize == pawn_index {
                    *id = u32::MAX;
                }
            }

            if mine_data.miners_count > 0 {
                mine_data.miners_count -= 1;
                mine_data.miners_ids.sort_unstable();
            }

            if mine_data.miners_count == 0 {
                mine.sprite = game.assets.structures.gold_mine_inactive.aabb;
            }
        },
        BehaviourState::Running(DISABLE_MINE) => {
            disable_mine(game, pawn_index);
        },
        _ => {}
    }

    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
    game.world.pawns_behaviour[pawn_index] = PawnBehaviour::idle();
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let state = game.world.pawns_behaviour[pawn_index].state;

    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVE_TO_MINE) => move_to_mine(game, pawn_index),
        BehaviourState::Running(ENTER_MINE) => enter_mine(game, pawn_index),
        BehaviourState::Running(MINING) => mining(game, pawn_index),
        BehaviourState::Running(DISABLE_MINE) => disable_mine(game, pawn_index),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;
    game.world.pawns_behaviour[pawn_index].state = BehaviourState::Running(MOVE_TO_MINE);
    
    move_to_mine(game, pawn_index);
}

fn move_to_mine(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let structure_index = params(behaviour.ty);
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    if !mine_data.can_be_mined() {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let mut target_position = mine.position;
    target_position.y += 10.0;

    let updated_position = move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        behaviour.state = BehaviourState::Running(ENTER_MINE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn enter_mine(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let structure_index = params(behaviour.ty);
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    if !mine_data.can_be_mined() {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let index = mine_data.miners_count as usize;
    mine_data.miners_ids[index] = pawn_index as u32;
    mine_data.miners_count += 1;

    if mine_data.miners_count == 1 {
        mine_data.last_drop_timestamp = game.global.time;
    }

    mine.sprite = game.assets.structures.gold_mine.aabb;

    // Hide the pawn
    pawn.animation = AnimationBase::default();

    behaviour.state = BehaviourState::Running(MINING);
}

fn mining(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];
    
    let structure_index = params(behaviour.ty);
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    if !mine_data.can_be_mined() {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let timer = 5000.0 - (300.0 * mine_data.miners_count as f64);
    let mut spawn_gold = false;

    if elapsed(game.global.time, mine_data.last_drop_timestamp, timer) {
        if mine_data.remaining_gold > 0 {
            mine_data.remaining_gold -= 1;
            spawn_gold = true;
        }

        mine_data.last_drop_timestamp = game.global.time;
    }
    
    if mine_data.remaining_gold == 0 {
        behaviour.state = BehaviourState::Running(DISABLE_MINE);
    }

    // This needs to be at the end because `spawn_resource::spawn_gold` borrows the game mutably
    if spawn_gold {
        let mut position = mine.position;
        position.y += fastrand::u32(30..60) as f32;
        position.x += fastrand::i32(-110..110) as f32;
        crate::behaviour::spawn_resources::spawn_gold(game, position);
    }
}

fn disable_mine(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let structure_index = params(behaviour.ty);
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    // If mine was already disabled
    if mine_data.remaining_gold == 0 && mine_data.miners_count == 0 {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    // Respawn pawns
    let mut position = mine.position;
    position.y += 10.0;
    position.x -= 70.0;

    for i in 0..(mine_data.miners_count as usize) {
        let pawn_index = mine_data.miners_ids[i] as usize;
        world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
        world.pawns[pawn_index].position = position;
        position.x += 70.0;
    }

    // Disable mine
    mine.sprite = game.assets.structures.gold_mine_destroyed.aabb;
    mine_data.miners_count = 0;
    mine_data.miners_ids = [0; 3];

    *behaviour = PawnBehaviour::idle();
}


#[inline(always)]
fn params(value: PawnBehaviourType) -> usize {
    match value {
        PawnBehaviourType::HarvestGold { structure_id } => structure_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
