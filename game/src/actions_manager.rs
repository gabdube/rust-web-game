use crate::data::actions::{self, Action, ActionType};
use crate::DemoGame;

/// Manages actions. Actions are bits of code that must be evaluated every frame.
pub struct ActionsManager {
    pub active: Vec<Action>,
    pub to_cancel: Vec<Action>,
    pub completed_actions: u32,
}

pub fn update(game: &mut DemoGame) {
    update_actions_from_game_state(game);
    cancel_actions(game);
    process_active_actions(game);

    if game.actions.completed_actions > 16 {
        game.actions.active.retain(|action| !action.is_done() );
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
            *action2 = Action::nop();
        }
    }
}

/// Copy the newly added actions from the game state to the manager state
fn update_actions_from_game_state(game: &mut DemoGame) {
    let manager = &mut game.actions;
    let actions = &mut game.data.actions.new;

    let mut action_index = 0;
    while action_index < actions.len() {
        let action = actions[action_index];
        filter_incompatible_actions(manager, action);
        manager.active.push(action);
        action_index += 1;
    }

    actions.clear()
}

/// Execute the cancel action callback for all actions cancelled
fn cancel_actions(game: &mut DemoGame) {
    let data = &mut game.data;
    for action in game.actions.to_cancel.iter_mut() {
        match action.ty {
            ActionType::MoveActor { .. } => { actions::move_actor::cancel(data, action); },
            ActionType::CutTree { .. } => { actions::cut_tree::cancel(data, action); },
            ActionType::SpawnResource { .. } => { actions::spawn_resource::cancel(data, action); },
            ActionType::GrabResource { .. } => { actions::grab_resource::cancel(data, action); },
            ActionType::StartMining { .. } => { actions::start_mining::cancel(data, action); },
            ActionType::Nop => {},
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
        if action.is_done() {
            action_index += 1;
            continue;
        }

        match action.ty {
            ActionType::MoveActor { .. } => { actions::move_actor::process(data, action); },
            ActionType::CutTree { .. } => { actions::cut_tree::process(data, action); },
            ActionType::SpawnResource { .. } => { actions::spawn_resource::process(data, action); },
            ActionType::GrabResource { .. } => { actions::grab_resource::process(data, action); },
            ActionType::StartMining { .. } => { actions::start_mining::process(data, action); },
            ActionType::Nop => { }
        }

        if action.is_done() {
            manager.completed_actions += 1;
            manager.active[action_index] = Action::nop();
        }

        action_index += 1;
    }
}

impl Default for ActionsManager {
    fn default() -> Self {
        ActionsManager {
            active: Vec::with_capacity(16),
            to_cancel: Vec::with_capacity(16),
            completed_actions: 0,
        }
    }
}

impl crate::store::SaveAndLoad for ActionsManager {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.active);
        writer.write_slice(&self.to_cancel);
        writer.write_u32(self.completed_actions);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let active = reader.read_slice().to_vec();
        let to_cancel = reader.read_slice().to_vec();
        let completed_actions = reader.read_u32();
        ActionsManager {
            active,
            to_cancel,
            completed_actions,
        }
    }
}
