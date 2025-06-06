pub use vello::util::RenderSurface as Surface;
use vello::wgpu::{CommandEncoderDescriptor, PresentMode, SurfaceTarget};
use vello::{AaConfig, Renderer, util::RenderContext};

use crate::UiContext;

pub struct Context {
    render_ctx: RenderContext,
    renderers: Vec<Option<Renderer>>,

    ui_ctx: UiContext,
}

impl Context {
    pub fn new() -> Self {
        Self {
            render_ctx: RenderContext::new(),
            renderers: Vec::new(),

            ui_ctx: UiContext::new(),
        }
    }

    pub async fn create_surface<'w>(
        &mut self,
        window: impl Into<SurfaceTarget<'w>>,
        size: (u32, u32),
    ) -> Result<Surface<'w>, vello::Error> {
        let surface = self
            .render_ctx
            .create_surface(window, size.0, size.1, PresentMode::AutoVsync)
            .await?;

        self.renderers
            .resize_with(self.render_ctx.devices.len(), || None);
        self.renderers[surface.dev_id].get_or_insert_with(|| {
            Renderer::new(
                &self.render_ctx.devices[surface.dev_id].device,
                Default::default(),
            )
            .expect("Failed to create renderer")
        });

        Ok(surface)
    }

    pub fn resize_surface<'w>(&mut self, surface: &mut Surface<'w>, size: (u32, u32)) {
        self.render_ctx.resize_surface(surface, size.0, size.1);
    }

    pub fn set_ui(&mut self, ui: impl FnOnce(&mut UiContext)) {
        self.ui_ctx.set_ui(ui);
    }

    pub fn render<'w>(&mut self, surface: &mut Surface<'w>) {
        let device_handle = &self.render_ctx.devices[surface.dev_id];
        let renderer = self.renderers[surface.dev_id].as_mut().unwrap();

        renderer
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.ui_ctx.scene,
                &surface.target_view,
                &vello::RenderParams {
                    base_color: vello::peniko::color::palette::css::BLACK,
                    width: surface.config.width,
                    height: surface.config.height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("Failed to render scene");

        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("Failed to get current texture");

        let mut encoder = device_handle
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Blit Surface Texture"),
            });

        surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &surface.target_view,
            &surface_texture.texture.create_view(&Default::default()),
        );
        device_handle
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        device_handle.device.poll(vello::wgpu::Maintain::Poll);
    }
}
