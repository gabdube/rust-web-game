use crate::world::Pawn;
use crate::state::GameState;
use crate::{DemoGame, pos};

impl DemoGame {

    pub fn init_gameplay(&mut self) {
        self.init_gameplay_test();
        self.state = GameState::Gameplay;
    }

    pub fn gameplay_update(&mut self) {

    }

    fn init_gameplay_test(&mut self) {
        self.world.pawns.push(Pawn {
            position: pos(100.0, 100.0),
        });
    }
}
