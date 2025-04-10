//! Special debugging state to test features
use crate::behaviour;
use crate::error::Error;
use crate::state::GameState;
use crate::world::{StructureData, WorldObject, WorldObjectType};
use crate::{DemoGame, DemoGameData, pos};

use super::gameplay_gui_state::GameplayGuiState;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TestId {
    None,
    PawnAi,
    WarriorAi,
    ArcherAi,
    PathfindingAi,
}

impl TestId {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => TestId::PawnAi,
            2 => TestId::WarriorAi,
            3 => TestId::ArcherAi,
            4 => TestId::PathfindingAi,
            _ => TestId::None,
        }
    }
}

pub struct EditorState {
    gui: GameplayGuiState,
    current_test: TestId,
    dragging_view: bool,
}

//
// Init
//

pub fn init(game: &mut DemoGame, test: TestId) -> Result<(), Error> {
    let mut inner_state = EditorState {
        gui: Default::default(),
        current_test: test,
        dragging_view: false,
    };

    game.data.init_terrain(32, 32);

    match test {
        TestId::None => {},
        TestId::PawnAi => init_pawn_tests(&mut game.data),
        TestId::WarriorAi => init_warrior_ai(&mut game.data),
        TestId::ArcherAi => init_archer_ai(&mut game.data),
        TestId::PathfindingAi => init_pathfinding_ai(&mut game.data),
    }

    inner_state.gui.build(&mut game.data)?;

    game.state = GameState::Editor(inner_state);

    Ok(())
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;

    world.create_pawn(pos(100.0, 100.0));
    world.create_pawn(pos(100.0, 200.0));
    world.create_pawn(pos(100.0, 300.0));
    
    world.create_tree(pos(300.0, 220.0));
    world.create_tree(pos(380.0, 300.0));
    world.create_tree(pos(230.0, 330.0));

    world.create_gold_mine(pos(200.0, 500.0));

    world.create_castle(pos(200.0, 760.0));
    world.create_tower(pos(500.0, 760.0));
    world.create_house(pos(700.0, 760.0));
    
    // Destroyed structures
    world.create_castle_with_data(pos(200.0, 960.0), crate::world::StructureCastleData { hp: 0, building: true, destroyed: true });
    world.create_tower_with_data(pos(500.0, 960.0), crate::world::StructureTowerData { hp: 0, building: true, destroyed: true });
    world.create_house_with_data(pos(700.0, 960.0), crate::world::StructureHouseData { hp: 0, building: true, destroyed: true });

    world.create_sheep(pos(650.0, 370.0));
    world.create_sheep(pos(690.0, 510.0));
    world.create_sheep(pos(620.0, 540.0));
    world.create_sheep(pos(590.0, 390.0));
    world.create_sheep(pos(700.0, 550.0));
}

fn init_warrior_ai(data: &mut DemoGameData) {
    let world = &mut data.world;

    world.create_warrior(pos(100.0, 100.0));
    world.create_warrior(pos(200.0, 100.0));
    world.create_warrior(pos(300.0, 100.0));

    world.create_goblin_hut(pos(90.0, 500.0));
    world.create_goblin_hut(pos(250.0, 430.0));
    world.create_goblin_hut(pos(180.0, 600.0));

    world.create_sheep(pos(650.0, 370.0));
    world.create_sheep(pos(690.0, 510.0));
    world.create_sheep(pos(620.0, 540.0));
    world.create_sheep(pos(590.0, 390.0));
    world.create_sheep(pos(700.0, 550.0));
}

fn init_archer_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    world.create_archer(pos(100.0, 100.0));
    world.create_archer(pos(200.0, 100.0));
    world.create_archer(pos(300.0, 100.0));

    world.create_goblin_hut(pos(90.0, 500.0));
    world.create_goblin_hut(pos(250.0, 430.0));
    world.create_goblin_hut(pos(180.0, 600.0));

    world.create_sheep(pos(650.0, 370.0));
    world.create_sheep(pos(690.0, 510.0));
    world.create_sheep(pos(620.0, 540.0));
    world.create_sheep(pos(590.0, 390.0));
    world.create_sheep(pos(700.0, 550.0));
}

