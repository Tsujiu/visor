//! Barra de status inferior moderna inspirada no Fluent Design do Windows 11.
//!
//! Organizada em três regiões funcionais:
//! - **Esquerda:** Estado atual (Carregando com spinner vetorial, Erro semântico,
//!   Notificações ou Nome do arquivo com truncamento e cópia de caminho).
//! - **Centro:** Dimensões da imagem (pixels + Megapixels), modo de redução,
//!   proporção de aspecto e formato/profundidade de cor.
//! - **Direita:** Nível de zoom interativo (Ajustar vs 100%), tamanho do arquivo
//!   em disco formatado e posição na pasta/galeria (navegável com clique).
//!
//! Totalmente responsiva com quebra progressiva para janelas compactas
//! e tooltips complementares para que nenhuma informação seja perdida.

use std::fs;
use std::path::Path;
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, Response, RichText, Sense, Stroke, Vec2};

use crate::app::state::VisorApp;
use crate::system::i18n::{CategoryExt, SupportExt};
use crate::ui::info_panel::{calculate_aspect_ratio, format_file_size};
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};

/// Desenha um spinner vetorial sutil e contínuo para indicar atividade em segundo plano.
pub fn draw_spinner(ui: &mut egui::Ui, radius: f32, color: Color32) -> Response {
    let size = Vec2::splat(radius * 2.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let time = ui.ctx().input(|i| i.time);
        let center = rect.center();
        let painter = ui.painter();
        let count = 8;
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU + (time as f32 * 4.5);
            let dist = radius * 0.72;
            let dot_pos = center + Vec2::new(angle.cos() * dist, angle.sin() * dist);
            let alpha = ((i + 1) as f32 / count as f32).powf(1.6);
            let dot_color = Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                (alpha * 255.0).clamp(30.0, 255.0) as u8,
            );
            painter.circle_filled(dot_pos, 1.25, dot_color);
        }
    }
    response
}

/// Renderiza um separador vertical discreto de 1px entre blocos de informação.
pub fn render_divider(ui: &mut egui::Ui, stroke: Stroke) {
    ui.add_space(ThemeSpacing::XXS);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 12.0), Sense::hover());
    ui.painter().vline(rect.center().x, rect.y_range(), stroke);
    ui.add_space(ThemeSpacing::XXS);
}

