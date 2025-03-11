use crate::actions::{Action, ActionState};
use crate::assets::Assets;
use crate::shared::{Position, pos};
use crate::world::World;
use crate::DemoGameData;

struct MovePawnParams<'a> {
    pub world: &'a mut World,
    pub assets: &'a Assets,
    pub frame_delta: f32,
    pub pawn_id: u32,
    pub move_target: Position<f32>,
}

pub fn move_pawn(data: &mut DemoGameData, action: &mut Action, pawn_id: u32, move_target: Position<f32>) {
    if data.world.pawns.get(pawn_id as usize).is_none() {
        action.state = ActionState::Finalized;
    }

    let mut params = MovePawnParams {
        world: &mut data.world,
        assets: &data.assets,
        frame_delta: data.timing.frame_delta,
        pawn_id,
        move_target
    };
    
    match action.state {
        ActionState::Initial => move_pawn_initial(action, &mut params),
        ActionState::Running => move_pawn_running(action, &mut params),
        ActionState::Finalizing => move_pawn_finalize(action, &mut params),
        ActionState::Finalized => {},
    }
}

fn move_pawn_initial(action: &mut Action, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    params.world.pawns[index].animation = params.assets.animations.pawn.walk;
    action.state = ActionState::Running;
}

fn move_pawn_running(action: &mut Action, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    let current_position = params.world.pawns[index].position;
    let target = params.move_target;

    let angle = f32::atan2(target.y - current_position.y, target.x - current_position.x);
    let speed = 0.2f32 * params.frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current_position.x + move_x, current_position.y + move_y);

    params.world.pawns[index].flipped = move_x < 0.0;

    if (move_x > 0.0 && updated_position.x > target.x) || (move_x < 0.0 && updated_position.x < target.x) {
        updated_position.x = target.x;
    }

    if (move_y > 0.0 && updated_position.y > target.y) || (move_y < 0.0 && updated_position.y < target.y) {
        updated_position.y = target.y;
    }

    if updated_position == target {
        action.state = ActionState::Finalizing;
    }

    params.world.pawns[index].position = updated_position;
}

fn move_pawn_finalize(action: &mut Action, params: &mut MovePawnParams) {
    let index = params.pawn_id as usize;
    params.world.pawns[index].animation = params.assets.animations.pawn.idle;
    action.state = ActionState::Finalized;
}
