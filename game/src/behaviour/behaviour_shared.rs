//! Shared logic between actions
use crate::shared::{Position, pos};
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;

pub fn move_to(current: Position<f32>, target: Position<f32>, frame_delta: f32) -> Position<f32> {
    move_to_with_speed(current, target, frame_delta, 0.2)
}

pub fn move_to_with_speed(current: Position<f32>, target: Position<f32>, frame_delta: f32, base_speed: f32) -> Position<f32> {
    let angle = f32::atan2(target.y - current.y, target.x - current.x);
    let speed = base_speed * frame_delta;
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

#[inline(always)]
pub fn elapsed(time: f64, timestamp: f64, timer: f64) -> bool {
    time - timestamp > timer
}

pub fn target_life(game: &DemoGameData, target: WorldObject) -> u8 {
    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => game.world.sheeps_data[target_index].life,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}

pub fn target_position(game: &DemoGameData, target: WorldObject, center: bool) -> Position<f32> {
    let mut base;
    let height;
    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => { 
            let sheep = game.world.sheeps[target_index];
            base = sheep.position;
            height = sheep.aabb().height();
        },
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }

    if center {
        base.y -= height * 0.5;
    }

    base
}
