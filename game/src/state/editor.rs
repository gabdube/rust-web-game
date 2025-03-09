//! Special debugging state to test features
use crate::state::GameState;
use crate::world::WorldObject;
use crate::{DemoGame, pos};

const DRAGGING_VIEW: u8 = 0b01;

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
    flags: u8,
}

impl EditorState {
    pub fn grab_view(&mut self) {
        self.flags |= DRAGGING_VIEW;
    }

    pub fn release_view(&mut self) {
        self.flags &= !DRAGGING_VIEW;
    }

    pub fn dragging_view(&mut self) -> bool {
        self.flags & DRAGGING_VIEW != 0
    }
}

impl DemoGame {

    pub fn init_editor(&mut self, test: TestId) {
        let inner_state = EditorState {
            current_test: test,
            selected_object: None,
            flags: 0,
        };

        self.init_test_terrain();

        match test {
            TestId::None => {},
            TestId::PawnAi => {
                self.init_pawn_tests();
            }
        }

        self.state = GameState::Editor(inner_state);
    }

    pub fn editor_update(&mut self) {
        dragging_view_updates(self);

        if self.inputs.left_mouse_clicked() {
            on_left_mouse(self);
        }
    }

    fn init_test_terrain(&mut self) {
        self.world.reset();
        self.world.init_terrain(16, 16);
        self.output.sync_world();
    }

    fn init_pawn_tests(&mut self) {
        self.world.create_pawn(pos(100.0, 100.0), &self.assets.animations.pawn.idle);
        self.world.create_pawn(pos(100.0, 200.0), &self.assets.animations.pawn.idle);
        self.world.create_pawn(pos(100.0, 300.0), &self.assets.animations.pawn.axe);
    }

}

fn on_left_mouse(game: &mut DemoGame) {
    let inputs = &game.inputs;
    let state = state(&mut game.state);

    let cursor_world_position = inputs.mouse_position + game.view_offset;
    let new_selected = game.world.object_at(cursor_world_position);

    match (state.selected_object, new_selected) {
        (None, None) => {},
        (None, Some(new)) => set_new_object_selection(game, new),
        (Some(old), None) => {},
        (Some(old), Some(new)) => replace_object_selection(game, old, new),
    }
}

fn set_new_object_selection(game: &mut DemoGame, new_selection: WorldObject) {
    let state = state(&mut game.state);
    game.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}

fn replace_object_selection(game: &mut DemoGame, old_selection: WorldObject, new_selection: WorldObject) {
    let state = state(&mut game.state);
    game.world.set_object_selected(old_selection, false);
    game.world.set_object_selected(new_selection, true);
    state.selected_object = Some(new_selection);
}


fn dragging_view_updates(game: &mut DemoGame) {
    let inputs = &game.inputs;
    let state = state(&mut game.state);
    
    if inputs.right_mouse_clicked() {
        state.grab_view();
    } else if inputs.right_mouse_released() {
        state.release_view();
    }

    if state.dragging_view() {
        if let Some(delta) = inputs.mouse_delta() {
            game.view_offset -= delta;
            game.output.sync_view();
        }
    }
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
        writer.write_u32(self.flags as u32);
        writer.save_option(&self.selected_object);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        let flags = reader.read_u32() as u8;
        let selected_object = reader.load_option();
        
        EditorState {
            current_test,
            selected_object,
            flags,
        }
    }
}

