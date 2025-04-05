use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{BaseAnimated, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{WarriorBehaviour, WarriorBehaviourType};

const MOVING: u8 = 0;

pub struct WarriorMoveParams {
    warrior: BaseAnimated,
    target_position: Position<f32>,
    new_behaviour: Option<WarriorBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, warrior: WorldObject, target_position: Position<f32>) {
    let warrior_index = warrior.id as usize;

    if warrior.ty != WorldObjectType::Warrior || warrior_index >= game.world.warriors.len() {
        return;
    }

    game.world.warriors_behaviour[warrior_index] = WarriorBehaviour {
        ty: WarriorBehaviourType::MoveTo { target_position },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, warrior_index: usize) {
    let mut params = read_params(game, warrior_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVING) => moving(game, &mut params),
        _ => {},
    }

    write_params(game, warrior_index, &params);
}

fn init(game: &DemoGameData, params: &mut WarriorMoveParams) {
    params.warrior.animation = game.assets.animations.warrior.walk;
    params.state = BehaviourState::Running(MOVING);
}

fn moving(game: &DemoGameData, params: &mut WarriorMoveParams) {
    use crate::behaviour::behaviour_shared::move_to;

    let updated_position = move_to(params.warrior.position, params.target_position, game.global.frame_delta);
    if updated_position == params.target_position {
        params.new_behaviour = Some(WarriorBehaviour::idle());
    } else {
        params.warrior.flipped = params.warrior.position.x > params.target_position.x;
    }

    params.warrior.position = updated_position;
}

fn read_params(game: &DemoGameData, warrior_index: usize) -> WarriorMoveParams {
    let warrior = unsafe { game.world.warriors.get_unchecked(warrior_index) };
    let warrior_behaviour = unsafe { game.world.warriors_behaviour.get_unchecked(warrior_index) };
    let target_position = match warrior_behaviour.ty {
        WarriorBehaviourType::MoveTo { target_position } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    WarriorMoveParams {
        warrior: *warrior,
        target_position,
        new_behaviour: None,
        state: warrior_behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, warrior_index: usize, params: &WarriorMoveParams) {
    let warrior = unsafe { game.world.warriors.get_unchecked_mut(warrior_index) };
    let warrior_behaviour = unsafe { game.world.warriors_behaviour.get_unchecked_mut(warrior_index) };

    *warrior = params.warrior;

    match params.new_behaviour {
        Some(new_behaviour) => { *warrior_behaviour = new_behaviour; }
        None => { warrior_behaviour.state = params.state; }
    }
}
