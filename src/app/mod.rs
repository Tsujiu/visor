//! Camada de coordenação do aplicativo, ciclo de vida e eventos de frame.

pub mod actions;
pub mod shortcuts;
pub mod state;

use eframe::egui;

pub use state::VisorApp;

use crate::ui::theme::BG;
use crate::ui::{
    render_association_banner, render_batch_window, render_canvas, render_color_picker_window,
    render_filmstrip, render_formats_window, render_info_panel, render_settings_window,
    render_status, render_toolbar, render_update_banner, render_update_dialog,
    render_whats_new_window,
};

impl eframe::App for VisorApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.pump(ui.ctx());
        self.take_drops(ui.ctx());
        shortcuts::handle_keys(self, ui);
        self.advance_animation(ui.ctx());

        // Detecta quando o aplicativo retorna ao primeiro plano após abrir as Configurações do Windows
        let is_focused = ui.ctx().input(|i| i.viewport().focused.unwrap_or(true));
        if is_focused && !self.was_window_focused {
            if self.was_settings_opened_for_assoc {
                self.was_settings_opened_for_assoc = false;
                self.trigger_association_check();
            }
        }
        self.was_window_focused = is_focused;

        if self.show_info {
            let total_width = ui.available_width();
            let panel_width = (total_width * 0.32)
                .clamp(260.0, 320.0)
                .min((total_width - 60.0).max(180.0));
            egui::Panel::right("info")
                .exact_size(panel_width)
                .frame(
                    egui::Frame::NONE
                        .fill(crate::ui::theme::ThemePalette::dark().surface_panel)
                        .stroke(crate::ui::theme::ThemePalette::dark().border_subtle),
                )
                .show(ui, |ui| render_info_panel(self, ui));
        }
        let palette = crate::ui::theme::ThemePalette::dark();
        egui::Panel::bottom("status")
            .exact_size(crate::ui::theme::ThemeDimensions::STATUS_BAR_HEIGHT)
            .frame(
                egui::Frame::NONE
                    .fill(palette.surface_panel)
                    .stroke(palette.border_subtle)
                    .inner_margin(egui::Margin::symmetric(
                        crate::ui::theme::ThemeSpacing::SM as i8,
                        2,
                    )),
            )
            .show(ui, |ui| render_status(self, ui));
        if self.show_filmstrip && !self.files.is_empty() {
            let film_height = if ui.available_width() < 480.0 {
                crate::ui::theme::ThemeDimensions::FILMSTRIP_HEIGHT_COMPACT
            } else {
                crate::ui::theme::ThemeDimensions::FILMSTRIP_HEIGHT
            };
            egui::Panel::bottom("film")
                .exact_size(film_height)
                .frame(
                    egui::Frame::NONE
                        .fill(crate::ui::theme::ThemePalette::dark().surface_panel)
                        .stroke(crate::ui::theme::ThemePalette::dark().border_subtle),
                )
                .show(ui, |ui| render_filmstrip(self, ui));
        }
        if self.has_active_update_banner() {
            egui::Panel::top("update_banner")
                .frame(
                    egui::Frame::NONE
                        .fill(crate::ui::theme::ThemePalette::dark().surface_panel)
                        .stroke(crate::ui::theme::ThemePalette::dark().border_subtle)
                        .inner_margin(egui::Margin::symmetric(
                            crate::ui::theme::ThemeSpacing::SM as i8,
                            6,
                        )),
                )
                .show(ui, |ui| render_update_banner(self, ui));
        }
        if self.show_association_banner {
            egui::Panel::top("association_banner")
                .frame(
                    egui::Frame::NONE
                        .fill(crate::ui::theme::ThemePalette::dark().surface_panel)
                        .stroke(crate::ui::theme::ThemePalette::dark().border_subtle)
                        .inner_margin(egui::Margin::symmetric(
                            crate::ui::theme::ThemeSpacing::SM as i8,
                            6,
                        )),
                )
                .show(ui, |ui| render_association_banner(self, ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG))
            .show(ui, |ui| {
                render_canvas(self, ui);
                render_toolbar(self, ui);
            });

        render_formats_window(self, ui.ctx());
        render_batch_window(self, ui.ctx());
        render_settings_window(self, ui.ctx());
        render_update_dialog(self, ui.ctx());
        render_whats_new_window(self, ui.ctx());
        render_color_picker_window(self, ui.ctx());

        if self.want_open {
            self.want_open = false;
            self.browse(frame);
        }
        if self.want_export {
            self.want_export = false;
            self.export_current(frame);
        }
        if self.want_batch_files {
            self.want_batch_files = false;
            self.pick_batch_files(frame);
        }
        if self.want_batch_dir {
            self.want_batch_dir = false;
            self.run_batch(frame);
        }
        self.pump(ui.ctx());
        self.sync_title(ui.ctx());
        self.handle_gallery_capture(ui);
    }
}

