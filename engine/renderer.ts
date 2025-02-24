import { EngineGameInstance } from "./game_interface";
import { EngineAssets } from "./assets";
import { set_last_error } from "./error";
import { Size } from "./helpers";
import { WebGL2Backend } from "./renderer/webgl2_renderer";


export class Renderer {
    type: string;
    backend: WebGL2Backend | null;

    constructor() {
        this.type = "undefined";
        this.backend = null;
    }

    init(): boolean {
        if (this.type !== "undefined") {
            set_last_error("Renderer was already initialized");
            return false;
        }

        const webgl2_backend = new WebGL2Backend();
        if (webgl2_backend.init()) {
            this.type = "webgl2";
            this.backend = webgl2_backend;
            return true;
        }
       
        set_last_error("Could not find a supported graphics backend");
        return false;
    }

    init_default_resources(assets: EngineAssets): boolean {
        if (this.backend) {
            return this.backend.init_default_resources(assets);
        } else {
            set_last_error("init_default_resources called on an uninitialized renderer")
            return false;
        }
    }

    canvas(): HTMLCanvasElement {
        if (this.backend) {
            return this.backend.canvas.element;
        } else {
            console.error("canvas_size was called on an uninitialized renderer");
            return new HTMLCanvasElement();
        }
    }

    update(game: EngineGameInstance) {
        this.backend?.update(game);
    }

    render() {
        this.backend?.render();
    }

    canvas_size(): Size {
        if (this.backend) {
            return this.backend.canvas_size();
        } else {
            console.error("canvas_size was called on an uninitialized renderer");
            return { width: 0, height: 0 };
        }
    }

    handle_resize() {
        if (this.backend) {
            return this.backend.handle_resize();
        } else {
            console.error("handle_resize was called on an uninitialized renderer");
        }
    }

}

