use crate::assets::Assets;
use crate::shared::{Position, pos};
use crate::world::{World, WorldObject, WorldObjectType};
use crate::DemoGame;

/// Different action types. See [Action]
#[derive(Copy, Clone)]
pub enum ActionType {
    Completed,
    MovePawn { id: u32, target: Position<f32> }
}

#[derive(Copy, Clone)]
pub enum ActionState {
    Initial,
    Running,
    Finalizing,
    Finalized,
}

impl ActionState {
    pub fn finalizing(&self) -> bool {
        matches!(self, ActionState::Finalizing)
    }
}

/// An action is a function that gets spread across a certain amount of time
/// Active actions gets evaluated at every game steps until they are completed
#[derive(Copy, Clone)]
pub struct Action {
    ty: ActionType,
    state: ActionState,
}

impl Action {

    pub fn move_to(obj: WorldObject, position: Position<f32>) -> Self {
        let ty = match obj.ty {
            WorldObjectType::Pawn => ActionType::MovePawn { id: obj.id, target: position },
            _ => ActionType::Completed
        };

        Action { ty, state: ActionState::Initial }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.ty, ActionType::Completed)
    }
    
}

/// Manages actions objects
pub struct ActionsManager {
    active: Vec<Action>,
}

impl ActionsManager {

    pub fn push(&mut self, action: Action) {
        if matches!(action.ty, ActionType::Completed) {
            return;
        }

        let last_completed = self.active.iter().position(|action| action.is_completed() );
        match last_completed {
            Some(index) => { self.active[index] = action; },
            None => self.active.push(action)
        }
    }

}

impl DemoGame {

    pub fn update_actions(&mut self) {
        let manager = &mut self.actions;
        let world = &mut self.world;
        let frame_delta = self.timing.frame_delta;

        for action in manager.active.iter_mut() {
            match action.ty {
                ActionType::MovePawn { id, target } => {
                    match action.state {
                        ActionState::Initial => move_pawn_initial(world, &self.assets, id),
                        ActionState::Running => move_pawn_running(world, &mut action.state, frame_delta, id, target),
                        ActionState::Finalizing => move_pawn_finalize(world, &self.assets, &mut action.state, id),
                        ActionState::Finalized => {},
                    }
                },
                ActionType::Completed => {},
            }

            match action.state {
                ActionState::Initial => { action.state = ActionState::Running; },
                ActionState::Finalized => { action.ty = ActionType::Completed; },
                _ => {},
            }
        }
    }

}

fn move_pawn_initial(world: &mut World, assets: &Assets, id: u32) {
    let index = id as usize;
    world.pawns[index].animation = assets.animations.pawn.walk;
}

fn move_pawn_running(world: &mut World, state: &mut ActionState, frame_delta: f32, id: u32, target: Position<f32>) {
    let index = id as usize;
    let current_position = world.pawns[index].position;

    let angle = f32::atan2(target.y - current_position.y, target.x - current_position.x);
    let speed = 0.2f32 * frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current_position.x + move_x, current_position.y + move_y);

    world.pawns[index].flipped = move_x < 0.0;

    if (move_x > 0.0 && updated_position.x > target.x) || (move_x < 0.0 && updated_position.x < target.x) {
        updated_position.x = target.x;
    }

    if (move_y > 0.0 && updated_position.y > target.y) || (move_y < 0.0 && updated_position.y < target.y) {
        updated_position.y = target.y;
    }

    if updated_position == target {
        *state = ActionState::Finalizing;
    }

    world.pawns[index].position = updated_position;
}

fn move_pawn_finalize(world: &mut World, assets: &Assets, state: &mut ActionState, id: u32) {
    let index = id as usize;
    world.pawns[index].animation = assets.animations.pawn.idle;
    *state = ActionState::Finalized;
}

impl Default for ActionsManager {
    fn default() -> Self {
        ActionsManager {
            active: Vec::with_capacity(32),
        }
    }
}

impl crate::store::SaveAndLoad for ActionsManager {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.active);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let active = reader.read_slice().to_vec();
        ActionsManager {
            active
        }
    }
}
