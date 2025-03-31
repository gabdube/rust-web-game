mod gameplay_gui_state;

pub mod gameplay;
pub use gameplay::GameplayState;

#[cfg(feature="editor")]
pub mod editor;
#[cfg(feature="editor")]
pub use editor::{EditorState, TestId};

use crate::inputs::InputState;
use crate::store::SaveAndLoad;
use crate::DemoGame;

pub enum GameState {
    Startup,
    MainMenu,
    Gameplay(GameplayState),

    #[cfg(feature="editor")]
    Editor(EditorState)
}

pub fn update(game: &mut DemoGame) {
    use crate::state::GameState;

    let data = &mut game.data;
    let state = &mut game.state;

    if data.inputs.view_resized() {
        data.gui.resize(data.inputs.view_size);
    }

    match state {
        GameState::MainMenu => {

        },
        GameState::Gameplay(_) => {
            
        },
        GameState::Editor(_) => {
            if data.inputs.left_mouse_clicked() {
                crate::state::editor::on_left_mouse(state, data);
            }

            if data.inputs.right_mouse_clicked() {
                crate::state::editor::on_right_mouse(state, data);
            }
        },
        GameState::Startup => {
        }
    }

    clear_inputs_after_state_process(&mut data.inputs);
}

fn clear_inputs_after_state_process(inputs: &mut InputState) {
    inputs.last_mouse_position = inputs.mouse_position;
    inputs.last_view_size = inputs.view_size;
    
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
