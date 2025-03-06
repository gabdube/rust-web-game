//! Special debugging state to test features
use crate::state::GameState;
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
        self.common_test_update();

        match state(&mut self.state).current_test {
            TestId::None => {},
            TestId::PawnAi => self.pawns_test_update(),
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
        self.world.create_pawn(pos(100.0, 300.0), &self.assets.animations.pawn.idle);
    }

    fn common_test_update(&mut self) {
        let inputs = &self.inputs;
        let state = state(&mut self.state);
        
        if inputs.right_mouse_clicked() {
            state.grab_view();
        } else if inputs.right_mouse_released() {
            state.release_view();
        }

        if state.dragging_view() {
            if let Some(delta) = inputs.mouse_delta() {
                self.view_offset -= delta;
                self.output.sync_view();
            }
        }
    }

    fn pawns_test_update(&mut self) {
        let inputs = &self.inputs;
        let state = state(&mut self.state);
        if inputs.left_mouse_clicked() {
            let world_position = inputs.mouse_position + self.view_offset;
            dbg!("{:?}", self.world.object_at(world_position));
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
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        let flags = reader.read_u32() as u8;
        
        EditorState {
            current_test,
            flags,
        }
    }
}

