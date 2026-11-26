use eframe::egui::{self, RichText};
use rash_render::WindowSize;

use crate::{App, update::Message};

impl eframe::App for App {
    fn logic(&mut self, _: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();

        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new()
                .config(
                    egui::menu::MenuConfig::new()
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                )
                .ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open…").clicked() {
                            self.update(Message::OpenProject);
                        }
                    });
                    ui.menu_button("Project", |ui| {
                        self.menu_project(ui);
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About Rash").clicked() {
                            self.impure_state.show_about = true;
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{:.1} FPS", self.fps));
                    });
                });
        });

        egui::Window::new(RichText::new("About Rash").size(16.0))
            .open(&mut self.impure_state.show_about)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Rash").size(28.0).heading());
                    ui.label(
                        RichText::new("A fast compiler and runtime for Scratch")
                            .size(16.0)
                            .heading(),
                    );
                    ui.add_space(4.0);
                    ui.label(RichText::new("Built on Rust, wgpu, egui and cranelift").size(12.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Proudly free and open source under the GNU GPLv3")
                            .size(12.0),
                    );
                    ui.add_space(4.0);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let available = ui.available_rect_before_wrap();
            let project_size = aspect_fit(available.size(), self.config.project_resolution);
            let rect = egui::Rect::from_center_size(available.center(), project_size);
            let response = ui.allocate_rect(rect, egui::Sense::click());
            let target_size = if self.config.render_at_high_resolution {
                WindowSize::new(
                    (rect.width() * ctx.pixels_per_point()).round().max(1.0) as u32,
                    (rect.height() * ctx.pixels_per_point()).round().max(1.0) as u32,
                )
            } else {
                WindowSize::new(
                    self.config.project_resolution[0],
                    self.config.project_resolution[1],
                )
            };
            let target_changed = self.ensure_target(target_size);

            self.renderer
                .state
                .on_left_click(response.is_pointer_button_down_on());
            self.step_project(target_changed);

            ui.painter().image(
                self.texture_id,
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        });
    }

    fn on_exit(&mut self) {
        if std::env::var("RASH_PRINT_MEMORY").is_ok() {
            rash_vm::print_memory(&self.vm.memory);
        }
    }
}

impl App {
    fn menu_project(&mut self, ui: &mut egui::Ui) {
        let default_widgets = egui::Visuals::dark().widgets;
        ui.visuals_mut().widgets.inactive = default_widgets.inactive;
        ui.visuals_mut().widgets.hovered = default_widgets.hovered;
        ui.visuals_mut().widgets.active = default_widgets.active;
        ui.visuals_mut().button_frame = true;

        let mut new_cfg = self.config;

        ui.horizontal(|ui| {
            if ui.button(" Restart ").clicked() {
                self.update(Message::RestartVm);
            }
            // if ui.button(" Pause ").clicked() {
            //     todo!()
            // }
            // if ui.button(" Stop ").clicked() {
            //     todo!()
            // }
        });

        ui.checkbox(&mut new_cfg.turbo_mode, "Turbo Mode");
        ui.checkbox(&mut new_cfg.render_at_high_resolution, "High Quality Pen");

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("FPS Limit:");
            ui.add(egui::DragValue::new(&mut new_cfg.fps_cap).range(0..=1000));
        });
        ui.label(
            RichText::new("Default: 30 fps\n0 matches your monitor's refresh rate").size(10.0),
        );

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Viewport Resolution:");
            if ui.button("Reset").clicked() {
                new_cfg.project_resolution = [480, 360];
            }
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Width:").size(10.0));
            ui.add(egui::DragValue::new(&mut new_cfg.project_resolution[0]).range(20..=8192));
            ui.label(RichText::new("Height:").size(10.0));
            ui.add(egui::DragValue::new(&mut new_cfg.project_resolution[1]).range(20..=8192));
        });

        if new_cfg != self.config {
            self.update(Message::UpdateConfig(new_cfg));
        }
    }
}

fn aspect_fit(available: egui::Vec2, project_resolution: [u32; 2]) -> egui::Vec2 {
    if available.x <= 0.0 || available.y <= 0.0 {
        return egui::Vec2::ZERO;
    }
    let aspect_ratio = project_resolution[0].max(1) as f32 / project_resolution[1].max(1) as f32;
    if available.x / available.y > aspect_ratio {
        egui::vec2(available.y * aspect_ratio, available.y)
    } else {
        egui::vec2(available.x, available.x / aspect_ratio)
    }
}
