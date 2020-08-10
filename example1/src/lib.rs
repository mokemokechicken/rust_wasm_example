use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log_1;

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[wasm_bindgen]
pub fn output_log(s: &String) {
    log(s);
}
