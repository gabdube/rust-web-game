use crate::behaviour::BehaviourState;
use crate::behaviour::behaviour_shared::{is_enemy_structure, move_to, elapsed};
use crate::shared::{Position, AABB, pos};
use crate::world::{BaseAnimated, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{ArcherBehaviour, ArcherBehaviourType};

const MOVING: u8 = 0;
const SHOOTING: u8 = 2;
const PAUSE: u8 = 3;

const MAX_SHOOTING_DISTANCE: f32 = 64.0 * 7.0;

pub struct ArcherShootParams {
    archer: BaseAnimated,
    target: WorldObject,
    target_position: Position<f32>,
    target_life: u8,
    last_timestamp: f32,
    spawn_arrow: bool,
    new_behaviour: Option<ArcherBehaviour>,
    state: BehaviourState,
}

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

    let archer_index = archer.id as usize;
    let target_index = target.id as usize;
    let target_invalid = match target.ty {
        WorldObjectType::Sheep => target_index >= game.world.sheeps.len(),
        WorldObjectType::Structure => is_enemy_structure(game, target_index) == false,
        _ => false
    };

    if archer.ty != WorldObjectType::Archer  || archer_index >= game.world.archers.len() || target_invalid  {
        return;
    }

    game.world.archers_behaviour[archer_index] = ArcherBehaviour {
        ty: ArcherBehaviourType::Shoot { target, last_timestamp: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, archer_index: usize) {
    let mut params = read_params(game, archer_index);

    if params.target_life == 0 {
        params.new_behaviour = Some(ArcherBehaviour::idle());
        write_params(game, archer_index, &params);
        return;
    } 

    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVING) => moving(game, &mut params),
        BehaviourState::Running(SHOOTING) => shooting(game, &mut params),
        BehaviourState::Running(PAUSE) => pause(game, &mut params),
        _ => {},
    }
    write_params(game, archer_index, &params);
}

fn init(game: &DemoGameData, params: &mut ArcherShootParams) {
    params.archer.current_frame = 0;
    
    if params.archer.position.distance(params.target_position) > MAX_SHOOTING_DISTANCE {
        params.archer.animation = game.assets.animations.archer.walk;
        params.archer.current_frame = 0;
        params.state = BehaviourState::Running(MOVING);
        moving(game, params);
    } else {
        params.state = BehaviourState::Running(SHOOTING);
        shooting(game, params);
    }
}

fn moving(game: &DemoGameData, params: &mut ArcherShootParams) {
    params.archer.position = move_to(params.archer.position, params.target_position, game.global.frame_delta);
    params.archer.flipped = params.archer.position.x > params.target_position.x;

    if params.archer.position.distance(params.target_position) < MAX_SHOOTING_DISTANCE {
        params.archer.current_frame = 0;
        params.state = BehaviourState::Running(SHOOTING);
    }
}

fn shooting(game: &DemoGameData, params: &mut ArcherShootParams) {
    fn select_animation(params: &ArcherShootParams) -> ShootingAnimation {
        let position = params.archer.position;
        let target_position = params.target_position;
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

    fn set_aim_animation(game: &DemoGameData, params: &mut ArcherShootParams) -> ShootingAnimation {
        let animation = select_animation(params);
        let animations = &game.assets.animations.archer;
        params.archer.animation = match animation {
            ShootingAnimation::Top => animations.fire_top,
            ShootingAnimation::Bottom => animations.fire_bottom,
            ShootingAnimation::BottomLeft | ShootingAnimation::BottomRight => animations.fire_bottom_h,
            ShootingAnimation::TopLeft | ShootingAnimation::TopRight => animations.fire_top_h,
            ShootingAnimation::Left | ShootingAnimation::Right => animations.fire_h,
        };
    
        params.archer.flipped = params.archer.position.x > params.target_position.x;
    
        animation
    }

    if params.archer.position.distance(params.target_position) > MAX_SHOOTING_DISTANCE {
        params.archer.animation = game.assets.animations.archer.idle;
        params.archer.current_frame = 0;
        params.last_timestamp = game.global.time as f32;
        params.state = BehaviourState::Running(PAUSE);
        return;
    }

    set_aim_animation(game, params);

    let animation_time = crate::ANIMATION_INTERVAL * 7.0;
    if params.archer.current_frame == 7 && elapsed(game.global.time, params.last_timestamp as f64, animation_time) {
        params.archer.animation = game.assets.animations.archer.idle;
        params.archer.current_frame = 0;
        params.last_timestamp = game.global.time as f32;
        params.state = BehaviourState::Running(PAUSE);
        params.spawn_arrow = true;
    }
}

fn pause(game: &DemoGameData, params: &mut ArcherShootParams) {
    if elapsed(game.global.time, params.last_timestamp as f64, 500.0) {
        init(game, params);
    }
}

fn read_params(game: &DemoGameData, archer_index: usize) -> ArcherShootParams {
    let archer = unsafe { game.world.archers.get_unchecked(archer_index) };
    let archer_behaviour = unsafe { game.world.archers_behaviour.get_unchecked(archer_index) };
    let (target, last_timestamp) = match archer_behaviour.ty {
        ArcherBehaviourType::Shoot { target, last_timestamp } => (target, last_timestamp),
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    let target_position = crate::behaviour::behaviour_shared::target_position(game, target, true);
    let target_life = crate::behaviour::behaviour_shared::target_life(game, target);

    ArcherShootParams {
        archer: *archer,
        target,
        target_position,
        target_life,
        last_timestamp,
        spawn_arrow: false,
        new_behaviour: None,
        state: archer_behaviour.state
    }
}

fn spawn_arrow(game: &mut DemoGameData, params: &ArcherShootParams) {
    use crate::world::{BaseProjectile, ArrowData};

    fn compute_arrow_tip_offset(sprite: &AABB, rotation: f32) -> Position<f32> {
        let x = sprite.width() / 2.0;
        let y = sprite.height() / 2.0;

        pos(
            (x * f32::cos(rotation)) - (y * f32::sin(rotation)),
            (x * f32::sin(rotation)) - (y * f32::cos(rotation)),
        )
    }

    let mut position = params.archer.position;
    position.y -= params.archer.aabb().height() / 2.0;

    let rotation = f32::atan2(params.target_position.y - position.y, params.target_position.x - position.x);
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
        target_position: params.target_position,
        target_entity: params.target,
        arrow_tip_offset,
    });
}

fn write_params(game: &mut DemoGameData, archer_index: usize, params: &ArcherShootParams) {
    if params.spawn_arrow {
        spawn_arrow(game, params);
    }
    
    let archer = unsafe { game.world.archers.get_unchecked_mut(archer_index) };
    let archer_behaviour = unsafe { game.world.archers_behaviour.get_unchecked_mut(archer_index) };
   
    *archer = params.archer;

    match params.new_behaviour {
        Some(new_behaviour) => { *archer_behaviour = new_behaviour; }
        None => { 
            archer_behaviour.ty = ArcherBehaviourType::Shoot { target: params.target, last_timestamp: params.last_timestamp };
            archer_behaviour.state = params.state;
        }
    }
}
