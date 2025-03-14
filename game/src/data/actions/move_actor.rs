use crate::data::DemoGameData;
use crate::shared::{Position, pos};
use crate::world::{WorldObject, WorldObjectType};
use super::{Action, ActionType, ActionState};

pub fn new(game: &mut DemoGameData, actor: WorldObject, target_position: Position<f32>) {
    game.actions.push(Action::from_type(ActionType::MoveActor { actor, target_position }));
}

pub fn cancel(game: &mut DemoGameData, action: &mut Action) {
    done(game, action);
}

pub fn process(game: &mut DemoGameData, action: &mut Action) {
    if !validate(game, action) {
        action.state = ActionState::Finalized;
        return;
    }

    match action.state {
        ActionState::Initial => init(game, action),
        ActionState::Running => run(game, action),
        ActionState::Finalizing => done(game, action),
        ActionState::Finalized => {}
    }
}

fn init(game: &mut DemoGameData, action: &mut Action) {
    let (actor, _) = params(action);
    let index = actor.id as usize;
    match actor.ty {
        WorldObjectType::Pawn => {
            game.world.pawns[index].animation = game.assets.animations.pawn.walk;
            action.state = ActionState::Running;
        },
        _ => {
            dbg!("Not implemented for {:?}", actor.ty);
            action.state = ActionState::Finalized;
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
    let angle = f32::atan2(target_position.y - current_position.y, target_position.x - current_position.x);
    let speed = 0.2f32 * game.global.frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current_position.x + move_x, current_position.y + move_y);

    if (move_x > 0.0 && updated_position.x > target_position.x) || (move_x < 0.0 && updated_position.x < target_position.x) {
        updated_position.x = target_position.x;
    }

    if (move_y > 0.0 && updated_position.y > target_position.y) || (move_y < 0.0 && updated_position.y < target_position.y) {
        updated_position.y = target_position.y;
    }

    if updated_position == target_position {
        action.state = ActionState::Finalizing;
    }

    actor_data.position = updated_position;
    actor_data.flipped = move_x < 0.0;
}

fn done(game: &mut DemoGameData, action: &mut Action) {
    let (actor, _) = params(action);
    let index = actor.id as usize;
    match actor.ty {
        WorldObjectType::Pawn => {
            game.world.pawns[index].animation = game.assets.animations.pawn.idle;
        },
        _ => {}
    }

    action.state = ActionState::Finalized;
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

fn params(action: &mut Action) -> (WorldObject, Position<f32>) {
    match action.ty {
        ActionType::MoveActor { actor, target_position } => (actor, target_position),
        _ => unsafe { ::std::hint::unreachable_unchecked()}
    }
}
