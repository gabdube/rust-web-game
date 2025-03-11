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

    /// Cancels all actions
    /// If the action has queued work, it also cancels all of it.
    pub fn cancel(&mut self, action: Action) {
        // TODO
        for action2 in self.active.iter_mut() {
            if action2.ty == action.ty {
                action2.ty = ActionType::Completed;
                action2.state = ActionState::Finalized;
            }
        }
    }

}

impl DemoGame {

    pub fn update_actions(&mut self) {
        let manager = &mut self.actions;
        let frame_delta = self.timing.frame_delta;

        for action in manager.active.iter_mut() {
            if matches!(action.ty, ActionType::Completed) {
                continue;
            }

            match action.ty {
                ActionType::MovePawn { id, target } => {
                    let mut params = move_pawn::MovePawnParams {
                        world: &mut self.world,
                        assets: &self.assets,
                        pawn_id: id,
                        frame_delta,
                        target
                    };
                    move_pawn::move_pawn(&mut action.state, &mut params);
                },
                ActionType::CutTree { pawn_id, tree_id } => {
                    let mut params = cut_tree::CutTreeParams {
                        world: &mut self.world,
                        assets: &self.assets,
                        pawn_id,
                        tree_id,
                    };
                    cut_tree::cut_tree(&mut action.state, &mut params);
                },
                ActionType::Completed => unreachable!(),
            }

            if matches!(action.state, ActionState::Finalized) {
                action.ty = ActionType::Completed;
                if action.next != u32::MAX {
                    ::std::mem::swap(action, &mut manager.queued[action.next as usize]);
                }
            }
        }
    }

}

impl Default for ActionsManager {
    fn default() -> Self {
        ActionsManager {
            active: Vec::with_capacity(32),
            queued: Vec::with_capacity(32)
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
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let active = reader.read_slice().to_vec();
        let queued = reader.read_slice().to_vec();
        ActionsManager {
            active,
            queued,
        }
    }
}
