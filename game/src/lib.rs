#[macro_use]
mod logging;

#[macro_use]
mod error;

mod store;
mod shared;
mod inputs;
mod assets;
mod gui;
mod world;
mod state;
mod data;
mod output;
mod behaviour;

use data::DemoGameData;
use shared::*;

use parking_lot::Mutex;
use wasm_bindgen::prelude::*;

static LAST_ERROR: Mutex<Option<error::Error>> = Mutex::new(None);

const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps

/// Initial data to initialize the game state 
#[wasm_bindgen]
pub struct DemoGameInit {
    pub(crate) text_assets: fnv::FnvHashMap<String, String>,
    pub(crate) font_assets: fnv::FnvHashMap<String, Box<[u8]>>,
    pub(crate) assets_bundle: String,
    pub(crate) seed: u64,
    pub(crate) initial_window_size: Size<f32>,
}

#[wasm_bindgen]
impl DemoGameInit {
    pub fn new() -> Self {
        DemoGameInit {
            text_assets: fnv::FnvHashMap::default(),
            font_assets: fnv::FnvHashMap::default(),
            assets_bundle: String::new(),
            seed: 0,
            initial_window_size: Size::default(),
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

    pub fn upload_font_asset(&mut self, name: String, data: Vec<u8>) {
        self.font_assets.insert(name, data.into_boxed_slice());
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }
}

/// The game data and the game state
#[wasm_bindgen]
pub struct DemoGame {
    data: DemoGameData,
    output: output::GameOutput,
}

#[wasm_bindgen]
impl DemoGame {

    pub fn initialize(init: DemoGameInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut demo_app = DemoGame::default();
        demo_app.data.global.seed = init.seed;
        demo_app.data.inputs.view_size = init.initial_window_size;
        demo_app.data.inputs.last_view_size = init.initial_window_size;

        fastrand::seed(init.seed);

        if let Err(e) = assets::init_assets(&mut demo_app, &init) {
            set_last_error(e);
            return None;
        }

        #[cfg(feature="editor")]
        if let Err(e) = state::editor::init(&mut demo_app.data, crate::state::TestId::PawnAi) {
            set_last_error(e);
            return None;
        }

        #[cfg(not(feature="editor"))]
        state::gameplay::init(&mut demo_app.data);
    
        dbg!("Game client initialized. Game client size: {}", size_of::<DemoGame>());

        Some(demo_app)
    }

    pub fn on_reload(&mut self) -> bool {
        #[cfg(feature="editor")]
        if let Err(e) = state::editor::init(&mut self.data, crate::state::TestId::PawnAi) {
            set_last_error(e);
            return false;
        }

        // #[cfg(not(feature="editor"))]
        // state::gameplay::init(&mut self.data);

        return true;
    }

    pub fn update(&mut self, time: f64) -> bool {
        self.update_timing(time);
        state::update(self);
        behaviour::update(self);
        output::update(self);
        return true
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

    pub fn update_view_size(&mut self, width: f32, height: f32) {
        self.data.inputs.view_size.width = width;
        self.data.inputs.view_size.height = height;
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.data.inputs.update_mouse_position(x, y);
    }

    pub fn update_mouse_buttons(&mut self, button: u8, pressed: bool) {
        let button = match inputs::MouseButton::try_from(button) {
            Ok(btn) => btn,
            Err(_) => { return; }
        };

        let state = match pressed {
            true => inputs::ButtonState::JustPressed,
            false => inputs::ButtonState::JustReleased,
        };

        self.data.inputs.update_mouse_buttons(button, state);
    }

    pub fn update_keys(&mut self, key_name: &str, pressed: bool) {
        let key = match inputs::Key::from_name(key_name) {
            Some(key) => key,
            None => { return; }
        };

        let state = match pressed {
            true => inputs::ButtonState::JustPressed,
            false => inputs::ButtonState::JustReleased,
        };

        self.data.inputs.update_keys(key, state);
    }

}

impl DemoGame { 

    fn update_timing(&mut self, new_time: f64) {
        let global = &mut self.data.global;
        global.frame_delta = (new_time - global.time) as f32;
        global.time = new_time;

        // Can happen if the application was paused.
        // In this case we set the delta to 0 for this frame so the game logic doesn't break.
        if global.frame_delta > 1000.0 {
            global.frame_delta = 0.0;
        }

        // This only sets a flag
        // Sprite animation are computed at sprite generation in `output.gen_sprites_with_animation` 
        let delta = new_time - global.last_animation_tick;
        if delta > ANIMATION_INTERVAL {
            global.flags.set_update_animations();
            global.last_animation_tick = new_time;
        }
    }

}

impl Default for DemoGame {
    fn default() -> Self {
        DemoGame {
            data: DemoGameData::default(),
            output: output::GameOutput::default(),
        }
    }
}

impl store::SaveAndLoad for DemoGame {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.save(&self.data);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        let mut demo_app = DemoGame::default();
        demo_app.data = reader.load();
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

    fastrand::seed(demo_app.data.global.seed);

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
