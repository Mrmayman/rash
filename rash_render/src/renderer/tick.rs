use std::time::Instant;

use crate::SpriteId;

use super::to_bytes;

use super::{
    buffers::{GlobalBuffer, GraphicsState},
    InnerRenderer,
};

impl InnerRenderer<'_> {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.global_state.resolution = [new_size.width as f32, new_size.height as f32];
            self.update_global_state();
        }
    }

    fn update_global_state(&mut self) {
        self.queue.write_buffer(
            &self.global_buffer,
            0,
            to_bytes(&self.global_state, std::mem::size_of::<GlobalBuffer>()),
        );
    }

    pub fn render(&mut self, sprite_order: &[SpriteId]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            for i in sprite_order {
                let i = i.0 as u32 * 6;
                render_pass.draw(i..(i + 6), 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn tick(
        &mut self,
        control_flow: &winit::event_loop::EventLoopWindowTarget<()>,
        graphics: &[GraphicsState],
        sprite_order: &[SpriteId],
    ) {
        let delta = self.last_time.elapsed().as_secs_f64() * 60.0;
        // This tells winit that we want another frame after this one
        self.window.request_redraw();

        // Write to the storage buffer

        self.queue.write_buffer(
            &self.sprites_buffer,
            0,
            to_bytes(
                graphics,
                graphics.len() * std::mem::size_of::<GraphicsState>(),
            ),
        );

        match self.render(sprite_order) {
            Ok(_) => {}
            // Reconfigure the surface if it's lost or outdated
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => self.resize(self.size),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("OutOfMemory");
                control_flow.exit();
            }

            // This happens when the a frame takes too long to present
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface timeout")
            }
        }

        self.last_time = Instant::now();
    }
}
