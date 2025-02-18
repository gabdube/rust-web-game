#[macro_use]
mod logging;

#[macro_use]
mod error;

mod store;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DemoGame {
    test: u32
}

#[wasm_bindgen]
pub fn init_app() -> DemoGame {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let demo_app = DemoGame {
        test: 99,
    };

    dbg!("Game client initialized. Game client size: {}", size_of::<DemoGame>());

    demo_app
}


#[wasm_bindgen]
pub fn save(client: DemoGame) -> Box<[u8]> {
    let mut writer = store::SaveFileWriter::new();
    writer.write_u32(client.test);
    let bytes = writer.finalize();
    bytes.into_boxed_slice()
}

#[wasm_bindgen]
pub fn load(data: Box<[u8]>) -> DemoGame {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut demo_app = DemoGame {
        test: 0
    };

    match store::SaveFileReader::new(&data) {
        Ok(mut reader) => {
            demo_app.test = reader.read_u32();
        },
        Err(e) => {
            log_err!(e);
        }
    }

    dbg!("Game client reloaded");

    demo_app
}
