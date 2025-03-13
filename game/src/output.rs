/// Structures and system to transfer data from the rust app to an external reader (in this case javascript)
/// Data with `repr(C)` will be directly read from memory by the engine

use crate::shared::{aabb, Position};
use crate::DemoGame;

const UPDATE_VIEW: u8 = 0b001;
const UPDATE_WORLD: u8 = 0b010;
const UPDATE_ANIMATION: u8 = 0b100;

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
#[derive(Copy, Clone, Debug)]
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

/// A generic draw update that will be read by the renderere
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawUpdate {
    graphics: DrawUpdateType,
    params: DrawUpdateParams,
}

/// Information on how to render a sprites on the GPU
/// Memory layout must match `in_instance_position`, `in_instance_texcoord`, `in_instance_data` in `sprites.vert.glsl`
/// If this struct size change, it must also be updatedin `game_interface.ts`
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct SpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
    pub data: i32,
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


/// Temporary storage for sprites when regrouping by texture_id and y position
pub struct TempSprite {
    pub texture_id: u32,
    pub y: f32,
    pub sprite: SpriteData,
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

    /// Buffers of generated sprites. Shared with the renderer
    pub sprite_data_buffer: Vec<SpriteData>,

    /// Buffers of generated terrain sprites. Shared with the renderer.
    pub terrain_data: Vec<TerrainChunkTexcoord>,

    /// Buffers of the generated draw update for the current frame. Shared with the renderer.
    pub commands: Vec<DrawUpdate>,

    /// Temporary storage used with generating and optimizing sprites draw calls
    pub sprites_builder: Vec<TempSprite>,

    /// Output update flags
    pub updates: u8,
}

impl GameOutput {

    pub fn clear(&mut self) {
        self.commands.clear();
        self.sprites_builder.clear();
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

    pub fn update_animations(&mut self) { self.updates |= UPDATE_ANIMATION; }
    fn must_update_animation(&self) -> bool { self.updates & UPDATE_ANIMATION > 0 }
    fn clear_update_animation(&mut self) { self.updates &= !UPDATE_ANIMATION; }

}

pub fn update(game: &mut DemoGame) {
    game.output.clear();
    update_view(game);
    update_terrain(game);
    render_terrain(game);
    render_sprites(game);
    game.output.write_index();
}

fn update_view(game: &mut DemoGame) {
    let output = &mut game.output;
    if !output.must_sync_view() {
        return;
    }

    output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::UpdateViewOffset,
        params: DrawUpdateParams { update_view_offset: game.data.global.view_offset },
    });

    output.clear_sync_view();
}

