/// Interface between the wasm client and the engine

import { DemoGame } from "../build/game/game";

const OUTPUT_INDEX_SIZE: number = 56;  // size_of(OutputIndex)
const DRAW_UPDATE_SIZE: number = 16;   // size_of(DrawUpdate)
export const SPRITE_DATA_SIZE: number = 36;       // size_of(SpriteData) && size_of(ProjectileSpriteData)
const TERRAIN_CHUNK_TEXT_COORD_SIZE: number = 32; // size_of(TerrainChunkTexcoord)
export const GUI_VERTEX_SIZE: number = 20;        // size_of(GuiVertex)

const OUTPUT_INDEX_DRAW_UPDATES_OFFSET: number = 4;
const OUTPUT_INDEX_DRAW_UPDATES_COUNT_OFFSET: number = 8;

const OUTPUT_INDEX_SPRITE_DATA_OFFSET: number = 12;
const OUTPUT_INDEX_SPRITE_DATA_COUNT_OFFSET: number = 16;

const OUTPUT_INDEX_PROJ_SPRITE_DATA_OFFSET: number = 20;
const OUTPUT_INDEX_PROJ_SPRITE_DATA_COUNT_OFFSET: number = 24;

const OUTPUT_INDEX_TERRAIN_DATA_OFFSET: number = 28;
const OUTPUT_INDEX_TERRAIN_DATA_COUNT_OFFSET: number = 32;

const OUTPUT_INDEX_GUI_INDICES_OFFSET: number = 36;
const OUTPUT_INDEX_GUI_INDICES_COUNT_OFFSET: number = 40;

const OUTPUT_INDEX_GUI_VERTEX_OFFSET: number = 44;
const OUTPUT_INDEX_GUI_VERTEX_COUNT_OFFSET: number = 48;

const OUTPUT_INDEX_VALIDATION_INDEX: number = 52;

const DRAW_UPDATE_GRAPHICS_MODULE_OFFSET: number = 0;

export const TERRAIN_CHUNK_STRIDE: number = 16;
export const TERRAIN_CHUNK_CELL_COUNT: number = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE;
export const TERRAIN_CHUNK_SIZE_BYTES: number = TERRAIN_CHUNK_CELL_COUNT * TERRAIN_CHUNK_TEXT_COORD_SIZE;

export enum DrawUpdateType {
    Undefined = 0,
    DrawSprites = 1,
    UpdateTerrainChunk = 2,
    DrawTerrainChunk = 3,
    UpdateViewOffset = 4,
    UpdateGui = 5,
    DrawProjectileSprites = 6,
}

export class EngineGameDrawUpdate {
    module: DrawUpdateType = DrawUpdateType.Undefined;

    // DrawTerrainChunk parameters
    chunk_id: number;
    chunk_x: number;
    chunk_y: number;

    // UpdateTerrainChunk params
    chunk_data_offset: number;

    // DrawSprites / DrawProjectileSprites parameters
    instance_base: number;
    instance_count: number;
    texture_id: number;

    // Update view params
    view_x: number;
    view_y: number;

    // Gui update params
    gui_indices_count: number;
    gui_vertex_count: number;
}

export class EngineGameInstanceUpdates {
    buffer: ArrayBuffer;
    index: DataView;

    // Reusable storage so javascript doesn't create a new object for every call to `get_draw_update`
    last_draw_update: EngineGameDrawUpdate = new EngineGameDrawUpdate();

    memory_layout_validation() {
        const ptr_size = this.index.getUint8(0)
        if (ptr_size != 4) {
            throw `Engine output pointer should always be 4 bytes in WASM (got ${ptr_size})`;
        }

        const validation_num = this.index.getUint32(OUTPUT_INDEX_VALIDATION_INDEX, true);
        if (validation_num != 33355) {
            throw `Failed to validate index memory layout. This usually means the layout of OutputIndex was modified in the rust source, but this file was not updated`;
        }
    }

    draw_updates_count(): number {
        return this.index.getUint32(OUTPUT_INDEX_DRAW_UPDATES_COUNT_OFFSET, true);
    }

