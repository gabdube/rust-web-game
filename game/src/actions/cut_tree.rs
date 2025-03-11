use crate::actions::{Action, ActionState};
use crate::assets::Assets;
use crate::world::World;
use crate::DemoGameData;

struct CutTreeParams<'a> {
    pub world: &'a mut World,
    pub assets: &'a Assets,
    pub pawn_id: u32,
    pub tree_id: u32,
}

pub fn cut_tree(data: &mut DemoGameData, action: &mut Action, pawn_id: u32, tree_id: u32) {
    if data.world.pawns.get(pawn_id as usize).is_none() || data.world.trees.get(tree_id as usize).is_none() {
        action.state = ActionState::Finalized;
    }

    let mut params = CutTreeParams {
        world: &mut data.world,
        assets: &data.assets,
        pawn_id,
        tree_id,
    };

    match action.state {
        ActionState::Initial => cut_tree_initial(action, &mut params),
        ActionState::Running => cut_tree_running(action, &mut params),
        ActionState::Finalizing => cut_tree_finalize(action, &mut params),
        ActionState::Finalized => {},
    }
}

fn cut_tree_initial(action: &mut Action, params: &mut CutTreeParams) {
    let pawn_index = params.pawn_id as usize;
    let tree_index = params.tree_id as usize;

    let pawn = &mut params.world.pawns[pawn_index];
    let tree = &mut params.world.trees[tree_index];
    let tree_data = &mut params.world.trees_data[tree_index];

    // A pawn is already harvesting the tree
    if tree_data.being_harvested {
        action.state = ActionState::Finalized;
        return;
    }

    pawn.animation = params.assets.animations.pawn.axe;
    tree.animation = params.assets.resources.tree_cut;
    tree_data.being_harvested = true;
    tree_data.last_drop_timestamp = 0.0;
  
    action.state = ActionState::Running;
}

fn cut_tree_running(action: &mut Action, params: &mut CutTreeParams) {

}

fn cut_tree_finalize(action: &mut Action, params: &mut CutTreeParams) {

}
