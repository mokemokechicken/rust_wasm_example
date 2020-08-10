use wasm_bindgen::prelude::*;
use web_sys::console::log_1;
use web_sys::{Window, Document};

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

