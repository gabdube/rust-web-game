use crate::shared::{aabb, Position};
use crate::DemoGame;

const UPDATE_VIEW: u8 = 0b001;
const UPDATE_WORLD: u8 = 0b010;

/// Tells the engine which "module" to use to process a draw update
/// This maps 1-1 to the `GraphicsModule` defined in the engine renderer
#[repr(u32)]
#[derive(Copy, Clone, Default)]
pub enum DrawUpdateType {
    #[default]
    Undefined = 0,
    DrawSprites = 1,
    UpdateTerrainChunk = 2,
    DrawTerrainChunk = 3,
    UpdateViewOffset = 4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawSpriteParams {
    pub instance_base: u32,
    pub instance_count: u32,
    pub texture_id: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UpdateTerrainChunkParams {
    pub chunk_id: u32,
    pub chunk_data_offset: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawTerrainChunkParams {
    pub chunk_id: u32,
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union DrawUpdateParams {
    pub draw_sprites: DrawSpriteParams,
    pub update_terrain_chunk: UpdateTerrainChunkParams,
    pub draw_terrain_chunk: DrawTerrainChunkParams,
    pub update_view_offset: Position<f32>,
}

/// A generic draw update that will be read by the renderer
/// Must be `repr(C)` because it will be directly read from memory by the engine
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawUpdate {
    graphics: DrawUpdateType,
    params: DrawUpdateParams,
}

/// Information on how to render a sprites on the GPU
/// Memory layout must match `in_instance_position` and `in_instance_texcoord` in `sprites.vert.glsl`
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct SpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
}

/// Texture coordinates for the 4 vertex of a sprite the terrain data buffer
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct TerrainChunkTexcoord {
    pub v0: [f32; 2],
    pub v1: [f32; 2],
    pub v2: [f32; 2],
    pub v3: [f32; 2],
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
    pub terrain_data_ptr: *const TerrainChunkTexcoord,
    pub terrain_data_count: usize,
    pub validation: usize
}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,
    pub sprite_data_buffer: Vec<SpriteData>,
    pub terrain_data: Vec<TerrainChunkTexcoord>,
    pub commands: Vec<DrawUpdate>,
    pub updates: u8,
}

impl DemoGame {

    /// Updates the current output buffers based on the game state
    pub fn update_output(&mut self) {
        self.output.clear();
        self.update_view();
        self.update_terrain();
        self.render_terrain();
        self.render_sprites();
        self.output.write_index();
    }

    fn update_view(&mut self) {
        let output = &mut self.output;
        if !output.must_sync_view() {
            return;
        }

        output.commands.push(DrawUpdate {
            graphics: DrawUpdateType::UpdateViewOffset,
            params: DrawUpdateParams { update_view_offset: self.view_offset },
        });

        output.clear_sync_view();
    }

    fn update_terrain(&mut self) {
        const CELL_TEXEL_SIZE: f32 = 64.0;

        let output = &mut self.output;
        if !output.must_sync_world() {
            return;
        }

        let terrain = &mut self.world.terrain;
        let terrain_tilemap = &self.assets.terrain;
        let mut params = UpdateTerrainChunkParams { chunk_id: 0, chunk_data_offset: 0 };

        for (index, update) in terrain.chunks_updates.iter_mut().enumerate() {
            if !*update {
                continue;
            }

            let chunk = &terrain.chunks[index];
            params.chunk_id = chunk.position;
            params.chunk_data_offset = output.terrain_data.len() as u32;

            for row in chunk.cells.iter() {
                for &cell in row.iter() {
                    let [x, y] = terrain_tilemap.get_cell_texcoord(cell);
                    output.terrain_data.push(TerrainChunkTexcoord {
                        v0: [x, y],
                        v1: [x+CELL_TEXEL_SIZE, y],
                        v2: [x, y+CELL_TEXEL_SIZE],
                        v3: [x+CELL_TEXEL_SIZE, y+CELL_TEXEL_SIZE],
                    });
                }
            }

            output.commands.push(DrawUpdate {
                graphics: DrawUpdateType::UpdateTerrainChunk,
                params: DrawUpdateParams { update_terrain_chunk: params },
            });
 
            *update = false;
        }

        output.clear_sync_world();
    }

    fn render_terrain(&mut self) {
        let output = &mut self.output;
        let view = aabb(self.view_offset, self.view_size);

        let mut params = DrawTerrainChunkParams { chunk_id: 0, x: 0.0, y: 0.0 };

        for chunk in self.world.terrain.chunks.iter() {
            let chunk_view = chunk.view();
            if !view.intersects(&chunk_view) {
                continue;
            }

            params.chunk_id = chunk.position;
            params.x = chunk_view.left;
            params.y = chunk_view.top;

            output.commands.push(DrawUpdate {
                graphics: DrawUpdateType::DrawTerrainChunk,
                params: DrawUpdateParams { draw_terrain_chunk: params },
            });
        }
    }

    fn render_sprites(&mut self) {
        let world = &self.world;
        let output = &mut self.output;
        let sprites_data = &mut output.sprite_data_buffer;

        let mut params = DrawSpriteParams {
            instance_base: 0,
            instance_count: 0,
            texture_id: 0,
        };

        let sprites_group = [
            (world.pawn_texture, &world.pawns_sprites),
            (world.warrior_texture, &world.warrior_sprites),
            (world.archer_texture, &world.archer_sprites),
            (world.torch_goblin_texture, &world.torch_goblins_sprites),
            (world.tnt_goblin_texture, &world.tnt_goblins_sprites),
            (world.sheep_texture, &world.sheep_sprites),
        ];

        for (texture, sprites) in sprites_group {
            if sprites.len() == 0 {
                continue;
            }

            params.instance_base = sprites_data.len() as u32;
            params.instance_count = 0;
            params.texture_id = texture.id;
            
            for &sprite in sprites {
                sprites_data.push(sprite);
                params.instance_count += 1;
            }

            output.commands.push(DrawUpdate {
                graphics: DrawUpdateType::DrawSprites,
                params: DrawUpdateParams { draw_sprites: params },
            });
        }

    }

}

impl GameOutput {

    pub fn clear(&mut self) {
        self.commands.clear();
        self.sprite_data_buffer.clear();
        self.terrain_data.clear();
    }

    pub fn write_index(&mut self) {
        let index = &mut self.output_index;
        index.commands_ptr = self.commands.as_ptr();
        index.commands_count = self.commands.len();
        index.sprites_data_ptr = self.sprite_data_buffer.as_ptr();
        index.sprites_data_count = self.sprite_data_buffer.len();
        index.terrain_data_ptr = self.terrain_data.as_ptr();
        index.terrain_data_count = self.terrain_data.len();
    }

    pub fn sync_view(&mut self) { self.updates |= UPDATE_VIEW; }
    fn must_sync_view(&self) -> bool { self.updates & UPDATE_VIEW > 0 }
    fn clear_sync_view(&mut self) { self.updates &= !UPDATE_VIEW; }

    pub fn sync_world(&mut self) { self.updates |= UPDATE_WORLD; }
    fn must_sync_world(&self) -> bool { self.updates & UPDATE_WORLD > 0 }
    fn clear_sync_world(&mut self) { self.updates &= !UPDATE_WORLD; }

}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            sprite_data_buffer: Vec::with_capacity(32),
            terrain_data: Vec::with_capacity(1024),
            commands: Vec::with_capacity(32),
            updates: 0,
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
            terrain_data_ptr: ::std::ptr::null(),
            terrain_data_count: 0,
            validation: 33355,
        }
    }
}
