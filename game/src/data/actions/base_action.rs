use crate::shared::Position;
use crate::world::{ResourceType, WorldObject, WorldObjectType};

/// Different action types. See [Action]
#[derive(Copy, Clone)]
pub enum ActionType {
    Nop,
    MoveActor { actor: WorldObject, target_position: Position<f32> },
    CutTree { pawn_id: u32, tree_id: u32 },
    SpawnResource { spawn_id: u32, resource_type: ResourceType },
    GrabResource { pawn_id: u32, resource_id: u32 },
}

#[derive(Copy, Clone)]
pub enum ActionState {
    Initial,
    Done,
    Running(u8),
}

/// An action is a function that gets spread across a certain amount of time
/// Active actions gets evaluated at every game step until their state is set to `Finalized`
#[derive(Copy, Clone)]
pub struct Action {
    pub ty: ActionType,
    pub state: ActionState,
}

impl Action {

    pub const fn from_type(ty: ActionType) -> Self {
        Action { 
            ty,
            state: ActionState::Initial
        }
    }

    pub fn nop() -> Self {
        Action {
            ty: ActionType::Nop,
            state: ActionState::Done,
        }
    }

    pub fn is_done(&self) -> bool {
        matches!(self.state, ActionState::Done)
    }

    /// Check if the action can exist at the same time as another action
    /// In this function, `self` is the old action and `other is the new one`
    pub fn is_incompatible(&self, other: Action) -> bool {
        match (self.ty, other.ty) {
            (ActionType::MoveActor { actor: actor1, .. }, ActionType::MoveActor { actor: actor2, .. }) => { actor1.id == actor2.id },
            (ActionType::MoveActor { actor, .. }, ActionType::CutTree { pawn_id, .. }) => { pawn_id == actor.id },

            (ActionType::CutTree { pawn_id, .. }, ActionType::MoveActor { actor, .. }) => { pawn_id == actor.id },
            (ActionType::CutTree { pawn_id: pid1, .. }, ActionType::GrabResource { pawn_id: pid2, .. }) => { pid1 == pid2 },

            (ActionType::GrabResource { pawn_id: pid1, .. }, ActionType::MoveActor { actor: actor1, .. }) => { actor1.id == pid1 && actor1.ty == WorldObjectType::Pawn } ,
            (ActionType::GrabResource { pawn_id: pid1, .. }, ActionType::CutTree { pawn_id: pid2, .. }) => { pid1 == pid2},
            _ => false,
        }
    }

    /// Return the value of `ActionState::Running(value)` or 255 if the state of the action is not running
    pub fn running_value(&self) -> u8 {
        match self.state {
            ActionState::Running(value) => value,
            _ => 255
        }
    }
}
