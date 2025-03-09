import { EngineGameInstance } from "./game_interface";
import { EngineAssets } from "./assets";
import { Renderer } from "./renderer";
import { GameWebSocket, WebSocketMessage } from "./websocket";
import { file_extension } from "./helpers";
import { set_last_error } from "./error";

import { Error, get_last_error } from "./error";

const UPDATE_MOUSE_POSITION = 0b1;
const UPDATE_MOUSE_BUTTONS = 0b10;

// Matches `MouseButton`
const MOUSE_BUTTON_LEFT = 0;
const MOUSE_BUTTON_RIGHT = 1;

class InputState {
    updates: number = 0;
    mouse_position: number[] = [0.0, 0.0];

    // true: button was pressed, false: button was released, null: button state wasn't changed
    left_mouse_button: boolean|null = null;    
    right_mouse_button: boolean|null = null;
}

class Engine {
    web_socket: GameWebSocket = new GameWebSocket();

    game: EngineGameInstance = new EngineGameInstance();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();
    input: InputState = new InputState();

    errors_count: number = 0;

    reload_client: boolean = false;
    reload: boolean = false;
    exit: boolean = false;
}

//
// Init
//


// Import the wasm game client modulle
// Note: Game client instance is created in `start_client`
// @ts-ignore
async function init_client(game: EngineGameInstance): Promise<boolean> {
    const GAME_SRC_PATH = "/game/game.js";
    game.module = await import(GAME_SRC_PATH)
        .catch((e) => { set_last_error(`Failed to load the game client`); return null; });

    if (!game.module) {
        return false;
    }

    await game.module.default();

    return true;
}

/// Fetch all the game assets. Starting from "bundle" which act as an index for all the different assets types in the projects
async function init_assets(engine: Engine): Promise<boolean> {
    return engine.assets.load();
}

/// Load the initial resources (like shaders) in the renderer
function init_renderer_default_resources(engine: Engine): boolean {
    return engine.renderer.init_default_resources(engine.assets);
}

/// Instance the game client
function start_game_client(engine: Engine): boolean {
    const game = engine.game.module;

    const init = game.DemoGameInit.new();
    init.set_assets_bundle(engine.assets.raw_bundle);

    const size = engine.renderer.canvas_size();
    init.set_initial_window_size(size.width, size.height);

    for (let [name, json] of engine.assets.csv.entries()) {
        init.upload_text_asset(name, json);
    }

    engine.game.instance = game.DemoGame.initialize(init);

    if (engine.game.instance) { 
        return true
    } else {
        set_last_error(engine.game.module.get_last_error());
        return false;
    }
}

function init_handlers(engine: Engine) {
    const canvas = engine.renderer.canvas();
    const input_state = engine.input;

    canvas.addEventListener("mousemove", (event) => { 
        input_state.mouse_position[0] = event.clientX;
        input_state.mouse_position[1] = event.clientY;
        input_state.updates |= UPDATE_MOUSE_POSITION;
    })

    canvas.addEventListener("mousedown", (event) => {
        input_state.mouse_position[0] = event.clientX;
        input_state.mouse_position[1] = event.clientY;
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = true; }
        else if (event.button === 2) { input_state.right_mouse_button = true; }
    })

    canvas.addEventListener("mouseup", (event) => {
        input_state.mouse_position[0] = event.clientX;
        input_state.mouse_position[1] = event.clientY;
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = false; }
        else if (event.button === 2) { input_state.right_mouse_button = false; }
    })

    canvas.addEventListener("contextmenu", (event) => { event.preventDefault(); });

    // window.addEventListener("keydown", (event) => { input_state.keys.set(event.code, true); });
    // window.addEventListener("keyup", (event) => { input_state.keys.set(event.code, false); });
}

async function init(): Promise<Engine | null> {
    const engine = new Engine();

    if ( !engine.renderer.init() ) {
        return null;
    }

    let load_client = init_client(engine.game);
    let load_assets = init_assets(engine);
    let [client_ok, assets_ok] = await Promise.all([load_client, load_assets]);
    if (!client_ok || !assets_ok) {
        return null;
    }

    if (!init_renderer_default_resources(engine)) {
        return null;
    }

    if (!start_game_client(engine)) {
        return null;
    }

    init_handlers(engine);

    return engine;
}

//
// Updates
//