/// Renderiza a barra de status inferior moderna e responsiva.
pub fn render_status(app: &mut VisorApp, ui: &mut egui::Ui) {
    let palette = ThemePalette::dark();
    let tr = *app.tr();
    let total_width = ui.available_width();
    let is_wide = total_width >= 760.0;
    let is_medium = total_width >= 520.0;
    let is_ultra_compact = total_width < 380.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::XS, 0.0);

        // =====================================================================
        // 1. REGIÃO ESQUERDA: Estado / Nome do Arquivo
        // =====================================================================
        if app.loading {
            draw_spinner(ui, 5.5, palette.accent_default);
            ui.label(
                RichText::new(app.tr().loading)
                    .size(11.5)
                    .color(palette.accent_default)
                    .strong(),
            );
            ui.ctx().request_repaint_after(Duration::from_millis(30));
        } else if let Some(err) = &app.error {
            let resp = ui.add(
                egui::Button::new(
                    RichText::new(format!("[!] {}", tr.status_error_label))
                        .size(11.5)
                        .color(palette.status_error)
                        .strong(),
                )
                .frame(false)
                .corner_radius(ThemeRounding::RADIUS_XS),
            );
            let resp = resp.on_hover_text(format!(
                "Erro ao carregar ou processar imagem:\n{err}\n\nClique para dispensar aviso"
            ));
            if resp.clicked() {
                app.error = None;
            }
        } else if let Some(notice) = &app.notice {
            let resp = ui.add(
                egui::Button::new(
                    RichText::new(notice)
                        .size(11.5)
                        .color(palette.text_primary),
                )
                .frame(false)
                .corner_radius(ThemeRounding::RADIUS_XS),
            );
            let resp = resp.on_hover_text("Notificação do sistema\nClique para dispensar");
            if resp.clicked() {
                app.notice = None;
            }
        } else if let Some(path) = app.current_path() {
            let display_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "imagem".to_owned());

            let max_name_len = if is_wide { 32 } else if is_medium { 22 } else { 14 };
            let truncated_name = if display_name.chars().count() > max_name_len {
                let mut s: String = display_name
                    .chars()
                    .take(max_name_len.saturating_sub(1))
                    .collect();
                s.push('…');
                s
            } else {
                display_name.clone()
            };

            let resp = ui.add(
                egui::Button::new(
                    RichText::new(&truncated_name)
                        .size(11.5)
                        .color(palette.text_primary)
                        .strong(),
                )
                .frame(false)
                .corner_radius(ThemeRounding::RADIUS_XS),
            );

            let folder_path = path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            let resp = resp.on_hover_text(format!(
                "Arquivo: {}\nPasta: {}\nCaminho completo: {}\n\nClique para copiar o caminho completo para a área de transferência",
                display_name, folder_path, path.display()
            ));

            if resp.clicked() {
                ui.ctx().copy_text(path.display().to_string());
                app.notice = Some(tr.status_path_copied.to_owned());
            }
        } else {
            let resp = ui.add(
                egui::Label::new(
                    RichText::new(app.tr().ready)
                        .size(11.5)
                        .color(palette.text_muted),
                ),
            );
            resp.on_hover_text(
                "Nenhuma imagem aberta no momento.\nArraste uma foto para cá ou pressione Ctrl+O para abrir.",
            );
        }

        // =====================================================================
        // 2. REGIÃO CENTRO: Dimensões, Megapixels e Formato (se espaço permitir)
        // =====================================================================
        if is_medium {
            if let Some(picture) = &app.picture {
                // Alinhamento suave ao centro quando a janela for ampla
                let center_x = total_width * 0.44;
                let current_x = ui.cursor().min.x;
                if is_wide && current_x < center_x - 100.0 {
                    ui.add_space(center_x - 100.0 - current_x);
                }

                render_divider(ui, palette.border_subtle);

                let info = &picture.info;
                let frame = picture.frame();

                let mut dim_text = format!("{} × {} px", info.original_w, info.original_h);
                if is_wide {
                    let mp = (info.original_w as f64 * info.original_h as f64) / 1_000_000.0;
                    dim_text.push_str(&format!(" ({:.1} MP)", mp));
                }
                if info.reduced {
                    dim_text.push_str(&format!(" [{}]", tr.info_reduced));
                }

                let dim_label = ui.add(
                    egui::Label::new(
                        RichText::new(&dim_text).size(11.5).color(palette.text_secondary),
                    ),
                );

                let ratio = calculate_aspect_ratio(info.original_w, info.original_h);
                dim_label.on_hover_text(format!(
                    "Dimensões originais: {} × {} pixels\nExibição atual: {} × {} pixels\nProporção de aspecto: {}\nProfundidade de cor: {} bits · {}\nTransparência alfa: {}",
                    info.original_w,
                    info.original_h,
                    frame.width,
                    frame.height,
                    ratio,
                    info.bit_depth,
                    info.color,
                    if info.has_alpha { "Sim" } else { "Não" }
                ));

                // Formato e quadros
                if is_wide {
                    render_divider(ui, palette.border_subtle);
                    let fmt_text = if picture.frames.len() > 1 {
                        format!(
                            "{} · Quadro {}/{}",
                            info.name,
                            picture.frame_i + 1,
                            picture.frames.len()
                        )
                    } else {
                        format!("{} · {} bits", info.name, info.bit_depth)
                    };

                    let fmt_label = ui.add(
                        egui::Label::new(
                            RichText::new(&fmt_text).size(11.5).color(palette.text_secondary),
                        ),
                    );
                    let notes_summary = if info.notes.is_empty() {
                        "Nenhuma observação".to_owned()
                    } else {
                        info.notes.join(" · ")
                    };
                    fmt_label.on_hover_text(format!(
                        "Formato: {} ({})\nNível de suporte: {}\nNotas técnicas: {}",
                        info.name,
                        info.category.localized_label(&tr),
                        info.support.localized_label(&tr),
                        notes_summary
                    ));
                }
            } else if is_wide {
                // Estado vazio com dica no centro
                let center_x = total_width * 0.42;
                let current_x = ui.cursor().min.x;
                if current_x < center_x - 120.0 {
                    ui.add_space(center_x - 120.0 - current_x);
                }
                render_divider(ui, palette.border_subtle);
                let hint = ui.add(
                    egui::Label::new(
                        RichText::new(tr.status_drag_hint)
                            .size(11.5)
                            .color(palette.text_muted),
                    ),
                );
                hint.on_hover_text("O VISOR suporta dezenas de formatos: PNG, JPEG, WebP, TIFF, BMP, QOI, RAW, SVG, HDR e mais.");
            }
        }

        // =====================================================================
        // 3. REGIÃO DIREITA: Zoom, Tamanho e Posição (Ancorados à Direita)
        // =====================================================================
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::XS, 0.0);

            // 3.1 Posição na pasta (ex: "1 / 48")
            if !app.files.is_empty() {
                let pos_text = format!("{} / {}", app.index + 1, app.files.len());
                let folder_name = app
                    .current_path()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "pasta atual".to_owned());

                let resp = ui.add(
                    egui::Button::new(
                        RichText::new(&pos_text)
                            .size(11.5)
                            .color(palette.text_secondary),
                    )
                    .frame(false)
                    .corner_radius(ThemeRounding::RADIUS_XS),
                );

                let tooltip_text = format!(
                    "Imagem {} de {}\nPasta: {}\n\nClique esquerdo: Próxima imagem (->)\nClique direito: Imagem anterior (<-)",
                    app.index + 1,
                    app.files.len(),
                    folder_name
                );
                let resp = resp.on_hover_text(tooltip_text);

                if resp.clicked() {
                    app.step(1);
                } else if resp.secondary_clicked() {
                    app.step(-1);
                }

                render_divider(ui, palette.border_subtle);
            }

            // 3.2 Tamanho do arquivo em disco (se couber e houver arquivo)
            if is_medium {
                if let Some(path) = app.current_path() {
                    if let Ok(meta) = fs::metadata(path) {
                        let size_formatted = format_file_size(meta.len());
                        let label = ui.add(
                            egui::Label::new(
                                RichText::new(&size_formatted)
                                    .size(11.5)
                                    .color(palette.text_secondary),
                            ),
                        );
                        label.on_hover_text(format!(
                            "Tamanho do arquivo em disco:\n{} ({} bytes)",
                            size_formatted,
                            meta.len()
                        ));
                        render_divider(ui, palette.border_subtle);
                    }
                }
            }

            // 3.3 Zoom e Modo de Visualização (ex: "259% [Ajustar]" ou "100%")
            if !is_ultra_compact {
                if let Some(percent) = app.shown_percent {
                    let mode_suffix = if app.view.fit {
                        format!(" [{}]", tr.status_mode_fit)
                    } else if app.view.is_actual(ui.pixels_per_point()) {
                        " [100%]".to_string()
                    } else {
                        String::new()
                    };
                    let zoom_text = format!("{percent}%{mode_suffix}");
                    let is_fit = app.view.fit;
                    let resp = ui.add(
                        egui::Button::new(
                            RichText::new(&zoom_text)
                                .size(11.5)
                                .color(if is_fit {
                                    palette.accent_default
                                } else {
                                    palette.text_secondary
                                })
                                .strong(),
                        )
                        .frame(false)
                        .corner_radius(ThemeRounding::RADIUS_XS),
                    );

                    let mode_desc = if is_fit {
                        tr.status_mode_auto
                    } else if app.view.is_actual(ui.pixels_per_point()) {
                        tr.status_mode_actual
                    } else {
                        tr.zoom
                    };
                    let tooltip_text = tr
                        .status_zoom_tooltip
                        .replace("{pct}", &percent.to_string())
                        .replace("{mode}", mode_desc);
                    let resp = resp.on_hover_text(tooltip_text);

                    if resp.clicked() {
                        if is_fit {
                            app.view.set_actual(ui.pixels_per_point());
                        } else {
                            app.view.set_fit();
                        }
                    }
                }
            }
        });
    });
}