fn init_pathfinding_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    
    world.create_pawn(pos(100.0, 100.0));
    world.create_warrior(pos(200.0, 100.0));
    world.create_archer(pos(300.0, 100.0));

    world.create_castle_with_data(pos(400.0, 480.0), crate::world::StructureCastleData { hp: crate::world::MAX_CASTLE_HP, building: false, destroyed: false });
    
    let tower_data = crate::world::StructureTowerData { hp: crate::world::MAX_TOWER_HP, building: false, destroyed: false };
    world.create_tower_with_data(pos(200.0, 600.0), tower_data);
    world.create_tower_with_data(pos(200.0, 380.0), tower_data);
    world.create_tower_with_data(pos(600.0, 380.0), tower_data);
    world.create_tower_with_data(pos(600.0, 600.0), tower_data);
    world.create_tower_with_data(pos(400.0, 650.0), tower_data);

    let house_data = crate::world::StructureHouseData { hp: crate::world::MAX_HOUSE_HP, building: false, destroyed: false };
    world.create_house_with_data(pos(750.0, 750.0), house_data);
    world.create_house_with_data(pos(850.0, 750.0), house_data);
    world.create_house_with_data(pos(950.0, 750.0), house_data);
    world.create_house_with_data(pos(1050.0, 750.0), house_data);

    world.create_house_with_data(pos(500.0, 850.0), house_data);
    world.create_house_with_data(pos(500.0, 950.0), house_data);
    world.create_house_with_data(pos(500.0, 1050.0), house_data);
    world.create_house_with_data(pos(500.0, 1150.0), house_data);

    world.create_house_with_data(pos(900.0, 1200.0), house_data);
    world.create_house_with_data(pos(1000.0, 1200.0), house_data);
    world.create_house_with_data(pos(1100.0, 1200.0), house_data);
    world.create_house_with_data(pos(1200.0, 1200.0), house_data);

    world.create_house_with_data(pos(1200.0, 900.0), house_data);
    world.create_house_with_data(pos(1200.0, 1000.0), house_data);
    world.create_house_with_data(pos(1200.0, 1100.0), house_data);

    world.create_goblin_hut(pos(1550.0, 250.0));
    world.create_goblin_hut(pos(1550.0, 350.0));
    world.create_goblin_hut(pos(1550.0, 450.0));
    world.create_goblin_hut(pos(1550.0, 550.0));
    world.create_goblin_hut(pos(1550.0, 650.0));
    world.create_goblin_hut(pos(1550.0, 750.0));
    world.create_goblin_hut(pos(1550.0, 850.0));
    world.create_goblin_hut(pos(1550.0, 950.0));
    world.create_goblin_hut(pos(1550.0, 1050.0));

    world.create_tree(pos(1000.0, 300.0));
    world.create_tree(pos(1000.0, 400.0));
    world.create_tree(pos(1000.0, 500.0));
    world.create_tree(pos(1100.0, 300.0));
    world.create_tree(pos(1100.0, 400.0));
    world.create_tree(pos(1100.0, 500.0));
    world.create_tree(pos(1200.0, 300.0));
    world.create_tree(pos(1200.0, 400.0));
    world.create_tree(pos(1200.0, 500.0));
}


//
// General updates
//

pub fn on_update(state: &mut GameState, data: &mut DemoGameData) {
    use crate::inputs::{MouseButton, ButtonState};
    
    let state = get_state(state);

    match data.inputs.mouse_button_state(MouseButton::Center) {
        ButtonState::JustPressed => { state.dragging_view = true; },
        ButtonState::JustReleased => { state.dragging_view = false; },
        _ => {}
    }

    if state.dragging_view {
        if let Some(delta) = data.inputs.mouse_delta() {
            data.set_view_offset(data.global.view_offset - delta);
        }
    }
}

