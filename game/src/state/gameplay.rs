use crate::state::GameState;
use crate::{DemoGame, pos};

const DRAGGING_VIEW: u8 = 0b01;

#[derive(Default)]
pub struct GameplayState {
    flags: u8,
}

impl GameplayState {
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

    pub fn init_gameplay(&mut self) {
        let inner_state = GameplayState {
            flags: 0,
        };

        self.init_gameplay_test();
    
        self.state = GameState::Gameplay(inner_state);
    }

    pub fn gameplay_update(&mut self) {
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
 
    fn init_gameplay_test(&mut self) {
        self.world.reset();
        self.world.init_terrain(32, 32);

        self.world.create_structure(pos(200.0, 500.0), &self.assets.structures.knights_castle);
        self.world.create_decoration(pos(500.0, 500.0), &self.assets.decorations.shroom_big);
        self.world.create_resource(pos(450.0, 500.0), &self.assets.resources.gold);

        self.output.sync_world();
    }

}

fn state(state: &mut GameState) -> &mut GameplayState {
    match state {
        GameState::Gameplay(inner) => inner,
        _ => unsafe { std::hint::unreachable_unchecked() }  // state will always be gameplay in this module
    }
}

impl crate::store::SaveAndLoad for GameplayState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.flags as u32);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let mut state = GameplayState::default();
        state.flags = reader.read_u32() as u8;
        state
    }
}
