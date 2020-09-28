mod geometry;

use crate::geometry::vec2d::Vec2d;
use crate::geometry::voronoi_diagram::types::{ClusterId, NodeId, VoronoiCenterPoint};
use crate::geometry::voronoi_diagram::voronoi_diagram::VoronoiDiagram;
use core::f64::consts::PI;
use js_sys::Math::random;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
const SIZE: f64 = 698.;

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
    init_my_app();

    my_app_mut().setup();

    // setup onClick canvas
    {
        let c = Closure::wrap(Box::new(move |e| {
            my_app_mut().on_click(e);
        }) as Box<dyn FnMut(JsValue)>);
        let elem: web_sys::HtmlCanvasElement = get_element_by_id("canvas");
        elem.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())
            .unwrap();
        c.forget(); // c を Rustのメモリ管理から外して JSのGCにわたす
    }

    // setup onClick add
    {
        let c = Closure::wrap(Box::new(move |e| {
            my_app_mut().on_add_points(e);
        }) as Box<dyn FnMut(JsValue)>);
        let elem: web_sys::HtmlElement = get_element_by_id("add");
        elem.add_event_listener_with_callback("click", c.as_ref().unchecked_ref())
            .unwrap();
        c.forget(); // c を Rustのメモリ管理から外して JSのGCにわたす
    }
}

pub struct MyApp {
    diagram: VoronoiDiagram,
}

impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            diagram: VoronoiDiagram::new(),
        }
    }

    pub fn setup(&mut self) {
        self.draw();
    }

    pub fn on_click(&mut self, _event: JsValue) {
        let cx = get_nested_property(&_event, vec!["offsetX"])
            .unwrap()
            .as_f64()
            .unwrap();
        let cy = get_nested_property(&_event, vec!["offsetY"])
            .unwrap()
            .as_f64()
            .unwrap();
        let x = (cx - 1.) / SIZE;
        let y = (cy - 1.) / SIZE;
        let id = self.diagram.cells.len();
        let vp = VoronoiCenterPoint {
            pos: Vec2d { x, y },
            node_id: id as NodeId,
            cluster_id: id as ClusterId,
        };
        self.diagram.add_point(&vp);
        self.draw();
    }

    pub fn on_add_points(&mut self, _e: JsValue) {
        for _idx in 0..100 {
            let x = random() * 0.8 + 0.1;
            let y = random() * 0.8 + 0.1;
            let id = self.diagram.cells.len();
            let vp = VoronoiCenterPoint {
                pos: Vec2d { x, y },
                node_id: id as NodeId,
                cluster_id: id as ClusterId,
            };
            self.diagram.add_point(&vp);
            self.draw();
        }
    }

    pub fn draw(&mut self) {
        let context = get_context2d_by_id(&String::from("canvas"));
        context.set_fill_style(&JsValue::from(format!("rgb(0, 0, 0, 1)")));
        context.fill_rect(0., 0., SIZE + 2., SIZE + 2.);
        context.fill();

        context.set_fill_style(&JsValue::from(format!("rgb(255, 0, 0, 1)")));
        context.set_stroke_style(&JsValue::from(format!("rgb(0, 255, 0, 1)")));

        context.begin_path();
        for cell in self.diagram.cells.iter() {
            for line in cell.borrow().lines.iter() {
                context.move_to(line.line.p1.x * SIZE + 1., line.line.p1.y * SIZE + 1.);
                context.line_to(line.line.p2.x * SIZE + 1., line.line.p2.y * SIZE + 1.)
            }
        }
        context.stroke();

        for _cell in self.diagram.cells.iter() {
            let cell = _cell.borrow();
            context.begin_path();
            context
                .arc(
                    cell.point.pos.x * SIZE + 1.,
                    cell.point.pos.y * SIZE + 1.,
                    2.,
                    0.,
                    PI * 2.,
                )
                .unwrap();
            context.fill();
        }
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

pub fn get_context2d_by_id(canvas_id: &String) -> web_sys::CanvasRenderingContext2d {
    let canvas: web_sys::HtmlCanvasElement = get_element_by_id(canvas_id.as_str());
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

pub fn get_element_by_id<T: JsCast>(id: impl AsRef<str>) -> T {
    document()
        .get_element_by_id(id.as_ref())
        .expect(&format!("not found: {}", id.as_ref()))
        .dyn_into::<T>()
        .map_err(|_| ())
        .unwrap()
}

pub fn get_nested_property(e: &JsValue, names: Vec<&str>) -> Option<JsValue> {
    let mut ret: Option<JsValue> = None;
    for name in names {
        let target = match ret {
            Some(ref v) => v,
            None => e,
        };
        match js_sys::Reflect::get(target, &JsValue::from(name)) {
            Ok(v) => {
                ret = Some(v);
            }
            Err(_) => return None,
        };
    }
    ret
}
