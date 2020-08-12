# はじめに
今回は、アプリケーションを作っていてよくやるような「HTTPでJSONデータをFetchしてきてPropertyに保存する」ことをやってみます。

JSなら数行でかけるようなこんなことが、結構難航しました。わかってしまえば、柔軟にいろいろな型にも対応できるしライブラリ化したりも簡単そうですが、わかるまでが大変なんですよね、、、こういうの。

# 内容
## コード
### Cargo.toml
- `wasm-bindgen-futures`, `futures` と Future関連が追加
- `web-sys` から `Headers`, ..., `Response` などのHTTP関連のものが追加

```toml:Cargo.toml
[dependencies]
wasm-bindgen = { version = "0.2.67", features = ["serde-serialize"] }
js-sys = "0.3.44"
serde = { version = "1.0", features = ["derive"] }
#
wasm-bindgen-futures = "0.4.17"
futures = "0.3.5"

[dependencies.web-sys]
version = "0.3.44"
features = [
  'Window',
  'console',
  # HTTP
  'Headers',
  'Request',
  'RequestInit',
  'RequestMode',
  'Response',
]
```

### lib.rs
いくつかポイントというかハマったところがあります。

#### load_data() 自体を async にすると、取得したデータを selfに保存しようとしてハマる

たぶん、それが原因だったと記憶していますが、 `async fn load_data(&mut self)` みたいにすると、lifetimeか何かの問題で上手くいかないです。
※ 今から再現するために書き戻す気がしないので、最終的な確認はしていないですが...

なので、 load_data() みたいな非同期的処理をする場合、 Promise や Future をreturnすることにして、そのFunction自体は asyncにはしない方がよさそうです。

#### `Rc<RefCell<型>>` を活用する
[スマートポインタ](https://doc.rust-jp.rs/book/second-edition/ch15-00-smart-pointers.html) の説明の先にこの使い方が載っています。
仕組みや詳しい使い方は読んでもらうとして、「実行時の自己責任において、mutなpointerを複製できちゃう＆値も更新できちゃう方法」です。 Closureのようにlifetimeが不明な処理が、その内部で外側のObjectの更新を可能にするためのほぼ必須な技なんじゃないかと思います。

```rust
pub struct MyApp {
    team: Rc<RefCell<Option<Team>>>,
}

...
fn load_data(&mut self) {
  let ptr_for_update = self.team.clone();
  ...
  (in closure)
    *ptr_for_update.borrow_mut() = <value>;
}
```

みたいなのがパターンかなと思います。


#### Promise -> Future に変換して処理をつなげる
`window.fetch_with_request(&request)` とか `response.json()` みたいな web_sys側の機能は Promise を返すようです。それを `JsFuture::from(promise)` という感じで Future に変換すると、色々書きやすい感じがあります。


<details>
<summary>Rustコード全体</summary>
<div>

```rust:lib.rs
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
```

</div></details>

### index.html
こんな感じで
`await myApp.load_data()` とか `myApp.load_data().then(function ...)` みたいに使うことができます。一応、こうかければ、初期化処理が終わったらメインループを始めるみたいなことをJS側にやらせれば全体としては困らないかなと思います。

ユーザのActionに応じて任意のタイミングで通信が発生するような場合だと、もう少し気を使わないといけないかもしれないですが。

```javascript
      async function run() {
        await init();
        const myApp = wasm.MyApp.new();

        myApp.show_team_info(); // -> There is no team.

        const ret = await myApp.load_data("data.json");
        console.log(ret); // -> load data success!

        myApp.show_team_info(); // -> Team Name=Cats ...
      }
      run();
```

さいごに
======

少しRustを理解してきた気がしますが、まだまだ先は長そうです。