fn update_terrain(game: &mut DemoGame) {
    const CELL_TEXEL_SIZE: f32 = 64.0;

    let output = &mut game.output;
    if !output.must_sync_world() {
        return;
    }

    let terrain = &mut game.data.world.terrain;
    let terrain_tilemap = &game.data.assets.terrain;
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

fn render_terrain(game: &mut DemoGame) {
    let output = &mut game.output;
    let view = aabb(game.data.global.view_offset, game.data.global.view_size);

    let mut params = DrawTerrainChunkParams { chunk_id: 0, x: 0.0, y: 0.0 };

    for chunk in game.data.world.terrain.chunks.iter() {
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

/**
    This function does a few things:

    * Update the animations if the UPDATE_ANIMATION flag was set
    * Generates sprites from the different world objects
    * Regroup and order sprites by their Y coordinates from the highest (rendered first), to the lowest (rendered last)
    * Generate batches of commands for the engine to render
*/
fn render_sprites(game: &mut DemoGame) {
    let world = &mut game.data.world;
    let output = &mut game.output;

    let total_sprites = world.total_sprites();
    if total_sprites == 0 {
        return;
    }

    output.sprites_builder.reserve(total_sprites);

    // Generate sprites
    if output.must_update_animation() {
        gen_sprites_with_animation(world, output);
    } else {
        gen_sprites(world, output);
    }

    gen_static_sprites(world, output);

    // Order
    order_sprites(output);

    // Commands
    gen_commands(output);
    
    output.clear_update_animation();
}

fn gen_sprites(world: &crate::world::World, output: &mut GameOutput) {
    let sprite_groups = [
        (world.units_texture.id, &world.pawns),
        (world.units_texture.id, &world.warriors),
        (world.units_texture.id, &world.archers),
        (world.units_texture.id, &world.torch_goblins),
        (world.units_texture.id, &world.tnt_goblins),
        (world.units_texture.id, &world.sheeps),
        (world.static_resources_texture.id, &world.resources_spawn),
        (world.static_resources_texture.id, &world.trees),
    ];

    let builder = &mut output.sprites_builder;
    for (texture_id, sprites) in sprite_groups {
        for unit in sprites.iter() {
            let sprite = build_actor_sprite(unit);
            builder.push(TempSprite {
                texture_id,
                y: unit.position.y,
                sprite
            });
        }
    }
}

fn gen_sprites_with_animation(world: &mut crate::world::World, output: &mut GameOutput) {
    let sprite_groups = [
        (world.units_texture.id, &mut world.pawns),
        (world.units_texture.id, &mut world.warriors),
        (world.units_texture.id, &mut world.archers),
        (world.units_texture.id, &mut world.torch_goblins),
        (world.units_texture.id, &mut world.tnt_goblins),
        (world.units_texture.id, &mut world.sheeps),
        (world.static_resources_texture.id, &mut world.resources_spawn),
        (world.static_resources_texture.id, &mut world.trees),
    ];

    let builder = &mut output.sprites_builder;
    for (texture_id, sprites) in sprite_groups {
        for unit in sprites.iter_mut() {
            unit.current_frame += 1;
            unit.current_frame = unit.current_frame * ((unit.current_frame <= unit.animation.last_frame) as u8);

            let sprite = build_actor_sprite(unit);
            builder.push(TempSprite {
                texture_id,
                y: unit.position.y,
                sprite
            });
        }
    }
}

fn gen_static_sprites(world: &crate::world::World, output: &mut GameOutput) {
    let texture_id = world.static_resources_texture.id;
    let sprites_groups = [&world.decorations, &world.structures, &world.resources];
    let builder = &mut output.sprites_builder;
    for group in sprites_groups {
        for unit in group {
            let sprite = build_static_sprite(unit);
            builder.push(TempSprite {
                texture_id,
                y: unit.position.y,
                sprite
            });
        }
    }
}

fn order_sprites(output: &mut GameOutput) {
    use std::cmp::Ordering;

    // Sprites with a lower Y value gets rendered first
    output.sprites_builder.sort_unstable_by(|v1, v2| {
        let diff = v1.y - v2.y;
        if diff < -1.0 {
            Ordering::Less
        } else if diff > 1.0 {
            Ordering::Greater
        } else {
            v1.texture_id.cmp(&v2.texture_id)
        }
    });
}

fn gen_commands(output: &mut GameOutput) {
    let texture_id = output.sprites_builder.first().map(|v| v.texture_id ).unwrap_or(0);
    let mut params = DrawSpriteParams {
        instance_base: 0,
        instance_count: 0,
        texture_id,
    };

    let sprites_data = &mut output.sprite_data_buffer;

    for build_sprite in output.sprites_builder.iter() {
        if build_sprite.texture_id != params.texture_id {
            output.commands.push(DrawUpdate {
                graphics: DrawUpdateType::DrawSprites,
                params: DrawUpdateParams { draw_sprites: params },
            });

            params.instance_base = sprites_data.len() as u32;
            params.instance_count = 0;
            params.texture_id = build_sprite.texture_id;
        };

        sprites_data.push(build_sprite.sprite);
        params.instance_count += 1; 
    }

    output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::DrawSprites,
        params: DrawUpdateParams { draw_sprites: params },
    });
}

fn build_actor_sprite(unit: &crate::world::BaseAnimated) -> SpriteData {
    let mut sprite = SpriteData::default();
    let position = unit.position;
    let animation = unit.animation;
    let i = unit.current_frame as f32;

    sprite.position[0] = position.x - (animation.sprite_width * 0.5);
    sprite.position[1] = position.y - animation.sprite_height;
    sprite.size[0] = animation.sprite_width;
    sprite.size[1] = animation.sprite_height;
    sprite.texcoord_offset[0] = animation.x + (animation.sprite_width * i);
    sprite.texcoord_offset[1] = animation.y;
    sprite.texcoord_size[0] = animation.sprite_width;
    sprite.texcoord_size[1] = animation.sprite_height;
    sprite.data += 1 * (unit.selected as i32);
    sprite.data += 2 * (unit.flipped as i32);

    sprite
}

fn build_static_sprite(base: &crate::world::BaseStatic) -> SpriteData {
        let mut sprite = SpriteData::default();
        let position = base.position;
        let aabb = base.aabb;
        sprite.position[0] = position.x - (aabb.width() * 0.5);
        sprite.position[1] = position.y - aabb.height();
        sprite.size[0] = aabb.width();
        sprite.size[1] = aabb.height();
        sprite.texcoord_offset[0] = aabb.left;
        sprite.texcoord_offset[1] = aabb.top;
        sprite.texcoord_size[0] = sprite.size[0];
        sprite.texcoord_size[1] = sprite.size[1];
        sprite.data = 1 * (base.selected as i32);
        sprite
    }

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            sprite_data_buffer: Vec::with_capacity(32),
            terrain_data: Vec::with_capacity(1024),
            commands: Vec::with_capacity(32),
            sprites_builder: Vec::with_capacity(128),
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
