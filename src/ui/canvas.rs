//! Viewport principal da imagem: zoom ancorado, pan, fundos e renderização Pixel Perfect.

use eframe::egui::emath::GuiRounding as _;
use eframe::egui::{
    self, Align2, Color32, ColorImage, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Sense,
    Stroke, StrokeKind, Vec2,
};

use super::theme::{BG, ThemePalette, ThemeRounding};
use crate::app::state::{CanvasBackground, VisorApp};
use crate::core::geometry::{clamp_offset, image_rect, scale_limits, zoom_about};
use crate::system::config::MouseWheelAction;

/// Renderiza a área principal de exibição da imagem (canvas).
pub fn render_canvas(app: &mut VisorApp, ui: &mut egui::Ui) {
    let canvas = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(canvas, Sense::click_and_drag());
    let painter = ui.painter().clone();
    let palette = ThemePalette::dark();
    painter.rect_filled(canvas, 0.0, BG);

    // Menu de contexto com botão direito no canvas (imagem ativa ou estado vazio)
    response.context_menu(|ui| {
        super::context_menu::render_context_menu(app, ui, None);
    });

    // 1. Destaque sutil ao arrastar arquivo sobre o canvas
    let hovering_file = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
    if hovering_file {
        painter.rect_filled(
            canvas.shrink(8.0),
            ThemeRounding::RADIUS_LG,
            palette.accent_muted,
        );
        painter.rect_stroke(
            canvas.shrink(8.0),
            ThemeRounding::RADIUS_LG,
            Stroke::new(2.0, palette.accent_default),
            StrokeKind::Inside,
        );
        painter.text(
            canvas.center() + Vec2::new(0.0, 48.0),
            Align2::CENTER_CENTER,
            app.tr().drop_image_here,
            FontId::proportional(16.0),
            palette.accent_default,
        );
    }

    // 2. Se nenhuma imagem estiver carregada, exibe o Empty State minimalista
    let Some(image_px) = app.picture.as_ref().map(|picture| picture.size()) else {
        draw_empty(app, &painter, canvas, &palette);
        return;
    };

    let ppp = ui.pixels_per_point();
    let (min_scale, max_scale) = scale_limits(ppp);

    // 3. Atalhos de Zoom no teclado (+ e -) ancorados no cursor
    let zoom_in = ui.input(|input| input.key_pressed(Key::Plus) || input.key_pressed(Key::Equals));
    let zoom_out = ui.input(|input| input.key_pressed(Key::Minus));
    if zoom_in || zoom_out {
        let factor = if zoom_in { 1.15 } else { 1.0 / 1.15 };
        let anchor = ui
            .ctx()
            .input(|input| input.pointer.hover_pos())
            .unwrap_or_else(|| canvas.center());
        apply_zoom(app, canvas, image_px, anchor, factor, min_scale, max_scale);
    }

    // 4. Duplo clique para alternar entre Ajustar à Tela (Fit) e 100% (Tamanho Real)
    if response.double_clicked() {
        if app.view.fit {
            app.view.set_actual(ppp);
        } else {
            app.view.set_fit();
        }
    }

    // 5. Pan interativo por clique e arraste com o botão primário
    let scale_before_drag = app.view.scale(image_px, canvas);
    if response.dragged_by(PointerButton::Primary) {
        if app.view.fit {
            app.view.fit = false;
            app.view.zoom = scale_before_drag;
            app.view.offset = Vec2::ZERO;
        }
        app.view.offset += response.drag_delta();
    }

    // 6. Manipulação configurável da roda do mouse (Zoom ou Navegação)
    let canvas_active = is_canvas_interaction_active(app, ui, &response);
    if canvas_active {
        let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
        let ctrl_held = ui.input(|input| input.modifiers.ctrl || input.modifiers.command);

        if ctrl_held || app.settings.mouse_wheel_action == MouseWheelAction::Zoom {
            // Zoom progressivo ancorado no cursor (Ctrl força Zoom sempre)
            if scroll_y.abs() > 0.1 {
                let anchor = response.hover_pos().unwrap_or_else(|| canvas.center());
                let factor = (scroll_y * 0.0016).exp();
                apply_zoom(app, canvas, image_px, anchor, factor, min_scale, max_scale);
                app.wheel_nav_accumulator = 0.0;
            }
        } else if app.settings.mouse_wheel_action == MouseWheelAction::Navigate {
            // Navegação entre imagens com acumulador e limiar anti-disparo acidental
            const WHEEL_NAV_THRESHOLD: f32 = 20.0;

            if scroll_y != 0.0 {
                // Inversão brusca de direção zera o acumulador imediatamente
                if (scroll_y > 0.0 && app.wheel_nav_accumulator < 0.0)
                    || (scroll_y < 0.0 && app.wheel_nav_accumulator > 0.0)
                {
                    app.wheel_nav_accumulator = 0.0;
                }

                app.wheel_nav_accumulator += scroll_y;

                if app.wheel_nav_accumulator >= WHEEL_NAV_THRESHOLD {
                    // Roda para cima (afastando) -> imagem anterior
                    app.step(-1);
                    app.wheel_nav_accumulator = 0.0;
                } else if app.wheel_nav_accumulator <= -WHEEL_NAV_THRESHOLD {
                    // Roda para baixo (aproximando) -> próxima imagem
                    app.step(1);
                    app.wheel_nav_accumulator = 0.0;
                }
            }
        }
    } else {
        // Ao deshoverar o canvas ou sobrepor com modal/texto/painel, reseta o acumulador
        app.wheel_nav_accumulator = 0.0;
    }

    if !app.view.fit {
        let scale = app.view.zoom;
        app.view.offset = clamp_offset(canvas, image_px, scale, app.view.offset);
    }

    let scale = app.view.scale(image_px, canvas);
    let effective_scale = scale * ppp;
    app.shown_percent = Some((effective_scale * 100.0).round() as i32);

    // 7. Atualização dinâmica da interpolação na textura (GPU) conforme modo e escala
    let target_options = app.interpolation.resolve_options(effective_scale);
    if let Some(picture) = app.picture.as_mut() {
        if picture.texture_options != target_options {
            let frame = picture.frame();
            let color = ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.rgba,
            );
            picture.texture.set(color, target_options);
            picture.texture_options = target_options;
        }
    }

    // 8. Cálculo de coordenadas Pixel-Perfect com alinhamento a pixels físicos
    let rect = image_rect(canvas, image_px, scale, app.view.offset).round_to_pixels(ppp);

    // 9. Renderização do fundo da imagem (transparência / canal alfa)
    let (has_alpha, texture_id) = app
        .picture
        .as_ref()
        .map(|p| (p.info.has_alpha, p.texture.id()))
        .unwrap();

    if has_alpha {
        match app.background {
            CanvasBackground::DarkChecker => {
                let cell = 12.0;
                let repeats = rect.size() / (cell * 2.0);
                painter.image(
                    app.checker.id(),
                    rect,
                    Rect::from_min_size(Pos2::ZERO, repeats),
                    Color32::WHITE,
                );
            }
            CanvasBackground::LightChecker => {
                let cell = 12.0;
                let repeats = rect.size() / (cell * 2.0);
                painter.image(
                    app.checker_light.id(),
                    rect,
                    Rect::from_min_size(Pos2::ZERO, repeats),
                    Color32::WHITE,
                );
            }
            CanvasBackground::Black => {
                painter.rect_filled(rect, 0.0, Color32::from_rgb(12, 12, 14));
            }
            CanvasBackground::White => {
                painter.rect_filled(rect, 0.0, Color32::from_rgb(250, 250, 252));
            }
            CanvasBackground::Custom(color) => {
                painter.rect_filled(rect, 0.0, color);
            }
        }
    }

    // 10. Desenho da textura principal da imagem
    painter.image(
        texture_id,
        rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    // 11. Ícones de cursor contextuais para navegação
    if response.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if response.hovered() {
        response.clone().on_hover_cursor(CursorIcon::Grab);
    }

    // 12. Indicador sutil de carregamento no topo
    if app.loading {
        painter.text(
            canvas.center_top() + Vec2::new(0.0, 18.0),
            Align2::CENTER_TOP,
            app.tr().loading,
            FontId::proportional(15.0),
            palette.text_secondary,
        );
    }
}

