# はじめに
[前々回](https://qiita.com/mokemokechicken/items/37fec7b97350a26a1932) で作ったアニメーションでは、 `requestAnimationLoop()` は JavaScript側から仕込んでもらっていました。では、これをRust内で完結させるにはどうしたらいいのか。また、on_clickのようなイベントハンドラを登録するにはどうしたら良いのか気になったので調べてみました。

# 内容

## requestAnimatinLoop() と on_click()の登録
ポイントは以下のようなところかと思います。

- `start()` の中で `MyApp::new()` して、それの `RC & RefCell` を `requestAnimationLoop()` や `on_click()` の Closureに渡すことで mutable な MyAppでメソッドが呼び出せるようになっている。
- `requestAnimationLoop()` の中でもう一度同じClosureを呼び出さないといけないので、`closure_captured`, `closure_cloned` という「こんなの普通にはなかなか思いつかないよ」というHackを使う。
- JSのイベントハンドラは、 `js_sys:Function` という型を要求するが `Closure<dyn FnMut(型)>` というもの(これは `Closure::wrap(Box::new(|変数: 型| {...}) as Box<dyn FnMut(型)>` で生成する)を `.as_ref().unchecked_ref()` すると得られる。呪文か。
- on_clickのときのようなClosureは、Closure自身がRustのScope外に出ると消滅するので、 `closure.forget()` として、Rustのメモリ管理から外れて JS側にGCにライフサイクルを委ねるのが良いようだ。ちなみに `forget()` 系は下手するとメモリリークを起こすので、使い所には気をつけないといけない。


```rust
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

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

```


## index.html
JS側は、単に `start()` を呼ぶだけになっています。

```HTML
    <script type="module">
      import init, * as wasm from './pkg/rust_wasm_example.js';

      async function run() {
        await init();
        wasm.start();
      }
      run();
    </script>
```

## MyApp
サンプル用のものです。値が更新できる(mutableである)、というのが重要です。

```rust
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
```

# さいごに

前々回のようなやり方と、今回のように閉じたやり方とどちらが良いのかはケースバイケースなのかな。
簡易なケースだと前々回のような書き方でも良さそう（簡単だし）だけど、固くきっちり作るならこういう感じになるのでしょうか。
MyApp自体がJS側に渡らなくなっていて(たぶん、MyApp自体を渡すことは所有権の関係でできない)JS側からちょっかいを出しにくくなっているので、 `start()` の返り値として、MyAppにちょっかいを出せるObjectをreturnしておくというのはあり得るかもしれない。

Rustは他の言語にはない設計が必要ですね。パズルみたいで面白いけど、大規模開発だとどういう規約で作っていくのだろうか。
