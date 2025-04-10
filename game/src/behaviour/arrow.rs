use crate::shared::{Position, AABB};
use crate::world::{WorldObject, WorldObjectType};
use crate::{DemoGame, DemoGameData};

pub(super) fn update_arrow(game: &mut DemoGame) {
    let data = &mut game.data;
    let arrow_count = data.world.arrows.len();
    let mut i = 0;
    let mut deleted_count = 0;

    while i < arrow_count {
        if data.world.arrows[i].deleted {
            deleted_count += 1;
            i += 1;
            continue;
        }

        let arrow_data = data.world.arrows_data[i];
        let position = data.world.arrows[i].position;
        let tip_position = position + arrow_data.arrow_tip_offset;
        
        let d = tip_position.distance(arrow_data.target_position);

        if d < 10.0 || d > 500.0 {
            if d > 500.0 || arrow_strike(data, tip_position, arrow_data.target_entity) {
                data.world.arrows[i].sprite = AABB::default();
                data.world.arrows[i].deleted = true;
            }
        }

        data.world.arrows[i].position = position + arrow_data.velocity;

        i += 1;
    }

    if deleted_count > 16 {
        clean_arrow(game);
    }
}

fn arrow_strike(data: &mut DemoGameData, arrow_position: Position<f32>, target: WorldObject) -> bool {
    let mut touched = false;
    let index = target.id as usize;

    match target.ty {
        WorldObjectType::Sheep => {
            let aabb = data.world.sheeps[index].aabb();
            if aabb.point_inside(arrow_position) {
                super::sheep::strike(data, index, 5);
                touched = true;
            }
        },
        WorldObjectType::Structure => {
            let aabb = data.world.structures[index].aabb();
            if aabb.point_inside(arrow_position) {
                crate::behaviour::behaviour_shared::damage_structure(data, index, 3);
                touched = true;
            }
        },
        _ => {},
    }

    touched
}

fn clean_arrow(game: &mut DemoGame) {
    let world = &mut game.data.world;
    let mut arrow_iter = world.arrows.iter().map(|arrow| !arrow.deleted );
    world.arrows_data.retain(|_| arrow_iter.next().unwrap_or(true) );
    world.arrows.retain(|arrow| !arrow.deleted );
}
