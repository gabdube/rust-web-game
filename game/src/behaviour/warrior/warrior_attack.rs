use crate::behaviour::BehaviourState;
use crate::behaviour::behaviour_shared::{elapsed, is_enemy_structure};
use crate::shared::Position;
use crate::world::{BaseAnimated, StructureData, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{WarriorBehaviour, WarriorBehaviourType};

const MOVING: u8 = 0;
const STRIKE: u8 = 1;
const PAUSE: u8 = 2;

const MAX_ATTACK_DISTANCE: f32 = 80.0;
const STRIKE_TIME: f64 = crate::ANIMATION_INTERVAL * 5.0;

pub struct WarriorAttackParams {
    warrior: BaseAnimated,
    target_position: Position<f32>,
    timestamp1: f64,
    timestamp2: f64,
    target_life: u8,
    compute_damage: bool,
    new_behaviour: Option<WarriorBehaviour>,
    state: BehaviourState,
}

enum AttackAnimation {
    Left1,
    Left2,
    Right1,
    Right2,
}

pub fn new(game: &mut DemoGameData, warrior: WorldObject, target: WorldObject) {
    let warrior_index = warrior.id as usize;
    let target_index = target.id as usize;

    let target_invalid = match target.ty {
        WorldObjectType::Sheep => target_index >= game.world.sheeps.len(),
        WorldObjectType::Structure => !is_enemy_structure(game, target_index),
        _ => false
    };

    if target_invalid || warrior.ty != WorldObjectType::Warrior || warrior_index >= game.world.warriors.len() {
        return;
    }

    game.world.warriors_behaviour[warrior_index] = WarriorBehaviour {
        ty: WarriorBehaviourType::Attack { target, timestamp1: 0.0, timestamp2: 0.0 },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, warrior_index: usize) {
    let mut params = read_params(game, warrior_index);

    if params.target_life == 0 {
        params.new_behaviour = Some(WarriorBehaviour::idle());
        write_params(game, warrior_index, &params);
        return;
    } 

    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVING) => moving(game, &mut params),
        BehaviourState::Running(STRIKE) => strike(game, &mut params),
        BehaviourState::Running(PAUSE) => pause(game, &mut params),
        _ => {},
    }
    write_params(game, warrior_index, &params);
}

fn init(game: &DemoGameData, params: &mut WarriorAttackParams) {
    let distance = params.warrior.position.distance(params.target_position);
    if distance > MAX_ATTACK_DISTANCE {
        params.warrior.animation = game.assets.animations.warrior.walk;
        params.warrior.current_frame = 0;
        params.state = BehaviourState::Running(MOVING);
        moving(game, params);
    } else {
        params.timestamp1 = game.global.time;
        params.state = BehaviourState::Running(STRIKE);
        strike(game, params);
    }
}

fn moving(game: &DemoGameData, params: &mut WarriorAttackParams) {
    use crate::behaviour::behaviour_shared::move_to;

    let mut target = params.target_position;
    target.y += 10.0;
    if params.warrior.position.x < target.x {
        target.x -= 60.0;
    } else {
        target.x += 60.0;
    }

    params.warrior.position = move_to(params.warrior.position, target, game.global.frame_delta);
    params.warrior.flipped = params.warrior.position.x > params.target_position.x;

    if params.warrior.position == target {
        params.warrior.current_frame = 0;
        params.timestamp1 = game.global.time;
        params.state = BehaviourState::Running(STRIKE);
    }
}

