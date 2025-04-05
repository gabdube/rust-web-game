use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::{BaseAnimated, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{ArcherBehaviour, ArcherBehaviourType};

const MOVING: u8 = 0;

pub struct ArcherMoveParams {
    archer: BaseAnimated,
    target_position: Position<f32>,
    new_behaviour: Option<ArcherBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, archer: WorldObject, target_position: Position<f32>) {
    let archer_index = archer.id as usize;

    if archer.ty != WorldObjectType::Archer || archer_index >= game.world.archers.len() {
        return;
    }

    game.world.archers_behaviour[archer_index] = ArcherBehaviour {
        ty: ArcherBehaviourType::MoveTo { target_position },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, archer_index: usize) {
    let mut params = read_params(game, archer_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVING) => moving(game, &mut params),
        _ => {},
    }

    write_params(game, archer_index, &params);
}

fn init(game: &DemoGameData, params: &mut ArcherMoveParams) {
    params.archer.animation = game.assets.animations.archer .walk;
    params.state = BehaviourState::Running(MOVING);
    moving(game, params);
}

fn moving(game: &DemoGameData, params: &mut ArcherMoveParams) {
    use crate::behaviour::behaviour_shared::move_to;

    let current_position = params.archer.position;
    let updated_position = move_to(current_position, params.target_position, game.global.frame_delta);
    if updated_position == params.target_position {
        params.new_behaviour = Some(ArcherBehaviour::idle());
    } else {
        params.archer.flipped = current_position.x > params.target_position.x;
    }

    params.archer.position = updated_position;
}

fn read_params(game: &DemoGameData, archer_index: usize) -> ArcherMoveParams {
    let archer = unsafe { game.world.archers.get_unchecked(archer_index) };
    let archer_behaviour = unsafe { game.world.archers_behaviour.get_unchecked(archer_index) };
    let target_position = match archer_behaviour.ty {
        ArcherBehaviourType::MoveTo { target_position } => target_position,
        _ => unsafe { ::std::hint::unreachable_unchecked(); }
    };

    ArcherMoveParams {
        archer: *archer,
        target_position,
        new_behaviour: None,
        state: archer_behaviour.state
    }
}

fn write_params(game: &mut DemoGameData, archer_index: usize, params: &ArcherMoveParams) {
    let archer = unsafe { game.world.archers.get_unchecked_mut(archer_index) };
    let archer_behaviour = unsafe { game.world.archers_behaviour.get_unchecked_mut(archer_index) };

    *archer = params.archer;

    match params.new_behaviour {
        Some(new_behaviour) => { *archer_behaviour = new_behaviour; }
        None => { archer_behaviour.state = params.state; }
    }
}
