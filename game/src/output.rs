use super::DemoGame;

/// Tells the engine which "module" to use to process a draw update
/// This maps 1-1 to the `GraphicsModule` defined in the engine renderer
#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum GraphicsModule {
    #[default]
    Undefined = 0,
    DrawSprite = 1,
}

/// A generic draw update that will be read by the renderer
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawUpdate {
    graphics: GraphicsModule,
}

/// The index of all the pointers and array size to share with the engine
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
pub struct OutputIndex {
    pub pointer_size: usize,
    pub commands_ptr: *const DrawUpdate,
    pub commands_count: usize,
    pub validation: usize
}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,    
    pub commands: Vec<DrawUpdate>,
}

impl DemoGame {

    /// Updates the current output buffers based on the game state
    pub fn update_output(&mut self) {
        self.output.clear();
        self.render_pawns();
        self.output.write_index();
    }

    fn render_pawns(&mut self) {
        let world = &self.world;
        let output = &mut self.output;

        let mut command = DrawUpdate {
            graphics: GraphicsModule::DrawSprite,

        };

        for pawn in world.pawns.iter() {

        }

        output.commands.push(command);
    }

}

impl GameOutput {

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn write_index(&mut self) {
        let index = &mut self.output_index;
        index.commands_ptr = self.commands.as_ptr();
        index.commands_count = self.commands.len();
    }

}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            commands: Vec::with_capacity(32),
        }
    }

}

impl Default for OutputIndex {
    fn default() -> Self {
        OutputIndex {
            pointer_size: size_of::<usize>(),
            commands_ptr: ::std::ptr::null(),
            commands_count: 0,
            validation: 33355,
        }
    }
}