//
// Input events
//

pub fn on_left_mouse(state: &mut GameState, data: &mut DemoGameData) {
    let state = get_state(state);

    let cursor_world_position = data.inputs.mouse_position + data.global.view_offset;
    let new_selected = data.world.object_at(cursor_world_position);

    match (state.gui.details_frame.displayed_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => {
            data.world.set_object_selected(new, true);
            state.gui.set_selected_object(data, new);
        },
        (Some(old), Some(new)) => {
            data.world.set_object_selected(old, false);
            data.world.set_object_selected(new, true);
            state.gui.set_selected_object(data, new);
        },
    }
}

pub fn on_right_mouse(state: &mut GameState, data: &mut DemoGameData) {
    let state = get_state(state);
    let selected_object = match state.gui.details_frame.displayed_object {
        Some(obj) => obj,
        None => { return; }
    };

    let cursor_world_position = data.inputs.mouse_position + data.global.view_offset;
    let target_object = data.world.object_at(cursor_world_position);

    match selected_object.ty {
        WorldObjectType::Pawn => pawn_actions(data, selected_object, target_object),
        WorldObjectType::Warrior => warrior_actions(data, selected_object, target_object),
        WorldObjectType::Archer => archer_actions(data, selected_object, target_object),
        _ => {},
    }
}

fn pawn_actions(game: &mut DemoGameData, pawn: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::pawn::pawn_move::new(game, pawn, cursor_world_position);
        return;
    }

    let target_object = target_object.unwrap();
    match target_object.ty {
        WorldObjectType::Tree => behaviour::pawn::harvest_wood::new(game, pawn, target_object),
        WorldObjectType::Resource => behaviour::pawn::grab_resource::new(game, pawn, target_object),
        WorldObjectType::Sheep => behaviour::pawn::hunt_sheep::new(game, pawn, target_object),
        WorldObjectType::Structure => {
            match game.world.structures_data[target_object.id as usize] {
                StructureData::GoldMine(_) => behaviour::pawn::harvest_gold::new(game, pawn, target_object),
                StructureData::Castle(_) | StructureData::House(_) | StructureData::Tower(_) => {
                    behaviour::pawn::build_structure::new(game, pawn, target_object);
                },
                StructureData::GoblinHut(_) => {},
            } 
        },
        _ => {},
    }
}

fn warrior_actions(game: &mut DemoGameData, warrior: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::warrior::warrior_move::new(game, warrior, cursor_world_position);
        return;
    }

    let target_object = target_object.unwrap();
    match target_object.ty {
        WorldObjectType::Sheep => behaviour::warrior::warrior_attack::new(game, warrior, target_object),
        WorldObjectType::Structure => behaviour::warrior::warrior_attack::new(game, warrior, target_object),
        _ => {}
    }
}

fn archer_actions(game: &mut DemoGameData, archer: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::archer::archer_move::new(game, archer, cursor_world_position);
        return;
    }

    let target_object = target_object.unwrap();
    match target_object.ty {
        WorldObjectType::Sheep => behaviour::archer::shoot::new(game, archer, target_object),
        WorldObjectType::Structure => behaviour::archer::shoot::new(game, archer, target_object),
        _ => {}
    }
}


//
// Other
//

fn get_state(state: &mut GameState) -> &mut EditorState {
    match state {
        GameState::Editor(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be editor in this module
    }
}

impl crate::store::SaveAndLoad for EditorState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.gui);
        writer.write_u32(self.current_test as u32);
        writer.write_u32(self.dragging_view as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let gui = reader.read();
        let current_test = TestId::from_u32(reader.read_u32());
        let dragging_view = reader.read_u32() == 1;
        
        EditorState {
            gui,
            current_test,
            dragging_view
        }
    }
}

