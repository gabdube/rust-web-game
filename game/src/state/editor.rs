//! Special debugging state to test features
use crate::actions::{Action, ActionsManager};
use crate::state::GameState;
use crate::world::WorldObject;
use crate::{DemoGame, DemoGameData, pos};

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

pub fn init(game: &mut DemoGame, test: TestId) {
    let inner_state = EditorState {
        current_test: test,
        selected_object: None,
    };

    game.data.world.reset();
    game.data.world.init_terrain(16, 16);
    game.output.sync_world();

    match test {
        TestId::None => {},
        TestId::PawnAi => {
            init_pawn_tests(&mut game.data);
        }
    }

    game.data.state = GameState::Editor(inner_state);
}

fn init_pawn_tests(data: &mut DemoGameData) {
    let world = &mut data.world;
    let assets = &data.assets;
    world.create_pawn(pos(100.0, 100.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 200.0), &assets.animations.pawn.idle);
    world.create_pawn(pos(100.0, 300.0), &assets.animations.pawn.idle);
    world.create_tree(pos(300.0, 250.0), &assets.resources.tree_idle);
}

pub fn on_left_mouse(game: &mut DemoGame) {
    let data = &mut game.data;
    let inputs = &data.inputs;
    let state = state(&mut data.state);

    let cursor_world_position = inputs.mouse_position + data.global.view_offset;
    let new_selected = data.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) | (Some(_), None) => {},
        (None, Some(new)) => set_new_object_selection(data, new),
        (Some(old), Some(new)) => replace_object_selection(data, old, new),
    }
}

pub fn on_right_mouse(game: &mut DemoGame) {
    use crate::world::WorldObjectType;

    let data = &mut game.data;
    let inputs = &data.inputs;
    let actions = &mut game.actions;
    let state = state(&mut data.state);

    let selected_object = match state.selected_object {
        Some(obj) => obj,
        None => { return; }
    };

    let cursor_world_position = inputs.mouse_position + data.global.view_offset;
    let target_object = data.world.object_at(cursor_world_position);

    match selected_object.ty {
        WorldObjectType::Pawn => {
            if target_object.is_none() {
                let action = Action::move_to(selected_object, cursor_world_position);
                actions.cancel(action);
                actions.push(action);
            }
            
            if let Some(target_object) = target_object {
                match target_object.ty {
                    WorldObjectType::Tree => move_pawn_to_tree(data, actions, selected_object, target_object),
                    _ => {},
                }
            }
        },
        _ => {},
    }
}

fn move_pawn_to_tree(
    data: &mut DemoGameData,
    actions: &mut ActionsManager,
    selected_object: WorldObject,
    target: WorldObject
) {
    let pawn_x = data.world.pawns[selected_object.id as usize].position.x;
    let mut target_position = data.world.trees[target.id as usize].position;

    target_position.y += 10.0;

    if target_position.x > pawn_x {
        target_position.x -= 60.0;
    } else {
        target_position.x += 60.0;
    }

    let action = Action::move_to(selected_object, target_position);
    let action2 = Action::cut_tree(selected_object, target);
    actions.cancel(action);
    actions.push_and_queue(action, action2);
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

