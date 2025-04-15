/// Structures and system to transfer data from the rust app to an external reader (in this case javascript)
/// Data with `repr(C)` will be directly read from memory by the engine

use crate::shared::{aabb, Position};
use crate::world::{BaseAnimated, BaseProjectile};
use crate::DemoGame;

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
    UpdateGui = 5,
    DrawProjectileSprites = 6,
    DrawDebugInfo = 7,
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
pub struct UpdateGuiParams {
    pub indices_count: u32,
    pub vertex_count: u32,
}

/// DrawDebugParams doesn't have any parameters
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawDebugParams;

#[repr(C)]
#[derive(Copy, Clone)]
pub union DrawUpdateParams {
    pub draw_sprites: DrawSpriteParams,
    pub update_terrain_chunk: UpdateTerrainChunkParams,
    pub draw_terrain_chunk: DrawTerrainChunkParams,
    pub update_view_offset: Position<f32>,
    pub update_gui: UpdateGuiParams,
    pub draw_debug: DrawDebugParams,
}

/// A generic draw update that will be read by the renderere
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DrawUpdate {
    graphics: DrawUpdateType,
    params: DrawUpdateParams,
}

/// Information on how to render a sprite on the GPU
/// Memory layout must match `in_instance_position`, `in_instance_texcoord`, `in_instance_data` in `sprites.vert.glsl`
/// If this struct size change, it must also be updated in `game_interface.ts`
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
    pub data: i32,
}

/// Information on how to render a projectile sprite on the GPU
/// Memory layout must match `in_instance_position`, `in_instance_texcoord`, `in_instance_data` in `proj_sprites.vert.glsl`
/// If this struct size change, it must also be updated in `game_interface.ts`
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct ProjectileSpriteData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub texcoord_offset: [f32; 2],
    pub texcoord_size: [f32; 2],
    pub rotation: f32,
}


/// Texture coordinates for the 4 vertex of a sprite the terrain data buffer
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TerrainChunkTexcoord {
    pub v0: [f32; 2],
    pub v1: [f32; 2],
    pub v2: [f32; 2],
    pub v3: [f32; 2],
}

/// A vertex used in the gui pipeline
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct GuiVertex {
    pub position: [f32; 2],
    pub texcoord: [f32; 2],
    pub color: [u8; 4],
}

/// A vertex in the debug pipeline
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct DebugVertex {
    pub position: [f32; 2],
    pub color: [u8; 4],
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
    pub projectile_sprites_data_ptr: *const ProjectileSpriteData,
    pub projectile_sprites_data_count: usize,
    pub terrain_data_ptr: *const TerrainChunkTexcoord,
    pub terrain_data_count: usize,
    pub gui_indices_ptr: *const u16,
    pub gui_indices_count: usize,
    pub gui_vertex_ptr: *const GuiVertex,
    pub gui_vertex_count: usize,
    pub debug_vertex_ptr: *const DebugVertex,
    pub debug_vertex_count: usize,
    pub validation: usize
}

/// Holds the data buffer shared between the game client and the engine 
pub struct GameOutput {
    /// This is a leaked box because we return the pointer to the client in `output` and `Box::as_ptr` is a nightly-only experimental API
    pub output_index: &'static mut OutputIndex,

    /// Buffers of generated sprites
    pub sprite_data_buffer: Vec<SpriteData>,

    /// Buffer of generated projectile sprites
    pub projectile_sprites_buffer: Vec<ProjectileSpriteData>,

    /// Buffers of generated terrain sprites
    pub terrain_data: Vec<TerrainChunkTexcoord>,

    /// Buffer holding the indices of the gui mesh
    pub gui_indices: Vec<u16>,

    /// Buffer holding the vertex of the gui mesh
    pub gui_vertex: Vec<GuiVertex>,

    /// Buffer holding the vertex of the debug info. Debug info do not use an index buffer
    pub debug_vertex: Vec<DebugVertex>,

    /// Buffers of the generated draw update for the current frame.
    pub commands: Vec<DrawUpdate>,

    /// Temporary storage used when generating and optimizing sprites draw calls
    pub sprites_builder: Vec<TempSprite>,
}

impl GameOutput {

    pub fn clear(&mut self) {
        self.commands.clear();
        self.sprites_builder.clear();
        self.projectile_sprites_buffer.clear();
        self.sprite_data_buffer.clear();
        self.terrain_data.clear();
        self.debug_vertex.clear();
    }

