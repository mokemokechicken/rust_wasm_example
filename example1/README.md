# consoleにログを出してみる
まあ、まずはとにかく動くようにするところからですが、それは拍子抜けするほど簡単でした。端的にいうと以下のファイルを用意しておけばOKです。(Makefileは別にいらんけど)。

```
.
├── Cargo.toml
├── Makefile
├── index.html
└── src
    └── lib.rs
```

一つずつみていきます。

## Cargo.toml
主に使用するcrateなどの宣言をするところですが、最初はとにかくこう書いておけ、という感じではあります。以下の３つのcrateを使用しますが、私の理解では、

- `js-sys`: JavaScriptのいろいろをRustで使うための何か
- `web-sys`: ブラウザの機能やDOM周りをRustで使うための何か
- `wasm-bindgen`: JavaScript <-> wasm のInterface周りをやってくれる何か

となっています。 `features` というのは、余分なコードを取り込まないように使う機能(コード)だけをbuildしましょうということで分けてあるんだと思います(ふんいき談)。

```toml:Cargo.toml
[package]
name = "rust_wasm_example"
version = "0.1.0"
authors = ["mokemokechicken"]
edition = "2018"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2.67"
js-sys = "0.3.44"

[dependencies.web-sys]
version = "0.3.44"
features = [
  'console',
]
```

ここで

```
[dependencies.web-sys]
version = "0.3.44"
features = [
  'console',
]
```

という部分は、`[dependencies]`の中に

```
web-sys = {"version" = "0.3.44", features=['console']}
```

と書いても同じなのですが、今後この`web-sys`については `features` がどんどん増えていくので、上記のような書き方にしておくほうが見やすい気がします（意味は同じ、、だと思う)。
大事なのは２つ同時に書いてしまうと、エラーになるので注意が必要ということです(これで１０分くらい悩んだし...)。

## src/lib.rs
Rust本体。ライブラリ系は `lib.rs` が起点になるというのが規約となっているようです。

```rust:lib.rs
use wasm_bindgen::prelude::*;
use web_sys::console::log_1;

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[wasm_bindgen]
pub fn output_log(s: &str) {
    log(&format!("Hello {}", s));
}
```

ポイントはやはり `#[wasm_bindgen]` の部分と `use web_sys::console::log_1` かな。

`#[wasm_bindgen]` となっているfunctionはJavaScriptから呼べるようになります。指定できる引数や返り値は結構なんでもいけます(書き方などは面倒なのもありますが)。文字列だとこんな感じです。

`console::log_1` というのは引数を１つとる JSでいう `console.log()` みたいなものっぽいです。 `log_2`, `log_3` というのもあります。
ログはやはりよく使うことになると思いますが、 `&JsValue::from(s)` みたいに `JsValue` という `JavaScript世界の型` にしてあげる必要があるようです。ので、簡単な関数を作っておくと便利でした。`String`型を作るには `format!("{}, {}", val1, val2)` みたいなマクロがだいたいどんなときでも使えるのでまずはこれだけ覚えておけばログには困らないという気がします(Rust歴数日の人間の言うことは信じてはいけない)。

※ 実はここでは `js-sys` は使ってないから Cargo.toml には書かなくて良かったのですが、どうせまあ今後使うので...

一応この `Cargo.toml` と `lib.rs` だけあればwasmのビルドができてしまいます。

## Makefile
ビルドコマンドを覚えるのが面倒なので、Makefileに書いておきます。
- `build` は wasm ビルド用。 `--target web` がポイント。他にも方法があるがこれが余計なツールを使わずに済んで好みだった。
- `serve` は wasmモジュールをLocalfileから読み込んではくれない(たぶん)のでHttpServerが必要です。なんでも良いのですがここでは Pythonの `SimpleHTTPServer` を使います。

```makefile:Makefile
build:
	wasm-pack build --target web

serve:
	echo "http://localhost:8082/"
	python -m SimpleHTTPServer 8082

clean:
	rm -rf pkg target
```

## Buildしてみる

```
make build
```

とします。成功すると `pkg/`, `target/` というDIRができると思います。使うのは `pkg/`の方です。

これが通れば、あとは簡単です。

## index.html
この[wasm-bindgenのGuide](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html)から借用します。

`<script type="module">` なんてあるんですね。知らなかった。

```html:index.html
<!DOCTYPE html>
<html lang="ja">
  <head>
    <meta content="text/html;charset=utf-8" http-equiv="Content-Type"/>
  </head>
  <body>
    <!-- Note the usage of `type=module` here as this is an ES6 module -->
    <script type="module">
      import init, * as wasm from './pkg/rust_wasm_example.js';
      async function run() {
        await init();
        wasm.output_log("Wasm!");
      }
      run();
    </script>
  </body>
</html>
```

## HTTPServerを起動してアクセス
`make serve` とします。

```
% make serve
echo "http://localhost:8082/"
http://localhost:8082/
python -m SimpleHTTPServer 8082
Serving HTTP on 0.0.0.0 port 8082 ...
```

こんな表示になって、http://localhost:8082/ にアクセスしてみて、consoleを開いてログが表示されていれば成功となります。
