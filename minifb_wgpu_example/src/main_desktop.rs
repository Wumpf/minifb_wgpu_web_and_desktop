use crate::Application;

pub fn main_desktop() {
    env_logger::init();

    let mut application = pollster::block_on(Application::new());

    while application.window.is_open() {
        application.window.update();
        if application
            .window
            .is_key_pressed(minifb::Key::Escape, minifb::KeyRepeat::No)
        {
            return;
        }

        application.draw_frame();
    }
}
