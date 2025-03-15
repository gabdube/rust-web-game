use crate::data::DemoGameData;
use crate::shared::Position;
use crate::world::{WorldObject, WorldObjectType};
use super::{Action, ActionType, ActionState};

const MOVING: u8 = 0;
const STOPPING: u8 = 1;

pub fn new(game: &mut DemoGameData, actor: WorldObject, target_position: Position<f32>) {
    game.actions.push(Action::from_type(ActionType::MoveActor { actor, target_position }));
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        return;
    }

    done(game, action);
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Done;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running(MOVING) => run(game, action),
        ActionState::Running(STOPPING) => done(game, action),
        _ => {},
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let (actor, _) = params(action);
    let index = actor.id as usize;
    match actor.ty {
        WorldObjectType::Pawn => {
            let animation = if game.world.pawns_data[index].grabbed_resource().is_some() {
                game.assets.animations.pawn.walk_hold
            } else {
                game.assets.animations.pawn.walk
            };

            game.world.pawns[index].animation = animation;
            action.state = ActionState::Running(MOVING);
        },
        _ => {
            action.state = ActionState::Done;
        }
    }

}

fn run(game: &mut DemoGameData, action: &mut Action) {
    let (actor, target_position) = params(action);
    let index = actor.id as usize;
    let actor_data = match actor.ty {
        WorldObjectType::Pawn => &mut game.world.pawns[index],
        _ => { return; }
    };

    let current_position = actor_data.position;
    let updated_position = super::actions_shared::move_to(current_position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        action.state = ActionState::Running(STOPPING);
    }

    actor_data.position = updated_position;
    actor_data.flipped = current_position.x > target_position.x;
}

fn done(game: &mut DemoGameData, action: &mut Action) {
    let (actor, _) = params(action);
    let index = actor.id as usize;
    match actor.ty {
        WorldObjectType::Pawn => {
            let animation = if game.world.pawns_data[index].grabbed_resource().is_some() {
                game.assets.animations.pawn.idle_hold
            } else {
                game.assets.animations.pawn.idle
            };

            game.world.pawns[index].animation = animation;
        },
        _ => {}
    }

    action.state = ActionState::Done;
}

fn validate(game: &mut DemoGameData, action: &mut Action) -> bool {
    let (actor, _) = params(action);
    let index = actor.id as usize;
    match actor.ty {
        WorldObjectType::Pawn => { index < game.world.pawns.len() },
        _ => {
            dbg!("Not implemented for {:?}", actor.ty);
            false
        }
    }
}

#[inline]
fn params(action: &mut Action) -> (WorldObject, Position<f32>) {
    match action.ty {
        ActionType::MoveActor { actor, target_position } => (actor, target_position),
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
