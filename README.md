Minimal [`wgpu`](https://github.com/gfx-rs/wgpu) and [`minifb`](https://github.com/emoon/rust_minifb) example for web & desktop
================================================

This is a minimal example of how to use [`wgpu`](https://github.com/gfx-rs/wgpu) and [`minifb`](https://github.com/emoon/rust_minifb) to render a triangle on the screen.

Run on desktop
--------------

WIP:

```sh
cargo run
```

Run on web
----------

```sh
cargo xtask run-wasm
```

This executes an xtask that builds the wasm binary and launches a web server.

The example will automatically pick WebGPU if available and fall back to WebGL otherwise.
Known issue: Chrome Linux will pick WebGPU even though not supported (will be addressed once https://github.com/gfx-rs/wgpu/pull/6371 lands).