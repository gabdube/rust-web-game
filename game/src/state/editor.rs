//! Special debugging state to test features
use crate::data::actions;
use crate::state::GameState;
use crate::world::{WorldObject, WorldObjectType};
use crate::{DemoGameData, pos};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum TestId {
    None,
    PawnAi
}

impl TestId {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => TestId::PawnAi,
            _ => TestId::None,
        }
    }
}

pub struct EditorState {
    current_test: TestId,
    selected_object: Option<WorldObject>,
}

pub fn init(game: &mut DemoGameData, test: TestId) {
    let inner_state = EditorState {
        current_test: test,
        selected_object: None,
    };

    game.init_terrain(16, 16);

    match test {
        TestId::None => {},
        TestId::PawnAi => {
            init_pawn_tests(game);
        }
    }

    game.state = GameState::Editor(inner_state);
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;
    world.create_pawn(pos(100.0, 100.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 200.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 300.0), &assets.animations.pawn.idle);
    world.create_tree(pos(300.0, 220.0), &assets.resources.tree_idle);
    world.create_tree(pos(380.0, 300.0), &assets.resources.tree_idle);
    world.create_tree(pos(230.0, 330.0), &assets.resources.tree_idle);
}

pub fn on_left_mouse(game: &mut DemoGameData) {
    let inputs = &game.inputs;
    let state = state(&mut game.state);

    let cursor_world_position = inputs.mouse_position + game.global.view_offset;
    let new_selected = game.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => set_new_object_selection(game, new),
        (Some(old), Some(new)) => replace_object_selection(game, old, new),
    }
}

pub fn on_right_mouse(game: &mut DemoGameData) {
    let state = state(&mut game.state);
    let selected_object = match state.selected_object {
        Some(obj) => obj,
        None => { return; }
    };

    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;
    let target_object = game.world.object_at(cursor_world_position);

    match selected_object.ty {
        WorldObjectType::Pawn => pawn_actions(game, selected_object, target_object),
        _ => {},
    }
}

fn pawn_actions(game: &mut DemoGameData, pawn: WorldObject, target_object: Option<WorldObject>) {
    let cursor_world_position = game.inputs.mouse_position + game.global.view_offset;

    if target_object.is_none() || game.inputs.left_shift.pressed() {
        actions::move_actor::new(game, pawn, cursor_world_position);
        return;
    }

    let target_object = target_object.unwrap();
    match target_object.ty {
        WorldObjectType::Tree => actions::cut_tree::new(game, pawn, target_object),
        _ => {},
    }
}

fn set_new_object_selection(data: &mut DemoGameData, new_selection: WorldObject) {
    let state = state(&mut data.state);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn replace_object_selection(data: &mut DemoGameData, old_selection: WorldObject, new_selection: WorldObject) {
    let state = state(&mut data.state);
    data.world.set_object_selected(old_selection, false);
    data.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn state(state: &mut GameState) -> &mut EditorState {
    match state {
        GameState::Editor(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be editor in this module
    }
}

impl crate::store::SaveAndLoad for EditorState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.current_test as u32);
        writer.save_option(&self.selected_object);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        let selected_object = reader.load_option();
        
        EditorState {
            current_test,
            selected_object,
        }
    }
}

