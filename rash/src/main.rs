use std::{collections::HashMap, path::PathBuf, sync::Arc};

use rash_loader_sb3::ProjectLoader;
use rash_render::{Costume, Renderer, WindowSize};
use rash_vm::{CostumeId, GraphicsState, RunState, Runtime, SpriteData, SpriteId, SpriteLoadData};
use svg_render::SvgRenderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const HELP_MSG: &str = r"Rash: A fast, experimental Scratch runtime
Usage: ./rash path/to/project.sb3

Commands:
    --help: Prints this help screen";

fn main() {
    let path = if let Some(arg) = std::env::args().nth(1) {
        if arg == "--help" {
            println!("{HELP_MSG}");
            return;
        } else {
            PathBuf::from(arg)
        }
    } else {
        let Some(p) = rfd::FileDialog::new()
            .add_filter("Scratch Project", &["sb3"])
            .set_title("Open Project")
            .pick_file()
        else {
            return;
        };
        p
    };

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rash")
            .build(&event_loop)
            .unwrap(),
    );

    let vm = ProjectLoader::new(&path).unwrap().build().unwrap();
    let mut app = pollster::block_on(App::new(vm, window)).unwrap();

    event_loop
        .run(|event, control_flow| match &event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => control_flow.exit(),
            _ => app.tick(event),
        })
        .unwrap();
}

pub struct App {
    renderer: Renderer,
    costumes: HashMap<CostumeId, Costume>,
    state: RunState,
    vm: Runtime,
    window: Arc<Window>,

    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl App {
    pub async fn new(vm: Runtime, window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY,
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .unwrap();

        let window_size = window.inner_size();
        let renderer = Renderer::new(
            WindowSize {
                width: window_size.width,
                height: window_size.height,
            },
            vm.sprite_load_info.len(),
            &surface,
            &adapter,
            &device,
        )
        .await;

        let svg_renderer = SvgRenderer::new();

        let costumes: anyhow::Result<HashMap<CostumeId, Costume>> = vm
            .costume_data
            .iter()
            .map(|(id, costume)| {
                if costume.is_svg
                    && let Ok(svg_text) = String::from_utf8(costume.bytes.clone())
                {
                    let img = svg_renderer.render(&svg_text)?;

                    return Ok((
                        *id,
                        Costume::from_image(
                            costume,
                            &device,
                            &queue,
                            &img,
                            &renderer.sampler,
                            &renderer.costume_layout,
                        ),
                    ));
                }

                Costume::from_bytes(
                    costume,
                    &device,
                    &queue,
                    &renderer.sampler,
                    &renderer.costume_layout,
                )
                .map(|n| (*id, n))
                .map_err(|err| err.into())
            })
            .collect();
        let costumes = costumes?;

        let sprites = vm
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
            state: RunState { sprites },
            costumes,
            vm,
            surface,
            device,
            queue,
        })
    }

    pub fn tick(&mut self, event: Event<()>) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => match event {
                WindowEvent::RedrawRequested => {
                    let _exited = self.vm.update(&mut self.state);

                    let mut graphics_state: Vec<(&SpriteId, &SpriteData)> =
                        self.state.sprites.iter().collect();
                    graphics_state.sort_by_key(|n| n.0);

                    let graphics_state: Vec<GraphicsState> = graphics_state
                        .into_iter()
                        .map(|(_, data)| data.graphics)
                        .collect();

                    self.window.request_redraw();

                    self.renderer.render(
                        &graphics_state,
                        &self.vm.sprite_order,
                        &self.costumes,
                        &self.device,
                        &self.queue,
                        &self.surface,
                    );
                }
                WindowEvent::Resized(s) => {
                    self.resize(s);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn resize(&mut self, s: &winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(
            WindowSize {
                width: s.width,
                height: s.height,
            },
            &self.device,
            &self.queue,
            &self.surface,
        );
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
