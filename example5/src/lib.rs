use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    log!("start");
    let closure_captured = Rc::new(RefCell::new(None));
    let closure_cloned = Rc::clone(&closure_captured);
    let mut my_app = MyApp::new();
    closure_cloned.replace(Some(Closure::wrap(Box::new(move |time: f64| {
        log!("in box");
        my_app.on_animation_frame(time);
        request_animation_frame(closure_captured.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>)));
    request_animation_frame(closure_cloned.borrow().as_ref().unwrap());
}

pub struct MyApp {
    pub my_counter: i32,
}

impl MyApp {
    pub fn new() -> MyApp {
        MyApp { my_counter: 0 }
    }

    pub fn on_animation_frame(&mut self, _time: f64) {
        self.my_counter += 1;
        log!("count={}", self.my_counter);
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
