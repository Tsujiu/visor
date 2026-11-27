//! Menu de contexto moderno (botão direito) inspirado no Windows 11 Fluent Design.
//!
//! Exibido sob o cursor ao clicar com o botão direito no Canvas ou nas miniaturas do Filmstrip.
//! Organizado em agrupamentos funcionais lógicos com separadores sutis:
//! - Área de Transferência & Sistema (Copiar Imagem, Copiar Caminho, Revelar no Explorer)
//! - Manipulação (Girar à Direita, Girar à Esquerda)
//! - Exibição & Zoom (Ajustar à Tela, Tamanho Real 100%, Interpolação, Alternar Fundo)
//! - Informações (Painel de Informações EXIF/Metadados)
//! - Destrutivo (Mover para a Lixeira)

use std::path::Path;

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Vec2};

use super::theme::{ThemePalette, ThemeRounding};
use crate::app::state::{InterpolationMode, VisorApp};

/// Item de menu estilizado com visual Fluent Design: hover suave, cantos arredondados,
/// rótulo alinhado à esquerda e atalho de teclado alinhado à direita.
pub fn render_menu_item(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
    is_destructive: bool,
) -> egui::Response {
    let palette = ThemePalette::dark();
    let height = 26.0;
    let min_width = 230.0;
    let available_w = ui.available_width().max(min_width);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(available_w, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered() && enabled;
        let is_active = response.is_pointer_button_down_on() && enabled;

        // Fundo com cantos arredondados sutis (5px)
        if is_active {
            let bg = if is_destructive {
                Color32::from_rgba_unmultiplied(220, 50, 50, 70)
            } else {
                palette.surface_active
            };
            ui.painter().rect_filled(rect, ThemeRounding::RADIUS_SM, bg);
        } else if is_hovered {
            let bg = if is_destructive {
                Color32::from_rgba_unmultiplied(220, 50, 50, 35)
            } else {
                palette.surface_hover
            };
            ui.painter().rect_filled(rect, ThemeRounding::RADIUS_SM, bg);
        }

        // Cor da tipografia
        let text_color = if !enabled {
            palette.text_muted
        } else if is_destructive && is_hovered {
            Color32::from_rgb(255, 120, 120)
        } else {
            palette.text_primary
        };

        let shortcut_color = if !enabled {
            palette.text_muted
        } else {
            palette.text_secondary
        };

        // Rótulo à esquerda (padding 10px)
        let label_pos = Pos2::new(rect.min.x + 10.0, rect.center().y);
        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0),
            text_color,
        );

        // Atalho à direita (padding 10px)
        if let Some(sc) = shortcut {
            let sc_pos = Pos2::new(rect.max.x - 10.0, rect.center().y);
            ui.painter().text(
                sc_pos,
                egui::Align2::RIGHT_CENTER,
                sc,
                FontId::proportional(11.5),
                shortcut_color,
            );
        }
    }

    response
}

/// Separador horizontal sutil de 1px entre grupos de ações.
pub fn render_menu_separator(ui: &mut egui::Ui) {
    let palette = ThemePalette::dark();
    let width = ui.available_width().max(230.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 7.0), Sense::hover());
    let y = rect.center().y;
    ui.painter().line_segment(
        [
            Pos2::new(rect.min.x + 6.0, y),
            Pos2::new(rect.max.x - 6.0, y),
        ],
        palette.border_subtle,
    );
}

