use std::time::Instant;

use rash_vm::{GraphicsState, SpriteId};

use super::to_bytes;
use crate::WindowSize;

use super::Renderer;

impl Renderer {
    pub fn resize(
        &mut self,
        new_size: WindowSize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::Surface,
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(device, &self.config);

            self.global_state.resolution = [new_size.width as f32, new_size.height as f32];
            self.update_global_state(queue);
        }
    }

    fn update_global_state(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.global_buffer, 0, to_bytes(&[self.global_state]));
    }

    fn render_inner(
        &mut self,
        sprite_order: &[SpriteId],
        graphics: &[GraphicsState],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::Surface,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                    depth_slice: None,
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            for i in sprite_order {
                let state = graphics.get(i.0 as usize).unwrap();
                if state.shown == 0 {
                    continue;
                }

                let costume_id = state.current_costume;
                let costume = self.costumes.get(&costume_id).unwrap();
                render_pass.set_bind_group(1, &costume.bind_group, &[]);

                let i = i.0 as u32 * 6;
                render_pass.draw(i..(i + 6), 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn render(
        &mut self,
        sprite_order: &[SpriteId],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::Surface,
    ) {
        let mut graphics: Vec<(_, _)> = self.state.sprites.iter().collect();
        graphics.sort_by_key(|n| n.0);
        let graphics: Vec<GraphicsState> = graphics.into_iter().map(|n| n.1.graphics).collect();

        queue.write_buffer(&self.sprites_buffer, 0, to_bytes(&graphics));

        match self.render_inner(sprite_order, &graphics, device, queue, surface) {
            Ok(()) => {}
            // Reconfigure the surface if it's lost or outdated
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.resize(self.window_size, device, queue, surface);
            }
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("[error] Graphics: Out Of Memory");
                return;
            }
            // This happens when the a frame takes too long to present
            Err(err) => {
                eprintln!("[error] {err}");
            }
        }

        let frametime = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        let target_frametime = 1.0 / 30.0;
        if frametime < target_frametime {
            std::thread::sleep(std::time::Duration::from_secs_f64(
                target_frametime - frametime,
            ));
        }
    }
}