function handle_game_err(engine: Engine) {
    const error = engine.game.module.get_last_error();
    
    console.log(error);

    engine.errors_count += 1;

    if (engine.errors_count > 5) {
        console.log("More than 5 errors logged, exiting...");
        engine.exit = true;
    }
}

function on_file_changed(engine: Engine, message: WebSocketMessage) {
    const ext = file_extension(message.data);
    switch (ext) {
        case "wasm": {
            engine.reload_client = true;
            engine.reload = true;
            break;
        }
        case "vert":
        case "frag": {
            // TODO: reload shaders
            break;
        }
    }
}

/// Handle the updates received from the development server
function websocket_messages(engine: Engine) {
    const ws = engine.web_socket;
    if (!ws.open) {
        // We're using a static client with no dev server
        return;
    }

    for (let i=0; i<ws.messages_count; i++) {
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

function handle_resize(engine: Engine) {
    if (engine.renderer.handle_resize()) {
        const game = engine.game.instance;
        const canvas_size = engine.renderer.canvas_size();
        game.update_view_size(canvas_size.width, canvas_size.height);
    }
}

function game_input_updates(engine: Engine): boolean {
    const inputs = engine.input;
    const game = engine.game.instance;
    let ok = true;

    if ((inputs.updates & UPDATE_MOUSE_POSITION) > 0) {
        game.update_mouse_position(inputs.mouse_position[0], inputs.mouse_position[1]);
    }

    if ((inputs.updates & UPDATE_MOUSE_BUTTONS) > 0) {
        if (inputs.left_mouse_button !== null) {
            ok = ok && game.update_mouse_buttons(MOUSE_BUTTON_LEFT, inputs.left_mouse_button);
        }

        if (inputs.right_mouse_button !== null) {
            ok = ok && game.update_mouse_buttons(MOUSE_BUTTON_RIGHT, inputs.right_mouse_button);
        }

        inputs.left_mouse_button = null;
        inputs.right_mouse_button = null;
    }

    inputs.updates = 0;

    return ok;
}

function game_updates(engine: Engine, time: DOMHighResTimeStamp) {
    if (!game_input_updates(engine)) {
        handle_game_err(engine);
        return;
    }
    
    if (!engine.game.instance.update(time)) {
        handle_game_err(engine);
    }
}

/// Reads the rendering updates generated by the game client
function renderer_updates(engine: Engine) {
    engine.renderer.update(engine.game);
}


export function update(engine: Engine, time: DOMHighResTimeStamp) {
    websocket_messages(engine);
    handle_resize(engine);
    game_updates(engine, time);
    renderer_updates(engine);
}

//
// Render
//

export function render(engine: Engine) {
    engine.renderer.render();
}

//
// Reload
//

async function reload_client(engine: Engine) {
    const game = engine.game;
    game.reload_count += 1;
    game.validate_memory_layout = true;

    const saved = game.module.save(engine.game.instance);

    game.module = await import(`/game/game.js?v=${game.reload_count}`);
    await game.module.default();

    game.instance = game.module.load(saved);
}

export async function reload(engine: Engine) {
    if (engine.reload_client) {
        await reload_client(engine);
        engine.reload_client = false;
    }
}

// Runtime

let boundedRun = () => {};

function show(element: HTMLElement) {
    element.classList.remove('hidden');
}

function show_critical_error(error: Error) {
    const panel = document.getElementById("errorPanel") as HTMLElement;
    const error_message = panel.children[2] as HTMLElement;
    const error_traceback = panel.lastElementChild as HTMLElement;

    error_message.textContent = error.message;

    if (error.traceback) {
        error_traceback.textContent = error.traceback.toString();
        show(document.getElementById("errorDetails") as HTMLElement);
    }

    show(panel);
}

let last_p = performance.now();

function run(engine: Engine) {
    update(engine, performance.now());
    render(engine);

    if (engine.exit) {
        return;
    }

    if (engine.reload) {
        reload(engine)
            .then(() => requestAnimationFrame(boundedRun) );
    } else {
        requestAnimationFrame(boundedRun);
    }
}

async function init_app() {
    const engine = await init();
    if (!engine) {
        const error = get_last_error();
        if (error) {
            show_critical_error(error);
        }
        return;
    }

    boundedRun = run.bind(null, engine);
    boundedRun();
}

init_app();

