pub mod pawn_move;
pub mod harvest_wood;
pub mod harvest_gold;
pub mod grab_resource;
pub mod hunt_sheep;
pub mod build_structure;

use crate::behaviour::BehaviourState;
use crate::shared::Position;
use crate::world::ResourceType;
use crate::DemoGameData;


#[derive(Copy, Clone)]
pub enum PawnBehaviourType {
    Idle,
    MoveTo { target_position: Position<f32> },
    HarvestWood { tree_id: u32, last_timestamp: f32 },
    HarvestGold { structure_id: u32, last_timestamp: f32 },
    GrabResource { resource_id: u32 },
    HuntSheep { sheep_id: u32, pause_timestamp: f32 },
    BuildStructure { structure_id: u32, last_timestamp: f32 },
}

#[derive(Copy, Clone)]
pub struct PawnBehaviour {
    pub ty: PawnBehaviourType,
    pub state: BehaviourState,
}

impl PawnBehaviour {

    pub fn idle() -> Self {
        PawnBehaviour {
            ty: PawnBehaviourType::Idle,
            state: BehaviourState::Initial
        }
    }

    pub fn cancel(game: &mut DemoGameData, pawn_id: u32, drop: bool) {
        let pawn_index = pawn_id as usize;

        if drop {
            if game.world.pawns_data[pawn_index].grabbed_resource().is_some() {
                drop_resource(game, pawn_index);
            }
        }

        let ty = game.world.pawns_behaviour[pawn_index].ty;
        match ty {
            PawnBehaviourType::HarvestWood { .. } => { harvest_wood::cancel(game, pawn_index); }
            PawnBehaviourType::HarvestGold { .. } => { harvest_gold::cancel(game, pawn_index); }
            _ => {},
        }
    }

}

pub fn idle(game: &mut DemoGameData, pawn_index: usize) {
    let world = &mut game.world;
    let behaviour = &mut world.pawns_behaviour[pawn_index];
    if let BehaviourState::Initial = behaviour.state {
        let pawn = &mut world.pawns[pawn_index];
        let pawn_data = world.pawns_data[pawn_index];
        
        pawn.animation = match pawn_data.grabbed_resource() {
            Some(_) => game.assets.animations.pawn.idle_hold,
            None => game.assets.animations.pawn.idle,
        };

        behaviour.state = BehaviourState::Running(0);
    }
}

fn drop_resource(game: &mut DemoGameData, pawn_index: usize) {
    let pawn_position = game.world.pawns[pawn_index].position;
    let pawn_data = &mut game.world.pawns_data[pawn_index];
    if pawn_data.grabbed_resource().is_none() {
        return;
    }

    let resource_index = pawn_data.grabbed_resource as usize;
    let resource_data = &mut game.world.resources_data[resource_index];
    let resource = &mut game.world.resources[resource_index];

    resource.position = pawn_position;
    resource.position.x = f32::ceil(resource.position.x);
    resource.position.y = f32::ceil(resource.position.y);

    resource.sprite = match resource_data.resource_type {
        ResourceType::Wood => game.assets.resources.wood,
        ResourceType::Food => game.assets.resources.meat,
        ResourceType::Gold => game.assets.resources.gold,
    };

    pawn_data.grabbed_resource = u32::MAX;
    resource_data.grabbed = false;
}
