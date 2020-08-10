use wasm_bindgen::prelude::*;
use web_sys::console::log_1;
use wasm_bindgen::JsCast;
use serde::{Serialize, Deserialize};


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

fn get_element_by_id<T: JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .expect("not found")
        .dyn_into::<T>()
        .map_err(|_| ())
        .unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct MyAppOptions {
    pub name: String,
    pub age: Option<u32>,
    pub email: Option<String>,
}

#[wasm_bindgen]
pub struct MyApp {
    name: String,
    age: u32,
    email: Option<String>,
}

#[wasm_bindgen]
impl MyApp {
    pub fn new(options: JsValue) -> MyApp {
        let opts: MyAppOptions = options.into_serde().unwrap();
        MyApp {
            name: opts.name,
            age: opts.age.unwrap_or(18),  // default値で埋めておく場合
            email: opts.email,                    // Option のままの場合
        }
    }

    pub fn show_status(&self, div_id: &str) {
        let div: web_sys::HtmlDivElement = get_element_by_id(div_id);
        div.set_inner_text(&format!("name={}, age={}, email={}",
                                    self.name,
                                    self.age,
                                    self.email.as_ref().unwrap_or(&"".into()) // 野暮ったい...
        ));
    }
}

