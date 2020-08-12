# はじめに
今回はなにかアニメーションみたいなのを作ってみようと思います。

## 作ったもの

なんとなく四角の中を赤いBallが大量に飛び回るものを作ってみます。

※ (clickでYoutubeに飛びます)
[![Demo](https://img.youtube.com/vi/isp4qD-G110/0.jpg)](https://www.youtube.com/watch?v=isp4qD-G110)

Ballが何個のときにFPSがどのくらいかなぁ、というのがちょっと気になるところです。

# 内容
## 困ったこと
アニメーションさせるのに `requestAnimationFrame()` というBrowser側?のAPIを使うことが多いと思いますが、`requestAnimationFrame()` は `setInterval` とかと違って定期的に呼ばれるわけではないので、 `requestAnimationFrame()` の内部でもう一度 `requestAnimationFrame()` を呼ばないといけません。

最初は、Rustの `on_animation_frame()` みたいな method の中で 再び `requestAnimationFrame()` を呼ぶようにしようと思ったのですが、そこで closureを渡すのが尋常じゃなく上手くいかなくて、、、諦めました。(今ならなんとかできるかもしれないが、少し面倒そうなのはかわらない)。したがって、JS側でそのループを作って毎回 Rustの`on_animation_frame()` を call してもらうようにしました。まあ、こういうのも無くはないかな、と。

それ以外は、素直なRustプログラムになって、かなりコンパイラに怒られながらも動くものが作れました。

## コード
### Cargo.toml

- `rand = { version="0.7.3", features =["wasm-bindgen"] }` が追加。
    - このfeaturesを指定しないと、WASM内でrandが使えないようなので注意が必要
- web-sys feature: `CanvasRenderingContext2d`
    - Context2d使うので必要

### index.html
ちょっと妥協の産物ですが、これで毎回AnimationのFrameを更新するようにしています。

```javascript:type="module"
  import init, * as wasm from './pkg/rust_wasm_example.js';
  async function run() {
    await init();
    const myApp = wasm.MyApp.new({canvas_id: "canvas", initial_n_balls: 1000});
    myApp.init();
    function onAnimationFrame(t) {
      const ret = myApp.on_animation_frame(t);
      if (ret) {
        requestAnimationFrame(onAnimationFrame);
      }
    }
    requestAnimationFrame(onAnimationFrame);
  }
  run();
```

### 他
Animationをさせるためにコードは色々書きましたが特筆するようなところはないかな。
ちょっと工夫したのは、Ballに動きの尾ひれ(残像?残影?軌跡?)をつけるために、過去数フレームの位置を保存させて描画しているところかな。全体を保存(コピー)させているけど、Ball毎にVecとかで履歴を持たせたほうが良かったかな。そうすると、１本の線とかにもしやすかっただろうし。

まあ、ちょっと固まったRustのコードを書くいい練習にはなりました。

ちなみに今回requestAnimationFrame() を使うと、1frameあたりの処理が十分早ければちょうどFPS=60になるようでした。遅くなると60を切っていきます。

「衝突」「引力 or 斥力」を全Ball間でしているので、Ballの数が増えると計算量が結構あがっていきます。私のPCだと4~500個くらいで FPSが60を切ってしまうかな、という感じでした。
上記の計算をしない場合は、だいたいCanvasへの描画処理ばかりになるのでもうちょっとBallが多くてもFPS６０が維持されます。

# さいごに

なんか、仕様&動作確認的な実験的なものばかりだと飽きるので作ってみた他愛のないものですが、せっかくなので書き残しておきました。
