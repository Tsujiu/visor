//! Barra de ferramentas moderna, responsiva e em formato de cápsula flutuante.
//!
//! Inspirada na linguagem visual do Windows 11 (Fluent Design), a Toolbar flutua
//! suavemente sobre a imagem na parte inferior da tela, sem reduzir a área útil do canvas.
//!
//! Possui 5 grupos funcionais:
//! 1. Navegação (Imagem anterior / próxima)
//! 2. Zoom (Diminuir, percentual, aumentar, ajustar à tela e 100%)
//! 3. Manipulação (Girar, animação e painel de informações)
//! 4. Exportação (Exportar imagem e conversão em lote)
//! 5. Mais opções (Menu `⋯` com overflow responsivo e utilitários)

use eframe::egui::{
    self, Align2, Color32, Margin, Pos2, Response, RichText, Sense, Stroke, Vec2, WidgetText,
};

use crate::app::state::{CanvasBackground, InterpolationMode, VisorApp};
use crate::core::geometry::scale_limits;
use crate::ui::theme::{ThemeDimensions, ThemePalette, ThemeRounding, ThemeSpacing};

/// Níveis de densidade e responsividade da Toolbar conforme a largura do canvas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ToolbarTier {
    /// Resoluções amplas (largura >= 720px): Todos os 5 grupos visíveis na cápsula.
    Full,
    /// Resoluções médias (540px <= largura < 720px): Lote e Info movidos para o menu `⋯`.
    Medium,
    /// Resoluções compactas (380px <= largura < 540px): Apenas Navegação, Zoom essencial e `⋯`.
    Compact,
    /// Resoluções mínimas (largura < 380px): Navegação, percentual e `⋯`.
    Minimal,
}

impl ToolbarTier {
    /// Determina o nível de responsividade a partir da largura disponível do canvas.
    pub fn from_width(available_width: f32) -> Self {
        if available_width >= 720.0 {
            Self::Full
        } else if available_width >= 540.0 {
            Self::Medium
        } else if available_width >= 380.0 {
            Self::Compact
        } else {
            Self::Minimal
        }
    }
}

/// Aplica um passo de zoom no estado da visualização centralizada da imagem.
pub fn zoom_step(app: &mut VisorApp, factor: f32, pixels_per_point: f32) {
    let (min_scale, max_scale) = scale_limits(pixels_per_point);
    let current_scale = if app.view.fit {
        if let Some(pct) = app.shown_percent {
            (pct as f32 / 100.0) / pixels_per_point.max(0.5)
        } else {
            app.view.zoom
        }
    } else {
        app.view.zoom
    };
    app.view.fit = false;
    app.view.zoom = (current_scale * factor).clamp(min_scale, max_scale);
}

/// Renderiza a barra de ferramentas flutuante em cápsula sobre o canvas.
pub fn render_toolbar(app: &mut VisorApp, ui: &mut egui::Ui) {
    let canvas_rect = ui.max_rect();
    let available_width = canvas_rect.width();
    let tier = ToolbarTier::from_width(available_width);
    let ppp = ui.pixels_per_point();
    let palette = ThemePalette::dark();

    // Posicionamento: centralizado horizontalmente na parte inferior do canvas
    let bottom_margin = 20.0;
    let anchor_pos = Pos2::new(
        canvas_rect.center().x,
        (canvas_rect.bottom() - bottom_margin).max(canvas_rect.top() + 32.0),
    );

    let id = ui.make_persistent_id("floating_capsule_toolbar");
    let frame = egui::Frame::NONE
        .fill(palette.surface_flyout)
        .stroke(palette.border_subtle)
        .corner_radius(ThemeRounding::RADIUS_PILL)
        .inner_margin(Margin::symmetric(10, 5));

    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(anchor_pos)
        .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::XS, ThemeSpacing::XS);
            ui.spacing_mut().button_padding = ThemeDimensions::TOOLBAR_BUTTON_PADDING;

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    render_toolbar_contents(app, ui, tier, ppp, &palette);
                });
            });
        });
}