    pub fn write_index(&mut self) {
        let index = &mut self.output_index;
        index.commands_ptr = self.commands.as_ptr();
        index.commands_count = self.commands.len();
        index.sprites_data_ptr = self.sprite_data_buffer.as_ptr();
        index.sprites_data_count = self.sprite_data_buffer.len();
        index.projectile_sprites_data_ptr = self.projectile_sprites_buffer.as_ptr();
        index.projectile_sprites_data_count = self.projectile_sprites_buffer.len();
        index.terrain_data_ptr = self.terrain_data.as_ptr();
        index.terrain_data_count = self.terrain_data.len();
        index.gui_indices_ptr = self.gui_indices.as_ptr();
        index.gui_indices_count = self.gui_indices.len();
        index.gui_vertex_ptr = self.gui_vertex.as_ptr();
        index.gui_vertex_count = self.gui_vertex.len();
        index.debug_vertex_ptr = self.debug_vertex.as_ptr();
        index.debug_vertex_count = self.debug_vertex.len();
    }

}

pub fn update(game: &mut DemoGame) {
    game.output.clear();
    update_view(game);
    update_terrain(game);
    render_terrain(game);
    render_sprites(game);
    render_projectiles(game);
    render_gui(game);

    #[cfg(feature="debug")]
    render_debug(game);
    
    game.output.write_index();
}

/**
    Update the engine view offset
*/
fn update_view(game: &mut DemoGame) {
    let flags = &mut game.data.global.flags;
    if !flags.get_sync_view() {
        return;
    }

    flags.clear_sync_view();

    game.output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::UpdateViewOffset,
        params: DrawUpdateParams { update_view_offset: game.data.global.view_offset },
    });
}

/**
    Synchronise the terrain chunk data with the engine
*/
fn update_terrain(game: &mut DemoGame) {
    const CELL_TEXEL_SIZE: f32 = 64.0;
    if !game.data.global.flags.get_sync_terrain() {
        return;
    }

    game.data.global.flags.clear_sync_terrain();

    let output = &mut game.output;
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
} 

/**
    Generate rendering command to draw the terrain from the data synchronized with `update_terrain`
*/
fn render_terrain(game: &mut DemoGame) {
    let output = &mut game.output;
    let view = aabb(game.data.global.view_offset, game.data.inputs.view_size);

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
    let flags = &mut game.data.global.flags;
    let world = &mut game.data.world;
    let output = &mut game.output;

    let total_sprites = world.total_sprites();
    if total_sprites == 0 {
        return;
    }

    output.sprites_builder.reserve(total_sprites);

    // Generate sprites
    if flags.get_update_animations() {
        gen_sprites_with_animation(world, output);
        flags.clear_update_animations();
    } else {
        gen_sprites(world, output);
    }

    gen_static_sprites(world, output);

    // Order
    order_sprites(output);

    // Commands
    gen_commands(output);
}

