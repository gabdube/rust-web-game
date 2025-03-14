use std::hint::unreachable_unchecked;

use crate::data::actions::{self, Action, ActionType};
use crate::DemoGame;

/// Manages actions. Actions are bits of code that must be evaluated every frame.
pub struct ActionsManager {
    pub active: Vec<Action>,
    pub queued: Vec<Action>,
    pub to_cancel: Vec<Action>,
    pub completed_actions: u32,
}

pub fn update(game: &mut DemoGame) {
    update_actions_from_game_state(game);
    cancel_actions(game);
    process_active_actions(game);

    // Cleanup (todo cleanup queued actions)
    if game.actions.completed_actions > 16 {
        game.actions.active.retain(|action| !action.is_completed() );
    }
}

/// Removes all active actions in the action manager that are incompatible with `filter_action`
/// For example, two move action targeting the same unit cannot exist at the same time.
/// Cancelled actions are added to the `to_cancel` list of the manager. See `cancel_actions`.
fn filter_incompatible_actions(manager: &mut ActionsManager, filter_action: Action) {
    for action2 in manager.active.iter_mut() {
        if action2.is_incompatible(filter_action) {
            manager.completed_actions += 1;
            manager.to_cancel.push(*action2);
            *action2 = Action::completed();
        }
    }
}

/// Copy the newly added actions from the game state to the manager state
fn update_actions_from_game_state(game: &mut DemoGame) {
    /// Push the next action into the manager queued actions list and return its index
    fn next_action_queued(manager: &mut ActionsManager, actions: &mut [Action], action_index: usize) -> u32 {
        let queued_index = manager.queued.len();

        let mut action = Action::completed();
        ::std::mem::swap(&mut action, &mut actions[action_index]);

        manager.queued.push(action);

        if action.next != u32::MAX {
            manager.queued[queued_index].next = next_action_queued(manager, actions, action.next as usize);
        }

        queued_index as u32
    }

    /// Push the next action into the manager active actions list and return its index
    fn next_action(manager: &mut ActionsManager, actions: &mut [Action], action_index: usize) {
        let mut action = Action::completed();
        ::std::mem::swap(&mut action, &mut actions[action_index]);

        filter_incompatible_actions(manager, action);

        if action.next != u32::MAX {
            action.next = next_action_queued(manager, actions, action.next as usize);
        }

        manager.active.push(action);
    }
    
    let manager = &mut game.actions;
    let actions = &mut game.data.actions.new;
    let mut action_index = 0;
    while action_index < actions.len() {
        if !actions[action_index].is_completed() {
            next_action(manager, actions, action_index);
        }

        action_index += 1;
    }

    actions.clear()
}

/// Execute the cancel action callback for all actions cancelled
fn cancel_actions(game: &mut DemoGame) {
    let data = &mut game.data;
    for action in game.actions.to_cancel.iter_mut() {
        if action.is_completed() {
            continue;
        }

        match action.ty {
            ActionType::MoveActor { .. } => { actions::move_actor::cancel(data, action); },
            ActionType::CutTree { .. } => { actions::cut_tree::cancel(data, action); },
            ActionType::SpawnResource { .. } => { actions::spawn_resource::cancel(data, action); },
            ActionType::Completed => {},
        }
    }

    game.actions.to_cancel.clear();
}

/// Execute the callback for each active action.
/// If an action is completed and it as another queued, register the queued action.
fn process_active_actions(game: &mut DemoGame) {
    let data = &mut game.data;
    let manager = &mut game.actions;

    let mut action_index = 0;
    while action_index < manager.active.len() {
        let action = &mut manager.active[action_index];
        if action.is_completed() {
            action_index += 1;
            continue;
        }

        match action.ty {
            ActionType::MoveActor { .. } => { actions::move_actor::process(data, action); },
            ActionType::CutTree { .. } => { actions::cut_tree::process(data, action); },
            ActionType::SpawnResource { .. } => { actions::spawn_resource::process(data, action); },
            ActionType::Completed => unsafe { unreachable_unchecked() },
        }

        if action.is_finalized() {
            if action.next == u32::MAX {
                *action = Action::completed();
                manager.completed_actions += 1;
            } else {
                let next_index = action.next as usize;
                let new_action = manager.queued[next_index];
                filter_incompatible_actions(manager, new_action);
                manager.active[action_index] = new_action;
                manager.queued[next_index] = Action::completed();
            }
        }

        action_index += 1;
    }
}

impl Default for ActionsManager {
    fn default() -> Self {
        ActionsManager {
            active: Vec::with_capacity(16),
            queued: Vec::with_capacity(16),
            to_cancel: Vec::with_capacity(16),
            completed_actions: 0,
        }
    }
}

impl crate::store::SaveAndLoad for ActionsManager {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.active);
        writer.write_slice(&self.queued);
        writer.write_slice(&self.to_cancel);
        writer.write_u32(self.completed_actions);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let active = reader.read_slice().to_vec();
        let queued = reader.read_slice().to_vec();
        let to_cancel = reader.read_slice().to_vec();
        let completed_actions = reader.read_u32();
        ActionsManager {
            active,
            queued,
            to_cancel,
            completed_actions,
        }
    }
}
