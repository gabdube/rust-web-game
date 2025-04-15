//! Shared logic between actions
use crate::shared::{Position, pos};
use crate::world::{WorldObject, WorldObjectType, StructureData};
use crate::DemoGameData;

pub fn move_to(current: Position<f32>, target: Position<f32>, frame_delta: f32) -> Position<f32> {
    move_to_with_speed(current, target, frame_delta, 0.2)
}

pub fn move_to_with_speed(current: Position<f32>, target: Position<f32>, frame_delta: f32, base_speed: f32) -> Position<f32> {
    let angle = f32::atan2(target.y - current.y, target.x - current.x);
    let speed = base_speed * frame_delta;
    let move_x = speed * f32::cos(angle);
    let move_y = speed * f32::sin(angle);
    let mut updated_position = pos(current.x + move_x, current.y + move_y);

    if f32::abs(updated_position.x - target.x) < 1.0 {
        updated_position.x = target.x;
    }

    if f32::abs(updated_position.y - target.y) < 1.0 {
        updated_position.y = target.y;
    }

    updated_position
}

#[inline(always)]
pub fn elapsed(time: f64, timestamp: f64, timer: f64) -> bool {
    time - timestamp > timer
}

pub fn target_life(game: &DemoGameData, target: WorldObject) -> u8 {
    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => game.world.sheeps_data[target_index].life,
        WorldObjectType::Structure => {
            match &game.world.structures_data[target_index] {
                StructureData::Castle(data) => data.hp,
                StructureData::Tower(data) => data.hp,
                StructureData::House(data) => data.hp,
                StructureData::GoblinHut(data) => data.hp,
                StructureData::GoldMine(_) => 0,
            }
        }
        _ => unimplemented!()
    }
}

pub fn target_position(game: &DemoGameData, target: WorldObject, center: bool) -> Position<f32> {
    let mut base;
    let height;
    let target_index = target.id as usize;
    match target.ty {
        WorldObjectType::Sheep => { 
            let sheep = game.world.sheeps[target_index];
            base = sheep.position;
            height = sheep.aabb().height();
        },
        WorldObjectType::Structure => {
            let structure = game.world.structures[target_index];
            base = structure.position;
            height = structure.aabb().height();
        }
        _ => unimplemented!()
    }

    if center {
        base.y -= height * 0.5;
    }

    base
}

/// Returns false if `structure_index` is out of data range
pub fn is_enemy_structure(game: &DemoGameData, structure_index: usize) -> bool {
    let mut is_enemy = false;
    if structure_index < game.world.structures.len() {
        match &game.world.structures_data[structure_index] {
            StructureData::GoblinHut(_) => { is_enemy = true; },
            _ => {}
        }
    }

    is_enemy
}

pub fn damage_structure(game: &mut DemoGameData, structure_index: usize, damage: u8) {
    if structure_index >= game.world.structures.len() {
        return;
    }

    let destroyed = match &mut game.world.structures_data[structure_index] {
        StructureData::Castle(data) => { 
            data.hp -= u8::min(data.hp, damage);
            data.destroyed = data.hp == 0;
            data.destroyed
        }
        StructureData::Tower(data) => { 
            data.hp -= u8::min(data.hp, damage);
            data.destroyed = data.hp == 0;
            data.destroyed
        }
        StructureData::House(data) => { 
            data.hp -= u8::min(data.hp, damage);
            data.destroyed = data.hp == 0;
            data.destroyed 
        }
        StructureData::GoblinHut(data) => { 
            data.hp -= u8::min(data.hp, damage);
            data.destroyed = data.hp == 0;
            data.destroyed
        }
        StructureData::GoldMine(_) => { false },
    };

    if !destroyed {
        return;
    }

    let structure = &mut game.world.structures[structure_index];
    match game.world.structures_data[structure_index] {
        StructureData::Castle(_) => { 
            structure.sprite = game.assets.structures.knights_castle_destroyed; 
            // game.world.pathfinding.unregister_static_collision(structure.aabb());
        }
        StructureData::Tower(_) => { structure.sprite = game.assets.structures.knights_tower_destroyed; }
        StructureData::House(_) => { structure.sprite = game.assets.structures.knights_house_destroyed;}
        StructureData::GoblinHut(_) => { structure.sprite = game.assets.structures.goblin_house_destroyed; },
        StructureData::GoldMine(_) => { },
    }
}
