use rash_render::Renderer;

use crate::{App, RuntimeConfig};

// I'm doing an elm-style architecture so this app doesn't delve into spaghetti code

pub enum Message {
    OpenProject,
    UpdateConfig(RuntimeConfig),
}

impl App {
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::OpenProject => {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("Scratch Project", &["sb3"])
                    .set_title("Open Project")
                    .pick_file()
                else {
                    return;
                };

                let mut vm = match rash_loader_sb3::load_from_path(&path) {
                    Ok(vm) => vm,
                    Err(err) => {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                };

                let renderer = Renderer::new_offscreen(
                    self.target.size(),
                    &self.wgpu.device,
                    &self.wgpu.queue,
                    &vm.sprite_load_info,
                    &mut vm.costumes,
                );

                self.vm = vm;
                self.renderer = renderer;
            }
            Message::UpdateConfig(new_config) => {
                self.config = new_config;
            }
        }
    }
}
