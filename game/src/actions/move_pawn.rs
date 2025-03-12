use crate::actions::{Action, ActionState};
use crate::assets::PawnAnimation;
use crate::shared::{Position, pos};
use crate::world::BaseAnimated;
use crate::DemoGameData;

struct MovePawnParams<'a> {
    pub pawn_animations: &'a PawnAnimation,
    pub pawn_base: &'a mut BaseAnimated,
    pub frame_delta: f32,
    pub move_target: Position<f32>,
}

pub fn move_pawn(data: &mut DemoGameData, action: &mut Action, pawn_id: u32, move_target: Position<f32>) {
    let pawn_index = pawn_id as usize;
    if pawn_index >= data.world.pawns.len() {
        action.state = ActionState::Finalized;
    }

    let mut params = MovePawnParams {
        pawn_animations: &data.assets.animations.pawn,
        pawn_base: &mut data.world.pawns[pawn_index],
        frame_delta: data.timing.frame_delta,
        move_target
    };
    
    match action.state {
        ActionState::Initial => move_pawn_initial(action, &mut params),
        ActionState::Running => move_pawn_running(action, &mut params),
        ActionState::Finalizing => move_pawn_finalize(action, &mut params),
        ActionState::Finalized => {},
    }
}

pub fn cancel(data: &mut DemoGameData, pawn_id: u32) {
    let pawn_index = pawn_id as usize;
    if pawn_index < data.world.pawns.len() {
        data.world.pawns[pawn_index].animation = data.assets.animations.pawn.idle;
    }
}

fn move_pawn_initial(action: &mut Action, params: &mut MovePawnParams) {
    params.pawn_base.animation = params.pawn_animations.walk;
    action.state = ActionState::Running;
}

fn move_pawn_running(action: &mut Action, params: &mut MovePawnParams) {
    let current_position = params.pawn_base.position;
    let target = params.move_target;

    let angle = f32::atan2(target.y - current_position.y, target.x - current_position.x);
    let speed = 0.2f32 * params.frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current_position.x + move_x, current_position.y + move_y);

    params.pawn_base.flipped = move_x < 0.0;

    if (move_x > 0.0 && updated_position.x > target.x) || (move_x < 0.0 && updated_position.x < target.x) {
        updated_position.x = target.x;
    }

    if (move_y > 0.0 && updated_position.y > target.y) || (move_y < 0.0 && updated_position.y < target.y) {
        updated_position.y = target.y;
    }

    if updated_position == target {
        action.state = ActionState::Finalizing;
    }

    params.pawn_base.position = updated_position;
}

fn move_pawn_finalize(action: &mut Action, params: &mut MovePawnParams) {
    params.pawn_base.animation = params.pawn_animations.idle;
    action.state = ActionState::Finalized;
}
