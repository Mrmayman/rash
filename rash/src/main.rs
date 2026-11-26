use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::{egui, egui_wgpu, wgpu};
use rash_core::RunState;
use rash_render::{RenderTarget, Renderer, WindowSize};
use rash_vm::{ProjectBuilder, Runtime, SpriteBuilder, runtime::Script};

mod update;
mod view;

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

    if std::env::var("RASH_PRINT_FUNCTIONS").is_ok() {
        rash_vm::print_function_addresses();
    }

    println!("Loading project from {}", path.display());
    let vm = match rash_loader_sb3::load_from_path(&path) {
        Ok(vm) => vm,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    // TODO: read from project (turbowarp comment)
    let config = RuntimeConfig {
        render_at_high_resolution: false,
        project_resolution: [480, 360],
        fps_cap: 30,
        turbo_mode: false,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 720.0]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration::default(),
        ..Default::default()
    };
    eframe::run_native(
        "Rash",
        options,
        Box::new(move |creation_context| Ok(Box::new(App::new(config, vm, creation_context)))),
    )
    .unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RuntimeConfig {
    /// # `true`
    ///
    /// The `project_resolution` is only taken as a guidance for aspect ratio.
    /// The project is rendered with the pixel density of the window.
    /// Bigger window -> higher resolution (but keeping the aspect ratio constant).
    ///
    /// # `false`
    ///
    /// The project is rendered at the exact resolution specified by `project_resolution`,
    /// and scaled up using nearest-neighbor.
    render_at_high_resolution: bool,
    project_resolution: [u32; 2],
    fps_cap: u32,
    turbo_mode: bool,
}

#[derive(Default)]
struct ImpureState {
    show_about: bool,
}

struct App {
    renderer: Renderer,
    vm: Runtime,
    wgpu: egui_wgpu::RenderState,
    target: RenderTarget,
    texture_id: egui::TextureId,

    config: RuntimeConfig,
    previous_config: RuntimeConfig,

    impure_state: ImpureState,

    next_project_frame: Instant,
    last_ui_frame: Instant,
    last_fps_sample: Instant,
    frames_since_sample: u32,
    fps: f64,
}

impl App {
    fn new(
        config: RuntimeConfig,
        mut vm: Runtime,
        creation_context: &eframe::CreationContext<'_>,
    ) -> Self {
        let wgpu = creation_context
            .wgpu_render_state
            .as_ref()
            .expect("Rash requires eframe's wgpu renderer")
            .clone();
        let target = RenderTarget::new(
            &wgpu.device,
            WindowSize::new(config.project_resolution[0], config.project_resolution[1]),
        );
        let texture_id = register_texture(&wgpu, target.view());
        let renderer = Renderer::new_offscreen(
            target.size(),
            &wgpu.device,
            &wgpu.queue,
            &vm.sprite_load_info,
            &mut vm.costumes,
        );
        let now = Instant::now();

        Self {
            renderer,
            vm,
            wgpu,
            target,
            texture_id,
            config,
            previous_config: config,
            next_project_frame: now,
            last_ui_frame: now,
            last_fps_sample: now,
            frames_since_sample: 0,
            fps: 0.0,
            impure_state: ImpureState::default(),
        }
    }

    fn ensure_target(&mut self, size: WindowSize) -> bool {
        if self.target.size().width == size.width && self.target.size().height == size.height {
            return false;
        }

        let target = RenderTarget::new(&self.wgpu.device, size);
        let texture_id = register_texture(&self.wgpu, target.view());
        self.wgpu.renderer.write().free_texture(&self.texture_id);
        self.target = target;
        self.texture_id = texture_id;
        true
    }

    fn step_project(&mut self, force_render: bool) {
        let now = Instant::now();
        let vsync_dt = now
            .saturating_duration_since(self.last_ui_frame)
            .clamp(Duration::from_millis(1), Duration::from_millis(100));
        self.last_ui_frame = now;

        if self.config != self.previous_config {
            self.previous_config = self.config;
            self.next_project_frame = now;
        }

        let mut ticks = 0u32;

        if self.config.turbo_mode {
            let budget = if self.config.fps_cap == 0 {
                vsync_dt
            } else {
                Duration::from_secs_f64(1.0 / self.config.fps_cap as f64)
            };
            let deadline = now + budget.saturating_sub(Duration::from_millis(1));
            loop {
                self.vm.update(&mut self.renderer.state);
                ticks += 1;
                if Instant::now() >= deadline {
                    break;
                }
            }
            self.next_project_frame = Instant::now();
        } else if self.config.fps_cap == 0 {
            self.vm.update(&mut self.renderer.state);
            ticks = 1;
        } else {
            let dt = Duration::from_secs_f64(1.0 / self.config.fps_cap as f64);
            while now >= self.next_project_frame {
                self.vm.update(&mut self.renderer.state);
                self.next_project_frame += dt;
                ticks += 1;

                if now.saturating_duration_since(self.next_project_frame)
                    > Duration::from_millis(250)
                {
                    self.next_project_frame = now + dt;
                    break;
                }
            }
        }

        if ticks > 0 || force_render {
            self.renderer.render_to_target(
                &self.vm.sprite_order,
                &self.target,
                &self.wgpu.device,
                &self.wgpu.queue,
            );
        }

        self.frames_since_sample += ticks;
        let sample_now = Instant::now();
        let elapsed = sample_now - self.last_fps_sample;
        if elapsed.as_secs_f64() >= 1.0 {
            self.fps = self.frames_since_sample as f64 / elapsed.as_secs_f64();
            self.frames_since_sample = 0;
            self.last_fps_sample = sample_now;
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.wgpu.renderer.write().free_texture(&self.texture_id);
    }
}

fn register_texture(wgpu: &egui_wgpu::RenderState, view: &wgpu::TextureView) -> egui::TextureId {
    wgpu.renderer
        .write()
        .register_native_texture_with_sampler_options(
            &wgpu.device,
            view,
            wgpu::SamplerDescriptor {
                label: Some("Rash Project Texture"),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            },
        )
}

fn run_demo() {
    let mut sprite = SpriteBuilder::new();
    sprite.add_script(Script::new_green_flag(rash_vm::builder::program_pi()));
    let mut builder = ProjectBuilder::new();
    builder.add_sprite(sprite);
    let mut vm = builder.build();
    let mut state = RunState::default();
    while !vm.update(&mut state) {}
    rash_vm::print_memory(&vm.memory);
}
