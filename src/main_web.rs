use minifb::{Window, WindowOptions};
use std::{cell::RefCell, panic, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wgpu::web_sys;

use crate::{HEIGHT, WIDTH};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let _window = Window::new(
        "Minimal wgpu + minifb",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // A reference counted pointer to the closure that will update and render the application.
    let update_closure = Rc::new(RefCell::new(None));
    let update_closure_cpy = update_closure.clone();

    // Create the closure for updating and rendering the application.
    *update_closure_cpy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // TODO: wgpu init and minimal drawing.
        // schedule this closure for running again at next frame
        request_animation_frame(update_closure.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut() + 'static>));

    // Start the animation loop.
    request_animation_frame(update_closure_cpy.borrow().as_ref().unwrap());
}
