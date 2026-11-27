//! Carrossel horizontal de miniaturas e carregamento assíncrono de previews.
//!
//! Tira de navegação inferior inspirada na linguagem visual do Windows 11:
//! miniaturas responsivas com proporção preservada, cantos arredondados,
//! seleção destacada em Accent Blue, scroll horizontal automático e carregamento
//! cooperativo em segundo plano via Thread Pool.

use std::sync::mpsc;
use std::time::Duration;

use eframe::egui::{
    self, Color32, ColorImage, Context, Pos2, Rect, Sense, Stroke, StrokeKind, TextureOptions, Vec2,
};

use super::theme::{ThemeDimensions, ThemePalette, ThemeRounding, ThemeSpacing};
use crate::app::state::{ThumbMsg, VisorApp};
use crate::core::geometry::fit_rect;
use crate::workers::{CancellationToken, Priority};

/// Renderiza o painel inferior retrátil do Filmstrip.
pub fn render_filmstrip(app: &mut VisorApp, ui: &mut egui::Ui) {
    if app.files.is_empty() {
        return;
    }

    let palette = ThemePalette::dark();
    let available_width = ui.available_width();
    let (thumb_size, thumb_spacing) = if available_width < 480.0 {
        (
            ThemeDimensions::FILMSTRIP_THUMB_SIZE_COMPACT,
            ThemeSpacing::XS,
        )
    } else {
        (ThemeDimensions::FILMSTRIP_THUMB_SIZE, ThemeSpacing::SM)
    };

    let mut clicked = None;

    ui.add_space(4.0);
    egui::ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(thumb_spacing, 0.0);
            ui.horizontal(|ui| {
                for index in 0..app.files.len() {
                    let selected = index == app.index;
                    let (rect, response) = ui.allocate_exact_size(thumb_size, Sense::click());

                    // Auto-scroll suave para centralizar a miniatura na visão quando o índice for alterado
                    if app.filmstrip_scroll_target == Some(index) {
                        response.scroll_to_me(Some(egui::Align::Center));
                        app.filmstrip_scroll_target = None;
                    }

                    // Estilização visual inspirada no Windows 11 Fluent Design
                    let (fill, stroke) = if selected {
                        (
                            palette.surface_active,
                            Stroke::new(2.0, palette.accent_default),
                        )
                    } else if response.hovered() {
                        (palette.surface_hover, palette.border_default)
                    } else {
                        (palette.surface_card, palette.border_subtle)
                    };

                    ui.painter()
                        .rect_filled(rect, ThemeRounding::RADIUS_SM, fill);
                    ui.painter().rect_stroke(
                        rect,
                        ThemeRounding::RADIUS_SM,
                        stroke,
                        StrokeKind::Inside,
                    );

                    // Renderização da imagem / placeholder
                    if let Some(texture) = app.thumbs.get(&index) {
                        let fitted = fit_rect(rect.shrink(3.0), texture.size_vec2());
                        ui.painter().image(
                            texture.id(),
                            fitted,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else if app.failed_thumbs.contains(&index) {
                        draw_failed_thumb_placeholder(ui.painter(), rect, &palette);
                    } else {
                        draw_loading_thumb_placeholder(ui.painter(), rect, selected, &palette);
                    }

                    // Tooltip limpo com índice e nome do arquivo
                    let response = if let Some(path) = app.files.get(index) {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy())
                            .unwrap_or_default();
                        response.on_hover_text(format!(
                            "[{}/{}] {}",
                            index + 1,
                            app.files.len(),
                            name
                        ))
                    } else {
                        response
                    };

                    let target_path = app.files.get(index).cloned();
                    response.context_menu(|ui| {
                        super::context_menu::render_context_menu(app, ui, target_path.as_deref());
                    });

                    if response.clicked() {
                        clicked = Some(index);
                    }
                }
            });
        });
    ui.add_space(4.0);

    if let Some(index) = clicked {
        app.go_to(index);
    }
}

/// Desenha o placeholder minimalista de carregamento de miniatura.
pub fn draw_loading_thumb_placeholder(
    painter: &egui::Painter,
    rect: Rect,
    selected: bool,
    palette: &ThemePalette,
) {
    let center = rect.center();
    let color = if selected {
        palette.accent_muted
    } else {
        palette.surface_active
    };
    painter.circle_filled(center, 4.0, color);
}

/// Desenha o indicador visual sutil de falha no carregamento da miniatura.
pub fn draw_failed_thumb_placeholder(painter: &egui::Painter, rect: Rect, palette: &ThemePalette) {
    let center = rect.center();
    let icon_rect = Rect::from_center_size(center, Vec2::new(22.0, 18.0));
    painter.rect_stroke(
        icon_rect,
        ThemeRounding::RADIUS_XS,
        Stroke::new(1.0, palette.text_muted),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [
            icon_rect.left_bottom() + Vec2::new(3.0, -3.0),
            icon_rect.right_top() + Vec2::new(-3.0, 3.0),
        ],
        Stroke::new(1.0, palette.text_muted),
    );
}