fn gen_sprites(world: &crate::world::World, output: &mut GameOutput) {
    let sprite_groups: [(u32, &[BaseAnimated]); 8] = [
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
    let sprite_groups: [(u32, &mut [BaseAnimated]); 8] = [
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
    let sprites_groups = [&world.decorations, &world.structures];
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

    // Grabbed resource have a different y position if they are grabbed
    for (resource, resource_data) in world.resources.iter().zip(world.resources_data.iter()) {
        let sprite = build_static_sprite(resource);
        let y = match resource_data.grabbed {
            true => resource.position.y + 60.0,
            false => resource.position.y,
        };

        builder.push(TempSprite {
            texture_id,
            y,
            sprite
        });
    }
}

fn order_sprites(output: &mut GameOutput) {
    use std::cmp::Ordering;

    // Sprites with a lower Y value gets rendered first
    output.sprites_builder.sort_unstable_by(|v1, v2| {
        match v1.y.total_cmp(&v2.y) {
            v @ (Ordering::Greater | Ordering::Less) => v,
            _ =>  v1.texture_id.cmp(&v2.texture_id)
        }
    });
}

fn gen_commands(output: &mut GameOutput) {
    let texture_id = output.sprites_builder.first().map(|v| v.texture_id ).unwrap_or(0);
    let mut draw_sprites = DrawSpriteParams {
        instance_base: 0,
        instance_count: 0,
        texture_id,
    };

    let sprites_data = &mut output.sprite_data_buffer;

    for build_sprite in output.sprites_builder.iter() {
        if build_sprite.texture_id != draw_sprites.texture_id {
            output.commands.push(DrawUpdate {
                graphics: DrawUpdateType::DrawSprites,
                params: DrawUpdateParams { draw_sprites },
            });

            draw_sprites.instance_base = sprites_data.len() as u32;
            draw_sprites.instance_count = 0;
            draw_sprites.texture_id = build_sprite.texture_id;
        };

        sprites_data.push(build_sprite.sprite);
        draw_sprites.instance_count += 1; 
    }

    output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::DrawSprites,
        params: DrawUpdateParams { draw_sprites },
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
        let aabb = base.sprite;

        sprite.size[0] = aabb.width();
        sprite.size[1] = aabb.height();
        sprite.position[0] = position.x - (sprite.size[0] * 0.5);
        sprite.position[1] = position.y - sprite.size[1];
        sprite.texcoord_offset[0] = aabb.left;
        sprite.texcoord_offset[1] = aabb.top;
        sprite.texcoord_size[0] = sprite.size[0];
        sprite.texcoord_size[1] = sprite.size[1];
        sprite.data = 1 * (base.selected as i32);
        sprite
    }

/**
    Unlike normal sprites, projectile sprites can be rotated and cannot be selected. So they have their own pipeline.
    Also projectile sprites don't need to be y-ordered, they all share the same texture (so far), and are always rendered on top of the other sprites.
*/
fn render_projectiles(game: &mut DemoGame) {
    let world = &mut game.data.world;
    let output = &mut game.output;

    let total_sprites = world.total_projectile_sprites();
    if total_sprites == 0 {
        return;
    }

    let projectile_groups: [&[BaseProjectile]; 1] = [
        &world.arrows,
    ];

    for projectile_group in projectile_groups {
        for projectile in projectile_group {
            let position = projectile.position;
            let aabb = projectile.sprite;
            let mut sprite = ProjectileSpriteData::default();

            sprite.size[0] = aabb.width();
            sprite.size[1] = aabb.height();
            sprite.position[0] = position.x;
            sprite.position[1] = position.y;
            sprite.texcoord_offset[0] = aabb.left;
            sprite.texcoord_offset[1] = aabb.top;
            sprite.texcoord_size[0] = sprite.size[0];
            sprite.texcoord_size[1] = sprite.size[1];
            sprite.rotation = projectile.rotation;

            output.projectile_sprites_buffer.push(sprite);
        }
    }

    let draw_sprites = DrawSpriteParams {
        instance_base: 0,
        instance_count: total_sprites as u32,
        texture_id: world.static_resources_texture.id,
    };

    output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::DrawProjectileSprites,
        params: DrawUpdateParams { draw_sprites },
    });
}

/**
    Generate the gui sprites. If the gui wasn't updated since the last frame, this doesn't do anything
*/
fn render_gui(game: &mut DemoGame) {
    let gui = &mut game.data.gui;
    let output = &mut game.output;

    if !gui.needs_sync() {
        return;
    }

    gui.build_sprites();
    output.gui_indices.clear();
    output.gui_vertex.clear();

    let mut v = 0;
    for sprite in gui.sprites().iter() {
        output.gui_indices.extend_from_slice(&[v+0, v+3, v+2, v+1, v+0, v+3]);
 
        let [left, top, right, bottom] = sprite.positions.splat();
        let [tleft, ttop, tright, tbottom] = sprite.texcoord.splat();
        
        // The alpha channels holds rendering flags
        let [r, g, b] = sprite.color.splat();
        let color = [r, g, b, sprite.flags];

        output.gui_vertex.extend_from_slice(&[
            GuiVertex { position: [left, top], texcoord: [tleft, ttop], color },
            GuiVertex { position: [right, top], texcoord: [tright, ttop], color },
            GuiVertex { position: [left, bottom], texcoord: [tleft, tbottom], color },
            GuiVertex { position: [right, bottom], texcoord: [tright, tbottom], color },
        ]);

        v += 4;
    }

    if output.gui_indices.len() > 0 {
        output.commands.push(DrawUpdate {
            graphics: DrawUpdateType::UpdateGui,
            params: DrawUpdateParams { 
                update_gui: UpdateGuiParams {
                    indices_count: output.gui_indices.len() as u32,
                    vertex_count: output.gui_vertex.len() as u32,
                } 
            },
        });
    }
}

