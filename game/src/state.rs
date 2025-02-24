mod gameplay;
use gameplay::GameplayState;

use crate::store::SaveAndLoad;


pub enum GameState {
    Startup,
    MainMenu,
    Gameplay(GameplayState)
}

impl SaveAndLoad for GameState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        match self {
            GameState::Startup => {
                writer.write_u32(1);
            },
            GameState::MainMenu => {
                writer.write_u32(2);
            },
            GameState::Gameplay(inner) => {
                writer.write_u32(3);
                writer.save(inner);
            },
        };
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        match reader.read_u32() {
            1 => GameState::Startup,
            2 => GameState::MainMenu,
            3 => {
                let inner_state = reader.load();
                GameState::Gameplay(inner_state)
            },
            _ => GameState::Startup
        }
    }
}
