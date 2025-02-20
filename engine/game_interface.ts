/// Interface between the wasm client and the engine

import { DemoGame } from "../build/game/game";

const OUTPUT_INDEX_SIZE: number = 16;  // size_of(OutputIndex)
const DRAW_UPDATE_SIZE: number = 4;    // size_of(DrawUpdate)

const OUPUT_INDEX_DRAW_UPDATES_OFFSET: number = 4;
const OUPUT_INDEX_DRAW_UPDATES_COUNT_OFFSET: number = 8;

const DRAW_UPDATE_GRAPHICS_MODULE_OFFSET: number = 0;

export enum GraphicsModule {
    Undefined = 0,
    DrawSprites = 1,
}

export class EngineGameDrawUpdate {
    module: GraphicsModule = GraphicsModule.Undefined;
}

export class EngineGameInstanceOutput {
    buffer: ArrayBuffer;
    index: DataView;

    // Reusable storage so javascript doesn't create a new object for every call to `get_draw_update`
    last_draw_update: EngineGameDrawUpdate = new EngineGameDrawUpdate();

    memory_layout_validation() {
        const ptr_size = this.index.getUint32(0, true)
        if (ptr_size != 4) {
            throw `Engine output pointer should always be 4 bytes in WASM (got ${ptr_size})`;
        }

        const validation_num = this.index.getUint32(12, true);
        if (validation_num != 33355) {
            throw `Failed to validate index memory layout. This usually means the layout of OutputIndex was modified in the rust source, but this file was not updated`;
        }
    }

    draw_updates_count(): number {
        return this.index.getUint32(OUPUT_INDEX_DRAW_UPDATES_COUNT_OFFSET, true);
    }

    get_draw_update(index: number): EngineGameDrawUpdate {
        const draw_updates_offset = this.index.getUint32(OUPUT_INDEX_DRAW_UPDATES_OFFSET, true);
        const local_offset = DRAW_UPDATE_SIZE * index;
        const draw_update_view = new DataView(this.buffer, draw_updates_offset + local_offset, DRAW_UPDATE_SIZE);
        this.last_draw_update.module = draw_update_view.getUint32(DRAW_UPDATE_GRAPHICS_MODULE_OFFSET, true);
        return this.last_draw_update;
    }

}

export class EngineGameInstance {
    instance: DemoGame;
    module: any;
    reload_count: number = 0;
    inner_output: EngineGameInstanceOutput = new EngineGameInstanceOutput();
    validate_memory_layout: boolean = true;

    updates(): EngineGameInstanceOutput {
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
