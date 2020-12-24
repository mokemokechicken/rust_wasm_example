// ２つのリクエスを同時に実行して JOIN させる実験

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::log_1;
//
use core::cell::RefCell;
use futures::future::join_all;
use futures::prelude::*;
use js_sys::Promise;
use std;
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
    teams: Rc<RefCell<Option<Vec<Team>>>>,
}

#[wasm_bindgen]
impl MyApp {
    pub fn new() -> MyApp {
        MyApp {
            // Rc<RefCell<任意のデータ>> というのはRustでよくある使い方
            // https://doc.rust-jp.rs/book/second-edition/ch15-00-smart-pointers.html
            // ここのスマートポインタの部分をよく読むと良い。
            teams: Rc::new(RefCell::new(None)),
        }
    }

    // この関数自体を async にすると &mut selfの辺りが上手くいかない(確か)。
    // selfをClosureにcaptureさせずに 更新したい部分を Rc<RefCell> で capture させるのが良さそう。
    // この関数自体はPromiseをreturnさせる。
    pub fn load_data(&mut self) -> Option<Promise> {
        let path_list = vec!["team1.json", "team2.json"];
        let mut future_list: Vec<_> = Vec::new();
        let ptr_for_update = self.teams.clone();

        for path in path_list {
            let future = MyApp::json_request(path).and_then(|json| async move {
                let team: Team = json.into_serde().unwrap(); // team: Team と型を明示するのがポイント
                Ok(team)
            });
            future_list.push(future);
        }
        let future = join_all(future_list).map(move |vs: Vec<Result<Team, JsValue>>| {
            let mut teams: Vec<Team> = Vec::new();
            for result_team in vs {
                teams.push(result_team.unwrap());
            }
            ptr_for_update.replace(Some(teams));
            Ok(JsValue::from("99"))
        });
        Some(future_to_promise(future))
    }

    fn json_request(path: &str) -> impl Future<Output = Result<JsValue, JsValue>> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(path, &opts).unwrap();
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
            .and_then(|resp_value| async {
                assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into().unwrap();
                resp.json()
            })
            .and_then(|value: Promise| JsFuture::from(value));
        future
    }

    pub fn show_team_info(&self) {
        if let Some(teams) = self.teams.borrow().as_ref() {
            for team in teams {
                log(&format!("Team Name={}", &team.name));
                for member in &team.members {
                    log(&format!(
                        "name={}, age={}, role={}",
                        &member.name, member.age, &member.role
                    ));
                }
            }
        } else {
            log(&format!("There is no team."));
        }
    }
}
