use crate::actions::ActionState;
use crate::assets::Assets;
use crate::world::World;

pub struct CutTreeParams<'a> {
    pub world: &'a mut World,
    pub assets: &'a Assets,
    pub pawn_id: u32,
    pub tree_id: u32,
}

pub fn cut_tree(state: &mut ActionState, params: &mut CutTreeParams) {
    if params.world.pawns.get(params.pawn_id as usize).is_none() || params.world.trees.get(params.tree_id as usize).is_none() {
        *state = ActionState::Finalized;
    }

    match *state {
        ActionState::Initial => cut_tree_initial(state, params),
        ActionState::Running => cut_tree_running(state, params),
        ActionState::Finalizing => cut_tree_finalize(state, params),
        ActionState::Finalized => {},
    }
}

fn cut_tree_initial(state: &mut ActionState, params: &mut CutTreeParams) {
    let pawn_index = params.pawn_id as usize;
    let tree_index = params.tree_id as usize;

    let pawn = &mut params.world.pawns[pawn_index];
    let tree = &mut params.world.trees[tree_index];
    let tree_data = &mut params.world.trees_data[tree_index];

    // A pawn is already harvesting the tree
    if tree_data.being_harvested {
        *state = ActionState::Finalized;
        return;
    }

    pawn.animation = params.assets.animations.pawn.axe;
    tree.animation = params.assets.resources.tree_cut;
    tree_data.being_harvested = true;
    tree_data.last_drop_timestamp = 0.0;
  
    *state = ActionState::Running;
}

fn cut_tree_running(state: &mut ActionState, params: &mut CutTreeParams) {

}

fn cut_tree_finalize(state: &mut ActionState, params: &mut CutTreeParams) {

}