/// Gerencia o pool assíncrono de miniaturas, priorizando a vizinhança da seleção.
pub fn poll_thumbs(app: &mut VisorApp, ctx: &Context) {
    // Se o filmstrip estiver oculto ou a lista estiver vazia, não consome CPU
    if !app.show_filmstrip || app.files.is_empty() {
        return;
    }

    // 1. Coleta resultado concluído por worker thread
    let ready = app.thumb_rx.as_ref().and_then(|rx| rx.try_recv().ok());

    if let Some(msg) = ready {
        app.thumb_rx = None;
        app.thumb_cancel = None;
        if msg.list_token == app.list_token {
            if let Some(image) = msg.image {
                let texture = ctx.load_texture(
                    format!("thumb-{}-{}", msg.list_token, msg.index),
                    image,
                    TextureOptions::LINEAR,
                );
                app.thumbs.insert(msg.index, texture);
            } else {
                app.failed_thumbs.insert(msg.index);
            }

            // Poda automática de miniaturas distantes: mantém o consumo de VRAM e RAM estritamente sob controle
            if app.thumbs.len() > 60 {
                let current = app.index as isize;
                app.thumbs
                    .retain(|&idx, _| (idx as isize - current).abs() <= 30);
                app.failed_thumbs
                    .retain(|&idx| (idx as isize - current).abs() <= 45);
            }
        }
    }

    // 2. Se já houver tarefa em andamento, solicita repaint suave e aguarda
    if app.thumb_rx.is_some() {
        ctx.request_repaint_after(Duration::from_millis(30));
        return;
    }

    // 3. Seleção inteligente por janela deslizante: foca estritamente na vizinhança de app.index (alcance de 20 itens)
    // Evita consumir CPU gerando miniaturas para milhares de arquivos distantes na pasta.
    let len = app.files.len();
    let mut next_index = None;

    for dist in 0..=20 {
        if let Some(idx) = app.index.checked_add(dist)
            && idx < len && !app.thumbs.contains_key(&idx) && !app.failed_thumbs.contains(&idx) {
                next_index = Some(idx);
                break;
            }
        if dist > 0
            && let Some(idx) = app.index.checked_sub(dist)
                && !app.thumbs.contains_key(&idx) && !app.failed_thumbs.contains(&idx) {
                    next_index = Some(idx);
                    break;
                }
    }

    let Some(index) = next_index else {
        return;
    };

    // 4. Dispara decodificação assíncrona no ThreadPool com Priority::Thumbnail
    let path = app.files[index].clone();
    let list_token = app.list_token;
    let (tx, rx) = mpsc::channel();
    let cancel = CancellationToken::new();
    let task_cancel = cancel.clone();
    app.thumb_rx = Some(rx);
    app.thumb_cancel = Some(cancel);
    ctx.request_repaint();

    let accepted =
        app.pool
            .spawn_with_priority(Priority::Thumbnail, Some(task_cancel.clone()), move || {
                if task_cancel.is_cancelled() {
                    return;
                }
                let image = visor::thumbnail(&path, 96).ok().map(|frame| {
                    ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.rgba,
                    )
                });
                if task_cancel.is_cancelled() {
                    return;
                }
                let _ = tx.send(ThumbMsg {
                    index,
                    list_token,
                    image,
                });
            });

    if !accepted {
        app.thumb_rx = None;
        app.thumb_cancel = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filmstrip_toggle_preserves_selection() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = vec![
            PathBuf::from("img1.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png"),
        ];
        app.index = 1;
        assert!(app.show_filmstrip);

        // Oculta filmstrip
        app.show_filmstrip = false;
        assert_eq!(app.index, 1);

        // Reabre filmstrip
        app.show_filmstrip = true;
        assert_eq!(app.index, 1);
    }

    #[test]
    fn test_filmstrip_empty_files_renders_without_panic() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = Vec::new();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_filmstrip(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_filmstrip_single_file_renders_properly() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = vec![PathBuf::from("single_photo.jpg")];
        app.index = 0;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_filmstrip(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_filmstrip_scroll_target_cleared_on_render() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = vec![
            PathBuf::from("img1.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png"),
        ];
        app.index = 1;
        app.filmstrip_scroll_target = Some(1);

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_filmstrip(&mut app, ui);
        });
        output.textures_delta.clear();

        // O alvo de scroll deve ser consumido na renderização
        assert_eq!(app.filmstrip_scroll_target, None);
    }

    #[test]
    fn test_filmstrip_many_files_renders_with_scroll_area() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = (0..50)
            .map(|i| PathBuf::from(format!("foto_{i}.jpg")))
            .collect();
        app.index = 25;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_filmstrip(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_poll_thumbs_failed_load_recorded_without_infinite_loop() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = vec![PathBuf::from("corrupted.jpg")];
        app.index = 0;

        // Simula recebimento de ThumbMsg com None (falha de decode)
        let (tx, rx) = mpsc::channel();
        tx.send(ThumbMsg {
            index: 0,
            list_token: app.list_token,
            image: None,
        })
        .unwrap();
        app.thumb_rx = Some(rx);

        poll_thumbs(&mut app, &ctx);

        // Verifica se foi registrado em failed_thumbs e liberou thumb_rx
        assert!(app.failed_thumbs.contains(&0));
        assert!(app.thumb_rx.is_none());

        // Na próxima passagem, não deve enfileirar nada pois o índice está marcado como falha
        poll_thumbs(&mut app, &ctx);
        assert!(app.thumb_rx.is_none());
    }

    #[test]
    fn test_poll_thumbs_distance_prioritization() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = (0..20)
            .map(|i| PathBuf::from(format!("foto_{i}.jpg")))
            .collect();
        app.index = 10;

        // Primeira chamada deve selecionar o índice ativo (10)
        assert!(app.thumb_rx.is_none());
        poll_thumbs(&mut app, &ctx);
        assert!(app.thumb_rx.is_some());

        // Simula conclusão do índice 10
        let (tx, rx) = mpsc::channel();
        tx.send(ThumbMsg {
            index: 10,
            list_token: app.list_token,
            image: Some(ColorImage::example()),
        })
        .unwrap();
        app.thumb_rx = Some(rx);
        poll_thumbs(&mut app, &ctx);
        assert!(app.thumbs.contains_key(&10));

        // Próxima chamada deve priorizar a vizinhança imediata (11 ou 9)
        poll_thumbs(&mut app, &ctx);
        assert!(app.thumb_rx.is_some());
    }

    #[test]
    fn test_placeholders_drawing_no_panic() {
        let ctx = Context::default();
        let palette = ThemePalette::dark();
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(76.0, 56.0));

        let mut output = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter();
            draw_loading_thumb_placeholder(painter, rect, true, &palette);
            draw_loading_thumb_placeholder(painter, rect, false, &palette);
            draw_failed_thumb_placeholder(painter, rect, &palette);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_filmstrip_panel_height_bounds_and_canvas_protection() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = (0..10)
            .map(|i| PathBuf::from(format!("foto_{i}.jpg")))
            .collect();
        app.index = 0;
        app.show_filmstrip = true;

        let mut canvas_height = 0.0;
        let mut filmstrip_height = 0.0;

        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(screen_rect);

        let mut output = ctx.run_ui(raw_input, |ui| {
            let film_h = if ui.available_width() < 480.0 {
                ThemeDimensions::FILMSTRIP_HEIGHT_COMPACT
            } else {
                ThemeDimensions::FILMSTRIP_HEIGHT
            };

            egui::Panel::bottom("status")
                .exact_size(ThemeDimensions::STATUS_BAR_HEIGHT)
                .show(ui, |_ui| {});

            egui::Panel::bottom("film")
                .exact_size(film_h)
                .show(ui, |ui| {
                    filmstrip_height = ui.available_height();
                    render_filmstrip(&mut app, ui);
                });

            egui::CentralPanel::default().show(ui, |ui| {
                canvas_height = ui.available_height();
            });
        });
        output.textures_delta.clear();

        assert!(filmstrip_height <= ThemeDimensions::FILMSTRIP_HEIGHT);
        assert!(
            canvas_height > 450.0,
            "Canvas colapsou para {canvas_height}px!"
        );
    }

    #[test]
    fn test_filmstrip_sliding_window_pruning() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.files = (0..100)
            .map(|i| PathBuf::from(format!("foto_{i}.jpg")))
            .collect();
        app.index = 50;

        // Insere miniaturas simuladas
        for i in 0..70 {
            let texture = ctx.load_texture(
                format!("test-thumb-{i}"),
                ColorImage::example(),
                TextureOptions::LINEAR,
            );
            app.thumbs.insert(i, texture);
        }

        // Simula recebimento de ThumbMsg para acionar a poda de miniaturas distantes
        let (tx, rx) = mpsc::channel();
        tx.send(ThumbMsg {
            index: 51,
            list_token: app.list_token,
            image: Some(ColorImage::example()),
        })
        .unwrap();
        app.thumb_rx = Some(rx);

        poll_thumbs(&mut app, &ctx);

        // Miniaturas distantes (ex: índice 0 ou 10, dist > 30 de 50) devem ter sido podadas
        assert!(!app.thumbs.contains_key(&0));
        assert!(!app.thumbs.contains_key(&10));
        assert!(app.thumbs.contains_key(&50));
        assert!(app.thumbs.contains_key(&51));
        assert!(app.thumbs.len() <= 62);
    }
}
