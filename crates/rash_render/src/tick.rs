use rash_core::{ShaderState, SpriteId};

use crate::{RenderTarget, WindowSize};

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
            let config = self.config.as_mut().expect("surface renderer required");
            self.window_size = new_size;
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(device, config);

            self.global_state.resolution = [new_size.width as f32, new_size.height as f32];
            self.update_global_state(queue);
        }
    }

    fn update_global_state(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.global_buffer,
            0,
            bytemuck::bytes_of(&self.global_state),
        );
    }

    fn render_to_view(
        &mut self,
        sprite_order: &[SpriteId],
        graphics: &[ShaderState],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) {
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
                let costume = self.textures.get(costume_id.0 as usize).unwrap();
                render_pass.set_bind_group(1, &costume.bind_group, &[]);

                let i = i.0 as u32 * 6;
                render_pass.draw(i..(i + 6), 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render_to_target(
        &mut self,
        sprite_order: &[SpriteId],
        target: &RenderTarget,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let size = target.size();
        if self.global_state.resolution != [size.width as f32, size.height as f32] {
            self.global_state.resolution = [size.width as f32, size.height as f32];
            self.update_global_state(queue);
        }

        let graphics: Vec<ShaderState> = self.state.generate_shader_buffer();
        queue.write_buffer(&self.sprites_buffer, 0, bytemuck::cast_slice(&graphics));
        self.render_to_view(sprite_order, &graphics, device, queue, target.view());
    }

    pub fn render(
        &mut self,
        sprite_order: &[SpriteId],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::Surface,
    ) {
        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.resize(self.window_size, device, queue, surface);
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => panic!("Surface lost"),
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let graphics: Vec<ShaderState> = self.state.generate_shader_buffer();
        queue.write_buffer(&self.sprites_buffer, 0, bytemuck::cast_slice(&graphics));
        self.render_to_view(sprite_order, &graphics, device, queue, &view);
        queue.present(output);
    }
}
