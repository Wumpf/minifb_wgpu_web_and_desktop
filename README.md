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

As of writing the following two fixes to `minifb` are required (the sample uses a fork that contains both):
* https://github.com/emoon/rust_minifb/pull/367
* https://github.com/emoon/rust_minifb/pull/368

Chrome(ium) on Linux incorrectly reports WebGPU support, causing the application to crash on startup.
This will likely be addressed in a future release of `wgpu` using a new `wgpu::util::new_instance_with_webgpu_detection` utility method:
* https://github.com/gfx-rs/wgpu/pull/6371
