use super::DemoGame;

/// A generic draw command that will be read by the renderer
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawCommand {

}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    pub commands: Vec<DrawCommand>,
}

impl DemoGame {

    /// Updates the current output buffers based on the game state
    pub fn update_output(&mut self) {
        let world = &self.world;
        let output = &mut self.output;

        for pawn in world.pawns.iter() {

        }
    }

}

impl Default for GameOutput {

    fn default() -> Self {
        GameOutput {
            commands: Vec::with_capacity(32),
        }
    }

}
