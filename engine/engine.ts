import { EngineGameInstance } from "./game_interface";
import { EngineAssets } from "./assets";
import { Renderer } from "./renderer";
import { GameWebSocket, WebSocketMessage } from "./websocket";
import { file_extension } from "./helpers";
import { set_last_error } from "./error";

export { get_last_error } from "./error";

class Engine {
    game: EngineGameInstance = new EngineGameInstance();
    assets: EngineAssets = new EngineAssets();
    renderer: Renderer = new Renderer();
    web_socket: GameWebSocket = new GameWebSocket();

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

    for (let [name, json] of engine.assets.json.entries()) {
        init.upload_json(name, json);
    }

    engine.game.instance = game.DemoGame.initialize(init);

    if (engine.game.instance) { 
        return true
    } else {
        set_last_error(`An error occured while initializing the game client: ${engine.game.module.get_last_error()}`);
        return false;
    }
}

export async function init(): Promise<Engine | null> {
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

/// Updates the game simulation
function game_updates(engine: Engine, time: DOMHighResTimeStamp) {
    if (!engine.game.instance.update(time)) {
        handle_game_err(engine);
    }
}

/// Reads the rendering updates generated by the game client and update the renderer accordingly
function renderer_updates(engine: Engine) {
    engine.renderer.update(engine.game);
}

export function update(engine: Engine, time: DOMHighResTimeStamp) {
    engine.renderer.handle_resize();
    websocket_messages(engine);
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
