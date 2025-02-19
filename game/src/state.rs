mod gameplay;

use crate::store::SaveAndLoad;

pub enum GameState {
    Startup,
    MainMenu,
    Gameplay
}

impl SaveAndLoad for GameState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        let state_id = match self {
            GameState::Startup => 1,
            GameState::MainMenu => 2,
            GameState::Gameplay => 3,
        };

        writer.write_u32(state_id);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        match reader.read_u32() {
            1 => GameState::Startup,
            2 => GameState::MainMenu,
            3 => GameState::Gameplay,
            _ => GameState::Startup
        }
    }
}
