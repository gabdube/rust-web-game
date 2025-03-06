mod shared_state;

mod gameplay;
use gameplay::GameplayState;

#[cfg(feature="editor")]
mod editor;
#[cfg(feature="editor")]
pub use editor::{EditorState, TestId};

use crate::store::SaveAndLoad;


pub enum GameState {
    Startup,
    MainMenu,
    Gameplay(GameplayState),
    
    #[cfg(feature="editor")]
    Editor(EditorState)
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

            #[cfg(feature="editor")]
            GameState::Editor(inner) => {
                writer.write_u32(4);
                writer.save(inner);
            }
        };
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        match reader.read_u32() {
            1 => GameState::Startup,
            2 => GameState::MainMenu,
            3 => GameState::Gameplay(reader.load()),
            4 => {
                #[cfg(feature="editor")]
                { GameState::Editor(reader.load()) }

                #[cfg(not(feature="editor"))]
                { GameState::Startup }
            },
            _ => GameState::Startup
        }
    }
}