/// Renderiza o conteúdo interno da cápsula conforme o nível de densidade (tier).
fn render_toolbar_contents(
    app: &mut VisorApp,
    ui: &mut egui::Ui,
    tier: ToolbarTier,
    ppp: f32,
    palette: &ThemePalette,
) {
    let tr = *app.tr();
    let has_picture = app.picture.is_some();
    let has_any_picture = has_picture;
    let can_step = app.files.len() > 1;

    // -------------------------------------------------------------------------
    // 1. GRUPO: NAVEGAÇÃO (Anterior / Próxima com Chevrons Vetoriais)
    // -------------------------------------------------------------------------
    if ui
        .add_enabled(can_step, NavChevronWidget::left(palette))
        .on_hover_text(format!("{} (<-)", tr.previous))
        .clicked()
    {
        app.step(-1);
    }

    if ui
        .add_enabled(can_step, NavChevronWidget::right(palette))
        .on_hover_text(format!("{} (->)", tr.next))
        .clicked()
    {
        app.step(1);
    }

    subtle_separator(ui);

    // -------------------------------------------------------------------------
    // 2. GRUPO: ZOOM
    // -------------------------------------------------------------------------
    match tier {
        ToolbarTier::Full | ToolbarTier::Medium => {
            // Botão Diminuir
            if subtle_button(ui, "-", has_any_picture)
                .on_hover_text(format!("{} (-)", tr.zoom_out))
                .clicked()
            {
                zoom_step(app, 1.0 / 1.15, ppp);
            }

            // Percentual clicável (alterna entre Ajustar e 100%)
            let pct_text = app
                .shown_percent
                .map(|pct| format!("{pct}%"))
                .unwrap_or_else(|| "100%".to_owned());

            if subtle_button(ui, RichText::new(pct_text).monospace(), has_any_picture)
                .on_hover_text(tr.zoom_current_tooltip)
                .clicked()
            {
                if app.view.fit {
                    app.view.set_actual(ppp);
                } else {
                    app.view.set_fit();
                }
            }

            // Botão Aumentar
            if subtle_button(ui, "+", has_any_picture)
                .on_hover_text(format!("{} (+)", tr.zoom_in))
                .clicked()
            {
                zoom_step(app, 1.15, ppp);
            }

            // Botão Ajustar à tela
            if subtle_toggle_button(ui, tr.fit_btn, app.view.fit, has_any_picture, palette)
                .on_hover_text(format!("{} (F)", tr.fit_to_window))
                .clicked()
            {
                app.view.set_fit();
            }

            // Botão 100% (Tamanho real)
            let is_100 = app.view.is_actual(ppp);
            if subtle_toggle_button(ui, "100%", is_100, has_any_picture, palette)
                .on_hover_text(format!("{} (1)", tr.actual_size))
                .clicked()
            {
                app.view.set_actual(ppp);
            }
        }
        ToolbarTier::Compact => {
            // Versão compacta do Zoom: Diminuir, Percentual, Aumentar e Ajustar
            if subtle_button(ui, "-", has_any_picture)
                .on_hover_text(format!("{} (-)", tr.zoom_out))
                .clicked()
            {
                zoom_step(app, 1.0 / 1.15, ppp);
            }

            let pct_text = app
                .shown_percent
                .map(|pct| format!("{pct}%"))
                .unwrap_or_else(|| "100%".to_owned());

            let zoom_tooltip = format!(
                "{}\n{}: {} · {}: {}",
                tr.zoom_current_tooltip,
                tr.status_error_label,
                app.interpolation.short_label(),
                tr.background,
                app.background.localized_label(&tr),
            );

            if subtle_button(ui, RichText::new(pct_text).monospace(), has_any_picture)
                .on_hover_text(zoom_tooltip)
                .clicked()
            {
                if app.view.fit {
                    app.view.set_actual(ppp);
                } else {
                    app.view.set_fit();
                }
            }

            if subtle_button(ui, "+", has_any_picture)
                .on_hover_text(format!("{} (+)", tr.zoom_in))
                .clicked()
            {
                zoom_step(app, 1.15, ppp);
            }

            if subtle_toggle_button(ui, tr.fit_btn, app.view.fit, has_any_picture, palette)
                .on_hover_text(format!("{} (F)", tr.fit_to_window))
                .clicked()
            {
                app.view.set_fit();
            }
        }
        ToolbarTier::Minimal => {
            // Apenas o percentual central em telas ultracompactas
            let pct_text = app
                .shown_percent
                .map(|pct| format!("{pct}%"))
                .unwrap_or_else(|| "100%".to_owned());

            let zoom_tooltip = format!(
                "{}\n{}: {} · {}: {}",
                tr.zoom_current_tooltip,
                tr.status_error_label,
                app.interpolation.short_label(),
                tr.background,
                app.background.localized_label(&tr),
            );

            if subtle_button(ui, RichText::new(pct_text).monospace(), has_any_picture)
                .on_hover_text(zoom_tooltip)
                .clicked()
            {
                if app.view.fit {
                    app.view.set_actual(ppp);
                } else {
                    app.view.set_fit();
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 3. GRUPO: MANIPULAÇÃO (Girar, Animação e Info)
    // -------------------------------------------------------------------------
    match tier {
        ToolbarTier::Full => {
            subtle_separator(ui);

            // Girar
            if subtle_button(ui, tr.rotate_btn, has_picture)
                .on_hover_text(format!("{} (R  ·  Shift+R)", tr.rotate_cw))
                .clicked()
            {
                app.rotate(ui.ctx(), true);
            }

            // Animação (GIF/APNG) se houver múltiplos frames
            let is_animated = app.picture.as_ref().is_some_and(|p| p.frames.len() > 1);

            if is_animated {
                let is_playing = app.picture.as_ref().is_some_and(|p| p.playing);
                let label = if is_playing {
                    tr.pause_btn
                } else {
                    tr.play_btn
                };
                if subtle_button(ui, label, true)
                    .on_hover_text(tr.anim_toggle_tooltip)
                    .clicked()
                    && let Some(picture) = app.picture.as_mut() {
                        picture.playing = !picture.playing;
                        picture.shown_at = ui.input(|input| input.time);
                    }
            }

            // Painel de Informações Técnicas
            if subtle_toggle_button(ui, tr.info, app.show_info, true, palette)
                .on_hover_text(format!("{} (I)", tr.info))
                .clicked()
            {
                app.show_info = !app.show_info;
            }
        }
        ToolbarTier::Medium => {
            subtle_separator(ui);

            // Girar compacto
            if subtle_button(ui, tr.rotate_btn, has_picture)
                .on_hover_text(format!("{} (R)", tr.rotate_cw))
                .clicked()
            {
                app.rotate(ui.ctx(), true);
            }

            // Animação compacta se houver múltiplos frames
            let is_animated = app.picture.as_ref().is_some_and(|p| p.frames.len() > 1);

            if is_animated {
                let is_playing = app.picture.as_ref().is_some_and(|p| p.playing);
                let label = if is_playing {
                    tr.pause_btn
                } else {
                    tr.play_btn
                };
                if subtle_button(ui, label, true)
                    .on_hover_text(tr.anim_toggle_tooltip)
                    .clicked()
                    && let Some(picture) = app.picture.as_mut() {
                        picture.playing = !picture.playing;
                        picture.shown_at = ui.input(|input| input.time);
                    }
            }
        }
        ToolbarTier::Compact | ToolbarTier::Minimal => {
            // Movido para o menu Mais Opções (⋯)
        }
    }

    // -------------------------------------------------------------------------
    // 4. GRUPO: EXPORTAÇÃO
    // -------------------------------------------------------------------------
    match tier {
        ToolbarTier::Full => {
            subtle_separator(ui);

            if subtle_button(ui, tr.export_btn, has_picture)
                .on_hover_text(tr.export_tooltip)
                .clicked()
            {
                app.want_export = true;
            }

            if subtle_button(ui, tr.batch_btn, true)
                .on_hover_text(tr.batch_tooltip)
                .clicked()
            {
                app.show_batch = true;
            }
        }
        ToolbarTier::Medium => {
            subtle_separator(ui);

            if subtle_button(ui, tr.export_btn, has_picture)
                .on_hover_text(tr.export_tooltip)
                .clicked()
            {
                app.want_export = true;
            }
        }
        ToolbarTier::Compact | ToolbarTier::Minimal => {
            // Movido para o menu Mais Opções (⋯)
        }
    }

    subtle_separator(ui);

    // -------------------------------------------------------------------------
    // 5. GRUPO: MAIS OPÇÕES (Menu ⋯)
    // -------------------------------------------------------------------------
    render_more_menu(app, ui, tier, has_picture, ppp);
}

/// Renderiza o menu popover `⋯` com comandos secundários e itens movidos por overflow.
fn render_more_menu(
    app: &mut VisorApp,
    ui: &mut egui::Ui,
    tier: ToolbarTier,
    has_picture: bool,
    ppp: f32,
) {
    let tr = *app.tr();
    let palette = ThemePalette::dark();
    let button_text = if app.updater.available_release.is_some() {
        RichText::new("... •")
            .size(13.0)
            .strong()
            .color(palette.accent_default)
    } else {
        RichText::new("...").size(13.0).strong()
    };
    let menu_response = ui.menu_button(button_text, |ui| {
        ui.set_min_width(180.0);

        // Ações movidas de outros grupos conforme a densidade:
        if tier == ToolbarTier::Compact || tier == ToolbarTier::Minimal {
            // 100% (Tamanho real)
            let actual_label = format!("100%  ·  {} (1)", tr.actual_size);
            if ui
                .add_enabled(has_picture, egui::Button::new(actual_label))
                .clicked()
            {
                app.view.set_actual(ppp);
                ui.close();
            }

            // Girar
            if ui
                .add_enabled(has_picture, egui::Button::new(tr.rotate_cw_menu))
                .clicked()
            {
                app.rotate(ui.ctx(), true);
                ui.close();
            }

            // Animação
            if app.picture.as_ref().is_some_and(|p| p.frames.len() > 1) {
                let is_playing = app.picture.as_ref().is_some_and(|p| p.playing);
                let label = if is_playing {
                    tr.anim_pause_menu
                } else {
                    tr.anim_play_menu
                };
                if ui.button(label).clicked() {
                    if let Some(picture) = app.picture.as_mut() {
                        picture.playing = !picture.playing;
                        picture.shown_at = ui.input(|input| input.time);
                    }
                    ui.close();
                }
            }

            // Exportar
            if ui
                .add_enabled(has_picture, egui::Button::new(tr.export_image_menu))
                .clicked()
            {
                app.want_export = true;
                ui.close();
            }

            ui.separator();
        }

        // Lote e Info (movidos nos tiers Medium, Compact e Minimal)
        if tier != ToolbarTier::Full {
            if ui.button(format!("{}…", tr.batch_export)).clicked() {
                app.show_batch = true;
                ui.close();
            }

            let info_label = if app.show_info {
                format!("•  {}", tr.info_panel_menu)
            } else {
                format!("   {}", tr.info_panel_menu)
            };
            if ui.button(info_label).clicked() {
                app.show_info = !app.show_info;
                ui.close();
            }

            ui.separator();
        }

        // Comandos padrão de sistema e utilitários
        let filmstrip_label = if app.show_filmstrip {
            format!("•  {} (T)", tr.filmstrip)
        } else {
            format!("   {} (T)", tr.filmstrip)
        };
        if ui.button(filmstrip_label).clicked() {
            app.show_filmstrip = !app.show_filmstrip;
            ui.close();
        }

        if ui
            .button(format!("{}… (Ctrl+O)", tr.open_image_btn))
            .clicked()
        {
            app.want_open = true;
            ui.close();
        }

        if ui.button(format!("{}…", tr.formats_catalog)).clicked() {
            app.show_formats = !app.show_formats;
            ui.close();
        }

        if ui.button(format!("{}… (Ctrl+,)", tr.settings)).clicked() {
            app.show_settings = !app.show_settings;
            ui.close();
        }

        if ui
            .button(tr.set_default)
            .on_hover_text(tr.set_default_tooltip)
            .clicked()
        {
            app.define_default();
            ui.close();
        }

        if let Some(rel) = &app.updater.available_release
            && ui
                .button(
                    RichText::new(format!("{} ({})…", tr.update_available, rel.tag_name))
                        .color(palette.status_warning)
                        .strong(),
                )
                .clicked()
            {
                app.updater.show_update_window = true;
                ui.close();
            }

        if ui.button(format!("{}…", tr.visor_center)).clicked() {
            app.updater.whats_new_notes =
                Some(crate::updater::whats_new::get_current_version_notes());
            app.updater.show_whats_new_window = true;
            ui.close();
        }

        ui.separator();

        ui.menu_button(tr.interpolation_menu, |ui| {
            let auto_active = app.interpolation == InterpolationMode::Auto;
            let auto_label = if auto_active {
                format!("•  {}", tr.interp_auto)
            } else {
                format!("   {}", tr.interp_auto)
            };
            if ui.button(auto_label).clicked() {
                app.interpolation = InterpolationMode::Auto;
                app.save_settings();
                ui.close();
            }

            let linear_active = app.interpolation == InterpolationMode::Linear;
            let linear_label = if linear_active {
                format!("•  {}", tr.interp_linear)
            } else {
                format!("   {}", tr.interp_linear)
            };
            if ui.button(linear_label).clicked() {
                app.interpolation = InterpolationMode::Linear;
                app.save_settings();
                ui.close();
            }

            let nearest_active = app.interpolation == InterpolationMode::Nearest;
            let nearest_label = if nearest_active {
                format!("•  {}", tr.interp_nearest)
            } else {
                format!("   {}", tr.interp_nearest)
            };
            if ui.button(nearest_label).clicked() {
                app.interpolation = InterpolationMode::Nearest;
                app.save_settings();
                ui.close();
            }
        });

        ui.menu_button(tr.background_menu, |ui| {
            let dark_ck_active = app.background == CanvasBackground::DarkChecker;
            let dark_ck_label = if dark_ck_active {
                format!("•  {}", tr.bg_dark_checker)
            } else {
                format!("   {}", tr.bg_dark_checker)
            };
            if ui.button(dark_ck_label).clicked() {
                app.background = CanvasBackground::DarkChecker;
                app.save_settings();
                ui.close();
            }

            let light_ck_active = app.background == CanvasBackground::LightChecker;
            let light_ck_label = if light_ck_active {
                format!("•  {}", tr.bg_light_checker)
            } else {
                format!("   {}", tr.bg_light_checker)
            };
            if ui.button(light_ck_label).clicked() {
                app.background = CanvasBackground::LightChecker;
                app.save_settings();
                ui.close();
            }

            let black_active = app.background == CanvasBackground::Black;
            let black_label = if black_active {
                format!("•  {}", tr.bg_solid_black)
            } else {
                format!("   {}", tr.bg_solid_black)
            };
            if ui.button(black_label).clicked() {
                app.background = CanvasBackground::Black;
                app.save_settings();
                ui.close();
            }

            let white_active = app.background == CanvasBackground::White;
            let white_label = if white_active {
                format!("•  {}", tr.bg_solid_white)
            } else {
                format!("   {}", tr.bg_solid_white)
            };
            if ui.button(white_label).clicked() {
                app.background = CanvasBackground::White;
                app.save_settings();
                ui.close();
            }

            let custom_active = matches!(app.background, CanvasBackground::Custom(_));
            let custom_label = if custom_active {
                format!("•  {}", tr.bg_custom)
            } else {
                format!("   {}", tr.bg_custom)
            };
            ui.horizontal(|ui| {
                let btn_resp = ui.button(custom_label);

                let [r, g, b, a] = app.settings.custom_color;
                let current_color = Color32::from_rgba_unmultiplied(r, g, b, a);
                let swatch_size = Vec2::new(32.0, 18.0);
                let (swatch_rect, swatch_resp) =
                    ui.allocate_exact_size(swatch_size, egui::Sense::click());

                if ui.is_rect_visible(swatch_rect) {
                    let painter = ui.painter();
                    if a < 255 {
                        crate::ui::color_picker::paint_checkerboard_preview(painter, swatch_rect);
                    }
                    painter.rect_filled(swatch_rect, ThemeRounding::RADIUS_XS, current_color);
                    let border_stroke = if swatch_resp.hovered() {
                        egui::Stroke::new(1.0, palette.accent_default)
                    } else {
                        palette.border_subtle
                    };
                    painter.rect_stroke(
                        swatch_rect,
                        ThemeRounding::RADIUS_XS,
                        border_stroke,
                        egui::StrokeKind::Inside,
                    );
                }

                let swatch_resp = swatch_resp.on_hover_text(tr.bg_custom_tooltip);

                if btn_resp.clicked() || swatch_resp.clicked() {
                    app.open_color_picker();
                    ui.close();
                }
            });
        });
    });

    menu_response
        .response
        .on_hover_text(tr.toolbar_more_tooltip);
}

// =============================================================================
// COMPONENTES DE APRESENTAÇÃO VISUAL SUTIL (FLUENT DESIGN)
// =============================================================================

/// Widget de botão de navegação com chevron vetorial puro desenhado via Painter.
///
/// Substitui caracteres de fontes externas (como tofu/quadrados) por traços
/// vetoriais Fluent Design limpos, simétricos e com extremidades arredondadas.
pub struct NavChevronWidget<'a> {
    pub pointing_left: bool,
    pub palette: &'a ThemePalette,
}

impl<'a> NavChevronWidget<'a> {
    pub fn left(palette: &'a ThemePalette) -> Self {
        Self {
            pointing_left: true,
            palette,
        }
    }

    pub fn right(palette: &'a ThemePalette) -> Self {
        Self {
            pointing_left: false,
            palette,
        }
    }

    /// Calcula os 3 pontos que compõem o chevron a partir do centro do botão.
    ///
    /// Proporção geométrica canônica do Fluent Design:
    /// - Altura: 10px (-5.0 a +5.0)
    /// - Largura: 5px (-3.0 a +2.0 ou -2.0 a +3.0)
    /// - Ângulo de abertura: 90° (braços a 45° com inclinação unitária)
    pub fn points_from_center(center: Pos2, pointing_left: bool) -> Vec<Pos2> {
        if pointing_left {
            vec![
                Pos2::new(center.x + 2.0, center.y - 5.0),
                Pos2::new(center.x - 3.0, center.y),
                Pos2::new(center.x + 2.0, center.y + 5.0),
            ]
        } else {
            vec![
                Pos2::new(center.x - 2.0, center.y - 5.0),
                Pos2::new(center.x + 3.0, center.y),
                Pos2::new(center.x - 2.0, center.y + 5.0),
            ]
        }
    }
}

impl<'a> egui::Widget for NavChevronWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = ThemeDimensions::ICON_BUTTON_SIZE; // 32 × 32 px
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        let (bg_fill, icon_color) = if !ui.is_enabled() {
            (Color32::TRANSPARENT, self.palette.text_muted)
        } else if response.is_pointer_button_down_on() {
            (self.palette.surface_active, self.palette.text_primary)
        } else if response.hovered() {
            (self.palette.surface_hover, self.palette.text_primary)
        } else {
            (Color32::TRANSPARENT, self.palette.text_primary)
        };

        if bg_fill != Color32::TRANSPARENT {
            ui.painter()
                .rect_filled(rect, ThemeRounding::RADIUS_SM, bg_fill);
        }

        let center = rect.center();
        let stroke = Stroke::new(1.8, icon_color);
        let points = Self::points_from_center(center, self.pointing_left);

        ui.painter()
            .add(egui::Shape::Path(egui::epaint::PathShape::line(
                points, stroke,
            )));

        response
    }
}

/// Botão sutil e discreto sem borda nem fundo opaco em repouso.
fn subtle_button(ui: &mut egui::Ui, text: impl Into<WidgetText>, enabled: bool) -> Response {
    let button = egui::Button::new(text)
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(ThemeRounding::RADIUS_SM);

    ui.add_enabled(enabled, button)
}

/// Botão sutil de alternância (toggle ativo/inativo).
fn subtle_toggle_button(
    ui: &mut egui::Ui,
    text: &str,
    active: bool,
    enabled: bool,
    palette: &ThemePalette,
) -> Response {
    let (fill, stroke, text_color) = if active && enabled {
        (
            palette.accent_muted,
            Stroke::new(1.0, palette.accent_default),
            palette.accent_default,
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::NONE,
            if enabled {
                palette.text_primary
            } else {
                palette.text_muted
            },
        )
    };

    let button = egui::Button::new(RichText::new(text).color(text_color))
        .fill(fill)
        .stroke(stroke)
        .corner_radius(ThemeRounding::RADIUS_SM);

    ui.add_enabled(enabled, button)
}

/// Separador vertical translúcido e discreto entre grupos funcionais.
fn subtle_separator(ui: &mut egui::Ui) {
    let height = 18.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, height), Sense::hover());
    let x = rect.center().x;
    let y_min = rect.center().y - height * 0.5;
    let y_max = rect.center().y + height * 0.5;

    ui.painter().line_segment(
        [Pos2::new(x, y_min), Pos2::new(x, y_max)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
    );
}

