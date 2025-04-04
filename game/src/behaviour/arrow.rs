use crate::shared::{Position, AABB};
use crate::world::{World, WorldObject, WorldObjectType};
use crate::DemoGame;

pub(super) fn update_arrow(game: &mut DemoGame) {
    let world = &mut game.data.world;
    let arrow_count = world.arrows.len();
    let mut i = 0;
    let mut deleted_count = 0;

    while i < arrow_count {
        if world.arrows[i].deleted {
            deleted_count += 1;
            i += 1;
            continue;
        }

        let data = world.arrows_data[i];
        let position = world.arrows[i].position;

        let tip_position = position + data.arrow_tip_offset;
        let d = tip_position.distance(data.target_position);
        if d < 10.0 || d > 500.0 {
            if arrow_strike(world, tip_position, data.target_entity) || d > 500.0 {
                world.arrows[i].sprite = AABB::default();
                world.arrows[i].deleted = true;
            }
        }

        world.arrows[i].position = position + data.velocity;

        i += 1;
    }

    if deleted_count > 16 {
        clean_arrow(game);
    }
}

fn arrow_strike(world: &mut World, arrow_position: Position<f32>, target: WorldObject) -> bool {
    let mut touched = false;
    let index = target.id as usize;

    match target.ty {
        WorldObjectType::Sheep => {
            let aabb = world.sheeps[index].aabb();
            if aabb.point_inside(arrow_position) {
                super::sheep::strike(world, index, 5);
                touched = true;
            }
        }
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
