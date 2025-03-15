use crate::shared::Position;
use crate::world::{WorldObject, ResourceType};

/// Different action types. See [Action]
#[derive(Copy, Clone)]
pub enum ActionType {
    Completed,
    MoveActor { actor: WorldObject, target_position: Position<f32> },
    CutTree { pawn_id: u32, tree_id: u32 },
    SpawnResource { spawn_id: u32, resource_type: ResourceType },
    GrabResource { pawn_id: u32, resource_id: u32 },
}

#[derive(Copy, Clone)]
pub enum ActionState {
    Initial,
    Running,
    Finalizing,
    Finalized,
}

/// An action is a function that gets spread across a certain amount of time
/// Active actions gets evaluated at every game step until their state is set to `Finalized`
#[derive(Copy, Clone)]
pub struct Action {
    pub ty: ActionType,
    pub next: u32,
    pub state: ActionState,
}

impl Action {

    pub const fn from_type(ty: ActionType) -> Self {
        Action { 
            ty,
            next: u32::MAX,
            state: ActionState::Initial
        }
    }

    pub fn completed() -> Self {
        Action {
            ty: ActionType::Completed,
            next: u32::MAX,
            state: ActionState::Finalized,
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.ty, ActionType::Completed)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(self.state, ActionState::Finalized)
    }

    /// Check if the action can exist at the same time as another action
    pub fn is_incompatible(&self, other: Action) -> bool {
        match (self.ty, other.ty) {
            (ActionType::MoveActor { actor: actor1, .. }, ActionType::MoveActor { actor: actor2, .. }) => { actor1.id == actor2.id },
            (ActionType::CutTree { pawn_id, .. }, ActionType::MoveActor { actor, .. }) => { pawn_id == actor.id },
            (ActionType::GrabResource { pawn_id: pid1, resource_id: rid1 }, ActionType::GrabResource { pawn_id: pid2, resource_id: rid2 }) => {
                pid1 == pid2 || rid1 == rid2
            },
            _ => false,
        }
    }
}
