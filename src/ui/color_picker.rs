//! Janela modal/flutuante Fluent Design para seleção interativa da cor de fundo personalizada.
//!
//! Oferece:
//! 1. Pré-visualização ao vivo com código hexadecimal (#RRGGBB / #RRGGBBAA) e valores RGBA/opacidade.
//! 2. Seletor 2D integrado (gradiente de matiz + mapa de saturação/brilho) com canal alfa/opacidade.
//! 3. Entrada direta de código hexadecimal com parsing flexível (#RGB, #RGBA, #RRGGBB, #RRGGBBAA).
//! 4. Paleta de cores recomendadas de alto contraste e estéticas para fotografia e design.
//! 5. Atualização em tempo real do canvas do VISOR enquanto o usuário ajusta a cor.
//! 6. Redefinição para a cor padrão e botão de conclusão.

use eframe::egui::{self, Color32, Context, RichText, Vec2};

use crate::app::state::{CanvasBackground, VisorApp};
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};

/// Cores recomendadas predefinidas para fundo de imagem (neutros, escuros, fotografia e clássicos).
pub const PRESET_COLORS: &[(&str, Color32)] = &[
    ("Preto Puro (#000000)", Color32::from_rgb(0, 0, 0)),
    ("Grafite Escuro (#18181B)", Color32::from_rgb(24, 24, 27)),
    ("Cinza Carvão (#27272A)", Color32::from_rgb(39, 39, 42)),
    (
        "Cinza Médio 50% (#808080)",
        Color32::from_rgb(128, 128, 128),
    ),
    ("Cinza Claro (#E4E4E7)", Color32::from_rgb(228, 228, 231)),
    ("Branco Puro (#FFFFFF)", Color32::from_rgb(255, 255, 255)),
    ("Azul Noite (#0F172A)", Color32::from_rgb(15, 23, 42)),
    ("Azul Marinho (#1E3A8A)", Color32::from_rgb(30, 58, 138)),
    ("Azul Cobalto (#2563EB)", Color32::from_rgb(37, 99, 235)),
    ("Verde Floresta (#14532D)", Color32::from_rgb(20, 83, 45)),
    ("Verde Esmeralda (#059669)", Color32::from_rgb(5, 150, 105)),
    ("Sépia Escuro (#451A03)", Color32::from_rgb(69, 26, 3)),
    ("Terracota (#9A3412)", Color32::from_rgb(154, 52, 18)),
    ("Vinho Profundo (#4C0519)", Color32::from_rgb(76, 5, 25)),
    ("Roxo Meia-Noite (#3B0764)", Color32::from_rgb(59, 7, 100)),
    ("Padrão Visor (#2D3C50)", Color32::from_rgb(45, 60, 80)),
];

