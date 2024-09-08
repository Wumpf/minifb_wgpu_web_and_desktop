const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[cfg(target_arch = "wasm32")]
mod main_web;

#[cfg(not(target_arch = "wasm32"))]
mod main_desktop;

fn main() {
    #[cfg(target_arch = "wasm32")]
    main_web::start();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // TODO
    }
}
