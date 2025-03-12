use crate::actions::{Action, ActionState};
use crate::assets::Assets;
use crate::world::{BaseAnimated, TreeData};
use crate::DemoGameData;

struct CutTreeParams<'a> {
    pub assets: &'a Assets,
    pub pawn: &'a mut BaseAnimated,
    pub tree: &'a mut BaseAnimated,
    pub tree_data: &'a mut TreeData,
    pub time: f64,
}

pub fn cut_tree(data: &mut DemoGameData, action: &mut Action, pawn_id: u32, tree_id: u32) {
    let pawn_index = pawn_id as usize;
    let tree_index = tree_id as usize;
    if pawn_index >= data.world.pawns.len() || tree_index >= data.world.trees.len()  {
        action.state = ActionState::Finalized;
    }

    let mut params = CutTreeParams {
        assets: &data.assets,
        pawn: &mut data.world.pawns[pawn_index],
        tree: &mut data.world.trees[tree_index],
        tree_data: &mut data.world.trees_data[tree_index],
        time: data.timing.time,
    };

    match action.state {
        ActionState::Initial => cut_tree_initial(action, &mut params),
        ActionState::Running => cut_tree_running(action, &mut params),
        ActionState::Finalizing => cut_tree_finalize(action, &mut params),
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

fn cut_tree_initial(action: &mut Action, params: &mut CutTreeParams) {
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

fn cut_tree_running(action: &mut Action, params: &mut CutTreeParams) {

}

fn cut_tree_finalize(action: &mut Action, params: &mut CutTreeParams) {
    params.pawn.animation = params.assets.animations.pawn.idle;
    action.state = ActionState::Finalized;
}

