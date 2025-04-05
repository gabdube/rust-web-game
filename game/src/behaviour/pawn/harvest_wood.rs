use crate::assets::AnimationBase;
use crate::behaviour::BehaviourState;
use crate::world::{BaseAnimated, WorldObject, WorldObjectType, TreeData};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_TREE: u8 = 0;
const BEGIN_CUT_TREE: u8 = 1;
const CUT_TREE: u8 = 2;
const SPAWN_WOOD: u8 = 3;

struct PawnHarvestWoodParams {
    pawn: BaseAnimated,
    tree: BaseAnimated,
    tree_data: TreeData,
    tree_id: u32,
    last_timestamp: f32,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

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

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::HarvestWood { tree_id: tree.id, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn cancel(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);

    match params.state {
        BehaviourState::Running(BEGIN_CUT_TREE) | BehaviourState::Running(CUT_TREE) => {
            params.pawn.animation = game.assets.animations.pawn.idle;
            params.tree.animation = game.assets.resources.tree_idle;
            params.tree_data.being_harvested = false;
        },
        BehaviourState::Running(SPAWN_WOOD) => {
            spawn_wood(game, &mut params);
        },
        _ => {}
    }

    write_params(game, pawn_index, &params);
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_TREE) => move_to_tree(game, &mut params),
        BehaviourState::Running(BEGIN_CUT_TREE) => begin_cut_tree(game, &mut params),
        BehaviourState::Running(CUT_TREE) => cut_tree(game, &mut params),
        BehaviourState::Running(SPAWN_WOOD) => spawn_wood(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &params);
}

fn init(game: &DemoGameData, params: &mut PawnHarvestWoodParams) {
    params.pawn.animation = game.assets.animations.pawn.walk;
    params.state = BehaviourState::Running(MOVE_TO_TREE);
    move_to_tree(game, params);
}

fn move_to_tree(game: &DemoGameData, params: &mut PawnHarvestWoodParams) {
    use crate::behaviour::behaviour_shared::move_to;
    
    if params.tree_data.being_harvested || params.tree_data.life == 0 {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let mut target_position = params.tree.position;
    if params.pawn.position.x > target_position.x {
        target_position.x += 60.0;
    } else {
        target_position.x -= 60.0;
    }

    target_position.y += 10.0;

    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.state = BehaviourState::Running(BEGIN_CUT_TREE);
    }

    params.pawn.position = updated_position;
    params.pawn.flipped = params.pawn.position.x > target_position.x;
}

fn begin_cut_tree(game: &DemoGameData, params: &mut PawnHarvestWoodParams) {
    params.pawn.animation = game.assets.animations.pawn.axe;
    params.pawn.flipped = params.pawn.position.x > params.tree.position.x;

    params.tree.animation = game.assets.resources.tree_cut;
    params.tree.current_frame = 0;
    params.tree_data.being_harvested = true;
    
    params.last_timestamp = game.global.time as f32;
    params.state = BehaviourState::Running(CUT_TREE);

    cut_tree(game, params);
}

fn cut_tree(game: &DemoGameData, params: &mut PawnHarvestWoodParams) {
    use crate::behaviour::behaviour_shared::elapsed;

    let total_animation_time = crate::ANIMATION_INTERVAL * 6.0;

    if elapsed(game.global.time, params.last_timestamp as f64, total_animation_time) {
        params.last_timestamp = game.global.time as f32;
        params.tree_data.life -= u8::min(params.tree_data.life, 1);
    }

    if params.tree_data.life == 0 {
        params.state = BehaviourState::Running(SPAWN_WOOD);
    }
}

fn spawn_wood(game: &mut DemoGameData, params: &mut PawnHarvestWoodParams) {
    params.tree.animation = AnimationBase::from_aabb(game.assets.resources.tree_stump);
    params.tree_data.being_harvested = false;

    params.new_behaviour = Some(PawnBehaviour::idle());

    // Spawns three wood resources around the tree
    let center_pos = params.tree.position;
    let mut position = center_pos;
    let mut angle = 0.0;
    for _ in 0..3 {
        angle += f32::to_radians(fastrand::u8(120..180) as f32);
        position.x = f32::ceil(center_pos.x + f32::cos(angle) * 64.0);
        position.y = f32::ceil(center_pos.y + f32::sin(angle) * 64.0);

        crate::behaviour::spawn_resources::spawn_wood(game, position);
    }
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnHarvestWoodParams {
    let pawn = game.world.pawns.get(pawn_index);
    let behaviour = game.world.pawns_behaviour.get(pawn_index);

    match (pawn, behaviour) {
        (Some(pawn), Some(behaviour)) => {
            let (tree_index, last_timestamp) = match behaviour.ty {
                PawnBehaviourType::HarvestWood { tree_id, last_timestamp } => (tree_id as usize, last_timestamp),
                _ => unsafe { ::std::hint::unreachable_unchecked()}
            };

            let (tree, tree_data) = match (game.world.trees.get(tree_index), game.world.trees_data.get(tree_index)) {
                (Some(tree), Some(tree_data)) => (tree, tree_data),
                _ => unsafe { ::std::hint::unreachable_unchecked()}
            };

            PawnHarvestWoodParams {
                pawn: *pawn,
                tree: *tree,
                tree_data: *tree_data,
                tree_id: tree_index as u32,
                last_timestamp,
                new_behaviour: None,
                state: behaviour.state
            }
        },
        _  => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnHarvestWoodParams) {
    let tree_index = params.tree_id as usize;
    let pawn = game.world.pawns.get_mut(pawn_index);
    let behaviour = game.world.pawns_behaviour.get_mut(pawn_index);
    let tree = game.world.trees.get_mut(tree_index);
    let tree_data = game.world.trees_data.get_mut(tree_index);

    match (pawn, behaviour, tree, tree_data) {
        (Some(pawn), Some(behaviour), Some(tree), Some(tree_data)) => {
            *pawn = params.pawn;
            *tree = params.tree;
            *tree_data = params.tree_data;

            match params.new_behaviour {
                Some(new_behaviour) => {
                    *behaviour = new_behaviour;
                },
                None => {
                    behaviour.ty = PawnBehaviourType::HarvestWood { 
                        tree_id: params.tree_id,
                        last_timestamp: params.last_timestamp
                    };

                    behaviour.state = params.state;
                }
            }
        },
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}
