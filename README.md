Minimal [`wgpu`](https://github.com/gfx-rs/wgpu) and [`minifb`](https://github.com/emoon/rust_minifb) example for web & desktop
================================================

This is a minimal example of how to use [`wgpu`](https://github.com/gfx-rs/wgpu) and [`minifb`](https://github.com/emoon/rust_minifb) to render a triangle on the screen.

![image](https://github.com/user-attachments/assets/89bce01d-3e96-435e-b897-038fa1cee340)

Run on desktop
--------------

```sh
cargo run
```

Run on web
----------

```sh
cargo xtask run-wasm
```

This executes an xtask that builds the wasm binary and launches a web server.

Known issues & limitations
--------------------------

* A bunch of fixes to `minifb` are required which haven't been released yet (as of writing). This project therefore depends on a specific commit of `minifb` for now.
