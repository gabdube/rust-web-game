mod shared_state;

pub mod gameplay;
pub use gameplay::GameplayState;

#[cfg(feature="editor")]
pub mod editor;
#[cfg(feature="editor")]
pub use editor::{EditorState, TestId};

use crate::inputs::InputState;
use crate::store::SaveAndLoad;

pub enum GameState {
    Startup,
    MainMenu,
    Gameplay(GameplayState),
    
    #[cfg(feature="editor")]
    Editor(EditorState)
}

pub fn update(game: &mut crate::DemoGameData) {
    use crate::state::GameState;

    match game.state {
        GameState::MainMenu => {

        },
        GameState::Gameplay(_) => {
            
        },
        GameState::Editor(_) => {
            if game.inputs.left_mouse_clicked() {
                crate::state::editor::on_left_mouse(game);
            }

            if game.inputs.right_mouse_clicked() {
                crate::state::editor::on_right_mouse(game);
            }
        },
        GameState::Startup => {
        }
    }

    clear_inputs_after_state_process(&mut game.inputs);
}

fn clear_inputs_after_state_process(inputs: &mut InputState) {
    inputs.last_mouse_position = inputs.mouse_position;
    
    for state in inputs.mouse_buttons.iter_mut() {
        state.flip();
    }
    
    inputs.left_shift.flip();
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
