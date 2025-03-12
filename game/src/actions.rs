mod move_pawn;
mod cut_tree;

use std::u32;

use crate::shared::Position;
use crate::world::{WorldObject, WorldObjectType};
use crate::DemoGame;

/// Different action types. See [Action]
#[derive(Copy, Clone)]
pub enum ActionType {
    Completed,
    MovePawn { id: u32, target: Position<f32> },
    CutTree { pawn_id: u32, tree_id: u32 }
}

#[derive(Copy, Clone)]
pub enum ActionState {
    Initial,
    Running,
    Finalizing,
    Finalized,
}

/// An action is a function that gets spread across a certain amount of time
/// Active actions gets evaluated at every game steps until they are completed
#[derive(Copy, Clone)]
pub struct Action {
    ty: ActionType,
    next: u32,
    state: ActionState,
}

impl Action {

    pub fn move_to(obj: WorldObject, position: Position<f32>) -> Self {
        let ty = match obj.ty {
            WorldObjectType::Pawn => ActionType::MovePawn { id: obj.id, target: position },
            _ => ActionType::Completed
        };

        Action { ty, next: u32::MAX, state: ActionState::Initial }
    }

    pub fn cut_tree(pawn: WorldObject, tree: WorldObject) -> Self {
        let ty = match [pawn.ty, tree.ty] {
            [WorldObjectType::Pawn, WorldObjectType::Tree] => ActionType::CutTree { pawn_id: pawn.id, tree_id: tree.id },
            _ => ActionType::Completed
        };

        Action { ty, next: u32::MAX, state: ActionState::Initial }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.ty, ActionType::Completed)
    }
    
}

/// Manages actions. Actions are bits of code that must be evaluated every frame.
pub struct ActionsManager {
    active: Vec<Action>,
    queued: Vec<Action>,
    cancelled: Vec<Action>,
}

impl ActionsManager {

    /// Push a new action into the action manager. If the action type is "Completed", this does nothing
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

    /// Push a new action into the action manager in a queued state and return the ID of the queued task.
    /// Queued task will be executed after its parent task is fully executed
    pub fn push_queued(&mut self, action: Action) -> u32 {
        let id;

        let last_completed = self.queued.iter().position(|action| action.is_completed() );
        match last_completed {
            Some(index) => { 
                id = index as u32;
                self.queued[index] = action;
            },
            None => {
                id = self.queued.len() as u32;
                self.queued.push(action)
            }
        }

        id
    }

    /// Push an action then execute a second action when it finish
    /// If the base action type is "Completed", this does nothing
    pub fn push_and_queue(&mut self, mut action: Action, then: Action) {
        if matches!(action.ty, ActionType::Completed) {
            return;
        }

        let queued_index = self.push_queued(then);
        action.next = queued_index;

        let last_completed = self.active.iter().position(|action| action.is_completed() );
        match last_completed {
            Some(index) => { 
                self.active[index] = action;
            },
            None => {
                self.active.push(action)
            }
        }
    }

    /// Cancels all actions related to `object`
    /// Queued actions will be set to `Completed` without executing the on cancelled logic
    pub fn cancel_object_actions(&mut self, object: WorldObject) {
        match object.ty {
            WorldObjectType::Pawn => {
                for action in self.active.iter_mut() {
                    let cancel = match action.ty {
                        ActionType::MovePawn { id, .. } => id == object.id,
                        ActionType::CutTree { pawn_id, .. } => pawn_id == object.id,
                        _ => false
                    };

                    if cancel {
                        if action.next != u32::MAX {
                            dbg!("TODO: cancel queued actions");
                        }

                        self.cancelled.push(*action);

                        action.ty = ActionType::Completed;
                        action.state = ActionState::Finalized;
                        action.next = u32::MAX;
                    }
                }
            },
            _ => {},
        }
    }

}

pub fn update(game: &mut DemoGame) {
    cancel_actions(game);
    process_active_actions(game);
}

fn cancel_actions(game: &mut DemoGame) {
    let data = &mut game.data;
    let actions = &mut game.actions;

    for action in actions.cancelled.iter_mut() {
        match action.ty {
            ActionType::Completed => {},
            ActionType::MovePawn { id, .. } => move_pawn::cancel(data, id),
            ActionType::CutTree { pawn_id, tree_id } => cut_tree::cancel(data, pawn_id, tree_id),
        }
    }

    actions.cancelled.clear();
}

fn process_active_actions(game: &mut DemoGame) {
    let data = &mut game.data;
    let actions = &mut game.actions;

    for action in actions.active.iter_mut() {
        if matches!(action.ty, ActionType::Completed) {
            continue;
        }

        match action.ty {
            ActionType::MovePawn { id, target } => {
                move_pawn::move_pawn(data, action, id, target);
            },
            ActionType::CutTree { pawn_id, tree_id } => {
                cut_tree::cut_tree(data, action, pawn_id, tree_id);
            },
            ActionType::Completed => unreachable!(),
        }

        if matches!(action.state, ActionState::Finalized) {
            action.ty = ActionType::Completed;
            if action.next != u32::MAX {
                let next_index = action.next as usize;
                action.next = u32::MAX;
                ::std::mem::swap(action, &mut actions.queued[next_index]);
            }
        }
    }
}

impl Default for ActionsManager {
    fn default() -> Self {
        ActionsManager {
            active: Vec::with_capacity(32),
            queued: Vec::with_capacity(32),
            cancelled: Vec::with_capacity(16),
        }
    }
}

impl PartialEq for ActionType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ActionType::Completed, ActionType::Completed) => true,
            (ActionType::MovePawn { id: id1, .. }, ActionType::MovePawn { id: id2, .. }) => { id1 == id2 },
            _ => false,
        }
    }
}

impl crate::store::SaveAndLoad for ActionsManager {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.active);
        writer.write_slice(&self.queued);
        writer.write_slice(&self.cancelled);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let active = reader.read_slice().to_vec();
        let queued = reader.read_slice().to_vec();
        let cancelled = reader.read_slice().to_vec();
        ActionsManager {
            active,
            queued,
            cancelled,
        }
    }
}
