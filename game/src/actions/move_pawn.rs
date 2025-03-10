use crate::actions::ActionState;
use crate::assets::Assets;
use crate::shared::{Position, pos};
use crate::world::World;

pub struct MovePawnParams<'a> {
    pub world: &'a mut World,
    pub assets: &'a Assets,
    pub pawn_id: u32,
    pub frame_delta: f32,
    pub target: Position<f32>,
}

pub fn move_pawn(state: &mut ActionState, params: &mut MovePawnParams) {
    if params.world.pawns.get(params.pawn_id as usize).is_none() {
        *state = ActionState::Finalized;
    }
    
    match *state {
        ActionState::Initial => move_pawn_initial(state, params),
        ActionState::Running => move_pawn_running(state, params),
        ActionState::Finalizing => move_pawn_finalize(state, params),
        ActionState::Finalized => {},
    }
}

fn move_pawn_initial(state: &mut ActionState, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    params.world.pawns[index].animation = params.assets.animations.pawn.walk;
    *state = ActionState::Running;
}

fn move_pawn_running(state: &mut ActionState, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    let current_position = params.world.pawns[index].position;
    let target = params.target;

    let angle = f32::atan2(target.y - current_position.y, target.x - current_position.x);
    let speed = 0.2f32 * params.frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current_position.x + move_x, current_position.y + move_y);

    params.world.pawns[index].flipped = move_x < 0.0;

    if (move_x > 0.0 && updated_position.x > params.target.x) || (move_x < 0.0 && updated_position.x < target.x) {
        updated_position.x = target.x;
    }

    if (move_y > 0.0 && updated_position.y > target.y) || (move_y < 0.0 && updated_position.y < target.y) {
        updated_position.y = target.y;
    }

    if updated_position == target {
        *state = ActionState::Finalizing;
    }

    params.world.pawns[index].position = updated_position;
}

fn move_pawn_finalize(state: &mut ActionState, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    params.world.pawns[index].animation = params.assets.animations.pawn.idle;
    *state = ActionState::Finalized;
}
