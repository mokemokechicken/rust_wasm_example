use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
//
use core::cell::RefCell;
use futures::prelude::*;
use js_sys::Promise;
use std::rc::Rc;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

