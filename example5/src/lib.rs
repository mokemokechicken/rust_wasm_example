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
    let my_app_rc = Rc::new(RefCell::new(MyApp::new()));

    // setup requestAnimationFrame Loop
    {
        let app_for_closure = Rc::clone(&my_app_rc);
        closure_cloned.replace(Some(Closure::wrap(Box::new(move |time: f64| {
            app_for_closure.borrow_mut().on_animation_frame(time);
            request_animation_frame(closure_captured.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut(f64)>)));
        request_animation_frame(closure_cloned.borrow().as_ref().unwrap());
    }

    // setup onClick
    {
        let app_for_closure = Rc::clone(&my_app_rc);
        let c = Closure::wrap(Box::new(move |e| {
            app_for_closure.borrow_mut().on_click(e);
        }) as Box<dyn FnMut(JsValue)>);
        // Closure to js_sys::Function: .as_ref().unchecked_ref()
        document().set_onclick(Some(c.as_ref().unchecked_ref()));
        c.forget(); // c を Rustのメモリ管理から外して JSのGCにわたす
    }
}

pub struct MyApp {
    pub my_name: String,
    pub my_counter: i32,
    pub clicks: u32,
}

impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            my_name: "App!".into(),
            my_counter: 0,
            clicks: 0,
        }
    }

    pub fn on_animation_frame(&mut self, time: f64) {
        self.my_counter += 1;
        if self.my_counter % 60 == 0 {
            log!(
                "name={}, time={}, count={}, clicks={}",
                &self.my_name,
                time,
                self.my_counter,
                self.clicks,
            );
        }
    }

    pub fn on_click(&mut self, event: JsValue) {
        self.clicks += 1;
        log!("{:#?}", &event);
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
