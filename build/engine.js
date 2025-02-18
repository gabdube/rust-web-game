let LAST_ERROR = null;
class Error {
    constructor(msg, tb) {
        this.message = msg;
        this.traceback = tb;
    }
}
function set_last_error(msg, tb) {
    LAST_ERROR = new Error(msg, null);
}
function get_last_error() {
    return LAST_ERROR;
}

class RendererCanvas {
    constructor(element) {
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}
class WebGL2Backend {
    init() {
        if (!this.setup_canvas()) {
            return false;
        }
        if (!this.setup_context()) {
            return false;
        }
        if (!this.setup_framebuffer()) {
            return false;
        }
        return true;
    }
    update() {
    }
    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);
        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }
    setup_canvas() {
        const canvas_elem = document.getElementById("app");
        if (!canvas_elem) {
            set_last_error("Canvas element was not found");
            return false;
        }
        const dpr = window.devicePixelRatio;
        const display_width = Math.round(canvas_elem.clientWidth * dpr);
        const display_height = Math.round(canvas_elem.clientHeight * dpr);
        canvas_elem.width = display_width;
        canvas_elem.height = display_height;
        this.canvas = new RendererCanvas(canvas_elem);
        this.canvas.width = display_width;
        this.canvas.height = display_height;
        return true;
    }
    setup_context() {
        const canvas = this.canvas;
        const ctx = canvas.element.getContext("webgl2", {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultipliedAlpha: true,
            preserveDrawingBuffer: false,
        });
        if (!ctx) {
            set_last_error("Webgl2 not supported");
            return false;
        }
        this.ctx = ctx;
        this.ctx.viewport(0, 0, canvas.width, canvas.height);
        return true;
    }
    setup_framebuffer() {
        const canvas = this.canvas;
        const ctx = this.ctx;
        const framebuffer = ctx.createFramebuffer();
        if (!framebuffer) {
            set_last_error("Failed to create the renderer framebuffer");
            return false;
        }
        const color = ctx.createRenderbuffer();
        if (!color) {
            set_last_error("Failed to create the renderer color render buffer");
            return false;
        }
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, framebuffer);
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height);
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, color);
        this.framebuffer = framebuffer;
        this.color = color;
        return true;
    }
    get_samples() {
        let max_samples = this.ctx.getParameter(this.ctx.MAX_SAMPLES);
        // We don't need more than 4x msaa
        if (max_samples > 4) {
            max_samples = 4;
        }
        return max_samples;
    }
}

class Renderer {
    constructor() {
        this.type = "undefined";
        this.backend = null;
    }
    init() {
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

const VALID_MESSAGE_NAMES = ["FILE_CHANGED"];
class WebSocketMessage {
    constructor(name, data) {
        this.name = name;
        this.data = data;
    }
}
class GameWebSocket {
    constructor() {
        const host = "localhost:3000";
        const socket = new WebSocket("ws://" + host);
        socket.binaryType = "arraybuffer";
        this.socket = socket;
        this.messages = [];
        this.messages_count = 0;
        this.open = false;
        socket.addEventListener("open", (event) => {
            this.open = true;
        });
        socket.addEventListener("message", (event) => {
            if (typeof event.data === "string") {
                on_text_message(this, JSON.parse(event.data));
            }
            else {
                on_bin_message(event.data);
            }
        });
        socket.addEventListener("close", (event) => {
            this.open = false;
        });
    }
}
function on_text_message(ws, message) {
    if (message.name && message.data) {
        if (!VALID_MESSAGE_NAMES.includes(message.name)) {
            console.error("Unknown message:", message);
            return;
        }
        let ws_message = new WebSocketMessage(message.name, message.data);
        ws.messages[ws.messages_count] = ws_message;
        ws.messages_count += 1;
    }
    else {
        console.error("Unknown message:", message);
    }
}
function on_bin_message(data) {
}

function file_extension(path) {
    const lastDotIndex = path.lastIndexOf('.');
    if (lastDotIndex !== -1) {
        return path.slice(lastDotIndex + 1);
    }
    return '';
}

class EngineGameInstance {
    constructor() {
        this.instance = null;
        this.module = null;
        this.reload_count = 0;
    }
}
class Engine {
    constructor() {
        this.game = new EngineGameInstance();
        this.renderer = new Renderer();
        this.web_socket = new GameWebSocket();
        this.reload_client = false;
        this.reload = false;
    }
}
//
// Init
//
// @ts-ignore
async function init_client(game) {
    const GAME_SRC_PATH = "/game/game.js";
    game.module = await import(GAME_SRC_PATH);
    await game.module.default();
    game.instance = game.module.init_app();
}
async function init() {
    const engine = new Engine();
    if (!engine.renderer.init()) {
        return;
    }
    await init_client(engine.game);
    return engine;
}
//
// Updates
//
function on_file_changed(engine, message) {
    const ext = file_extension(message.data);
    switch (ext) {
        case "wasm": {
            engine.reload_client = true;
            engine.reload = true;
            break;
        }
    }
}
function handle_websocket_messages(engine) {
    const ws = engine.web_socket;
    for (let i = 0; i < ws.messages_count; i++) {
        let message = ws.messages[i];
        switch (message.name) {
            case "FILE_CHANGED": {
                on_file_changed(engine, message);
                break;
            }
            default: {
                console.log("Unknown message:", message);
            }
        }
    }
    ws.messages_count = 0;
}
function update(engine) {
    handle_websocket_messages(engine);
    engine.renderer.update();
}
//
// Render
//
function render(engine) {
    engine.renderer.render();
}
//
// Reload
//
async function reload_client(engine) {
    const game = engine.game;
    game.reload_count += 1;
    const saved = game.module.save(engine.game.instance);
    game.module = await import(`/game/game.js?v=${game.reload_count}`);
    await game.module.default();
    game.instance = game.module.load(saved);
}
async function reload(engine) {
    if (engine.reload_client) {
        await reload_client(engine);
        engine.reload_client = false;
    }
}

export { get_last_error, init, reload, render, update };