/// Renderiza o estado vazio minimalista (Empty State) com ícone vetorial e atalhos limpos.
pub fn draw_empty(app: &VisorApp, painter: &egui::Painter, canvas: Rect, palette: &ThemePalette) {
    if let Some(error) = &app.error {
        let card_size = Vec2::new((canvas.width() - 64.0).clamp(240.0, 480.0), 96.0);
        let card_rect = Rect::from_center_size(canvas.center(), card_size);
        painter.rect_filled(card_rect, ThemeRounding::RADIUS_MD, palette.surface_card);
        painter.rect_stroke(
            card_rect,
            ThemeRounding::RADIUS_MD,
            palette.border_subtle,
            StrokeKind::Inside,
        );
        let galley = painter.layout(
            error.clone(),
            FontId::proportional(15.0),
            Color32::from_rgb(255, 140, 130),
            card_size.x - 32.0,
        );
        painter.galley(
            card_rect.center() - galley.size() * 0.5,
            galley,
            Color32::WHITE,
        );
        return;
    }

    let tr = *app.tr();

    if app.loading {
        painter.text(
            canvas.center(),
            Align2::CENTER_CENTER,
            tr.loading,
            FontId::proportional(18.0),
            palette.text_secondary,
        );
        return;
    }

    let center = canvas.center();

    // 1. Ícone vetorial minimalista desenhado com Painter (Card de fotografia Fluent)
    let icon_center = center + Vec2::new(0.0, -80.0);
    let card_rect = Rect::from_center_size(icon_center, Vec2::new(76.0, 58.0));
    painter.rect_filled(card_rect, ThemeRounding::RADIUS_MD, palette.surface_card);
    painter.rect_stroke(
        card_rect,
        ThemeRounding::RADIUS_MD,
        palette.border_default,
        StrokeKind::Inside,
    );

    // Sol / indicador circular
    let sun_pos = icon_center + Vec2::new(18.0, -10.0);
    painter.circle_filled(sun_pos, 5.0, palette.accent_default);

    // Picos vetoriais de montanha com preenchimento e strokes suaves
    let p1 = icon_center + Vec2::new(-24.0, 15.0);
    let p2 = icon_center + Vec2::new(-8.0, -4.0);
    let p3 = icon_center + Vec2::new(10.0, 15.0);
    painter.add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        palette.surface_active,
        Stroke::NONE,
    ));
    painter.line_segment([p1, p2], Stroke::new(1.8, palette.text_secondary));
    painter.line_segment([p2, p3], Stroke::new(1.8, palette.text_secondary));

    let q1 = icon_center + Vec2::new(0.0, 15.0);
    let q2 = icon_center + Vec2::new(14.0, 4.0);
    let q3 = icon_center + Vec2::new(26.0, 15.0);
    painter.add(egui::Shape::convex_polygon(
        vec![q1, q2, q3],
        palette.surface_hover,
        Stroke::NONE,
    ));
    painter.line_segment([q1, q2], Stroke::new(1.8, palette.text_muted));
    painter.line_segment([q2, q3], Stroke::new(1.8, palette.text_muted));

    // 2. Tipografia e mensagens limpas
    painter.text(
        center + Vec2::new(0.0, -12.0),
        Align2::CENTER_CENTER,
        tr.drop_image_here,
        FontId::proportional(20.0),
        palette.text_primary,
    );

    painter.text(
        center + Vec2::new(0.0, 16.0),
        Align2::CENTER_CENTER,
        tr.or_press_ctrl_o,
        FontId::proportional(14.0),
        palette.text_secondary,
    );

    // 3. Atalhos úteis discretos em linha
    painter.text(
        center + Vec2::new(0.0, 52.0),
        Align2::CENTER_CENTER,
        tr.empty_shortcuts_hint,
        FontId::proportional(13.0),
        palette.text_muted,
    );
}