/// Renderiza o menu de contexto completo.
///
/// Se `target_path` for `None`, as ações operam sobre a imagem ativa atualmente no canvas.
/// Se `target_path` for `Some(path)`, as ações referem-se àquele arquivo específico (ex: miniatura do Filmstrip).
pub fn render_context_menu(app: &mut VisorApp, ui: &mut egui::Ui, target_path: Option<&Path>) {
    let tr = *app.tr();
    let current_path = app.current_path().map(Path::to_path_buf);
    let is_current = match (&current_path, target_path) {
        (Some(curr), Some(target)) => curr == target,
        (_, None) => true,
        _ => false,
    };

    let effective_path = target_path
        .map(Path::to_path_buf)
        .or_else(|| current_path.clone());
    let has_target_file = effective_path.is_some();
    let has_picture = app.picture.is_some();

    ui.spacing_mut().item_spacing = Vec2::new(0.0, 1.0);

    // =========================================================================
    // CASO A: Menu acionado em miniatura diferente da imagem ativa no momento
    // =========================================================================
    if !is_current && target_path.is_some() {
        let path = target_path.unwrap();
        let target_idx = app.files.iter().position(|p| p == path);

        if let Some(idx) = target_idx
            && render_menu_item(ui, tr.open_image_btn, None, true, false).clicked() {
                app.go_to(idx);
                ui.close();
            }

        if render_menu_item(ui, tr.copy_path, None, true, false).clicked() {
            app.copy_path(ui.ctx(), Some(path));
            ui.close();
        }

        if render_menu_item(ui, tr.reveal_in_explorer, None, true, false).clicked() {
            app.reveal_in_explorer(Some(path));
            ui.close();
        }

        if render_menu_item(ui, tr.open_with, None, true, false).clicked() {
            app.open_with_dialog(Some(path));
            ui.close();
        }

        render_menu_separator(ui);

        if render_menu_item(ui, tr.move_to_trash, Some("Del"), true, true).clicked() {
            app.delete_image_at_path(path.to_path_buf(), ui.ctx());
            ui.close();
        }
        return;
    }

    // =========================================================================
    // CASO B: Menu acionado no Canvas vazio (nenhuma imagem aberta)
    // =========================================================================
    if !has_picture {
        if render_menu_item(ui, tr.open_image_btn, Some("Ctrl+O"), true, false).clicked() {
            app.want_open = true;
            ui.close();
        }
        if render_menu_item(ui, tr.formats_catalog, None, true, false).clicked() {
            app.show_formats = true;
            ui.close();
        }
        if render_menu_item(ui, tr.batch_export, None, true, false).clicked() {
            app.want_batch_files = true;
            ui.close();
        }
        return;
    }

    // =========================================================================
    // CASO C: Imagem ativa carregada (Canvas ou miniatura ativa)
    // =========================================================================

    // 1. Área de Transferência & Integração com o Windows
    if render_menu_item(ui, tr.copy_image, Some("Ctrl+C"), has_picture, false).clicked() {
        app.copy_current_image(ui.ctx());
        ui.close();
    }

    if render_menu_item(ui, tr.copy_path, None, has_target_file, false).clicked() {
        app.copy_path(ui.ctx(), effective_path.as_deref());
        ui.close();
    }

    if render_menu_item(ui, tr.reveal_in_explorer, None, has_target_file, false).clicked() {
        app.reveal_in_explorer(effective_path.as_deref());
        ui.close();
    }

    if render_menu_item(ui, tr.open_with, None, has_target_file, false).clicked() {
        app.open_with_dialog(effective_path.as_deref());
        ui.close();
    }

    render_menu_separator(ui);

    // 2. Manipulação e Rotação
    if render_menu_item(ui, tr.rotate_cw, Some("R"), has_picture, false).clicked() {
        app.rotate(ui.ctx(), true);
        ui.close();
    }

    if render_menu_item(ui, tr.rotate_ccw, Some("Shift+R"), has_picture, false).clicked() {
        app.rotate(ui.ctx(), false);
        ui.close();
    }

    render_menu_separator(ui);

    // 3. Exibição, Enquadramento e Interpolação
    if render_menu_item(ui, tr.fit_to_window, Some("F"), has_picture, false).clicked() {
        app.view.set_fit();
        ui.close();
    }

    if render_menu_item(ui, tr.actual_size, Some("1"), has_picture, false).clicked() {
        let ppp = ui.pixels_per_point();
        app.view.set_actual(ppp);
        ui.close();
    }

    let interp_label = app.interpolation.localized_label(&tr);
    if render_menu_item(ui, interp_label, Some("P"), has_picture, false).clicked() {
        app.interpolation = match app.interpolation {
            InterpolationMode::Nearest => InterpolationMode::Auto,
            _ => InterpolationMode::Nearest,
        };
        ui.close();
    }

    let bg_label = format!("{}: {}", tr.background, app.background.localized_label(&tr));
    if render_menu_item(ui, &bg_label, Some("B"), has_picture, false).clicked() {
        app.background = app.background.cycle();
        ui.close();
    }

    if matches!(
        app.background,
        crate::app::state::CanvasBackground::Custom(_)
    )
        && render_menu_item(ui, tr.customize_bg, None, has_picture, false).clicked() {
            app.open_color_picker();
            ui.close();
        }

    render_menu_separator(ui);

    // 4. Painel de Informações & Metadados
    let info_label = if app.show_info { tr.hide_info } else { tr.info };
    if render_menu_item(ui, info_label, Some("I"), has_picture, false).clicked() {
        app.show_info = !app.show_info;
        ui.close();
    }

    render_menu_separator(ui);

    // 5. Ação Destrutiva (Separada e destacada)
    if render_menu_item(ui, tr.move_to_trash, Some("Del"), has_target_file, true).clicked() {
        if let Some(path) = effective_path {
            app.delete_image_at_path(path, ui.ctx());
        }
        ui.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use visor::{Decoded, Frame, ImageInfo};

    fn mock_decoded() -> Arc<Decoded> {
        Arc::new(Decoded {
            frames: vec![Frame {
                rgba: vec![255u8; 16 * 4],
                width: 4,
                height: 4,
                delay_ms: 0,
            }],
            info: ImageInfo {
                id: "test",
                name: "teste.png".to_owned(),
                category: visor::Category::Raster,
                support: visor::Support::Full,
                bit_depth: 8,
                color: "RGBA".to_owned(),
                original_w: 4,
                original_h: 4,
                display_w: 4,
                display_h: 4,
                reduced: false,
                has_alpha: true,
                frame_count: 1,
                loop_forever: false,
                mipmaps: 0,
                exif: Vec::new(),
                icc: None,
                notes: Vec::new(),
            },
        })
    }

    #[test]
    fn test_menu_item_rendering_enabled_and_disabled() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            let resp1 = render_menu_item(ui, "Copiar Imagem", Some("Ctrl+C"), true, false);
            assert!(resp1.sense.interactive());

            let resp2 = render_menu_item(ui, "Item Desativado", None, false, false);
            assert!(!resp2.sense.interactive());

            let resp3 = render_menu_item(ui, "Mover para a Lixeira", Some("Del"), true, true);
            assert!(resp3.sense.interactive());
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_menu_separator_rendering() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_menu_separator(ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_context_menu_renders_empty_state_without_panic() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        assert!(app.picture.is_none());

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_context_menu(&mut app, ui, None);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_context_menu_renders_with_loaded_image_without_panic() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.install(&ctx, mock_decoded());
        app.files = vec![PathBuf::from("C:\\fotos\\paisagem.png")];
        app.index = 0;
        assert!(app.picture.is_some());

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_context_menu(&mut app, ui, None);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_context_menu_renders_for_filmstrip_target_without_panic() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.install(&ctx, mock_decoded());
        let file1 = PathBuf::from("C:\\fotos\\img1.png");
        let file2 = PathBuf::from("C:\\fotos\\img2.png");
        app.files = vec![file1, file2.clone()];
        app.index = 0;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            // Clicando na miniatura do arquivo secundário (não o ativo)
            render_context_menu(&mut app, ui, Some(&file2));
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_copy_path_formatting_and_notice() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let file = PathBuf::from("Cargo.toml");
        assert!(file.exists());
        app.files = vec![file];
        app.index = 0;

        app.copy_path(&ctx, None);
        assert_eq!(
            app.notice.as_deref(),
            Some("Caminho copiado para a área de transferência.")
        );
    }

    #[test]
    fn test_reveal_in_explorer_error_handling() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let ghost = Path::new("C:\\arquivo_que_nao_existe_visor_xyz.png");
        app.reveal_in_explorer(Some(ghost));

        assert!(app.error.is_some());
        let err = app.error.clone().unwrap();
        assert!(err.contains("não encontrado"));
    }
}
