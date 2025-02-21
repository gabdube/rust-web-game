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

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DrawId {
    DrawPawns = 0
}

/// A generic draw update that will be read by the renderer
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawUpdate {
    graphics: GraphicsModule,
    draw_id: DrawId,
    instance_base: u32,
    instance_count: u32,
    texture_id: u32,
}

/// Information on how to render a sprites on the GPU
/// Memory layout must match `in_instance_position` and `in_instance_texcoord` in `sprites.vert.glsl`
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
}

/// The index of all the pointers and array size to share with the engine
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
pub struct OutputIndex {
    pub pointer_size: usize,
    pub commands_ptr: *const DrawUpdate,
    pub commands_count: usize,
    pub sprites_data_ptr: *const SpriteData,
    pub sprites_data_count: usize,
    pub validation: usize
}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,
    pub sprite_data_buffer: Vec<SpriteData>,
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
        let sprites_data = &mut output.sprite_data_buffer;

        let pawn_texture = self.assets.textures.get("pawn").unwrap().id;
        let mut command = DrawUpdate {
            graphics: GraphicsModule::DrawSprite,
            draw_id: DrawId::DrawPawns,
            instance_base: 0,
            instance_count: 0,
            texture_id: pawn_texture,
        };

        let texcoord_offset = [0.0, 0.0];
        let texcoord_size = [60.0, 59.0];
        let size = [60.0, 59.0];

        for pawn in world.pawns.iter() {
            let [x, y] = pawn.position.splat();

            sprites_data.push(SpriteData {
                position: [
                    x - (texcoord_size[0] * 0.5),
                    y - (texcoord_size[0] * 0.5)
                ],
                size,
                texcoord_offset,
                texcoord_size,
            });

            command.instance_count += 1;
        }

        output.commands.push(command);
    }

}

impl GameOutput {

    pub fn clear(&mut self) {
        self.commands.clear();
        self.sprite_data_buffer.clear();
    }

    pub fn write_index(&mut self) {
        let index = &mut self.output_index;
        index.commands_ptr = self.commands.as_ptr();
        index.commands_count = self.commands.len();
        index.sprites_data_ptr = self.sprite_data_buffer.as_ptr();
        index.sprites_data_count = self.sprite_data_buffer.len();
    }

}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            sprite_data_buffer: Vec::with_capacity(32),
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
            sprites_data_ptr: ::std::ptr::null(),
            sprites_data_count: 0,
            validation: 33355,
        }
    }
}
