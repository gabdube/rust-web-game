use crate::assets::AnimationBase;
use crate::behaviour::BehaviourState;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

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

    PawnBehaviour::cancel(game, pawn.id);

    if game.world.pawns_data[pawn_index].grabbed_resource().is_some() {
        super::drop_resource(game, pawn_index);
    }

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::HarvestWood { tree_id: tree.id },
        state: BehaviourState::Initial,
    };
}

pub fn cancel(game: &mut DemoGameData, pawn_index: usize) {
    let behaviour = &mut game.world.pawns_behaviour[pawn_index];
    let tree_index = params(behaviour.ty);

    match behaviour.state {
        BehaviourState::Running(BEGIN_CUT_TREE) | BehaviourState::Running(CUT_TREE) => {
            game.world.pawns[pawn_index].animation = game.assets.animations.pawn.idle;
            game.world.trees[tree_index].animation = game.assets.resources.tree_idle;
            game.world.trees_data[tree_index].being_harvested = false;
        },
        BehaviourState::Running(SPAWN_WOOD) => {
            spawn_wood(game, pawn_index);
        },
        _ => {}
    }

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour::idle();
}


pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let state = game.world.pawns_behaviour[pawn_index].state;

    match state {
        BehaviourState::Initial => init(game, pawn_index),
        BehaviourState::Running(MOVE_TO_TREE) => move_to_tree(game, pawn_index),
        BehaviourState::Running(BEGIN_CUT_TREE) => begin_cut_tree(game, pawn_index),
        BehaviourState::Running(CUT_TREE) => cut_tree(game, pawn_index),
        BehaviourState::Running(SPAWN_WOOD) => spawn_wood(game, pawn_index),
        _ => {}
    }
}

fn init(game: &mut DemoGameData, pawn_index: usize) {
    game.world.pawns[pawn_index].animation = game.assets.animations.pawn.walk;
    game.world.pawns_behaviour[pawn_index].state = BehaviourState::Running(MOVE_TO_TREE);
    
    move_to_tree(game, pawn_index);
}

fn move_to_tree(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let tree_index = params(behaviour.ty);
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    if tree_data.being_harvested || tree_data.life == 0 {
        *behaviour = PawnBehaviour::idle();
        return;
    }

    let mut target_position = tree.position;
    if pawn.position.x > target_position.x {
        target_position.x += 60.0;
    } else {
        target_position.x -= 60.0;
    }
    
    target_position.y += 10.0;

    let updated_position = move_to(pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        behaviour.state = BehaviourState::Running(BEGIN_CUT_TREE);
    }

    pawn.position = updated_position;
    pawn.flipped = pawn.position.x > target_position.x;
}

fn begin_cut_tree(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let tree_index = params(behaviour.ty);
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    pawn.animation = game.assets.animations.pawn.axe;
    pawn.flipped = pawn.position.x > tree.position.x;

    tree.animation = game.assets.resources.tree_cut;
    tree.current_frame = 0;

    tree_data.being_harvested = true;
    tree_data.last_drop_timestamp = game.global.time;

    behaviour.state = BehaviourState::Running(CUT_TREE);

    cut_tree(game, pawn_index);
}

fn cut_tree(game: &mut DemoGameData, pawn_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let world = &mut game.world;
    let pawn = &mut world.pawns[pawn_index];
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let tree_index = params(behaviour.ty);
    let tree_data = &mut world.trees_data[tree_index];

    let total_animation_time = crate::ANIMATION_INTERVAL * 6.0;
    
    if pawn.current_frame == 5 && elapsed(game.global.time, tree_data.last_drop_timestamp, total_animation_time) {
        tree_data.life -= u8::min(tree_data.life, 1);
        tree_data.last_drop_timestamp = game.global.time;
    }

    if tree_data.life == 0 {
        behaviour.state = BehaviourState::Running(SPAWN_WOOD);
    }
}

fn spawn_wood(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];

    let tree_index = params(behaviour.ty);
    let tree = &mut world.trees[tree_index];
    let tree_data = &mut world.trees_data[tree_index];

    tree.animation = AnimationBase::from_aabb(game.assets.resources.tree_stump.aabb);
    tree_data.being_harvested = false;

    *behaviour = PawnBehaviour::idle();

    // Spawns three wood resources around the tree
    let center_pos = tree.position;
    let mut position = center_pos;
    let mut angle = 0.0;
    for _ in 0..3 {
        angle += f32::to_radians(fastrand::u8(120..180) as f32);
        position.x = f32::ceil(center_pos.x + f32::cos(angle) * 64.0);
        position.y = f32::ceil(center_pos.y + f32::sin(angle) * 64.0);

        crate::behaviour::spawn_resources::spawn_wood(game, position);
    }
}


#[inline(always)]
fn params(value: PawnBehaviourType) -> usize {
    match value {
        PawnBehaviourType::HarvestWood { tree_id } => tree_id as usize,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