fn strike(game: &DemoGameData, params: &mut WarriorAttackParams) {

    fn select_animation(game: &DemoGameData, params: &WarriorAttackParams) -> AttackAnimation {
        let x1 = params.warrior.position.x;
        let x2 = params.target_position.x;

        match (x1 < x2, elapsed(game.global.time, params.timestamp1, STRIKE_TIME)) {
            (false, false) => AttackAnimation::Left1,
            (false, true) => AttackAnimation::Left2,
            (true, false) => AttackAnimation::Right1,
            (true, true) => AttackAnimation::Right2,
        }
    }

    fn set_attack_animation(game: &DemoGameData, params: &mut WarriorAttackParams) {
        let animation = select_animation(game, params);
        let animations = &game.assets.animations.warrior;
        params.warrior.animation = match animation {
            AttackAnimation::Right1 | AttackAnimation::Left1 => animations.strike_h1,
            AttackAnimation::Right2 | AttackAnimation::Left2 => animations.strike_h2
        };

        params.warrior.flipped = match animation {
            AttackAnimation::Left1 | AttackAnimation::Left2 => true,
            AttackAnimation::Right1 | AttackAnimation::Right2 => false,
        };
    }

    if params.warrior.position.distance(params.target_position) > MAX_ATTACK_DISTANCE {
        params.warrior.animation = game.assets.animations.warrior.idle;
        params.warrior.current_frame = 0;
        params.timestamp1 = game.global.time;
        params.state = BehaviourState::Running(PAUSE);
        return;
    }

    set_attack_animation(game, params);

    if params.warrior.current_frame == 5 {
        if elapsed(game.global.time, params.timestamp1,  STRIKE_TIME * 2.0) {
            params.warrior.animation = game.assets.animations.warrior.idle;
            params.timestamp1 = game.global.time;
            params.state = BehaviourState::Running(PAUSE);
        }

        if elapsed(game.global.time, params.timestamp2, 200.0) {
            params.compute_damage = true;
            params.timestamp2 = game.global.time;
        }
    }
}

fn pause(game: &DemoGameData, params: &mut WarriorAttackParams) {
    if elapsed(game.global.time, params.timestamp1, 500.0) {
        init(game, params);
    }
}

fn read_params(game: &DemoGameData, warrior_index: usize) -> WarriorAttackParams {
    let warrior = unsafe { game.world.warriors.get_unchecked(warrior_index) };
    let warrior_behaviour = unsafe { game.world.warriors_behaviour.get_unchecked(warrior_index) };
    let (target, timestamp1, timestamp2) = match warrior_behaviour.ty {
        WarriorBehaviourType::Attack { target, timestamp1, timestamp2 } => (target, timestamp1, timestamp2),
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    let target_position = crate::behaviour::behaviour_shared::target_position(game, target, false);
    let target_life = crate::behaviour::behaviour_shared::target_life(game, target);

    WarriorAttackParams {
        warrior: *warrior,
        target_position,
        timestamp1,
        timestamp2,
        target_life,
        compute_damage: false,
        new_behaviour: None,
        state: warrior_behaviour.state
    }
}

fn compute_damage(game: &mut DemoGameData, warrior_index: usize) {
    let warrior_behaviour = unsafe { game.world.warriors_behaviour.get_unchecked_mut(warrior_index) };
    let target = match warrior_behaviour.ty {
        WarriorBehaviourType::Attack { target, .. } => target,
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => crate::behaviour::sheep::strike(game, target_index, 5),
        WorldObjectType::Structure => crate::behaviour::behaviour_shared::damage_structure(game, target_index, 5),
        _ => {},
    }
}

fn write_params(game: &mut DemoGameData, warrior_index: usize, params: &WarriorAttackParams) {
    if params.compute_damage {
        compute_damage(game, warrior_index);
    }
    
    let warrior = unsafe { game.world.warriors.get_unchecked_mut(warrior_index) };
    let warrior_behaviour = unsafe { game.world.warriors_behaviour.get_unchecked_mut(warrior_index) };

    *warrior = params.warrior;

    match params.new_behaviour {
        Some(new_behaviour) => { *warrior_behaviour = new_behaviour; }
        None => { 
            warrior_behaviour.ty = match warrior_behaviour.ty {
                WarriorBehaviourType::Attack { target, .. } => WarriorBehaviourType::Attack { 
                    target,
                    timestamp1: params.timestamp1,
                    timestamp2: params.timestamp2,
                },
                _ => unsafe { ::std::hint::unreachable_unchecked(); }
            };

            warrior_behaviour.state = params.state; 
        }
    }
}
