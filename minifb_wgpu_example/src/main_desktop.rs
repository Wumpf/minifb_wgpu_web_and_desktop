use crate::Application;

pub fn main_desktop() {
    env_logger::init();

    let mut application = pollster::block_on(Application::new());

    loop {
        application.window.update();
        if application
            .window
            .is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No)
        {
            return;
        }

        // It's important to check openness after updating the window.
        // Otherwise, wgpu's surface might be invalid now.
        if !application.window.is_open() {
            return;
        }

        application.draw_frame();
    }
}
