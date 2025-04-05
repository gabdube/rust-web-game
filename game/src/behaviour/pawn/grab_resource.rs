use crate::behaviour::BehaviourState;
use crate::world::{BaseAnimated, PawnData, ResourceType, WorldObject, WorldObjectType};
use crate::DemoGameData;
use super::{PawnBehaviour, PawnBehaviourType};

const MOVE_TO_RESOURCE: u8 = 0;
const GRAB_RESOURCE: u8 = 1;

struct PawnGrabResourceParams {
    pawn: BaseAnimated,
    pawn_data: PawnData,
    resource_index: usize,
    new_behaviour: Option<PawnBehaviour>,
    state: BehaviourState,
}

pub fn new(game: &mut DemoGameData, pawn: WorldObject, resource: WorldObject) {
    match (pawn.ty, resource.ty) {
        (WorldObjectType::Pawn, WorldObjectType::Resource) => {},
        _ => { return; }
    }

    let pawn_index = pawn.id as usize;
    let resource_index = resource.id as usize;
    if resource_index >= game.world.resources.len() || pawn_index >= game.world.pawns.len() {
        return;
    }

    let resource_data = game.world.resources_data[resource_index];
    if resource_data.grabbed {
        return;
    }

    PawnBehaviour::cancel(game, pawn.id, true);

    game.world.pawns_behaviour[pawn_index] = PawnBehaviour {
        ty: PawnBehaviourType::GrabResource { resource_id: resource.id },
        state: BehaviourState::Initial,
    };
}

pub fn process(game: &mut DemoGameData, pawn_index: usize) {
    let mut params = read_params(game, pawn_index);
    match params.state {
        BehaviourState::Initial => init(game, &mut params),
        BehaviourState::Running(MOVE_TO_RESOURCE) => move_to_resource(game, &mut params),
        BehaviourState::Running(GRAB_RESOURCE) => grab_resource(game, &mut params),
        _ => {}
    }

    write_params(game, pawn_index, &params)
}

fn init(game: &DemoGameData, params: &mut PawnGrabResourceParams) {
    params.state = BehaviourState::Running(MOVE_TO_RESOURCE);
    params.pawn.animation = game.assets.animations.pawn.walk;
    move_to_resource(game, params);
}

fn move_to_resource(game: &DemoGameData, params: &mut PawnGrabResourceParams) {
    use crate::behaviour::behaviour_shared::move_to;
    
    let resource_data = game.world.resources_data[params.resource_index];
    if resource_data.grabbed {
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    let target_position = game.world.resources[params.resource_index].position;
    let updated_position = move_to(params.pawn.position, target_position, game.global.frame_delta);
    if updated_position == target_position {
        params.state = BehaviourState::Running(GRAB_RESOURCE);
    }

    params.pawn.position = updated_position;
    params.pawn.flipped = params.pawn.position.x > target_position.x;
}

fn grab_resource(game: &mut DemoGameData, params: &mut PawnGrabResourceParams) {
    let world = &mut game.world;
    let resource = &mut world.resources[params.resource_index];
    let resource_data = &mut world.resources_data[params.resource_index];
    if resource_data.grabbed {
        // Targeted resource was grabbed by another pawn
        params.new_behaviour = Some(PawnBehaviour::idle());
        return;
    }

    params.pawn.animation = game.assets.animations.pawn.idle_hold;
    params.pawn_data.grabbed_resource = params.resource_index as u32;

    resource_data.grabbed = true;

    resource.position = params.pawn.position;
    resource.position.y -= 60.0;

    resource.sprite = match resource_data.resource_type {
        ResourceType::Gold => game.assets.resources.gold_shadowless,
        ResourceType::Food => game.assets.resources.meat_shadowless,
        ResourceType::Wood => game.assets.resources.wood_shadowless,
    };

    params.new_behaviour = Some(PawnBehaviour::idle());
}

fn read_params(game: &DemoGameData, pawn_index: usize) -> PawnGrabResourceParams {
    let pawn = game.world.pawns.get(pawn_index);
    let pawn_data = game.world.pawns_data.get(pawn_index);
    let behaviour = game.world.pawns_behaviour.get(pawn_index);

    match (pawn, pawn_data, behaviour) {
        (Some(pawn), Some(pawn_data), Some(behaviour)) => {
            let resource_index = match behaviour.ty {
                PawnBehaviourType::GrabResource { resource_id } => resource_id as usize,
                _ => unsafe { ::std::hint::unreachable_unchecked()}
            };

            PawnGrabResourceParams {
                pawn: *pawn,
                pawn_data: *pawn_data,
                resource_index,
                new_behaviour: None,
                state: behaviour.state
            }
        },
        _  => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}

fn write_params(game: &mut DemoGameData, pawn_index: usize, params: &PawnGrabResourceParams) {
    let pawn = game.world.pawns.get_mut(pawn_index);
    let pawn_data = game.world.pawns_data.get_mut(pawn_index);
    let behaviour = game.world.pawns_behaviour.get_mut(pawn_index);

    match (pawn, pawn_data, behaviour) {
        (Some(pawn), Some(pawn_data), Some(behaviour)) => {
            *pawn = params.pawn;
            *pawn_data = params.pawn_data;

            match params.new_behaviour {
                Some(new_behaviour) => {
                    *behaviour = new_behaviour;
                },
                None => {
                    behaviour.state = params.state;
                }
            }
        },
        _ => {
            unsafe { ::std::hint::unreachable_unchecked(); }
        }
    }
}
