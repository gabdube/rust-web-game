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


/// The game state
#[wasm_bindgen]
pub struct DemoGame {
    time: f64,
    view_offset: Position<f32>,
    view_size: Size<f32>,
    assets: assets::Assets,
    inputs: inputs::InputState,
    world: world::World,
    output: output::GameOutput,
    state: state::GameState,
}

#[wasm_bindgen]
impl DemoGame {

    pub fn initialize(init: DemoGameInit) -> Option<Self> {
        ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut demo_app = DemoGame {
            view_size: init.initial_window_size,
            ..DemoGame::default()
        };

        demo_app.load_asset_bundle(&init)?;

        #[cfg(feature="editor")]
        {
            demo_app.init_editor(crate::state::TestId::PawnAi);
        }

        #[cfg(not(feature="editor"))]
        {
            demo_app.init_gameplay();
        }
    
        dbg!("Game client initialized. Game client size: {}", size_of::<DemoGame>());

        Some(demo_app)
    }

    pub fn on_reload(&mut self) {
        #[cfg(feature="editor")]
        {
            self.init_editor(crate::state::TestId::PawnAi);
        }

        #[cfg(not(feature="editor"))]
        {
            self.init_gameplay();
        }
    }

    pub fn update(&mut self, time: f64) -> bool {
        self.update_time(time);

        match &mut self.state {
            state::GameState::MainMenu => {},
            state::GameState::Gameplay(_) => {
                self.gameplay_update();
            },

            #[cfg(feature="editor")]
            state::GameState::Editor(_) => {
                self.editor_update();
            },

            state::GameState::Startup => {
                set_last_error(undefined_err!("Update should never be called while in startup state"));
                return false;
            },
        }

        self.update_animations();
        self.update_output();
        self.inputs.after_update();

        return true
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

    pub fn update_view_size(&mut self, width: f32, height: f32) {
        self.view_size.width = width;
        self.view_size.height = height;
    }

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.inputs.update_mouse_position(x, y);
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

        self.inputs.update_mouse_buttons(button, state);

        return true;
    }

}

impl DemoGame {

    fn update_time(&mut self, new_time: f64) {
        self.time = new_time;
    }

    fn update_animations(&mut self) {
        const ANIMATION_INTERVAL: f64 = 1000.0 / 16.0; // 16fps

        let world = &mut self.world;
        let delta = self.time - world.last_animation_tick;
        if delta < ANIMATION_INTERVAL {
            return;
        }

        self.output.update_animations();
        world.last_animation_tick = self.time;
    }

    fn init_world_assets(&mut self) -> Option<()> {
        if let Err(e) = self.world.init_assets(&self.assets) {
            set_last_error(e);
            None
        } else {
            Some(())
        }
    }

    fn load_asset_bundle(&mut self, init: &DemoGameInit) -> Option<()> {
        if let Err(e) = self.assets.load_bundle(&init) {
            set_last_error(e);
            return None;
        }

        self.init_world_assets()
    }

}

impl Default for DemoGame {
    fn default() -> Self {
        DemoGame {
            time: 0.0,
            view_offset: Position::default(),
            view_size: Size::default(),
            assets: assets::Assets::default(),
            inputs: inputs::InputState::default(),
            output: output::GameOutput::default(),
            world: world::World::default(),
            state: state::GameState::Startup,
        }
    }
}

impl store::SaveAndLoad for DemoGame {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.save(&self.view_offset);
        writer.save(&self.view_size);
        writer.save(&self.state);
        writer.save(&self.world);
        writer.save(&self.assets);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        let mut demo_app = DemoGame::default();
        demo_app.view_offset = reader.load();
        demo_app.view_size = reader.load();
        demo_app.state = reader.load();
        demo_app.world = reader.load();
        demo_app.assets = reader.load();

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
