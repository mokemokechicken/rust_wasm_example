use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

static mut MY_APP: Option<Box<MyApp>> = None;

pub fn init_my_app() {
    unsafe {
        MY_APP = Some(Box::new(MyApp::new()));
    }
}

pub fn my_app() -> &'static MyApp {
    unsafe { MY_APP.as_ref().unwrap() }
}

pub fn my_app_mut() -> &'static mut MyApp {
    unsafe { MY_APP.as_mut().unwrap() }
}

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    log!("start");
    init_my_app();
    let closure_captured = Rc::new(RefCell::new(None));
    let closure_cloned = Rc::clone(&closure_captured);

    let mut x = my_app_mut();
    x.clicks += 1;
    let y = my_app();
    log!("clicks={}", y.clicks);

    // setup requestAnimationFrame Loop
    {
        closure_cloned.replace(Some(Closure::wrap(Box::new(move |time: f64| {
            my_app_mut().on_animation_frame(time);
            request_animation_frame(closure_captured.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut(f64)>)));
        request_animation_frame(closure_cloned.borrow().as_ref().unwrap());
    }

    // setup onClick
    {
        let c = Closure::wrap(Box::new(move |e| {
            my_app_mut().on_click(e);
        }) as Box<dyn FnMut(JsValue)>);
        document()
            .add_event_listener_with_callback("click", c.as_ref().unchecked_ref())
            .unwrap();
        c.forget(); // c を Rustのメモリ管理から外して JSのGCにわたす
    }
}

pub struct MyApp {
    pub my_name: String,
    pub my_counter: i32,
    pub clicks: u32,
    pub async_count: u32,
}

impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            my_name: "App!".into(),
            my_counter: 0,
            clicks: 0,
            async_count: 0,
        }
    }

    pub fn on_animation_frame(&mut self, time: f64) {
        self.my_counter += 1;
        if self.my_counter % 60 == 0 {
            log!(
                "name={}, time={}, count={}, clicks={}, async_count={}",
                &self.my_name,
                time,
                self.my_counter,
                self.clicks,
                self.async_count,
            );
        }
    }

    pub fn on_click(&mut self, _event: JsValue) {
        self.clicks += 1;
        // 仮にこれをselfが持っていれば渡せることになる
        let x: Rc<RefCell<MyApp>> = Rc::new(RefCell::new(MyApp::new()));
        spawn_local(my_async_process(self.clicks));
        spawn_local(my_async_process_with_ref_cell(x.clone(), self.clicks));
        spawn_local(my_async_process_with_raw_pointer(
            self as *mut MyApp,
            self.clicks,
        ));
    }
}

pub async fn my_async_process(param: u32) {
    // 何か await とかしたりできる(fetchとか)
    // spawn_local() で使えるのは static な 参照しかないが、 my_app_mut() はそれを満たしている。
    my_app_mut().async_count += param;
}

pub async fn my_async_process_with_ref_cell(my_app: Rc<RefCell<MyApp>>, param: u32) {
    // 何か await とかしたりできる(fetchとか)
    // spawn_local() で使えるのは static な 参照しかないが、 Rcに包まれたものは有効。
    my_app.borrow_mut().async_count += param;
}

pub async fn my_async_process_with_raw_pointer(my_app: *mut MyApp, param: u32) {
    // 何か await とかしたりできる(fetchとか)
    // spawn_local() で使えるのは static な 参照しかないが、 Raw Pointerなら渡せる!!
    unsafe {
        (*my_app).async_count += param;
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
