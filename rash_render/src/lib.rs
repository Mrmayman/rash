use std::{collections::HashMap, sync::Arc};

use rash_vm::{CostumeId, GraphicsState, RunState, Runtime, SpriteData, SpriteId, SpriteLoadData};
use renderer::{texture::Costume, InnerRenderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

mod renderer;
mod svg_to_png;

pub struct Renderer<'a> {
    renderer: InnerRenderer<'a>,
    #[allow(unused)]
    window: Arc<Window>,
    event_loop: EventLoop<()>,
    costumes: HashMap<CostumeId, Costume>,
    graphics: RunState,
    vm: Runtime,
}

impl Renderer<'_> {
    pub async fn new(title: &str, num_sprites: usize, vm: Runtime) -> anyhow::Result<Self> {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );

        let renderer = InnerRenderer::new(window.clone(), num_sprites).await;

        let mut font_database = usvg_text_layout::fontdb::Database::new();
        font_database.load_system_fonts();

        let costumes: anyhow::Result<HashMap<CostumeId, Costume>> = vm
            .costume_data
            .iter()
            .map(|(id, costume)| {
                if costume.is_svg {
                    if let Ok(svg_text) = String::from_utf8(costume.bytes.clone()) {
                        let img = svg_to_png::render(&svg_text, &font_database)?;

                        return Ok((
                            *id,
                            Costume::from_image(
                                &costume,
                                &renderer.device,
                                &renderer.queue,
                                &img,
                                &renderer.sampler,
                                &renderer.costume_layout,
                            ),
                        ));
                    }
                }

                Costume::from_bytes(
                    &costume,
                    &renderer.device,
                    &renderer.queue,
                    &renderer.sampler,
                    &renderer.costume_layout,
                )
                .map(|n| (*id, n))
                .map_err(|err| err.into())
            })
            .collect();
        let costumes = costumes?;

        let graphics = vm
            .sprite_load_info
            .iter()
            .map(|(id, sprite_info)| {
                let costume = costumes.get(&sprite_info.costume).unwrap();
                let graphics = graphics(sprite_info, costume);
                (*id, SpriteData { graphics })
            })
            .collect();

        Ok(Self {
            renderer,
            window,
            event_loop,
            graphics: RunState { graphics },
            costumes,
            vm,
        })
    }

    pub fn run(mut self) {
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.renderer.window.id() => match event {
                    WindowEvent::RedrawRequested => {
                        let _exited = self.vm.update(&mut self.graphics);

                        let mut graphics_state: Vec<(&SpriteId, &SpriteData)> =
                            self.graphics.graphics.iter().collect();
                        graphics_state.sort_by_key(|n| n.0);

                        let graphics_state: Vec<GraphicsState> = graphics_state
                            .into_iter()
                            .map(|(_, data)| data.graphics)
                            .collect();

                        self.renderer.render(
                            control_flow,
                            &graphics_state,
                            &&self.vm.sprite_order,
                            &self.costumes,
                        );
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => control_flow.exit(),
                    _ => {}
                },
                _ => {}
            })
            .unwrap();
    }
}

fn graphics(sprite_info: &SpriteLoadData, costume_info: &Costume) -> GraphicsState {
    GraphicsState {
        x: sprite_info.x as f32,
        y: sprite_info.y as f32,
        texture_width: costume_info.texture_width as f32,
        texture_height: costume_info.texture_height as f32,
        size: sprite_info.size as f32,
        current_costume: sprite_info.costume,
        center_x: costume_info.rotation_center_x as f32,
        center_y: costume_info.rotation_center_y as f32,
    }
}

// Code taken from tutorial: https://sotrh.github.io/learn-wgpu/