impl VisorApp {
    fn handle_gallery_capture(&mut self, ui: &mut egui::Ui) {
        let Some(state) = self.capture_state else {
            return;
        };
        let Some(dir) = self.capture_dir.clone() else {
            return;
        };

        if self.capture_wait > 0 {
            self.capture_wait -= 1;
            ui.ctx().request_repaint();
            return;
        }

        match state {
            0 => {
                // Tier Full Clean (960x640, sem info, sem filmstrip)
                self.show_info = false;
                self.files.truncate(1);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        960.0, 640.0,
                    )));
                self.capture_state = Some(1);
                self.capture_wait = 15;
            }
            1 => {
                // Captura Full Clean
                let path = dir.join("toolbar_full_clean.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(2);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            2 => {
                // Tier Full Panels (960x640, com info e filmstrip)
                self.show_info = true;
                if self.files.len() == 1 {
                    self.files.push(self.files[0].clone());
                }
                self.capture_state = Some(3);
                self.capture_wait = 15;
            }
            3 => {
                // Captura Full Panels
                let path = dir.join("toolbar_full_panels.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(4);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            4 => {
                // Tier Medium Clean (640x480, sem info, sem filmstrip)
                self.show_info = false;
                self.files.truncate(1);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        640.0, 480.0,
                    )));
                self.capture_state = Some(5);
                self.capture_wait = 15;
            }
            5 => {
                // Captura Medium Clean
                let path = dir.join("toolbar_medium_clean.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(6);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            6 => {
                // Tier Medium Panels (640x480, com info e filmstrip)
                self.show_info = true;
                if self.files.len() == 1 {
                    self.files.push(self.files[0].clone());
                }
                self.capture_state = Some(7);
                self.capture_wait = 15;
            }
            7 => {
                // Captura Medium Panels
                let path = dir.join("toolbar_medium_panels.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(8);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            8 => {
                // Tier Compact Clean (460x380, sem info, sem filmstrip)
                self.show_info = false;
                self.files.truncate(1);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        460.0, 380.0,
                    )));
                self.capture_state = Some(9);
                self.capture_wait = 15;
            }
            9 => {
                // Captura Compact Clean
                let path = dir.join("toolbar_compact_clean.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(10);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            10 => {
                // Tier Compact Panels (460x380, com filmstrip inferior)
                self.show_info = false;
                if self.files.len() == 1 {
                    self.files.push(self.files[0].clone());
                }
                self.capture_state = Some(11);
                self.capture_wait = 15;
            }
            11 => {
                // Captura Compact Panels
                let path = dir.join("toolbar_compact_panels.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(12);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            12 => {
                // Tier Minimal Clean (340x300, sem info, sem filmstrip)
                self.show_info = false;
                self.files.truncate(1);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        340.0, 300.0,
                    )));
                self.capture_state = Some(13);
                self.capture_wait = 15;
            }
            13 => {
                // Captura Minimal Clean
                let path = dir.join("toolbar_minimal_clean.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(14);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            14 => {
                // Tier Minimal Panels (340x300, com filmstrip inferior)
                self.show_info = false;
                if self.files.len() == 1 {
                    self.files.push(self.files[0].clone());
                }
                self.capture_state = Some(15);
                self.capture_wait = 15;
            }
            15 => {
                // Captura Minimal Panels
                let path = dir.join("toolbar_minimal_panels.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(16);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            16 => {
                // Fundo Light Checkerboard (960x640)
                self.show_info = false;
                self.files.truncate(1);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        960.0, 640.0,
                    )));
                self.background = crate::app::state::CanvasBackground::LightChecker;
                self.view.set_fit();
                self.capture_state = Some(17);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            17 => {
                // Captura Light Checkerboard
                let path = dir.join("canvas_bg_light_checker.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(18);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            18 => {
                // Fundo Solid Black
                self.background = crate::app::state::CanvasBackground::Black;
                self.capture_state = Some(19);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            19 => {
                // Captura Solid Black
                let path = dir.join("canvas_bg_black.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(20);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            20 => {
                // Fundo Solid White
                self.background = crate::app::state::CanvasBackground::White;
                self.capture_state = Some(21);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            21 => {
                // Captura Solid White
                let path = dir.join("canvas_bg_white.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(22);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            22 => {
                // Fundo Dark Checkerboard (Padrão)
                self.background = crate::app::state::CanvasBackground::DarkChecker;
                self.capture_state = Some(23);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            23 => {
                // Captura Dark Checkerboard
                let path = dir.join("canvas_bg_dark_checker.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(24);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            24 => {
                // Zoom 400% Nearest (Pixel Perfect)
                self.interpolation = crate::app::state::InterpolationMode::Nearest;
                self.view.fit = false;
                self.view.zoom = 4.0;
                self.view.offset = egui::Vec2::ZERO;
                self.capture_state = Some(25);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            25 => {
                // Captura Pixel Perfect (Nearest 400%)
                let path = dir.join("canvas_pixel_perfect_nearest.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(26);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            26 => {
                // Zoom 400% Linear (Bilinear)
                self.interpolation = crate::app::state::InterpolationMode::Linear;
                self.capture_state = Some(27);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            27 => {
                // Captura Linear (Bilinear 400%)
                let path = dir.join("canvas_pixel_linear.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(28);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            28 => {
                // Empty state: limpa picture
                self.picture = None;
                self.view.set_fit();
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        960.0, 640.0,
                    )));
                self.capture_state = Some(29);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            29 => {
                // Captura Empty state
                let path = dir.join("canvas_empty_state.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(30);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            30 => {
                // Filmstrip com múltiplas imagens e seleção centralizada (960x640)
                let sample = std::path::PathBuf::from("assets/moldura.png");
                self.files = vec![
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                ];
                self.index = 3;
                self.show_filmstrip = true;
                self.show_info = false;
                self.open_input(sample);
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        960.0, 640.0,
                    )));
                self.capture_state = Some(31);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            31 => {
                // Captura Filmstrip Aberto com Múltiplas Imagens
                let path = dir.join("filmstrip_open_multiple.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(32);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            32 => {
                // Filmstrip Oculto (Retrátil - ganho total de canvas)
                self.show_filmstrip = false;
                self.capture_state = Some(33);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            33 => {
                // Captura Canvas com Filmstrip Oculto
                let path = dir.join("filmstrip_hidden_canvas_gain.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(34);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            34 => {
                // Filmstrip com Apenas Uma Imagem
                self.show_filmstrip = true;
                self.files.truncate(1);
                self.index = 0;
                self.capture_state = Some(35);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            35 => {
                // Captura Filmstrip com Uma Imagem
                let path = dir.join("filmstrip_single_image.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(36);
                self.capture_wait = 15;
                ui.ctx().request_repaint();
            }
            36 => {
                // Filmstrip Compacto (< 480px)
                let sample = std::path::PathBuf::from("assets/moldura.png");
                self.files = vec![
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                    sample.clone(),
                ];
                self.index = 1;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        420.0, 360.0,
                    )));
                self.capture_state = Some(37);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            37 => {
                // Captura Filmstrip Compacto
                let path = dir.join("filmstrip_compact_width.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(38);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            38 => {
                // Preparar Painel de Informações Completo
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1024.0, 720.0,
                    )));
                self.show_info = true;
                self.show_filmstrip = false;
                self.show_formats = false;
                self.show_batch = false;
                self.capture_state = Some(39);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            39 => {
                // Captura Painel de Informações
                let path = dir.join("info_panel_complete.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(40);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            40 => {
                // Preparar Janela de Formatos Suportados
                self.show_info = false;
                self.show_formats = true;
                self.show_batch = false;
                self.format_search = String::new();
                self.capture_state = Some(41);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            41 => {
                // Captura Janela de Formatos Suportados
                let path = dir.join("dialogs_formats_window.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(42);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            42 => {
                // Preparar Janela de Conversão em Lote
                self.show_formats = false;
                self.show_batch = true;
                let sample = std::path::PathBuf::from("assets/moldura.png");
                self.batch_files = vec![
                    sample.clone(),
                    std::path::PathBuf::from("paisagem_serra_gaucha.jpg"),
                    std::path::PathBuf::from("foto_estudio_macro.png"),
                ];
                self.batch_kind = visor::ExportKind::Webp;
                self.notice = Some("Lote: 3 arquivos prontos para conversão em WebP.".to_owned());
                self.capture_state = Some(43);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            43 => {
                // Captura Janela de Conversão em Lote
                let path = dir.join("dialogs_batch_window.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(44);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            44 => {
                // Passo 6: Resolução Ampla (Wide: 1024x640)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1024.0, 640.0,
                    )));
                self.show_info = false;
                self.show_filmstrip = false;
                self.show_formats = false;
                self.show_batch = false;
                self.files = vec![
                    std::path::PathBuf::from("assets/moldura.png"),
                    std::path::PathBuf::from("paisagem_serra_gaucha.jpg"),
                    std::path::PathBuf::from("foto_estudio_macro.png"),
                ];
                self.index = 0;
                self.capture_state = Some(45);
                self.capture_wait = 20;
                ui.ctx().request_repaint();
            }
            45 => {
                // Captura Status Bar Ampla
                let path = dir.join("status_wide_resolution.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(46);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            46 => {
                // Passo 6: Resolução Média com Filmstrip (720x520)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        720.0, 520.0,
                    )));
                self.show_filmstrip = true;
                self.capture_state = Some(47);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            47 => {
                // Captura Status Bar Média com Filmstrip
                let path = dir.join("status_medium_resolution.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(48);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            48 => {
                // Passo 6: Resolução Compacta (420x360)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        420.0, 360.0,
                    )));
                self.show_filmstrip = false;
                self.capture_state = Some(49);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            49 => {
                // Captura Status Bar Compacta
                let path = dir.join("status_compact_resolution.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(50);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            50 => {
                // Passo 6: Estado de Carregamento com Spinner (800x500)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        800.0, 500.0,
                    )));
                self.loading = true;
                self.capture_state = Some(51);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            51 => {
                // Captura Estado de Carregamento com Microinteração
                let path = dir.join("status_loading_state.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(52);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            52 => {
                // Passo 6: Coexistência com Painéis Abertos (Info + Filmstrip: 1080x700)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1080.0, 700.0,
                    )));
                self.loading = false;
                self.show_info = true;
                self.show_filmstrip = true;
                self.capture_state = Some(53);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            53 => {
                // Captura Coexistência com Painéis Abertos
                let path = dir.join("status_full_panels_coexistence.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(54);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            54 => {
                // Concluído com sucesso!
                println!(
                    "Todas as capturas do Passo 6 (Status Bar e Microinterações) finalizadas com sucesso!"
                );
                self.capture_state = None;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            60 => {
                // Validação Final: 1920x1080 (Ampla) com Toolbar, Filmstrip e Status Bar
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1920.0, 1080.0,
                    )));
                self.show_info = false;
                self.show_filmstrip = true;
                self.show_formats = false;
                self.show_batch = false;
                self.files = vec![
                    std::path::PathBuf::from("assets/moldura.png"),
                    std::path::PathBuf::from("paisagem_serra_gaucha.jpg"),
                    std::path::PathBuf::from("foto_estudio_macro.png"),
                ];
                self.index = 0;
                self.capture_state = Some(61);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            61 => {
                // Captura 1920x1080 Ampla
                let path = dir.join("val_1920x1080_full_ampla.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(62);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            62 => {
                // Validação Final: 1280x720 (Convencional) com Painel de Informações e Filmstrip
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1280.0, 720.0,
                    )));
                self.show_info = true;
                self.show_filmstrip = true;
                self.capture_state = Some(63);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            63 => {
                // Captura 1280x720 Convencional
                let path = dir.join("val_1280x720_convencional.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(64);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            64 => {
                // Validação Final: 800x600 (Compacta) com Filmstrip
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        800.0, 600.0,
                    )));
                self.show_info = false;
                self.show_filmstrip = true;
                self.capture_state = Some(65);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            65 => {
                // Captura 800x600 Compacta
                let path = dir.join("val_800x600_compacta.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(66);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            66 => {
                // Validação Final: 640x480 (Pequena) sem Filmstrip
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        640.0, 480.0,
                    )));
                self.show_info = false;
                self.show_filmstrip = false;
                self.capture_state = Some(67);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            67 => {
                // Captura 640x480 Pequena
                let path = dir.join("val_640x480_pequena.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(68);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            68 => {
                // Validação Final: 420x360 (Ultracompacta)
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        420.0, 360.0,
                    )));
                self.show_info = false;
                self.show_filmstrip = false;
                self.capture_state = Some(69);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            69 => {
                // Captura 420x360 Ultracompacta
                let path = dir.join("val_420x360_ultracompacta.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(70);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            70 => {
                // Validação Final: Pixel Perfect / Nearest em Zoom 400%
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        1024.0, 640.0,
                    )));
                self.interpolation = crate::app::state::InterpolationMode::Nearest;
                self.view.zoom = 4.0;
                self.view.fit = false;
                self.capture_state = Some(71);
                self.capture_wait = 25;
                ui.ctx().request_repaint();
            }
            71 => {
                // Captura Pixel Perfect
                let path = dir.join("val_pixel_perfect_art.png");
                ui.ctx().request_screenshot(move |image| {
                    save_screenshot_image(&path, &image);
                });
                self.capture_state = Some(72);
                self.capture_wait = 10;
                ui.ctx().request_repaint();
            }
            72 => {
                // Concluído com sucesso!
                println!("Todas as capturas da validação final concluídas com sucesso!");
                self.capture_state = None;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => {}
        }
    }
}

fn save_screenshot_image(path: &std::path::Path, image: &egui::ColorImage) {
    let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
    for c in &image.pixels {
        rgba.extend_from_slice(&c.to_array());
    }
    let _ = image::save_buffer(
        path,
        &rgba,
        image.width() as u32,
        image.height() as u32,
        image::ExtendedColorType::Rgba8,
    );
    println!("Captura salva: {}", path.display());
}