#[cfg(feature="debug")]
fn render_debug(game: &mut DemoGame) {
    use crate::debug::DebugElement;
    use crate::shared::{AABB, aabb, pos, size};

    fn debug_rect(vertex: &mut Vec<DebugVertex>, aabb: &AABB, color: [u8; 4]) {
        vertex.push(DebugVertex { position: [aabb.left, aabb.top],     color });
        vertex.push(DebugVertex { position: [aabb.left, aabb.bottom],  color });
        vertex.push(DebugVertex { position: [aabb.right, aabb.bottom], color });

        vertex.push(DebugVertex { position: [aabb.right, aabb.top],    color });
        vertex.push(DebugVertex { position: [aabb.left, aabb.top],     color });
        vertex.push(DebugVertex { position: [aabb.right, aabb.bottom], color });
    }

    fn debug_line(vertex: &mut Vec<DebugVertex>, p1: Position<f32>, p2: Position<f32>, color: [u8; 4]) {
        let angle = f32::atan2(p2.y-p1.y, p2.x-p1.x);
        let y = 1.0 * f32::cos(angle);
        let x = 1.0 * f32::sin(angle);

        vertex.push(DebugVertex { position: [p1.x+x, p1.y-y],  color });
        vertex.push(DebugVertex { position: [p1.x-x, p1.y+y],  color });
        vertex.push(DebugVertex { position: [p2.x-x, p2.y+y],      color });

        vertex.push(DebugVertex { position: [p2.x+x, p2.y-y],      color });
        vertex.push(DebugVertex { position: [p1.x+x, p1.y-y],  color });
        vertex.push(DebugVertex { position: [p2.x-x, p2.y+y],      color });
    }
    
    let vertex = &mut game.output.debug_vertex;
    let elements = &game.data.debug.elements;
    if elements.len() == 0 {
        return;
    }

    for element in elements.iter() {
        match *element {
            DebugElement::Point { pt, size: size_value, color } => {
                let half_size = size_value / 2.0;
                let position = pos(pt.x - half_size, pt.y-half_size);
                let size = size(size_value, size_value);
                debug_rect(vertex, &aabb(position, size), color);
            },
            DebugElement::Line { start, end, color } => {
                debug_line(vertex, start, end, color);
            },
            DebugElement::Triangle { v0, v1, v2, color } => {
                debug_line(vertex, v0, v1, color);
                debug_line(vertex, v1, v2, color);
                debug_line(vertex, v0, v2, color);
            }
            DebugElement::Rect { base, color } => {
                let rect_size = base.size();
                debug_rect(vertex, &aabb(pos(base.left, base.top), size(rect_size.width, 2.0)), color);         // Top
                debug_rect(vertex, &aabb(pos(base.left, base.bottom-2.0), size(rect_size.width, 2.0)), color);  // Bottom
                debug_rect(vertex, &aabb(pos(base.left, base.top), size(2.0, rect_size.height)), color);        // Left
                debug_rect(vertex, &aabb(pos(base.right-2.0, base.top), size(2.0, rect_size.height)), color);   // Right
            },
        }
    }

    game.output.commands.push(DrawUpdate {
        graphics: DrawUpdateType::DrawDebugInfo,
        params: DrawUpdateParams { draw_debug: DrawDebugParams },
    });

    game.data.debug.clear();
}

impl Default for GameOutput {

    fn default() -> Self {
        let output_index: Box<OutputIndex> = Box::default();
        GameOutput {
            output_index: Box::leak(output_index),
            sprite_data_buffer: Vec::with_capacity(64),
            projectile_sprites_buffer: Vec::with_capacity(32),
            terrain_data: Vec::with_capacity(1024),
            gui_indices: Vec::with_capacity(1500),
            gui_vertex: Vec::with_capacity(1000),
            debug_vertex: Vec::with_capacity(256),
            commands: Vec::with_capacity(32),
            sprites_builder: Vec::with_capacity(64),
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
            projectile_sprites_data_ptr: ::std::ptr::null(),
            projectile_sprites_data_count: 0,
            terrain_data_ptr: ::std::ptr::null(),
            terrain_data_count: 0,
            gui_indices_ptr: ::std::ptr::null(),
            gui_indices_count: 0,
            gui_vertex_ptr: ::std::ptr::null(),
            gui_vertex_count: 0,
            debug_vertex_ptr: ::std::ptr::null(),
            debug_vertex_count: 0,
            validation: 33355,
        }
    }
}
