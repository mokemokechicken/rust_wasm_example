はじめに
========
## Crates
今回使うcrateたちはこんな感じになっています。
- `wasm-bindgen` の features が増えた
- `serde` が登場
- `web-sys` の features に `Window`, `Document`, `HtmlDivElement` が増えた

```Cargo.toml
...略
[dependencies]
wasm-bindgen = { version = "0.2.67", features = ["serde-serialize"] }
js-sys = "0.3.44"
serde = { version = "1.0", features = ["derive"] }

[dependencies.web-sys]
version = "0.3.44"
features = [
  'Window',
  'Document',
  'HtmlDivElement',
  'console',
]
```

# 内容
## Rust側に状態をもたせる
いくつか方法はありそうですが、一番シンプルだと思うのは、JS側でRustのObjectを生成し、保持させておくことかなと思います。JS側としても特に違和感のないところです。

例えば、以下のようにRust側をしておきます。

```rust
#[wasm_bindgen]   // これ大事
pub struct MyApp {
    age: u32,
}

#[wasm_bindgen]  // この中のpubメソッドはJSから呼べるようになる。
impl MyApp {
    pub fn new(age: u32) -> MyApp {
        MyApp { age }
    }
}
```

で、 JS側で `MyApp.new()` を呼ぶとRust側のObjectをKeepできます。

```javascript
const myApp = wasm.MyApp.new(88);
console.log(myApp.age);  // -> 88
```

## JS ObjectをRustに渡す
例えば、各種設定のようなものをまとめてRust側に渡したいときに

```javascript
const myApp = MyApp.new({"key1": value1, "key2": value2});
```

みたいにやりたい。全部指定するのは面倒なので default値も設定しておきたい。

という場合があります。
以下のようにやると、だいぶイメージに近い感じになりました。

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MyAppOptions {
    pub name: String,
    pub age: Option<u32>,
    pub email: Option<String>,
}

#[wasm_bindgen]
pub struct MyApp {
    name: String,
    age: u32,
    email: Option<String>,
}

#[wasm_bindgen]
impl MyApp {
    pub fn new(options: JsValue) -> MyApp {
        let opts: MyAppOptions = options.into_serde().unwrap();
        MyApp {
            name: opts.name,
            age: opts.age.unwrap_or(18),  // default値で埋めておく場合
            email: opts.email,            // Option のままの場合
        }
    }
}
```

ポイントは、

```rust
let opts: MyAppOptions = options.into_serde().unwrap();
```

の辺りかな。 `MyAppOptions` に `#[derive(Serialize, Deserialize)]` がついているので `JsValue` からこの構造体に値をうつしてくれます。

呼び出すJS側は、以下のように書くことができます。

```javascript
const myApp = wasm.MyApp.new({name: "mokemoke", email: "hello@example.com", other: 88});
```

`age` が省略できていることと、 `other`という関係ないKeyがあってもエラーにならないことがポイントです。

## DOMにアクセスする
DOMにアクセスするには web_sys の ものを色々使います。
`Window`や`Document`にはよくアクセスするので以下のようなショートカットを作っておくとちょっとだけ便利です。

```rust
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}
```

`document.getElementById()` などをRustで書くとこんな感じになります。

```rust

fn get_element_by_id<T: JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .expect("not found")
        .dyn_into::<T>()
        .map_err(|_| ())
        .unwrap()
}
```

この `.dyn_into::<型>()` とかはJs系のCastをするときに結構出てきます。

で、
例えば、指定したdiv要素のinner_textを更新するにはこんな感じにかけば良いです。

```rust
#[wasm_bindgen]
impl MyApp {
    pub fn show_status(&self, div_id: &str) {
        let div: web_sys::HtmlDivElement = get_element_by_id(div_id);
        div.set_inner_text(&format!("name={}, age={}, email={}",
                                    self.name,
                                    self.age,
                                    self.email.as_ref().unwrap_or(&"".into()) // 野暮ったい...
        ));
    }
}
```

それぞれどういうMethodがあるかは [web_sysのAPI Document](https://docs.rs/web-sys/0.3.44/web_sys/struct.HtmlDivElement.html) などを見ると良いです。

# さいごに

なんか、 unwrap() したり dyn_into() したり、serdeがどうこうしたり、、と慣れるまで何がなんだかわかりません。今もよくわかってないところも多いですが、コンパイラのエラーメッセージ（これが結構親切なのがとても助かる）を見ながら治せるくらいにはなってきました。
