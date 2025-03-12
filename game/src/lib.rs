#[macro_use]
mod logging;

#[macro_use]
mod error;

mod store;
mod shared;
mod inputs;
mod assets;
mod world;
mod output;
mod state;
mod actions;

use shared::*;

use parking_lot::Mutex;
use wasm_bindgen::prelude::*;

static LAST_ERROR: Mutex<Option<error::Error>> = Mutex::new(None);

/// Initial data to initialize the game state 
#[wasm_bindgen]
pub struct DemoGameInit {
    pub(crate) assets_bundle: String,
    pub(crate) initial_window_size: Size<f32>,
    pub(crate) text_assets: fnv::FnvHashMap<String, String>,
}

#[wasm_bindgen]
impl DemoGameInit {
    pub fn new() -> Self {
        DemoGameInit {
            assets_bundle: String::new(),
            initial_window_size: Size::default(),
            text_assets: fnv::FnvHashMap::default(),
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn set_initial_window_size(&mut self, width: f32, height: f32) {
        self.initial_window_size = Size { width, height };
    }

    pub fn upload_text_asset(&mut self, name: String, value: String) {
        self.text_assets.insert(name, value);
    }
}

#[derive(Default)]
pub struct DemoGameTiming {
    time: f64,
    last_animation_tick: f64,
    frame_delta: f32,
}

#[derive(Default, Copy, Clone)]
pub struct DemoGameGlobalData {
    view_offset: Position<f32>,
    view_size: Size<f32>,
}

/// The game data
pub struct DemoGameData {
    global: DemoGameGlobalData,
    timing: DemoGameTiming,
    inputs: inputs::InputState,
    assets: assets::Assets,
    world: world::World,
    state: state::GameState,
}

/// The game data and the game state
#[wasm_bindgen]
pub struct DemoGame {
    data: DemoGameData,
    output: output::GameOutput,
    actions: actions::ActionsManager,
}

#[wasm_bindgen]
impl DemoGame {

    pub fn initialize(init: DemoGameInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut demo_app = DemoGame {
            data: DemoGameData {
                global: DemoGameGlobalData {
                    view_size: init.initial_window_size,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..DemoGame::default()
        };

        if let Err(e) = assets::init_assets(&mut demo_app, &init) {
            set_last_error(e);
            return None;
        }

        #[cfg(feature="editor")]
        state::editor::init(&mut demo_app, crate::state::TestId::PawnAi);

        #[cfg(not(feature="editor"))]
        state::gameplay::init(&mut demo_app);
    
        dbg!("Game client initialized. Game client size: {}", size_of::<DemoGame>());

        Some(demo_app)
    }

    pub fn on_reload(&mut self) {
        #[cfg(feature="editor")]
        state::editor::init(self, crate::state::TestId::PawnAi);

        #[cfg(not(feature="editor"))]
        state::gameplay::init(self);
    }

    pub fn update(&mut self, time: f64) -> bool {
        self.update_timing(time);
        state::update(self);
        actions::update(self);
        output::update(self);
        return true
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

    pub fn update_view_size(&mut self, width: f32, height: f32) {
        self.data.global.view_size.width = width;
        self.data.global.view_size.height = height;
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.data.inputs.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) -> bool {
        let button = match inputs::MouseButton::try_from(button) {
            Ok(btn) => btn,
            Err(error) => {
                set_last_error(error);
                return false;
            }
        };

        let state = match pressed {
            true => inputs::ButtonState::JustPressed,
            false => inputs::ButtonState::JustReleased,
        };

        self.data.inputs.update_mouse_buttons(button, state);

        return true;
    }

}

impl DemoGame {

    fn update_timing(&mut self, new_time: f64) {
        const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps
        let timing = &mut self.data.timing;
        timing.frame_delta = (new_time - timing.time) as f32;
        timing.time = new_time;

        // This only sets the update animation flag in output
        // Sprite animation are computed at sprite generation in `output.gen_sprites_with_animation` 
        let delta = new_time - timing.last_animation_tick;
        if delta > ANIMATION_INTERVAL {
            self.output.update_animations();
            timing.last_animation_tick = new_time;
        }
    }

}

impl Default for DemoGameData {
    fn default() -> Self {
        DemoGameData {
            global: DemoGameGlobalData::default(),
            inputs: inputs::InputState::default(),
            assets: assets::Assets::default(),
            world: world::World::default(),
            timing: DemoGameTiming::default(),
            state: state::GameState::Startup,
        }
    }
}

impl Default for DemoGame {
    fn default() -> Self {
        DemoGame {
            data: DemoGameData::default(),
            output: output::GameOutput::default(),
            actions: actions::ActionsManager::default(),
        }
    }
}

impl store::SaveAndLoad for DemoGame {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.write(&self.data.global);
        writer.save(&self.data.assets);
        writer.save(&self.data.world);
        writer.save(&self.data.state);
        writer.save(&self.actions);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        let mut demo_app = DemoGame::default();
        demo_app.data.global = reader.read();
        demo_app.data.assets = reader.load();
        demo_app.data.world = reader.load();
        demo_app.data.state = reader.load();
        demo_app.actions = reader.load();

        demo_app
    }
}

/// Export the game client into an array of bytes
#[wasm_bindgen]
pub fn save(client: DemoGame) -> Box<[u8]> {
    let mut writer = store::SaveFileWriter::new();
    writer.save(&client);
    writer.finalize().into_boxed_slice()
}

/// Load the game client from an array of bytes
#[wasm_bindgen]
pub fn load(data: Box<[u8]>) -> DemoGame {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut demo_app = match store::SaveFileReader::new(&data) {
        Ok(mut reader) => reader.load(),
        Err(e) => {
            log_err!(e);
            DemoGame::default()
        }
    };

    demo_app.on_reload();

    dbg!("Game client reloaded");

    demo_app
}

fn set_last_error(error: error::Error) {
    *LAST_ERROR.lock() = Some(error);
}

#[wasm_bindgen]
pub fn get_last_error() -> Option<String> {
    LAST_ERROR.lock().take()
        .map(|err| format!("{}", err) )
}