    get_draw_update(index: number): EngineGameDrawUpdate {
        const draw_updates_offset = this.index.getUint32(OUTPUT_INDEX_DRAW_UPDATES_OFFSET, true);
        const local_offset = DRAW_UPDATE_SIZE * index;
        const draw_update_view = new DataView(this.buffer, draw_updates_offset + local_offset, DRAW_UPDATE_SIZE);

        const draw = this.last_draw_update
        draw.module = draw_update_view.getUint32(DRAW_UPDATE_GRAPHICS_MODULE_OFFSET, true);

        switch (draw.module) {
            case DrawUpdateType.UpdateTerrainChunk: {
                draw.chunk_id = draw_update_view.getUint32(4, true);
                draw.chunk_data_offset = draw_update_view.getUint32(8, true);
                break;
            }
            case DrawUpdateType.DrawTerrainChunk: {
                draw.chunk_id = draw_update_view.getUint32(4, true);
                draw.chunk_x = draw_update_view.getFloat32(8, true);
                draw.chunk_y = draw_update_view.getFloat32(12, true);
                break;
            }
            case DrawUpdateType.DrawSprites: {
                draw.instance_base = draw_update_view.getUint32(4, true);
                draw.instance_count = draw_update_view.getUint32(8, true);
                draw.texture_id = draw_update_view.getUint32(12, true);
                break;
            }
            case DrawUpdateType.UpdateViewOffset: {
                draw.view_x = draw_update_view.getFloat32(4, true);
                draw.view_y = draw_update_view.getFloat32(8, true);
                break;
            }
            case DrawUpdateType.UpdateGui: {
                draw.gui_indices_count = draw_update_view.getUint32(4, true);
                draw.gui_vertex_count = draw_update_view.getUint32(8, true);
                break;
            }
            case DrawUpdateType.DrawProjectileSprites: {
                draw.instance_base = draw_update_view.getUint32(4, true);
                draw.instance_count = draw_update_view.getUint32(8, true);
                draw.texture_id = draw_update_view.getUint32(12, true);
                break;
            }
            default: {
                console.error("Error: Received unknown draw update type");
            }
        }
    
        return draw;
    }

    get_sprites_data(instance_base: number, instance_count: number): ArrayBuffer {
        const sprites_data_base = this.index.getUint32(OUTPUT_INDEX_SPRITE_DATA_OFFSET, true);
        const sprites_data_begin = sprites_data_base + (SPRITE_DATA_SIZE * instance_base);
        const sprites_data_end = sprites_data_begin + (SPRITE_DATA_SIZE * instance_count);
        return this.buffer.slice(sprites_data_begin, sprites_data_end);
    } 

    get_projectile_sprites_data(instance_base: number, instance_count: number): ArrayBuffer {
        const sprites_data_base = this.index.getUint32(OUTPUT_INDEX_PROJ_SPRITE_DATA_OFFSET, true);
        const sprites_data_begin = sprites_data_base + (SPRITE_DATA_SIZE * instance_base);
        const sprites_data_end = sprites_data_begin + (SPRITE_DATA_SIZE * instance_count);
        return this.buffer.slice(sprites_data_begin, sprites_data_end);
    }

    get_terrain_data(chunk_data_offset: number): ArrayBuffer {
        const terrain_data_base = this.index.getUint32(OUTPUT_INDEX_TERRAIN_DATA_OFFSET, true);
        const terrain_data_begin = terrain_data_base + (chunk_data_offset * TERRAIN_CHUNK_TEXT_COORD_SIZE);
        const terrain_data_end = terrain_data_begin + TERRAIN_CHUNK_SIZE_BYTES;
        return this.buffer.slice(terrain_data_begin, terrain_data_end);
    }

    get_gui_indices_data(): ArrayBuffer {
        const INDEX_SIZE = 2;
        const gui_indices_base = this.index.getUint32(OUTPUT_INDEX_GUI_INDICES_OFFSET, true);
        const gui_indices_count = this.index.getUint32(OUTPUT_INDEX_GUI_INDICES_COUNT_OFFSET, true);
        return this.buffer.slice(gui_indices_base, gui_indices_base+(gui_indices_count*INDEX_SIZE));
    }

    get_gui_vertex_data(): ArrayBuffer {
        const gui_vertex_base = this.index.getUint32(OUTPUT_INDEX_GUI_VERTEX_OFFSET, true);
        const gui_vertex_count = this.index.getUint32(OUTPUT_INDEX_GUI_VERTEX_COUNT_OFFSET, true);
        return this.buffer.slice(gui_vertex_base, gui_vertex_base+(gui_vertex_count*GUI_VERTEX_SIZE));
    }

}

export class EngineGameInstance {
    instance: DemoGame;
    module: any;
    reload_count: number = 0;
    inner_output: EngineGameInstanceUpdates = new EngineGameInstanceUpdates();
    validate_memory_layout: boolean = true;

    updates(): EngineGameInstanceUpdates {
        let output = this.inner_output;
        output.buffer = this.get_memory();
        
        const index_offset = this.instance.updates_ptr();
        const index_size = OUTPUT_INDEX_SIZE;  
        output.index = new DataView(output.buffer, index_offset, index_size);

        if (this.validate_memory_layout) {
            output.memory_layout_validation();
            this.validate_memory_layout = false;
        }

        return output;
    }

    get_memory(): ArrayBuffer {
        if (this.module) {
            // If the module was already initialized, this only returns the wasm memory
            return this.module.initSync().memory.buffer;
        } else {
            throw "Client module is not loaded";
        }
    }
}