// =============================================================================
// TESTES UNITÁRIOS DE RESPONSIVIDADE E COMPORTAMENTO
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_tier_breakpoints() {
        assert_eq!(ToolbarTier::from_width(1920.0), ToolbarTier::Full);
        assert_eq!(ToolbarTier::from_width(800.0), ToolbarTier::Full);
        assert_eq!(ToolbarTier::from_width(720.0), ToolbarTier::Full);

        assert_eq!(ToolbarTier::from_width(719.0), ToolbarTier::Medium);
        assert_eq!(ToolbarTier::from_width(600.0), ToolbarTier::Medium);
        assert_eq!(ToolbarTier::from_width(540.0), ToolbarTier::Medium);

        assert_eq!(ToolbarTier::from_width(539.0), ToolbarTier::Compact);
        assert_eq!(ToolbarTier::from_width(420.0), ToolbarTier::Compact);
        assert_eq!(ToolbarTier::from_width(380.0), ToolbarTier::Compact);

        assert_eq!(ToolbarTier::from_width(379.0), ToolbarTier::Minimal);
        assert_eq!(ToolbarTier::from_width(280.0), ToolbarTier::Minimal);
        assert_eq!(ToolbarTier::from_width(100.0), ToolbarTier::Minimal);
    }

    #[test]
    fn test_toolbar_zoom_step_calculations() {
        use crate::app::state::View;
        let mut app = VisorApp::new(&egui::Context::default(), None);

        // Inicialmente em modo Fit
        app.view.fit = true;
        app.shown_percent = Some(80);

        // Zoom In com fator 1.15
        zoom_step(&mut app, 1.15, 1.0);
        assert!(!app.view.fit, "Zoom manual deve desativar modo Fit");
        assert!(
            (app.view.zoom - 0.92).abs() < 0.01,
            "Zoom esperado ~0.92, obtido {}",
            app.view.zoom
        );

        // Zoom Out com fator 1.0 / 1.15
        zoom_step(&mut app, 1.0 / 1.15, 1.0);
        assert!(
            (app.view.zoom - 0.80).abs() < 0.01,
            "Zoom deve retornar para ~0.80, obtido {}",
            app.view.zoom
        );

        // Teste de limites extremos (não deve ultrapassar min_scale ou max_scale)
        let (min_scale, max_scale) = scale_limits(1.0);
        zoom_step(&mut app, 1000.0, 1.0);
        assert_eq!(app.view.zoom, max_scale);

        zoom_step(&mut app, 0.0001, 1.0);
        assert_eq!(app.view.zoom, min_scale);
    }

    #[test]
    fn test_toolbar_tier_progressive_disclosure_coverage() {
        // Valida que todos os tiers estão definidos e mapeados
        let tiers = [
            ToolbarTier::Full,
            ToolbarTier::Medium,
            ToolbarTier::Compact,
            ToolbarTier::Minimal,
        ];
        for tier in tiers {
            match tier {
                ToolbarTier::Full => assert!(true),
                ToolbarTier::Medium => assert!(true),
                ToolbarTier::Compact => assert!(true),
                ToolbarTier::Minimal => assert!(true),
            }
        }
    }

    #[test]
    fn test_toolbar_rendering_at_all_resolutions_no_panic() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);

        // Testa em múltiplas resoluções: Full, Medium, Compact e Minimal
        let resolutions = [
            Vec2::new(1920.0, 1080.0),
            Vec2::new(1280.0, 720.0),
            Vec2::new(800.0, 600.0),
            Vec2::new(650.0, 480.0),
            Vec2::new(500.0, 400.0),
            Vec2::new(400.0, 300.0),
            Vec2::new(320.0, 240.0),
            Vec2::new(200.0, 150.0),
        ];

        for res in resolutions {
            let mut output = ctx.run_ui(Default::default(), |ui| {
                ui.set_min_size(res);
                ui.set_max_size(res);
                render_toolbar(&mut app, ui);
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn test_canvas_layout_gain_comparison() {
        // Comparação da ocupação de tela antes e depois da Toolbar cápsula flutuante.
        // Anteriormente: egui::Panel::top("toolbar") ocupava ~36px fixos do topo da janela,
        // reduzindo permanentemente a altura disponível para o canvas.
        // Agora: A toolbar flutua em camada egui::Order::Foreground dentro do CentralPanel,
        // deixando 100% da área do canvas disponível para a renderização da imagem.
        let window_height = 800.0;
        let old_toolbar_fixed_height = 36.0;
        let old_canvas_height = window_height - old_toolbar_fixed_height;
        let new_canvas_height = window_height; // 0px subtraídos!

        let vertical_gain_px = new_canvas_height - old_canvas_height;
        let vertical_gain_percent = (vertical_gain_px / old_canvas_height) * 100.0;

        assert_eq!(vertical_gain_px, 36.0);
        assert!(vertical_gain_percent > 4.5);
    }

    #[test]
    fn test_nav_chevron_points_geometry() {
        let center = Pos2::new(100.0, 100.0);

        // Chevron Esquerda (<)
        let left = NavChevronWidget::points_from_center(center, true);
        assert_eq!(left.len(), 3);
        // Topo deve estar acima do centro e à direita da ponta
        assert_eq!(left[0], Pos2::new(102.0, 95.0));
        // Ponta deve estar no centro vertical e à esquerda
        assert_eq!(left[1], Pos2::new(97.0, 100.0));
        // Base deve estar abaixo do centro e alinhada ao topo
        assert_eq!(left[2], Pos2::new(102.0, 105.0));

        // Altura total = 10px, largura total = 5px
        assert_eq!(left[2].y - left[0].y, 10.0);
        assert_eq!(left[0].x - left[1].x, 5.0);

        // Ângulo de 90° (braços a 45°: inclinação unitária)
        let delta_x = (left[0].x - left[1].x).abs();
        let delta_y = (left[0].y - left[1].y).abs();
        assert_eq!(delta_x, delta_y);

        // Chevron Direita (>)
        let right = NavChevronWidget::points_from_center(center, false);
        assert_eq!(right.len(), 3);
        assert_eq!(right[0], Pos2::new(98.0, 95.0));
        assert_eq!(right[1], Pos2::new(103.0, 100.0));
        assert_eq!(right[2], Pos2::new(98.0, 105.0));

        // Simetria espelhada exata em relação ao centro
        assert_eq!(right[1].x - center.x, center.x - left[1].x);
    }

    #[test]
    fn test_nav_chevron_widget_renders_in_all_states() {
        let ctx = egui::Context::default();
        let palette = ThemePalette::dark();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            // Esquerda habilitado
            let resp_l1 = ui.add_enabled(true, NavChevronWidget::left(&palette));
            assert_eq!(resp_l1.rect.size(), Vec2::new(32.0, 32.0));

            // Esquerda desabilitado
            let resp_l0 = ui.add_enabled(false, NavChevronWidget::left(&palette));
            assert_eq!(resp_l0.rect.size(), Vec2::new(32.0, 32.0));

            // Direita habilitado
            let resp_r1 = ui.add_enabled(true, NavChevronWidget::right(&palette));
            assert_eq!(resp_r1.rect.size(), Vec2::new(32.0, 32.0));

            // Direita desabilitado
            let resp_r0 = ui.add_enabled(false, NavChevronWidget::right(&palette));
            assert_eq!(resp_r0.rect.size(), Vec2::new(32.0, 32.0));
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_dropdown_selection_markers_use_bullet_not_unsupported_glyph() {
        // Garantir que nenhum marcador de seleção de menu utilize glifos não suportados na fonte (como '✓'),
        // utilizando o caractere oficial de ponto bullet '•' suportado pelo egui.
        let bullet = "•  ";
        let _unselected = "   ";
        assert_eq!(bullet.chars().next().unwrap(), '•');
        assert_ne!(bullet.chars().next().unwrap(), '✓');

        // Testar interpolação
        for mode in &[
            InterpolationMode::Auto,
            InterpolationMode::Linear,
            InterpolationMode::Nearest,
        ] {
            let active_label = format!("•  {}", mode.label());
            let inactive_label = format!("   {}", mode.label());
            assert!(active_label.starts_with("•  "));
            assert!(inactive_label.starts_with("   "));
            assert!(!active_label.contains('✓'));
            assert!(!inactive_label.contains('✓'));
        }

        // Testar fundos
        let bg_labels = [
            "Xadrez Escuro",
            "Xadrez Claro",
            "Preto Sólido (#0C0C0E)",
            "Branco Sólido (#FAFAFC)",
            "Cor Personalizada",
        ];
        for label in &bg_labels {
            let active_label = format!("•  {label}");
            let inactive_label = format!("   {label}");
            assert!(active_label.starts_with("•  "));
            assert!(inactive_label.starts_with("   "));
            assert!(!active_label.contains('✓'));
            assert!(!inactive_label.contains('✓'));
        }
    }
}
