use wasm_bindgen::JsCast;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

pub fn get_element_by_id<T: JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .expect("not found")
        .dyn_into::<T>()
        .map_err(|_| ())
        .unwrap()
}

pub fn canvas(canvas_id: &str) -> web_sys::HtmlCanvasElement {
    get_element_by_id(&canvas_id)
}

pub fn context2d(canvas_id: &str) -> web_sys::CanvasRenderingContext2d {
    canvas(canvas_id)
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}