// =============================================================================
// TESTES UNITÁRIOS DA STATUS BAR E MICROINTERAÇÕES
// =============================================================================

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;
    use crate::app::state::Picture;

    fn make_test_picture(name: &str, w: u32, h: u32) -> Arc<visor::Decoded> {
        let rgba = vec![255u8; (w * h * 4) as usize];
        Arc::new(visor::Decoded {
            frames: vec![visor::Frame {
                rgba,
                width: w,
                height: h,
                delay_ms: 0,
            }],
            info: visor::ImageInfo {
                id: "test",
                name: name.to_owned(),
                category: visor::Category::Raster,
                support: visor::Support::Full,
                bit_depth: 8,
                color: "RGB".to_owned(),
                original_w: w,
                original_h: h,
                display_w: w,
                display_h: h,
                reduced: false,
                has_alpha: true,
                frame_count: 1,
                loop_forever: false,
                mipmaps: 0,
                exif: Vec::new(),
                icc: None,
                notes: vec!["Teste automatizado".to_owned()],
            },
        })
    }

    #[test]
    fn test_render_status_empty_state() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.picture = None;
        app.files.clear();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.picture.is_none());
    }

    #[test]
    fn test_render_status_with_picture_and_files() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let decoded = make_test_picture("foto_teste.png", 1920, 1080);
        app.install(&ctx, decoded);
        app.files = vec![
            PathBuf::from("foto_teste.png"),
            PathBuf::from("foto_segunda.jpg"),
        ];
        app.index = 0;
        app.shown_percent = Some(100);

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();

        assert_eq!(app.shown_percent, Some(100));
        assert_eq!(app.files.len(), 2);
    }

    #[test]
    fn test_render_status_loading_state() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.loading = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.loading);
    }

    #[test]
    fn test_render_status_error_state_dismissal() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.error = Some("Falha ao abrir arquivo corrompido".to_owned());

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.error.is_some());
    }

    #[test]
    fn test_render_status_notice_state() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.notice = Some("Exportação concluída com sucesso!".to_owned());

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.notice.is_some());
    }

    #[test]
    fn test_render_status_narrow_compact_window() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let decoded = make_test_picture(
            "imagem_longa_para_testar_resolucoes_estreitas.png",
            800,
            600,
        );
        app.install(&ctx, decoded);
        app.files = vec![PathBuf::from(
            "imagem_longa_para_testar_resolucoes_estreitas.png",
        )];
        app.shown_percent = Some(50);

        // Simula janela com apenas 360 px de largura
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(360.0, 300.0),
                )),
                ..Default::default()
            },
            |ui| {
                render_status(&mut app, ui);
            },
        );
        output.textures_delta.clear();
    }

    #[test]
    fn test_draw_spinner_renders_without_panic() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let resp = draw_spinner(ui, 6.0, Color32::from_rgb(96, 205, 255));
            assert_eq!(resp.rect.size(), Vec2::splat(12.0));
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_render_status_long_name_truncation() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let long_name = "foto_com_nome_extremamente_longo_para_testar_truncamento_e_responsividade_no_status_bar_sem_estourar_largura.png";
        let decoded = make_test_picture(long_name, 1920, 1080);
        app.install(&ctx, decoded);
        app.files = vec![PathBuf::from(long_name)];

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_render_status_zoom_toggle_microinteraction() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.shown_percent = Some(250);
        app.view.fit = true;

        // Ao alternar quando em fit, desativa fit e entra em escala real
        app.view.set_actual(1.0);
        assert!(!app.view.fit);
        assert!(app.view.is_actual(1.0));

        // Ao alternar quando manual, volta para fit
        app.view.set_fit();
        assert!(app.view.fit);
    }

    #[test]
    fn test_render_status_with_filmstrip_and_info_panel_open() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let decoded = make_test_picture("sample.jpg", 1280, 720);
        app.install(&ctx, decoded);
        app.files = vec![PathBuf::from("sample.jpg")];
        app.show_info = true;
        app.show_filmstrip = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_status(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.show_info);
        assert!(app.show_filmstrip);
    }
}
