# Running Yafa Express in a browser

The game targets `wasm32-unknown-unknown`. Everything it needs is compiled in -
the three typefaces included - so there is nothing to serve but the two files
`wasm-bindgen` produces and the page that loads them.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli        # once; must match the wasm-bindgen crate

cargo build --profile wasm-release --target wasm32-unknown-unknown -p yafa_game
wasm-bindgen --no-typescript --target web \
  --out-dir web --out-name yafa-express \
  target/wasm32-unknown-unknown/wasm-release/yafa-express.wasm

python3 -m http.server --directory web 8080
```

Then open <http://localhost:8080>. It has to be served over HTTP rather than
opened from the filesystem: the page is an ES module, and browsers refuse those
over `file://`.

The `wasm-release` profile optimises for size and strips symbol names, which
between them take most of a very large build down to something servable. It is
still a whole game engine: if you mean to put it on the public internet, run
[`wasm-opt -Oz`](https://github.com/WebAssembly/binaryen) over the result and
serve it with compression, which together roughly halve it again.

## What differs from the desktop build

The passport is not written to disk. There is no filesystem to write it to, so
on the web the year, the money and the stamps last as long as the tab does.
Everything else - the whole map, the economy, both languages - is the same
build of the same code.
