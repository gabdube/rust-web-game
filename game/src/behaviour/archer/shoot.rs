use crate::behaviour::BehaviourState;
use crate::shared::{Position, AABB, pos};
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{ArcherBehaviour, ArcherBehaviourType};

const MOVING: u8 = 0;
const SHOOTING: u8 = 2;
const PAUSE: u8 = 3;

const MAX_SHOOTING_DISTANCE: f32 = 64.0 * 6.0;

enum ShootingAnimation {
    Top,
    TopLeft,
    TopRight,
    Left,
    Right,
    BottomLeft,
    BottomRight,
    Bottom,
}

pub fn new(game: &mut DemoGameData, archer: WorldObject, target: WorldObject) {
    let archer_index;
    match (archer.ty, target.ty) {
        (WorldObjectType::Archer, WorldObjectType::Sheep) => {
            archer_index = archer.id as usize;
            if archer_index >= game.world.archers.len() {
                return;
            }

        },
        _ => { return; }
    }

    game.world.archers_behaviour[archer_index] = ArcherBehaviour {
        ty: ArcherBehaviourType::Shoot { target, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, archer_index: usize) {
    let state = game.world.archers_behaviour[archer_index].state;
    match state {
        BehaviourState::Initial => init(game, archer_index),
        BehaviourState::Running(MOVING) => moving(game, archer_index),
        BehaviourState::Running(SHOOTING) => shooting(game, archer_index),
        BehaviourState::Running(PAUSE) => pause(game, archer_index),
        _ => {},
    }
}

fn init(game: &mut DemoGameData, archer_index: usize) {
    let target_position = target_position(game, archer_index);

    let archer = &mut game.world.archers[archer_index];
    let behaviour = &mut game.world.archers_behaviour[archer_index];

    if archer.position.distance(target_position) > MAX_SHOOTING_DISTANCE {
        archer.animation = game.assets.animations.archer.walk;
        archer.current_frame = 0;
        behaviour.state = BehaviourState::Running(MOVING);
    } else {
        behaviour.state = BehaviourState::Running(SHOOTING);
    }
}

fn moving(game: &mut DemoGameData, archer_index: usize) {
    use crate::behaviour::behaviour_shared::move_to;

    let target_position = target_position(game, archer_index);

    let world = &mut game.world;
    let archer = &mut world.archers[archer_index];
    let behaviour = &mut world.archers_behaviour[archer_index];

    archer.position = move_to(archer.position, target_position, game.global.frame_delta);
    archer.flipped = archer.position.x > target_position.x;

    if archer.position.distance(target_position) < MAX_SHOOTING_DISTANCE {
        archer.current_frame = 0;
        behaviour.state = BehaviourState::Running(SHOOTING);
    }
}

fn shooting(game: &mut DemoGameData, archer_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let target_position = target_position(game, archer_index);
    let target_life = target_life(game, archer_index);
    
    if target_life == 0 {
        game.world.archers_behaviour[archer_index] = ArcherBehaviour::idle();
        return;
    }
    
    let archer = &mut game.world.archers[archer_index];
    if archer.position.distance(target_position) > MAX_SHOOTING_DISTANCE {
        archer.animation = game.assets.animations.archer.idle;
        archer.current_frame = 0;

        let behaviour = &mut game.world.archers_behaviour[archer_index];
        behaviour.state = BehaviourState::Running(PAUSE);
        params_set_last_timestamp(&mut behaviour.ty, game.global.time);

        return;
    }

    set_aim_animation(game, archer_index);

    let archer = &mut game.world.archers[archer_index];
    let behaviour = &mut game.world.archers_behaviour[archer_index];
    let timestamp = params_timestamp(behaviour.ty);
    let animation_time = crate::ANIMATION_INTERVAL * 7.0;
    if archer.current_frame == 7 && elapsed(game.global.time, timestamp, animation_time) {
        params_set_last_timestamp(&mut behaviour.ty, game.global.time);
        spawn_arrow(game, archer_index);
    }
}

fn pause(game: &mut DemoGameData, archer_index: usize) {
    use crate::behaviour::behaviour_shared::elapsed;

    let archer = &mut game.world.archers[archer_index];
    let behaviour = &mut game.world.archers_behaviour[archer_index];
    let timestamp = params_timestamp(behaviour.ty);
    if elapsed(game.global.time, timestamp, 500.0) {
        archer.animation = game.assets.animations.archer.walk;
        archer.current_frame = 0;
        behaviour.state = BehaviourState::Running(MOVING);
    }
}

fn set_aim_animation(game: &mut DemoGameData, archer_index: usize) -> ShootingAnimation {
    fn select_animation(game: &mut DemoGameData, archer_index: usize) -> ShootingAnimation {
        let target_position = target_position(game, archer_index);
        let position = game.world.archers[archer_index].position;
        let angle = f32::atan2(target_position.y - position.y, target_position.x - position.x);
        if angle < 0.0 {
            let frac = -std::f32::consts::FRAC_PI_8;
            if angle > frac {
                ShootingAnimation::Right
            } else if angle > (frac * 3.0) {
                ShootingAnimation::TopRight
            } else if angle > (frac * 5.0) {
                ShootingAnimation::Top
            } else if angle > (frac * 7.0) {
                ShootingAnimation::TopLeft
            } else {
                ShootingAnimation::Left
            }
        } else {
            let frac = std::f32::consts::FRAC_PI_8;
            if angle < frac {
                ShootingAnimation::Right
            } else if angle < (frac * 3.0) {
                ShootingAnimation::BottomRight
            } else if angle < (frac * 5.0) {
                ShootingAnimation::Bottom
            } else if angle < (frac * 7.0) {
                ShootingAnimation::BottomLeft
            } else {
                ShootingAnimation::Left
            }
        }
    }
    
    let target_position = target_position(game, archer_index);
    let animation = select_animation(game, archer_index);
    let archer = &mut game.world.archers[archer_index];

    let animations = &game.assets.animations.archer;
    archer.animation = match animation {
        ShootingAnimation::Top => animations.fire_top,
        ShootingAnimation::Bottom => animations.fire_bottom,
        ShootingAnimation::BottomLeft | ShootingAnimation::BottomRight => animations.fire_bottom_h,
        ShootingAnimation::TopLeft | ShootingAnimation::TopRight => animations.fire_top_h,
        ShootingAnimation::Left | ShootingAnimation::Right => animations.fire_h,
    };

    archer.flipped = archer.position.x > target_position.x;

    animation
}

fn spawn_arrow(
    game: &mut DemoGameData,
    archer_index: usize,
) {
    use crate::world::{BaseProjectile, ArrowData};

    fn compute_arrow_tip_offset(sprite: &AABB, rotation: f32) -> Position<f32> {
        let x = sprite.width() / 2.0;
        let y = sprite.height() / 2.0;

        pos(
            (x * f32::cos(rotation)) - (y * f32::sin(rotation)),
            (x * f32::sin(rotation)) - (y * f32::cos(rotation)),
        )
    }

    let target_position = target_position(game, archer_index);
    let target = params(game.world.archers_behaviour[archer_index].ty);
    let archer = game.world.archers[archer_index];

    let mut position = archer.position;
    position.y -= archer.aabb().height() / 2.0;

    let rotation = f32::atan2(target_position.y - position.y, target_position.x - position.x);
    let velocity = pos(f32::cos(rotation) * 5.0, f32::sin(rotation) * 5.0);

    let sprite = game.assets.resources.arrow;
    let arrow_tip_offset = compute_arrow_tip_offset(&sprite, rotation);

    game.world.arrows.push(BaseProjectile {
        position,
        sprite,
        rotation,
        deleted: false,
    });

    game.world.arrows_data.push(ArrowData {
        velocity,
        target_position,
        target_entity: target,
        arrow_tip_offset,
    });
}

fn target_position(game: &mut DemoGameData, archer_index: usize) -> Position<f32> {
    let target = params(game.world.archers_behaviour[archer_index].ty);
    let target_index = target.id as usize;

    let mut base;
    let height;
    match target.ty {
        WorldObjectType::Sheep => { 
            let sheep = game.world.sheeps[target_index];
            base = sheep.position;
            height = sheep.aabb().height();
         },
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }

    base.y -= height * 0.5;

    base
}

fn target_life(game: &mut DemoGameData, archer_index: usize) -> u8 {
    let target = params(game.world.archers_behaviour[archer_index].ty);
    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => { 
            game.world.sheeps_data[target_index].life
         },
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}

#[inline(always)]
fn params(value: ArcherBehaviourType) -> WorldObject {
    match value {
        ArcherBehaviourType::Shoot { target, .. } => target,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

#[inline(always)]
fn params_timestamp(value: ArcherBehaviourType) -> f64 {
    match value {
        ArcherBehaviourType::Shoot { last_timestamp, .. } => last_timestamp as f64,
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}

#[inline(always)]
fn params_set_last_timestamp(value: &mut ArcherBehaviourType, time: f64) {
    match value {
        ArcherBehaviourType::Shoot { last_timestamp, .. } => *last_timestamp = time as f32,
        _ => unsafe { ::std::hint::unreachable_unchecked() }
    }
}
