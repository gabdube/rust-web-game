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
}

impl TestId {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => TestId::PawnAi,
            2 => TestId::WarriorAi,
            3 => TestId::ArcherAi,
            _ => TestId::None,
        }
    }
}

pub struct EditorState {
    gui: GameplayGuiState,
    current_test: TestId,
}

//
// Init
//

pub fn init(game: &mut DemoGame, test: TestId) -> Result<(), Error> {
    let mut inner_state = EditorState {
        gui: Default::default(),
        current_test: test,
    };

    game.data.init_terrain(16, 16);

    match test {
        TestId::None => {},
        TestId::PawnAi => init_pawn_tests(&mut game.data),
        TestId::WarriorAi => init_warrior_ai(&mut game.data),
        TestId::ArcherAi => init_archer_ai(&mut game.data),
    }

    inner_state.gui.build(&mut game.data)?;

    game.state = GameState::Editor(inner_state);

    Ok(())
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_pawn(pos(100.0, 100.0));
    world.create_pawn(pos(100.0, 200.0));
    world.create_pawn(pos(100.0, 300.0));
    
    world.create_tree(pos(300.0, 220.0), &assets.resources.tree_idle);
    world.create_tree(pos(380.0, 300.0), &assets.resources.tree_idle);
    world.create_tree(pos(230.0, 330.0), &assets.resources.tree_idle);
    
    world.create_gold_mine(pos(200.0, 500.0), assets.structures.gold_mine_inactive);

    create_sheeps(data);
}

fn init_warrior_ai(data: &mut DemoGameData) {
    let world = &mut data.world;

    world.create_warrior(pos(100.0, 100.0));
    world.create_warrior(pos(200.0, 100.0));
    world.create_warrior(pos(300.0, 100.0));

    create_sheeps(data);
}

fn init_archer_ai(data: &mut DemoGameData) {
    let world = &mut data.world;
    world.create_archer(pos(100.0, 100.0));
    world.create_archer(pos(200.0, 100.0));
    world.create_archer(pos(300.0, 100.0));

    create_sheeps(data);
}

fn create_sheeps(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;

    world.create_sheep(pos(550.0, 170.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(590.0, 210.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(520.0, 240.0), &assets.animations.sheep.idle);
    world.create_sheep(pos(490.0, 190.0), &assets.animations.sheep.idle);
}

//
// On state events
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
}

fn archer_actions(game: &mut DemoGameData, archer: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        behaviour::archer::archer_move::new(game, archer, cursor_world_position);
        return;
    }
}

fn get_state(state: &mut GameState) -> &mut EditorState {
    match state {
        GameState::Editor(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be editor in this module
    }
}

//
// Other
//

impl crate::store::SaveAndLoad for EditorState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write(&self.gui);
        writer.write_u32(self.current_test as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let gui = reader.read();
        let current_test = TestId::from_u32(reader.read_u32());
        
        EditorState {
            gui,
            current_test,
        }
    }
}

