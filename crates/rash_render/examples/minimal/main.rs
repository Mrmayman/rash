use std::{path::PathBuf, sync::Arc};

use rash_render::{Renderer, WindowSize};
use rash_vm::Runtime;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const HELP_MSG: &str = r"Rash renderer minimal example
Usage: cargo run -p rash_render --example minimal -- path/to/project.sb3

Commands:
    --help: Prints this help screen";

fn main() {
    let path = if let Some(arg) = std::env::args().nth(1) {
        if arg == "--help" {
            println!("{HELP_MSG}");
            return;
        }
        PathBuf::from(arg)
    } else {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Scratch Project", &["sb3"])
            .set_title("Open Project")
            .pick_file()
        else {
            return;
        };
        path
    };

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Rash renderer minimal example")
            .build(&event_loop)
            .unwrap(),
    );

    println!("Loading project from {}", path.display());
    let vm = rash_loader_sb3::load_from_path(&path).unwrap();
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

struct App {
    renderer: Renderer,
    vm: Runtime,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl App {
    async fn new(mut vm: Runtime, window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        let size = window.inner_size();
        let renderer = Renderer::new(
            WindowSize::new(size.width, size.height),
            &surface,
            &adapter,
            &device,
            &queue,
            &vm.sprite_load_info,
            &mut vm.costumes,
        );

        Ok(Self {
            renderer,
            vm,
            window,
            surface,
            device,
            queue,
        })
    }

    fn tick(&mut self, event: Event<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == self.window.id() => {
                self.vm.update(&mut self.renderer.state);
                self.window.request_redraw();
                self.renderer.render(
                    &self.vm.sprite_order,
                    &self.device,
                    &self.queue,
                    &self.surface,
                );
                std::thread::sleep(std::time::Duration::from_millis(33));
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == self.window.id() => self.renderer.resize(
                WindowSize::new(size.width, size.height),
                &self.device,
                &self.queue,
                &self.surface,
            ),
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state,
                        button: winit::event::MouseButton::Left,
                        ..
                    },
                window_id,
            } if window_id == self.window.id() => self
                .renderer
                .state
                .on_left_click(matches!(state, winit::event::ElementState::Pressed)),
            _ => {}
        }
    }
}