/// Interpreta uma string hexadecimal de cor (com ou sem `#`, formatos de 3, 4, 6 ou 8 dígitos).
pub fn parse_hex_color(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(Color32::from_rgb(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? * 17;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

/// Desenha uma grade de xadrez sutil para demonstrar transparência / canal alfa.
pub fn paint_checkerboard_preview(painter: &egui::Painter, rect: egui::Rect) {
    let cell_size = 6.0;
    let dark = Color32::from_gray(32);
    let light = Color32::from_gray(56);
    let cols = ((rect.width() / cell_size).ceil() as usize).max(1);
    let rows = ((rect.height() / cell_size).ceil() as usize).max(1);

    for r in 0..rows {
        for c in 0..cols {
            let min_x = rect.min.x + c as f32 * cell_size;
            let min_y = rect.min.y + r as f32 * cell_size;
            let max_x = (min_x + cell_size).min(rect.max.x);
            let max_y = (min_y + cell_size).min(rect.max.y);
            if min_x < max_x && min_y < max_y {
                let cell_rect =
                    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));
                let color = if (r + c) % 2 == 0 { dark } else { light };
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }
    }
}

/// Renderiza um botão de amostra de cor pré-definida.
fn render_preset_swatch(
    ui: &mut egui::Ui,
    color: Color32,
    tooltip: &str,
    is_selected: bool,
    palette: &ThemePalette,
) -> egui::Response {
    let size = Vec2::new(26.0, 26.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = ThemeRounding::RADIUS_XS;
        painter.rect_filled(rect, radius, color);

        let stroke = if is_selected {
            egui::Stroke::new(2.0, palette.accent_default)
        } else if resp.hovered() {
            egui::Stroke::new(1.0, palette.text_primary)
        } else {
            palette.border_subtle
        };
        painter.rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    }
    resp.on_hover_text(tooltip)
}

/// Renderiza a janela flutuante/modal de seleção de cor de fundo personalizada.
pub fn render_color_picker_window(app: &mut VisorApp, ctx: &Context) {
    if !app.show_color_picker {
        return;
    }

    let mut open = app.show_color_picker;
    let palette = ThemePalette::dark();
    let tr = *app.tr();
    let mut changed = false;
    let mut should_close = false;

    // Tecla Escape fecha a janela
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    let [mut r, mut g, mut b, mut a] = app.settings.custom_color;
    let mut current_color = Color32::from_rgba_unmultiplied(r, g, b, a);

    egui::Window::new(tr.color_picker_title)
        .open(&mut open)
        .default_size([380.0, 480.0])
        .min_size([340.0, 440.0])
        .resizable(false)
        .collapsible(false)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.input(|i| {
            i.raw
                .screen_rect
                .map(|rect| egui::pos2(rect.center().x, rect.center().y))
                .unwrap_or(egui::pos2(400.0, 300.0))
        }))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::SM, ThemeSpacing::MD);

            // 1. Card de Pré-visualização com código Hex e valores RGBA
            egui::Frame::NONE
                .fill(palette.surface_active)
                .stroke(palette.border_subtle)
                .corner_radius(ThemeRounding::RADIUS_MD)
                .inner_margin(egui::Margin::symmetric(
                    ThemeSpacing::MD as i8,
                    ThemeSpacing::SM as i8,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Amostra de cor grande
                        let swatch_size = Vec2::new(56.0, 36.0);
                        let (rect, _) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());
                        if ui.is_rect_visible(rect) {
                            let painter = ui.painter();
                            if a < 255 {
                                paint_checkerboard_preview(painter, rect);
                            }
                            painter.rect_filled(rect, ThemeRounding::RADIUS_SM, current_color);
                            painter.rect_stroke(
                                rect,
                                ThemeRounding::RADIUS_SM,
                                palette.border_default,
                                egui::StrokeKind::Inside,
                            );
                        }

                        ui.vertical(|ui| {
                            let hex_str = if a < 255 {
                                format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
                            } else {
                                format!("#{:02X}{:02X}{:02X}", r, g, b)
                            };
                            ui.label(
                                RichText::new(hex_str)
                                    .size(15.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "R: {r}  G: {g}  B: {b}  {}: {}%",
                                    tr.color_picker_opacity,
                                    (a as f32 / 2.55).round() as u8
                                ))
                                .size(11.5)
                                .color(palette.text_muted),
                            );
                        });
                    });
                });

            // 2. Seletor de Cores Visual Integrado (Gradiente 2D + Sliders de Matiz e Alfa)
            ui.label(
                RichText::new(tr.color_picker_adjust)
                    .size(12.5)
                    .strong()
                    .color(palette.text_secondary),
            );

            if egui::color_picker::color_picker_color32(
                ui,
                &mut current_color,
                egui::color_picker::Alpha::OnlyBlend,
            ) {
                [r, g, b, a] = current_color.to_srgba_unmultiplied();
                app.settings.custom_color = [r, g, b, a];
                app.color_picker_hex_input = if a < 255 {
                    format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
                } else {
                    format!("#{:02X}{:02X}{:02X}", r, g, b)
                };
                changed = true;
            }

            // 3. Entrada Hexadecimal Direta
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.color_picker_hex)
                        .size(12.0)
                        .color(palette.text_muted),
                );
                let hex_edit = ui.add(
                    egui::TextEdit::singleline(&mut app.color_picker_hex_input)
                        .desired_width(120.0)
                        .hint_text("#RRGGBB"),
                );
                if hex_edit.changed() {
                    let clean = app.color_picker_hex_input.trim();
                    if let Some(parsed) = parse_hex_color(clean) {
                        current_color = parsed;
                        [r, g, b, a] = current_color.to_srgba_unmultiplied();
                        app.settings.custom_color = [r, g, b, a];
                        changed = true;
                    }
                }
            });

            // 4. Paleta de Cores de Acesso Rápido (Presets Profissionais)
            ui.separator();
            ui.label(
                RichText::new(tr.color_picker_presets)
                    .size(12.0)
                    .strong()
                    .color(palette.text_secondary),
            );

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                for (name, preset_color) in PRESET_COLORS {
                    let swatch_btn = render_preset_swatch(
                        ui,
                        *preset_color,
                        *name,
                        current_color == *preset_color,
                        &palette,
                    );
                    if swatch_btn.clicked() {
                        current_color = *preset_color;
                        [r, g, b, a] = current_color.to_srgba_unmultiplied();
                        app.settings.custom_color = [r, g, b, a];
                        app.color_picker_hex_input = if a < 255 {
                            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
                        } else {
                            format!("#{:02X}{:02X}{:02X}", r, g, b)
                        };
                        changed = true;
                    }
                }
            });

            ui.add_space(4.0);
            ui.separator();

            // 5. Barra Inferior com Botões de Ação
            ui.horizontal(|ui| {
                if ui
                    .button(
                        RichText::new(tr.color_picker_reset)
                            .size(12.0)
                            .color(palette.text_muted),
                    )
                    .on_hover_text(tr.color_picker_reset_tooltip)
                    .clicked()
                {
                    current_color = Color32::from_rgb(45, 60, 80);
                    [r, g, b, a] = current_color.to_srgba_unmultiplied();
                    app.settings.custom_color = [r, g, b, a];
                    app.color_picker_hex_input = "#2D3C50".to_string();
                    changed = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        RichText::new(tr.color_picker_done)
                            .size(12.5)
                            .strong()
                            .color(palette.text_on_accent),
                    )
                    .fill(palette.accent_default)
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui.add(close_btn).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if should_close {
        open = false;
    }

    if changed {
        app.background = CanvasBackground::Custom(current_color);
        app.settings.background = CanvasBackground::Custom(current_color);
        app.save_settings();
    }

    app.show_color_picker = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid_formats() {
        // Formato 6 dígitos
        let red = parse_hex_color("#FF0000").expect("hex vermelho");
        assert_eq!(red, Color32::from_rgb(255, 0, 0));

        let blue_no_hash = parse_hex_color("0000FF").expect("hex azul sem hash");
        assert_eq!(blue_no_hash, Color32::from_rgb(0, 0, 255));

        // Formato 3 dígitos (shorthand)
        let white_short = parse_hex_color("#FFF").expect("hex branco shorthand");
        assert_eq!(white_short, Color32::from_rgb(255, 255, 255));

        let black_short = parse_hex_color("000").expect("hex preto shorthand");
        assert_eq!(black_short, Color32::from_rgb(0, 0, 0));

        // Formato 8 dígitos (RGBA)
        let semitransparent = parse_hex_color("#FF880080").expect("hex laranja semitransparente");
        assert_eq!(
            semitransparent,
            Color32::from_rgba_unmultiplied(255, 136, 0, 128)
        );

        // Formato 4 dígitos (RGBA shorthand)
        let short_rgba = parse_hex_color("#F008").expect("hex vermelho com alfa shorthand");
        assert_eq!(short_rgba, Color32::from_rgba_unmultiplied(255, 0, 0, 136));
    }

    #[test]
    fn test_parse_hex_color_invalid_formats() {
        assert!(parse_hex_color("").is_none());
        assert!(parse_hex_color("#").is_none());
        assert!(parse_hex_color("12").is_none());
        assert!(parse_hex_color("12345").is_none());
        assert!(parse_hex_color("1234567").is_none());
        assert!(parse_hex_color("123456789").is_none());
        assert!(parse_hex_color("ZZZZZZ").is_none());
        assert!(parse_hex_color("#GG0000").is_none());
    }

    #[test]
    fn test_preset_colors_integrity() {
        assert!(
            PRESET_COLORS.len() >= 12,
            "Deve conter ao menos 12 cores pré-definidas"
        );
        for (name, color) in PRESET_COLORS {
            assert!(!name.is_empty(), "Nome do preset não pode ser vazio");
            let [_, _, _, a] = color.to_srgba_unmultiplied();
            assert_eq!(a, 255, "Cores predefinidas devem ser sólidas");
        }
    }

    #[test]
    fn test_color_picker_open_state() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        assert!(!app.show_color_picker);

        app.open_color_picker();
        assert!(app.show_color_picker);
        assert!(matches!(app.background, CanvasBackground::Custom(_)));
        assert!(!app.color_picker_hex_input.is_empty());
    }

    #[test]
    fn test_render_color_picker_window_renders_when_open() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.open_color_picker();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_color_picker_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.show_color_picker);
    }

    #[test]
    fn test_render_color_picker_window_skips_when_closed() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_color_picker = false;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_color_picker_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(!app.show_color_picker);
    }
}
