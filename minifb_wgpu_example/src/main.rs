use std::mem::ManuallyDrop;

use minifb::{Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

#[cfg(target_arch = "wasm32")]
mod main_web;

#[cfg(not(target_arch = "wasm32"))]
mod main_desktop;

struct Application<'a> {
    window: Window,
    surface: ManuallyDrop<wgpu::Surface<'a>>,
    surface_format: wgpu::TextureFormat,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    triangle_render_pipeline: wgpu::RenderPipeline,
}

impl Drop for Application<'_> {
    fn drop(&mut self) {
        // Drop surface before dropping the window, to ensure it always refers to valid window handles.
        unsafe {
            ManuallyDrop::drop(&mut self.surface);
        }
    }
}

impl<'a> Application<'a> {
    /// Initializes the application.
    ///
    /// There's various ways for this to fail, all of which are handled via `expect` right now.
    /// Of course there's be better ways to handle these (e.g. show something nice on screen or try a bit harder).
    async fn new() -> Self {
        // TODO: Use `wgpu::util::new_instance_with_webgpu_detection` once https://github.com/gfx-rs/wgpu/pull/6371 lands.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let window = Window::new(
            "minifb",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // Unfortunately, mini_fb's window type isn't `Send` which is required for wgpu's `WindowHandle` trait.
        // We instead have to use the unsafe variant to create a surface directly from the window handle.
        //
        // SAFETY:
        // * The window handles are valid at this point
        // * The window is guranteed to outlive the surface since we're ensuring so in `Application's` Drop impl
        let surface = unsafe {
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(&window)
                    .expect("Failed to create surface target."),
            )
        }
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
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(), // Needed if you want to support WebGL!
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Make all errors forward to the console before panicking, this way they also show up on the web!
        device.on_uncaptured_error(Box::new(|err| {
            log::error!("{}", err);
            panic!("{}", err);
        }));

        let surface_format = Self::pick_surface_format(&surface, &adapter);
        let triangle_render_pipeline = Self::create_render_pipeline(&device, surface_format);

        let mut application = Application {
            window,
            surface: ManuallyDrop::new(surface),
            surface_format,
            adapter,
            device,
            queue,

            triangle_render_pipeline,
        };

        // Initial surface configuration - required at least on web since otherwise first get_current_texture() will panic.
        application.configure_surface();

        application
    }

    fn pick_surface_format(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
    ) -> wgpu::TextureFormat {
        // WebGPU doesn't support sRGB(-converting-on-write) output formats, but on native the first format is often an sRGB one.
        // So if we just blindly pick the first, we'll end up with different colors!
        // Since all the colors used in this example are _already_ in sRGB, pick the first non-sRGB format!
        let surface_capabilitites = surface.get_capabilities(&adapter);
        for format in &surface_capabilitites.formats {
            if !format.is_srgb() {
                return *format;
            }
        }

        log::warn!("Couldn't find a non-sRGB format, defaulting to the first one");
        surface_capabilitites.formats[0]
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[], // No need for vertex buffers, the shader generates all the data.
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(target_format.into())],
            }),
            multiview: None,
            cache: None,
        })
    }

    fn configure_surface(&mut self) {
        // Need to reconfigure the surface and try again.
        let (width, height) = self.window.get_size();
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                format: self.surface_format,
                ..self
                    .surface
                    .get_default_config(&self.adapter, width as u32, height as u32)
                    .expect("Surface is not supported by the active adapter.")
            },
        );
    }

    pub fn draw_frame(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(err) => match err {
                wgpu::SurfaceError::Timeout => {
                    log::warn!("Surface texture acquisition timed out.");
                    return; // Try again next frame. TODO: does this make always sense?
                }
                wgpu::SurfaceError::Outdated => {
                    // Need to reconfigure the surface and try again.
                    self.configure_surface();
                    return;
                }
                wgpu::SurfaceError::Lost => {
                    log::error!("Swapchain has been lost.");
                    return; // Try again next frame. TODO: does this make always sense?
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
            let cornflower_blue = wgpu::Color {
                r: 0.39215686274509803,
                g: 0.5843137254901961,
                b: 0.9294117647058824,
                a: 1.0,
            };

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(cornflower_blue),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.triangle_render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        let command_buffer = encoder.finish();
        self.queue.submit(Some(command_buffer));
        frame.present()
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    return; // Not used on web, this method is merely a placeholder.

    #[cfg(not(target_arch = "wasm32"))]
    main_desktop::main_desktop();
}
