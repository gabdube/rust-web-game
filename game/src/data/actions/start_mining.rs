use crate::assets::AnimationBase;
use crate::data::actions::{Action, ActionType, ActionState};
use crate::world::{StructureData, WorldObject, WorldObjectType};
use crate::DemoGameData;

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

    game.actions.push(Action::from_type(ActionType::StartMining { pawn_id: pawn.id, structure_id: mine_structure.id }));
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    if action.running_value() == MINING {
        let [pawn_index, structure_index] = params(action);
        let world = &mut game.world;
        let mine = &mut world.structures[structure_index];
        let mine_data = world.structures_data[structure_index].gold_mine_mut();

        for id in &mut mine_data.miners_ids {
            if *id as usize == pawn_index {
                *id = u32::MAX;
            }
        }

        if mine_data.miners_count > 0 {
            mine_data.miners_count -= 1;
            mine_data.miners_ids.sort();
        }

        if mine_data.miners_count == 0 {
            mine.sprite = game.assets.structures.gold_mine_inactive.aabb;
        }

    } else if action.running_value() == DISABLE_MINE {
        disable_mine(game, action);
    }
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Done;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running(MOVE_TO_MINE) => move_to_mine(game, action),
        ActionState::Running(ENTER_MINE) => enter_mine(game, action),
        ActionState::Running(MINING) => mining(game, action),
        ActionState::Running(DISABLE_MINE) => disable_mine(game, action),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, _] = params(action);

    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;

    action.state = ActionState::Running(MOVE_TO_MINE);
    
    move_to_mine(game, action);
}

fn move_to_mine(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, structure_index] = params(action);

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    if !mine_data.can_be_mined() {
        pawn.animation = game.assets.animations.pawn.idle;
        action.state = ActionState::Done;
        return;
    }

    let mut target_position = mine.position;
    target_position.y += 10.0;

    let updated_position = super::actions_shared::move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        action.state = ActionState::Running(ENTER_MINE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn enter_mine(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, structure_index] = params(action);

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let pawn_data = &mut world.pawns_data[pawn_index];
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    let index = mine_data.miners_count as usize;
    mine_data.miners_ids[index] = pawn_index as u32;
    mine_data.miners_count += 1;

    if mine_data.miners_count == 1 {
        mine_data.last_drop_timestamp = game.global.time;
    }

    mine.sprite = game.assets.structures.gold_mine.aabb;

    pawn_data.assigned_mine = structure_index as u32;
    pawn.animation = AnimationBase::default();

    action.state = ActionState::Running(MINING);
}

fn mining(game: &mut DemoGameData, action: &mut Action) {
    let [_, structure_index] = params(action);

    let world = &mut game.world;
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();
    let timer = 5000.0 - (300.0 * mine_data.miners_count as f64);
    let mut spawn_gold = false;
    
    if game.global.time - mine_data.last_drop_timestamp > timer {
        if mine_data.remaining_gold > 0 {
            mine_data.remaining_gold -= 1;
            spawn_gold = true;
        }

        mine_data.last_drop_timestamp = game.global.time;
    }

    if mine_data.remaining_gold == 0 {
        if mine_data.miners_count > 0 {
            action.state = ActionState::Running(DISABLE_MINE);
        } else {
            action.state = ActionState::Done;
        }
    }

    // This needs to be at the end because `spawn_gold` borrows the game mutably
    if spawn_gold {
        let mut position = mine.position;
        position.y += fastrand::u32(10..50) as f32;
        position.x += fastrand::i32(-150..150) as f32;
        crate::data::actions::spawn_resource::spawn_gold(game, position);
    }
}

fn disable_mine(game: &mut DemoGameData, action: &mut Action) {
    let [_, structure_index] = params(action);

    let world = &mut game.world;
    let mine = &mut world.structures[structure_index];
    let mine_data = world.structures_data[structure_index].gold_mine_mut();

    // Respawn pawns
    let mut position = mine.position;
    position.y += 10.0;
    position.x -= 70.0;

    for i in 0..(mine_data.miners_count as usize) {
        let pawn_index = mine_data.miners_ids[i] as usize;
        world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
        world.pawns[pawn_index].position = position;
        world.pawns_data[pawn_index].assigned_mine = u32::MAX;
        position.x += 70.0;
    }

    // Disable mine
    mine.sprite = game.assets.structures.gold_mine_destroyed.aabb;
    mine_data.miners_count = 0;
    mine_data.miners_ids = [0; 3];

    action.state = ActionState::Done;
}

fn validate(game: &mut DemoGameData, action: &mut Action) -> bool {
    let [pawn_index, structure_index] = params(action);
    game.world.pawns.len() > pawn_index && game.world.structures.len() > structure_index
}

#[inline]
fn params(action: &mut Action) -> [usize; 2] {
    match action.ty {
        ActionType::StartMining { pawn_id, structure_id } => [pawn_id as usize, structure_id as usize],
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

