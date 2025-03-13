use crate::actions::{Action, ActionState};
use crate::assets::{Assets, AnimationBase};
use crate::shared::pos;
use crate::world::{World, BaseAnimated, TreeData};
use crate::DemoGameData;

// For initial & Running
struct CutTreeParams1<'a> {
    assets: &'a Assets,
    pawn: &'a mut BaseAnimated,
    tree: &'a mut BaseAnimated,
    tree_data: &'a mut TreeData,
    time: f64,
}

impl<'a> CutTreeParams1<'a> {
    pub fn new(data: &'a mut DemoGameData, pawn_index: usize, tree_index: usize) -> Self {
        CutTreeParams1 {
            assets: &data.assets,
            pawn: &mut data.world.pawns[pawn_index],
            tree: &mut data.world.trees[tree_index],
            tree_data: &mut data.world.trees_data[tree_index],
            time: data.timing.time,
        }
    }
}

/// For Finalizing
struct CutTreeParams2<'a> {
    assets: &'a Assets,
    world: &'a mut World,
    pawn_index: usize,
    tree_index: usize,
    time: f64,
}

impl<'a> CutTreeParams2<'a> {
    pub fn new(data: &'a mut DemoGameData, pawn_index: usize, tree_index: usize) -> Self {
        CutTreeParams2 {
            assets: &data.assets,
            world: &mut data.world,
            pawn_index,
            tree_index,
            time: data.timing.time,
        }
    }
}

pub fn cut_tree(data: &mut DemoGameData, action: &mut Action, pawn_id: u32, tree_id: u32) {
    let pawn_index = pawn_id as usize;
    let tree_index = tree_id as usize;
    if pawn_index >= data.world.pawns.len() || tree_index >= data.world.trees.len()  {
        action.state = ActionState::Finalized;
    }

    match action.state {
        ActionState::Initial => cut_tree_initial(action, CutTreeParams1::new(data, pawn_index, tree_index)),
        ActionState::Running => cut_tree_running(action, CutTreeParams1::new(data, pawn_index, tree_index)),
        ActionState::Finalizing => cut_tree_finalize(action, CutTreeParams2::new(data, pawn_index, tree_index)),
        ActionState::Finalized => {},
    }
}

pub fn cancel(data: &mut DemoGameData, pawn_id: u32, tree_id: u32) {
    let pawn_index = pawn_id as usize;
    if pawn_index < data.world.pawns.len() {
        data.world.pawns[pawn_index].animation = data.assets.animations.pawn.idle;
    }
   
    let tree_index = tree_id as usize;
    if tree_index < data.world.trees.len() {
        data.world.trees[tree_index].animation = data.assets.resources.tree_idle;
        data.world.trees_data[tree_index].being_harvested = false;
    }
}

fn cut_tree_initial(action: &mut Action, params: CutTreeParams1) {
    // A pawn is already harvesting the tree
    if params.tree_data.being_harvested || params.tree_data.life == 0 {
        action.state = ActionState::Finalizing;
        return;
    }

    params.pawn.animation = params.assets.animations.pawn.axe;
    params.pawn.flipped = params.pawn.position.x > params.tree.position.x;

    params.tree.animation = params.assets.resources.tree_cut;
    params.tree.current_frame = 0;
    params.tree_data.being_harvested = true;
    params.tree_data.last_drop_timestamp = params.time;
  
    action.state = ActionState::Running;
}

fn cut_tree_running(action: &mut Action, params: CutTreeParams1) {
    if params.tree_data.life > 0 && params.time - params.tree_data.last_drop_timestamp > 300.0 {
        params.tree_data.life -= 1;
        params.tree_data.last_drop_timestamp = params.time;
    }

    if params.tree_data.life == 0 {
        action.state = ActionState::Finalizing;
    }
}

fn cut_tree_finalize(action: &mut Action, params: CutTreeParams2) {
    let pawn = &mut params.world.pawns[params.pawn_index];
    let tree = &mut params.world.trees[params.tree_index];
    let tree_data = &mut params.world.trees_data[params.tree_index];
    pawn.animation = params.assets.animations.pawn.idle;
    tree.animation = AnimationBase::from_aabb(params.assets.resources.tree_stump.aabb);
    tree_data.being_harvested = false;

    // Spawns three wood resource around the tree
    let center_pos = tree.position;
    for _ in 0..3 {
        let x = fastrand::i8(-10..10);
        let y = fastrand::i8(-10..10);
        let pos = pos(center_pos.x + x as f32, center_pos.y + y as f32);
    }

    action.state = ActionState::Finalized;
}

