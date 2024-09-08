use std::sync::Arc;

use minifb::{Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

#[cfg(target_arch = "wasm32")]
mod main_web;

#[cfg(not(target_arch = "wasm32"))]
mod main_desktop;

struct Application<'a> {
    window: Arc<Window>,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> Application<'a> {
    /// Initializes the application.
    ///
    /// There's various ways for this to fail, all of which are handled via `expect` right now.
    /// Of course there's be better ways to handle these (e.g. show something nice on screen or try a bit harder).
    async fn new() -> Self {
        let window = Arc::new(
            Window::new(
                "Minimal wgpu + minifb",
                WIDTH,
                HEIGHT,
                WindowOptions::default(),
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            }),
        );

        // WebGL fallback is not enabled, this would require some special handling to do gracefully today,
        // see https://github.com/gfx-rs/wgpu/issues/6166
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut application = Application {
            window,
            surface,
            adapter,
            device,
            queue,
        };

        // Initial surface configuration - required at least on web since otherwise first get_current_texture() will panic.
        application.configure_surface();

        application
    }

    fn configure_surface(&mut self) {
        // Need to reconfigure the surface and try again.
        let (width, height) = self.window.get_size();
        self.surface.configure(
            &self.device,
            &self
                .surface
                .get_default_config(&self.adapter, width as u32, height as u32)
                .expect("Surface is not supported by the active adapter."),
        );
    }

    pub fn draw_frame(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(err) => match err {
                wgpu::SurfaceError::Timeout => {
                    log::warn!("Surface texture acquisition timed out.");
                    return; // Try again next frame. TODO: does this make sense?
                }
                wgpu::SurfaceError::Outdated => {
                    // Need to reconfigure the surface and try again.
                    self.configure_surface();
                    return;
                }
                wgpu::SurfaceError::Lost => {
                    log::error!("Swapchain has been lost.");
                    return; // Try again next frame. TODO: does this make sense?
                }
                wgpu::SurfaceError::OutOfMemory => panic!("Out of memory on surface acquisition"),
            },
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main encoder"),
            });

        {
            let mut _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // TODO: draw something
        }

        let command_buffer = encoder.finish();
        self.queue.submit(Some(command_buffer));
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    return; // Not used on web, this method is merely a placeholder.

    #[cfg(not(target_arch = "wasm32"))]
    {
        // TODO
    }
}
