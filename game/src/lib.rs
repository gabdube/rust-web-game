#[macro_use]
mod logging;

#[macro_use]
mod error;

mod store;
mod shared;
mod world;
mod output;
mod state;

use shared::*;
use wasm_bindgen::prelude::*;

/// Initial data to initialize the game state 
#[wasm_bindgen]
pub struct DemoGameInit {
    pub(crate) assets_bundle: String,
    pub(crate) initial_window_size: Size<f32>,
}

#[wasm_bindgen]
impl DemoGameInit {
    pub fn new() -> Self {
        DemoGameInit {
            assets_bundle: String::new(),
            initial_window_size: Size::default(),
        }
    }

    pub fn set_assets_bundle(&mut self, text: String) {
        self.assets_bundle = text;
    }

    pub fn set_initial_window_size(&mut self, width: f32, height: f32) {
        self.initial_window_size = Size { width, height };
    }
}

/// The game state
#[wasm_bindgen]
pub struct DemoGame {
    window_size: Size<f32>,
    world: world::World,
    state: state::GameState,
    output: output::GameOutput,
    last_error: Option<error::Error>,
}

#[wasm_bindgen]
impl DemoGame {

    pub fn initialize(init: DemoGameInit) -> Self {
        ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut demo_app = DemoGame {
            window_size: init.initial_window_size,
            ..DemoGame::default()
        };
    
        demo_app.init_gameplay();
    
        dbg!("Game client initialized. Game client size: {}", size_of::<DemoGame>());
    
        demo_app
    }

    pub fn update(&mut self) -> bool {
        match self.state {
            state::GameState::MainMenu => {},
            state::GameState::Gameplay => {
                self.gameplay_update();
            },
            state::GameState::Startup => {
                self.last_error = Some(undefined_err!("Update should never be called while in startup state"));
                return false;
            }
        }

        self.update_output();

        return true;
    }

    pub fn get_last_error(&mut self) -> Option<String> {
        self.last_error.take()
            .map(|err| format!("{}", err) )
    }

    pub fn updates_ptr(&self) -> *const output::OutputIndex {
        self.output.output_index
    }

}

impl Default for DemoGame {
    fn default() -> Self {
        DemoGame {
            window_size: Size::default(),
            output: output::GameOutput::default(),
            world: world::World::default(),
            state: state::GameState::Startup,
            last_error: None,
        }
    }
}

impl store::SaveAndLoad for DemoGame {
    fn save(&self, writer: &mut store::SaveFileWriter) {
        writer.save(&self.window_size);
        writer.save(&self.state);
        writer.save(&self.world);
    }

    fn load(reader: &mut store::SaveFileReader) -> Self {
        let mut demo_app = DemoGame::default();
        demo_app.window_size = reader.load();
        demo_app.state = reader.load();
        demo_app.world = reader.load();

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

    let demo_app = match store::SaveFileReader::new(&data) {
        Ok(mut reader) => reader.load(),
        Err(e) => {
            log_err!(e);
            DemoGame::default()
        }
    };

    dbg!("Game client reloaded");

    demo_app
}
