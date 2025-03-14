use crate::assets::AnimationBase;
use crate::data::actions::{Action, ActionType, ActionState};
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;

pub fn new(game: &mut DemoGameData, pawn: WorldObject, tree: WorldObject) {
    match (pawn.ty, tree.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Tree) => {},
        _ => { return; }
    };

    let pawn_index = pawn.id as usize;
    let tree_index = tree.id as usize;
    if tree_index >= game.world.trees.len() || pawn_index >= game.world.pawns.len() {
        return;
    }

    let tree_data = game.world.trees_data[tree_index];
    if tree_data.life == 0 || tree_data.being_harvested  {
        return;
    }

    let mut target_position = game.world.trees[tree_index].position;
    let pawn_x = game.world.pawns[pawn_index as usize].position.x;

    target_position.y += 10.0;
    if target_position.x > pawn_x {
        target_position.x -= 60.0;
    } else {
        target_position.x += 60.0;
    }

    let move_action = Action::from_type(ActionType::MoveActor { actor: pawn, target_position });
    let cut_tree_action = Action::from_type(ActionType::CutTree { pawn_id: pawn.id, tree_id: tree.id });
    game.actions.push_queue(move_action, cut_tree_action);
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
        action.state = ActionState::Finalized;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running => run(game, action),
        ActionState::Finalizing => done(game, action),
        ActionState::Finalized => {}
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
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

    action.state = ActionState::Running;
}

fn run(game: &mut DemoGameData, action: &mut Action) {
    let [_, tree_index] = params(action);
    let world = &mut game.world;

    let tree_data = &mut world.trees_data[tree_index];

    if tree_data.life > 0 && game.global.time - tree_data.last_drop_timestamp > 300.0 {
        tree_data.life -= 1;
        tree_data.last_drop_timestamp = game.global.time;
    }

    if tree_data.life == 0 {
        action.state = ActionState::Finalizing;
    }
}

fn done(game: &mut DemoGameData, action: &mut Action) {
    let [pawn_index, tree_index] = params(action);
    let world = &mut game.world;

    let pawn = &mut world.pawns[pawn_index];
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    pawn.animation = game.assets.animations.pawn.idle;
    tree.animation = AnimationBase::from_aabb(game.assets.resources.tree_stump.aabb);
    tree_data.being_harvested = false;

    // Spawns three wood resource around the tree
    let center_pos = tree.position;
    let mut position = center_pos;
    let mut angle = 0.0;
    for _ in 0..3 {
        angle += f32::to_radians(fastrand::u8(120..180) as f32);
        position.x = f32::ceil(center_pos.x + f32::cos(angle) * 64.0);
        position.y = f32::ceil(center_pos.y + f32::sin(angle) * 64.0);
        crate::data::actions::spawn_resource::spawn_wood(game, position);
    }

    action.state = ActionState::Finalized;
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

