use crate::state::GameState;
use crate::DemoGame;

#[derive(Default)]
pub struct GameplayState {
}

pub fn init(game: &mut DemoGame) {
    let inner_state = GameplayState {
    };

    game.data.state = GameState::Gameplay(inner_state);
}

// fn dragging_view_updates(game: &mut DemoGame) {
//     let inputs = &game.inputs;
//     let state = state(&mut game.state);
    
//     if inputs.right_mouse_clicked() {
//         state.grab_view();
//     } else if inputs.right_mouse_released() {
//         state.release_view();
//     }

//     if state.dragging_view() {
//         if let Some(delta) = inputs.mouse_delta() {
//             game.view_offset -= delta;
//             game.output.sync_view();
//         }
//     }
// }

fn state(state: &mut GameState) -> &mut GameplayState {
    match state {
        GameState::Gameplay(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be gameplay in this module
    }
}

impl crate::store::SaveAndLoad for GameplayState {
    fn save(&self, _writer: &mut crate::store::SaveFileWriter) {
    }

    fn load(_reader: &mut crate::store::SaveFileReader) -> Self {
        GameplayState { }
    }
}
