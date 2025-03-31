use wasm_bindgen::prelude::*;


#[allow(unused_macros)]
macro_rules! dbg {
    ($($arg:tt)*) => {
        crate::logging::log(format!("[DEBUG][{}:{}] {}", file!(), line!(), format!($($arg)*)))
    };
}

#[allow(unused_macros)]
macro_rules! warn {
    ($($arg:tt)*) => {
        crate::logging::log(format!("[WARNING][{}:{}] {}", file!(), line!(), format!($($arg)*)))
    };
}

#[allow(unused_macros)]
macro_rules! log_err {
    ($err:expr) => {
        crate::logging::log(format!("{}", $err));
    };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: String);
}
