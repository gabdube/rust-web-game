import { EngineGameInstance } from "./game_interface";
import { EngineAssets } from "./assets";
import { Renderer } from "./renderer";
import { GameWebSocket, WebSocketMessage } from "./websocket";
import { file_extension } from "./helpers";
import { set_last_error } from "./error";

import { Error, get_last_error } from "./error";

const UPDATE_MOUSE_POSITION = 0b001;
const UPDATE_MOUSE_BUTTONS  = 0b010;
const UPDATE_KEYS           = 0b100;

// Matches `MouseButton` in `game\src\inputs.rs`
const MOUSE_BUTTON_LEFT = 0;
const MOUSE_BUTTON_RIGHT = 1;
const MOUSE_BUTTON_CENTER = 2;

class InputState {
    updates: number = 0;
    mouse_position: number[] = [0.0, 0.0];

    // true: button was pressed, false: button was released, null: button state wasn't changed
    left_mouse_button: boolean|null = null;    
    right_mouse_button: boolean|null = null;
    center_mouse_button: boolean|null = null;

    keys: Map<string, boolean> = new Map();
}

class Engine {
    web_socket: GameWebSocket = new GameWebSocket();

    game: EngineGameInstance = new EngineGameInstance();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();
    input: InputState = new InputState();

    errors_count: number = 0;

    reload_client: boolean = false;
    reload_assets: string[] = [];
    reload: boolean = false;
    exit: boolean = false;
}

//
// Init
//


// Import the wasm game client module
// Note: Game client instance is created in `start_game_client`
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

/// Fetch all the game assets. Starting from "bundle", the index for all the different assets in the project
async function init_assets(engine: Engine): Promise<boolean> {
    return engine.assets.load();
}

/// Load the initial resources in the renderer
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

    const seed = new BigUint64Array(1);
    crypto.getRandomValues(seed);
    init.set_seed(seed[0]);

    for (let [name, json] of engine.assets.csv.entries()) {
        init.upload_text_asset(name, json);
    }

    for (let [name, font] of engine.assets.fonts.entries()) {
        init.upload_font_asset(name, new Uint8Array(font.atlas_data));
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
        else if (event.button === 1) { input_state.center_mouse_button = true; }
        else if (event.button === 2) { input_state.right_mouse_button = true; }
        
        event.preventDefault();
    })

    canvas.addEventListener("mouseup", (event) => {
        input_state.mouse_position[0] = event.clientX;
        input_state.mouse_position[1] = event.clientY;
        input_state.updates |= UPDATE_MOUSE_BUTTONS;

        if (event.button === 0) { input_state.left_mouse_button = false; }
        else if (event.button === 1) { input_state.center_mouse_button = false; }
        else if (event.button === 2) { input_state.right_mouse_button = false; }
        
        event.preventDefault();
    })

    canvas.addEventListener("contextmenu", (event) => { event.preventDefault(); });

    window.addEventListener("keydown", (event) => {
        input_state.keys.set(event.code, true);
        input_state.updates |= UPDATE_KEYS;
    });
    window.addEventListener("keyup", (event) => {
        // console.log(event.code);
        input_state.keys.set(event.code, false);
        input_state.updates |= UPDATE_KEYS;
    });
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
    // Reloading is async so we don't execute it right away in the game loop.
    // See the `reload` function in this file
    const ext = file_extension(message.data);
    switch (ext) {
        case "wasm": {
            engine.reload_client = true;
            engine.reload = true;
            break;
        }
        case "png": {
            engine.reload_assets.push(message.data);
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

function game_input_updates(engine: Engine) {
    const inputs = engine.input;
    const game = engine.game.instance;

    if ((inputs.updates & UPDATE_MOUSE_POSITION) > 0) {
        game.update_mouse_position(inputs.mouse_position[0], inputs.mouse_position[1]);
    }

    if ((inputs.updates & UPDATE_MOUSE_BUTTONS) > 0) {
        if (inputs.left_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_LEFT, inputs.left_mouse_button);
        }

        if (inputs.right_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_RIGHT, inputs.right_mouse_button);
        }

        if (inputs.center_mouse_button !== null) {
            game.update_mouse_buttons(MOUSE_BUTTON_CENTER, inputs.center_mouse_button);
        }

        inputs.left_mouse_button = null;
        inputs.right_mouse_button = null;
        inputs.center_mouse_button = null;
    }

    if ((inputs.updates & UPDATE_KEYS) > 0) {
        for (let entry of inputs.keys.entries()) {
            game.update_keys(entry[0], entry[1]);
        }
    }

    inputs.keys.clear();
    inputs.updates = 0;
}

function game_updates(engine: Engine, time: DOMHighResTimeStamp) {
    game_input_updates(engine)
    
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

async function reload_assets(engine: Engine): Promise<boolean> {
    const reload_result = await engine.assets.reload_assets(engine.reload_assets);
    if (!reload_result) {
        return false;
    }

    return engine.renderer.reload_assets(engine.reload_assets);
}

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
    let ok = true;

    if (engine.reload_assets.length > 0) {
        ok &&= await reload_assets(engine);
        engine.reload_assets = [];
    }

    if (engine.reload_client) {
        await reload_client(engine);
        engine.reload_client = false;
    }

    if (!ok) {
        let error = get_last_error();
        if (error) {
            show_critical_error(error);
            engine.exit = true;
        }
    }

    engine.reload = false;
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

function run(engine: Engine) {
    if (engine.exit) {
        return;
    }

    update(engine, performance.now());
    render(engine);

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

