// @ts-ignore
import { DemoGame } from "../build/app/app";
import { Renderer } from "./renderer";
import { GameWebSocket, WebSocketMessage } from "./websocket";
import { file_extension } from "./helpers";

export { get_last_error } from "./error";

class EngineGameInstance {
    instance: DemoGame | null = null;
    module: any = null;
    reload_count: number = 0;
}

class Engine {
    game: EngineGameInstance = new EngineGameInstance();
    renderer: Renderer = new Renderer();
    web_socket: GameWebSocket = new GameWebSocket();

    reload_client: boolean = false;
    reload: boolean = false;
}

//
// Init
//

// @ts-ignore
async function init_client(game: EngineGameInstance) {
    const GAME_SRC_PATH = "/game/game.js";
    game.module = await import(GAME_SRC_PATH);
    await game.module.default();
    game.instance = game.module.init_app();
}

export async function init(): Promise<Engine | undefined> {
    const engine = new Engine();

    if ( !engine.renderer.init() ) {
        return;
    }

    await init_client(engine.game);

    return engine;
}

//
// Updates
//

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

function handle_websocket_messages(engine: Engine) {
    const ws = engine.web_socket;

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


export function update(engine: Engine) {
    handle_websocket_messages(engine);
    engine.renderer.update();
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
