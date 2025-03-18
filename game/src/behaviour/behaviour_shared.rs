//! Shared logic between actions
use crate::shared::{Position, pos};

pub fn move_to(current: Position<f32>, target: Position<f32>, frame_delta: f32) -> Position<f32> {
    let angle = f32::atan2(target.y - current.y, target.x - current.x);
    let speed = 0.2f32 * frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current.x + move_x, current.y + move_y);

    if f32::abs(updated_position.x - target.x) < 1.0 {
        updated_position.x = target.x;
    }

    if f32::abs(updated_position.y - target.y) < 1.0 {
        updated_position.y = target.y;
    }

    updated_position
}

