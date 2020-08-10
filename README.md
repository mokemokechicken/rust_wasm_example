はじめに
========
RustにもWebAssembly(以降wasmと呼ぶ)にも興味はあるが、馴染みがない今日このごろ。Rustでwasmをやってみれば一石二鳥なんじゃあないか！？と思って始めてみました。
やってみると、Hello World的なものは意外とすぐにできたのですが、 AnimationLoop しようとしたり、データをHTTPでFetchしようとしたりするとなかなか尋常じゃなく躓きました（それぞれ１日くらい解決までかかった...)。
Rustも発展途上でちょいちょい言語仕様も変わっているので、過去のWebの記事通りでは動かなかったり、そもそもRustへの理解が足りてなかったりするのですが、自分用のメモ兼もしかしたら誰かの参考になるかもしれないので、何回かに分けてざっくりこれまで理解したことを書いておこうと思います。

## Version
これから示すコードが動くかどうかについて、RustやcrateのVersionはおそらくとても大事です。

OSはMacでもWindowsでもLinuxでもあまり差はないと思います。
ブラウザもいわゆるモダンなブラウザなら大丈夫だと思います。IEだとダメかもしれない。Edgeはどうなのかな。(`<script type="module">`みたいなので動くかどうかで、うまくいかない場合は他の方法を使えばwasm自体は動くと思います)。

- rustc 1.45.2 (d3fb005a3 2020-07-31): [Install方法](https://www.rust-lang.org/tools/install)
- wasm-pack 0.9.1: [Install方法](https://rustwasm.github.io/wasm-pack/installer/)
- crate
    - [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) = "0.2.67"
    - [js-sys](https://rustwasm.github.io/wasm-bindgen/api/js_sys/) = "0.3.44"
    - [web-sys](https://rustwasm.github.io/wasm-bindgen/api/web_sys/) = "0.3.44"
- MacOS: 10.15.6
- Google Chrome: 84.0.4147.105
- Python3 (単純なHttpServer用)

Contents
=========

- example1: [consoleにログをだしてみる](example1/)
- example2: [DOMアクセス & JS ObjectをRustに渡す](example2/)
- example3: [Context2d でアニメーションしてみる](example3/)
