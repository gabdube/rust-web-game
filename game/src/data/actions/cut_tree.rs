use crate::assets::AnimationBase;
use crate::data::actions::{Action, ActionType, ActionState};
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;

const MOVE_TO_TREE: u8 = 0;
const BEGIN_CUT_TREE: u8 = 1;
const CUT_TREE: u8 = 2;
const SPAWN_WOOD: u8 = 3;

pub fn new(game: &mut DemoGameData, pawn: WorldObject, tree: WorldObject) {
    match (pawn.ty, tree.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Tree) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let tree_index = tree.id as usize;
    if tree_index >= game.world.trees.len() || pawn_index >= game.world.pawns.len() {
        return;
    }

    let tree_data = game.world.trees_data[tree_index];
    if tree_data.life == 0 || tree_data.being_harvested  {
        return;
    }

    game.actions.push(Action::from_type(ActionType::CutTree { pawn_id: pawn.id, tree_id: tree.id }));
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    let [pawn_index, tree_index] = params(action);
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
    game.world.trees[tree_index].animation = game.assets.resources.tree_idle;
    game.world.trees_data[tree_index].being_harvested = false;
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Done;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running(MOVE_TO_TREE) => move_to_tree(game, action),
        ActionState::Running(BEGIN_CUT_TREE) => begin_cut_tree(game, action),
        ActionState::Running(CUT_TREE) => cut_tree(game, action),
        ActionState::Running(SPAWN_WOOD) => spawn_wood(game, action),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, _] = params(action);

    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;

    action.state = ActionState::Running(MOVE_TO_TREE);
    
    move_to_tree(game, action);
}

fn move_to_tree(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, tree_index] = params(action);

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    if tree_data.being_harvested || tree_data.life == 0 {
        pawn.animation = game.assets.animations.pawn.idle;
        action.state = ActionState::Done;
        return;
    }

    let mut target_position = tree.position;

    if pawn.position.x > tree.position.x {
        target_position.x += 60.0;
    } else {
        target_position.x -= 60.0;
    }
    
    target_position.y += 10.0;

    let updated_position = super::actions_shared::move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        action.state = ActionState::Running(BEGIN_CUT_TREE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn begin_cut_tree(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, tree_index] = params(action);

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    pawn.animation = game.assets.animations.pawn.axe;
    pawn.flipped = pawn.position.x > tree.position.x;

    tree.animation = game.assets.resources.tree_cut;
    tree.current_frame = 0;

    tree_data.being_harvested = true;
    tree_data.last_drop_timestamp = game.global.time;

    action.state = ActionState::Running(CUT_TREE);
}

fn cut_tree(game: &mut DemoGameData, action: &mut Action) {
    let [_, tree_index] = params(action);
    let world = &mut game.world;
    let tree_data = &mut world.trees_data[tree_index];

    if tree_data.life > 0 && game.global.time - tree_data.last_drop_timestamp > 300.0 {
        tree_data.life -= 1;
        tree_data.last_drop_timestamp = game.global.time;
    }

    if tree_data.life == 0 {
        action.state = ActionState::Running(SPAWN_WOOD);
    }
}

fn spawn_wood(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, tree_index] = params(action);
    let world = &mut game.world;

    let pawn = &mut world.pawns[pawn_index];
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    pawn.animation = game.assets.animations.pawn.idle;
    tree.animation = AnimationBase::from_aabb(game.assets.resources.tree_stump.aabb);
    tree_data.being_harvested = false;

    // Spawns three wood resources around the tree
    let center_pos = tree.position;
    let mut position = center_pos;
    let mut angle = 0.0;
    for _ in 0..3 {
        angle += f32::to_radians(fastrand::u8(120..180) as f32);
        position.x = f32::ceil(center_pos.x + f32::cos(angle) * 64.0);
        position.y = f32::ceil(center_pos.y + f32::sin(angle) * 64.0);
        crate::data::actions::spawn_resource::spawn_wood(game, position);
    }

    action.state = ActionState::Done;
}

fn validate(game: &mut DemoGameData, action: &mut Action) -> bool {
    let [pawn_index, tree_index] = params(action);
    game.world.pawns.len() > pawn_index && game.world.trees.len() > tree_index
}

#[inline]
fn params(action: &mut Action) -> [usize; 2] {
    match action.ty {
        ActionType::CutTree { pawn_id, tree_id } => [pawn_id as usize, tree_id as usize],
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

