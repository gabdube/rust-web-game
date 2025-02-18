import { set_last_error } from "./error";
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

    update() {
        this.backend?.update();
    }

    render() {
        this.backend?.render();
    }

}

