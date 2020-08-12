use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log_1;
//
use core::cell::RefCell;
use futures::prelude::*;
use js_sys::Promise;
use std::rc::Rc;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{Request, RequestInit, RequestMode, Response};

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[derive(Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub members: Vec<Member>,
}

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub name: String,
    pub age: u32,
    pub role: String,
}

#[wasm_bindgen]
pub struct MyApp {
    team: Rc<RefCell<Option<Team>>>,
}

#[wasm_bindgen]
impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            // Rc<RefCell<任意のデータ>> というのはRustでよくある使い方
            // https://doc.rust-jp.rs/book/second-edition/ch15-00-smart-pointers.html
            // ここのスマートポインタの部分をよく読むと良い。
            team: Rc::new(RefCell::new(None)),
        }
    }

    // この関数自体を async にすると &mut selfの辺りが上手くいかない(確か)。
    // selfをClosureにcaptureさせずに 更新したい部分を Rc<RefCell> で capture させるのが良さそう。
    // この関数自体はPromiseをreturnさせる。
    pub fn load_data(&mut self, path: String) -> Promise {
        // Closureの中で更新するために Rc のスマートポインタで株分けしておく
        // のちに *ptr_for_update.borrow_mut() = ... で更新できるようになる。
        // borrow_mut() は unsafe らしいので  実行時にエラーになることはありえる。
        // まあ、しかし、例えば初回に１回だけ実行するような場合ではまったく問題にならないだろう。
        let ptr_for_update = self.team.clone();

        // HTTP アクセスのための準備
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);
        let request = Request::new_with_str_and_init(&path, &opts).unwrap();
        request
            .headers()
            .set("Accept", "application/octet-stream")
            .unwrap();
        let window = web_sys::window().unwrap();
        let request_promise: Promise = window.fetch_with_request(&request);
        // ↑ ここまではお約束

        // Promise -> Future を繰り返す感じになる。 web_sysの methodが Promiseを返すから?
        // Future は RustのNativeっぽい型、 JsFuture::from(promise) で Promise->Future に変換される
        let future = JsFuture::from(request_promise)
            // async {} は Future を返すので .and_then でつなげることができる
            // ここは move は不要。あっても問題はない。
            .and_then(|resp_value| async {
                // `resp_value` is a `Response` object.
                assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into().unwrap();
                resp.json()
            })
            // ここは別の書き方もあるのかな...? でも流れ的にこうかくことにする。
            .and_then(|value: Promise| JsFuture::from(value))
            // async move {} と書くのがポイント
            // move とすると、closure内でcaptureしている ptr_for_update の所有権を外から奪うことになり、
            // *ptr_for_update.borrow_mut() = ... が使えるようになる。
            // 当然 move を削るとコンパイルエラーになる。
            .and_then(|json| async move {
                let team: Team = json.into_serde().unwrap(); // team: Team と型を明示するのがポイント
                *ptr_for_update.borrow_mut() = Some(team);
                Ok(JsValue::from("load_data success!"))
            });
        future_to_promise(future) // Promise としてreturnするので JS側から await や .then() などで呼ぶ。
    }

    pub fn show_team_info(&self) {
        if let Some(team) = self.team.borrow().as_ref() {
            log(&format!("Team Name={}", &team.name));
            for member in &team.members {
                log(&format!(
                    "name={}, age={}, role={}",
                    &member.name, member.age, &member.role
                ));
            }
        } else {
            log(&format!("There is no team."));
        }
    }
}