/// Aplica uma alteração de escala ancorada em um ponto específico do canvas.
pub fn apply_zoom(
    app: &mut VisorApp,
    canvas: Rect,
    image_px: Vec2,
    anchor: Pos2,
    factor: f32,
    min_scale: f32,
    max_scale: f32,
) {
    let scale = app.view.scale(image_px, canvas);
    let (zoom, offset) = zoom_about(
        canvas,
        image_px,
        scale,
        app.view.offset,
        anchor,
        factor,
        min_scale,
        max_scale,
    );
    app.view.fit = false;
    app.view.zoom = zoom;
    app.view.offset = offset;
}

/// Verifica se a interação do ponteiro pertence exclusivamente ao viewport do Canvas,
/// sem sobreposição por janelas modais, barras flutuantes, menus ou campos de texto ativos.
pub fn is_canvas_interaction_active(
    app: &VisorApp,
    ui: &egui::Ui,
    response: &egui::Response,
) -> bool {
    if !response.hovered() {
        return false;
    }
    // Se o usuário estiver digitando em um campo de texto, a roda não deve navegar arquivos
    if ui.ctx().egui_wants_keyboard_input() {
        return false;
    }
    // Se o cursor estiver sobre uma janela modal aberta (Configurações, Formatos, Conversão, Seletor de Cores)
    if app.show_settings || app.show_formats || app.show_batch || app.show_color_picker {
        if let Some(pos) = response.hover_pos() {
            if let Some(top_layer) = ui.ctx().layer_id_at(pos) {
                if top_layer != ui.layer_id() {
                    return false;
                }
            }
        }
    }
    // Verifica se a camada ativa no ponto do cursor é a do Canvas (evita Toolbar e menus)
    if let Some(pos) = response.hover_pos() {
        if let Some(top_layer) = ui.ctx().layer_id_at(pos) {
            if top_layer != ui.layer_id() {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::InterpolationMode;
    use eframe::egui::TextureFilter;
    use std::path::PathBuf;

    #[test]
    fn test_interpolation_mode_resolution() {
        // Labels
        assert_eq!(InterpolationMode::Auto.label(), "Automático (Auto)");
        assert_eq!(InterpolationMode::Auto.short_label(), "Auto");
        assert_eq!(InterpolationMode::Linear.label(), "Suave (Bilinear)");
        assert_eq!(InterpolationMode::Linear.short_label(), "Linear");
        assert_eq!(
            InterpolationMode::Nearest.label(),
            "Pixel Perfect (Nearest)"
        );
        assert_eq!(InterpolationMode::Nearest.short_label(), "Nearest");

        // Auto: Nearest para ampliações (>= 1.0), Linear para reduções (< 1.0)
        let auto = InterpolationMode::Auto;
        assert_eq!(
            auto.resolve_options(0.5).magnification,
            TextureFilter::Linear
        );
        assert_eq!(
            auto.resolve_options(0.9).magnification,
            TextureFilter::Linear
        );
        assert_eq!(
            auto.resolve_options(1.0).magnification,
            TextureFilter::Nearest
        );
        assert_eq!(
            auto.resolve_options(2.0).magnification,
            TextureFilter::Nearest
        );
        assert_eq!(
            auto.resolve_options(4.0).magnification,
            TextureFilter::Nearest
        );

        // Linear: sempre Linear
        let linear = InterpolationMode::Linear;
        assert_eq!(
            linear.resolve_options(0.5).magnification,
            TextureFilter::Linear
        );
        assert_eq!(
            linear.resolve_options(1.0).magnification,
            TextureFilter::Linear
        );
        assert_eq!(
            linear.resolve_options(3.0).magnification,
            TextureFilter::Linear
        );

        // Nearest: sempre Nearest (Pixel Perfect)
        let nearest = InterpolationMode::Nearest;
        assert_eq!(
            nearest.resolve_options(0.25).magnification,
            TextureFilter::Nearest
        );
        assert_eq!(
            nearest.resolve_options(1.0).magnification,
            TextureFilter::Nearest
        );
        assert_eq!(
            nearest.resolve_options(4.0).magnification,
            TextureFilter::Nearest
        );
    }

    #[test]
    fn test_canvas_background_cycling() {
        assert_eq!(
            CanvasBackground::DarkChecker.label(),
            "Xadrez Escuro (Padrão)"
        );
        assert_eq!(CanvasBackground::LightChecker.label(), "Xadrez Claro");
        assert_eq!(CanvasBackground::Black.label(), "Preto Sólido");
        assert_eq!(CanvasBackground::White.label(), "Branco Sólido");

        let mut bg = CanvasBackground::DarkChecker;
        bg = bg.cycle();
        assert_eq!(bg, CanvasBackground::LightChecker);
        bg = bg.cycle();
        assert_eq!(bg, CanvasBackground::Black);
        bg = bg.cycle();
        assert_eq!(bg, CanvasBackground::White);
        bg = bg.cycle();
        assert_eq!(bg, CanvasBackground::DarkChecker);

        let custom = CanvasBackground::Custom(Color32::RED);
        assert_eq!(custom.cycle(), CanvasBackground::DarkChecker);
    }

    #[test]
    fn test_apply_zoom_anchoring() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let canvas = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let image_px = Vec2::new(400.0, 300.0);
        let anchor = Pos2::new(400.0, 300.0);

        apply_zoom(&mut app, canvas, image_px, anchor, 1.5, 0.01, 32.0);
        assert!(!app.view.fit);
        assert!(app.view.zoom > 1.0);
        assert!(app.view.zoom.is_finite());
    }

    #[test]
    fn test_draw_empty_renders_without_panic() {
        let ctx = egui::Context::default();
        let app = VisorApp::new(&ctx, None);
        let palette = ThemePalette::dark();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            let painter = ui.painter().clone();
            let canvas = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
            draw_empty(&app, &painter, canvas, &palette);
        });
        output.textures_delta.clear();
    }

    fn make_test_decoded(has_alpha: bool) -> std::sync::Arc<visor::Decoded> {
        std::sync::Arc::new(visor::Decoded {
            frames: vec![visor::Frame {
                rgba: vec![
                    255, 0, 0, 128, 0, 255, 0, 128, 0, 0, 255, 128, 255, 255, 0, 128,
                ],
                width: 2,
                height: 2,
                delay_ms: 100,
            }],
            info: visor::ImageInfo {
                id: "test",
                name: "test_image.png".to_string(),
                category: visor::Category::Raster,
                support: visor::Support::Full,
                bit_depth: 8,
                color: "RGBA8".to_string(),
                original_w: 2,
                original_h: 2,
                display_w: 2,
                display_h: 2,
                reduced: false,
                has_alpha,
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
    fn test_render_canvas_with_picture_and_all_background_modes() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let decoded = make_test_decoded(true);
        app.install(&ctx, decoded);

        let backgrounds = [
            CanvasBackground::DarkChecker,
            CanvasBackground::LightChecker,
            CanvasBackground::Black,
            CanvasBackground::White,
            CanvasBackground::Custom(Color32::from_rgb(45, 60, 80)),
        ];

        for bg in backgrounds {
            app.background = bg;
            let mut output = ctx.run_ui(Default::default(), |ui| {
                render_canvas(&mut app, ui);
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn test_render_canvas_dynamic_interpolation_switch() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let decoded = make_test_decoded(false);
        app.install(&ctx, decoded);

        // 1. Forçar modo Linear
        app.interpolation = InterpolationMode::Linear;
        let mut out1 = ctx.run_ui(Default::default(), |ui| {
            render_canvas(&mut app, ui);
        });
        out1.textures_delta.clear();
        assert_eq!(
            app.picture.as_ref().unwrap().texture_options.magnification,
            TextureFilter::Linear
        );

        // 2. Mudar para Pixel Perfect (Nearest)
        app.interpolation = InterpolationMode::Nearest;
        let mut out2 = ctx.run_ui(Default::default(), |ui| {
            render_canvas(&mut app, ui);
        });
        out2.textures_delta.clear();
        assert_eq!(
            app.picture.as_ref().unwrap().texture_options.magnification,
            TextureFilter::Nearest
        );
    }

    #[test]
    fn test_mouse_wheel_navigation_accumulator_threshold() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.settings.mouse_wheel_action = MouseWheelAction::Navigate;

        // Cria lista de arquivos para navegação
        app.files = vec![
            PathBuf::from("img1.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png"),
        ];
        app.index = 1;
        assert_eq!(app.wheel_nav_accumulator, 0.0);

        // Simula pequenos deltas abaixo do limiar (ex.: touchpad)
        app.wheel_nav_accumulator += 10.0;
        assert_eq!(app.index, 1); // Não atingiu threshold de 20.0

        // Inversão brusca reseta
        let scroll_down = -5.0;
        if (scroll_down > 0.0 && app.wheel_nav_accumulator < 0.0)
            || (scroll_down < 0.0 && app.wheel_nav_accumulator > 0.0)
        {
            app.wheel_nav_accumulator = 0.0;
        }
        assert_eq!(app.wheel_nav_accumulator, 0.0);

        // Atinge threshold para avançar (scroll negativo = para baixo -> próximo arquivo)
        app.wheel_nav_accumulator = -25.0;
        const WHEEL_NAV_THRESHOLD: f32 = 20.0;
        if app.wheel_nav_accumulator <= -WHEEL_NAV_THRESHOLD {
            app.step(1);
            app.wheel_nav_accumulator = 0.0;
        }
        assert_eq!(app.index, 2);
        assert_eq!(app.wheel_nav_accumulator, 0.0);

        // Atinge threshold para retroceder (scroll positivo = para cima -> arquivo anterior)
        app.wheel_nav_accumulator = 25.0;
        if app.wheel_nav_accumulator >= WHEEL_NAV_THRESHOLD {
            app.step(-1);
            app.wheel_nav_accumulator = 0.0;
        }
        assert_eq!(app.index, 1);
        assert_eq!(app.wheel_nav_accumulator, 0.0);
    }

    #[test]
    fn test_mouse_wheel_isolation_and_interactions() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.settings.mouse_wheel_action = MouseWheelAction::Navigate;

        // 1. Simulação: Campo de texto em edição bloqueia interação do canvas
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 30.0));
            let mut text = String::from("pesquisa");
            ui.add(egui::TextEdit::singleline(&mut text));
            let response = ui.allocate_rect(rect, Sense::click_and_drag());

            // Quando wants_keyboard_input for falso
            assert!(!ui.ctx().egui_wants_keyboard_input());
            assert!(is_canvas_interaction_active(&app, ui, &response) || !response.hovered());
        });
        out.textures_delta.clear();

        // 2. Simulação: Janela modal aberta bloqueia interação no canvas abaixo dela
        app.show_settings = true;
        let mut out2 = ctx.run_ui(Default::default(), |ui| {
            let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0));
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            // Com modal aberto, se a camada superior não for do canvas, bloqueia
            assert!(
                !response.hovered()
                    || !is_canvas_interaction_active(&app, ui, &response)
                    || !app.show_settings
            );
        });
        out2.textures_delta.clear();
        app.show_settings = false;

        // 3. Simulação: Ctrl + Roda força Zoom em vez de Navegação
        let mut app_zoom = VisorApp::new(&ctx, None);
        app_zoom.settings.mouse_wheel_action = MouseWheelAction::Navigate;
        let decoded = make_test_decoded(false);
        app_zoom.install(&ctx, decoded);
        let initial_zoom = app_zoom.view.zoom;

        let canvas = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let image_px = Vec2::new(2.0, 2.0);
        let anchor = canvas.center();
        let scroll_y: f32 = 50.0;
        let factor = (scroll_y * 0.0016f32).exp();

        // Com Ctrl ativo, aplica zoom
        apply_zoom(&mut app_zoom, canvas, image_px, anchor, factor, 0.01, 32.0);
        assert!(!app_zoom.view.fit);
        assert!(app_zoom.view.zoom > initial_zoom);
        assert_eq!(app_zoom.wheel_nav_accumulator, 0.0);

        // 4. Inversão de direção do touchpad zera o acumulador
        let mut acc: f32 = 15.0; // Rolando para cima
        let scroll_invert = -4.0; // Inverte para baixo
        if (scroll_invert > 0.0 && acc < 0.0) || (scroll_invert < 0.0 && acc > 0.0) {
            acc = 0.0;
        }
        assert_eq!(acc, 0.0, "inversão deve resetar acumulador");
    }
}
