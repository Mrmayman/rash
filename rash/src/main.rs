use std::{collections::HashMap, path::PathBuf, sync::Arc};

use rash_loader_sb3::ProjectLoader;
use rash_render::{Renderer, WindowSize};
use rash_vm::{
    MEMORY, ProjectBuilder, RunState, Runtime, ScratchBlock, ScratchObject, SpriteBuilder,
    SpriteData, SpriteId, runtime::Script,
};
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
        } else if arg == "--demo" {
            run_demo();
            print_memory();
            return;
        }
        PathBuf::from(arg)
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

    // rash_vm::print_function_addresses();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rash")
            .build(&event_loop)
            .unwrap(),
    );

    let vm = match ProjectLoader::new(&path).unwrap().build() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
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
                memory_hints: wgpu::MemoryHints::default(),
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
            &vm,
            &surface,
            &adapter,
            &device,
            &queue,
        )
        .await;

        Ok(Self {
            renderer,
            window,
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
                    let _exited = self.vm.update(&mut self.renderer.state);

                    self.window.request_redraw();

                    self.renderer.render(
                        &self.vm.sprite_order,
                        &self.device,
                        &self.queue,
                        &self.surface,
                    );
                }
                WindowEvent::Resized(s) => {
                    self.resize(*s);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn resize(&mut self, s: winit::dpi::PhysicalSize<u32>) {
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

fn run_demo() {
    // TODO: All memory is a global variable
    // I *will* refactor this in the future
    let memory = MEMORY.lock().unwrap();

    let mut sprite = SpriteBuilder::new(SpriteId(0));
    sprite.add_script(
        &Script::new_green_flag(vec![
            ScratchBlock::Log("Hello World".into()),
            ScratchBlock::Log(ScratchBlock::OpBNot(true.into()).into()),
        ]),
        &memory,
    );
    let mut builder = ProjectBuilder::new();
    builder.add_sprite(sprite);
    let mut vm = builder.build();
    let mut state = RunState {
        // We won't do any graphics operations here
        sprites: HashMap::from([(SpriteId(0), SpriteData::default())]),
    };

    while !vm.update(&mut state) {}
}

fn print_memory() {
    let lock = rash_vm::MEMORY.lock().unwrap();

    println!("MEMORY: {:X}", lock.as_ptr() as usize);

    // Only print the changed values that aren't zero.
    let print_until_idx = lock
        .iter()
        .enumerate()
        .rev()
        .find(|(_, n)| !matches!(**n, ScratchObject::Number(0.0)))
        .map(|(i, _)| i);
    if let Some(print_until_idx) = print_until_idx {
        for (i, obj) in lock.iter().enumerate().take(print_until_idx + 1) {
            println!("{i}: {obj:?}");
        }
    }
    println!("...: {:?}", ScratchObject::Number(0.0));
}
