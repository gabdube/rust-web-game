/// Interface between the wasm client and the engine

import { DemoGame } from "../build/game/game";

const OUTPUT_INDEX_SIZE: number = 32;  // size_of(OutputIndex)
const DRAW_UPDATE_SIZE: number = 16;   // size_of(DrawUpdate)
export const SPRITE_DATA_SIZE: number = 32;       // size_of(SpriteData)
const TERRAIN_CHUNK_TEXT_COORD_SIZE: number = 32; //size_of(TerrainChunkTexcoord)

const OUTPUT_INDEX_DRAW_UPDATES_OFFSET: number = 4;
const OUTPUT_INDEX_DRAW_UPDATES_COUNT_OFFSET: number = 8;
const OUTPUT_INDEX_SPRITE_DATA_OFFSET: number = 12;
const OUTPUT_INDEX_SPRITE_DATA_COUNT_OFFSET: number = 16;
const OUTPUT_INDEX_TERRAIN_DATA_OFFSET: number = 20;
const OUTPUT_INDEX_TERRAIN_DATA_COUNT_OFFSET: number = 24;
const OUTPUT_INDEX_VALIDATION_INDEX: number = 28;

const DRAW_UPDATE_GRAPHICS_MODULE_OFFSET: number = 0;

export const TERRAIN_CHUNK_STRIDE: number = 16;
export const TERRAIN_CHUNK_CELL_COUNT: number = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE;
export const TERRAIN_CHUNK_SIZE_BYTES: number = TERRAIN_CHUNK_CELL_COUNT * TERRAIN_CHUNK_TEXT_COORD_SIZE;

export enum GraphicsModule {
    Undefined = 0,
    DrawSprites = 1,
    UpdateTerrainChunk = 2,
    DrawTerrainChunk = 3,
}

export class EngineGameDrawUpdate {
    module: GraphicsModule = GraphicsModule.Undefined;

    // DrawTerrainChunk parameters
    chunk_id: number;
    chunk_x: number;
    chunk_y: number;

    // UpdateTerrainChunk params
    chunk_data_offset: number;

    // DrawSprites parameters
    instance_base: number;
    instance_count: number;
    texture_id: number;
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
            case GraphicsModule.UpdateTerrainChunk: {
                draw.chunk_id = draw_update_view.getUint32(4, true);
                draw.chunk_data_offset = draw_update_view.getUint32(8, true);
                break;
            }
            case GraphicsModule.DrawTerrainChunk: {
                draw.chunk_id = draw_update_view.getUint32(4, true);
                draw.chunk_x = draw_update_view.getFloat32(8, true);
                draw.chunk_y = draw_update_view.getFloat32(12, true);
                break;
            }
            case GraphicsModule.DrawSprites: {
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

    get_terrain_data(chunk_data_offset: number): ArrayBuffer {
        const terrain_data_base = this.index.getUint32(OUTPUT_INDEX_TERRAIN_DATA_OFFSET, true);
        const terrain_data_begin = terrain_data_base + (chunk_data_offset * TERRAIN_CHUNK_TEXT_COORD_SIZE);
        const terrain_data_end = terrain_data_begin + TERRAIN_CHUNK_SIZE_BYTES;
        return this.buffer.slice(terrain_data_begin, terrain_data_end);
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
