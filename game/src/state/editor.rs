//! Special debugging state to test features
use crate::state::GameState;
use crate::DemoGame;

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
    pub current_test: TestId,
}

impl DemoGame {

    pub fn init_editor(&mut self, test: TestId) {
        let inner_state = EditorState {
            current_test: test,
        };

        self.init_test_terrain();

        self.state = GameState::Editor(inner_state);
    }

    pub fn editor_update(&mut self) {

    }

    fn init_test_terrain(&mut self) {
        self.world.reset();
        self.world.init_terrain(16, 16);
        self.output.sync_world();
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
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let current_test = TestId::from_u32(reader.read_u32());
        
        EditorState {
            current_test,
        }
    }
}

