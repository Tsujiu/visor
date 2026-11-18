//! Diálogo modal elegante para apoiar o desenvolvimento independente do VISOR.

use eframe::egui::{self, Color32, Context, RichText, Stroke};

use crate::app::VisorApp;
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};

/// Links oficiais de apoio ao projeto.
pub const SUPPORT_URL_INFINITEPAY: &str = "https://linknabio.gg/mutsujii";
pub const SUPPORT_URL_STRIPE_GLOBAL: &str = "https://donate.stripe.com/test_eVq3cu6eK5tO5yb2yc3VC04";
pub const SUPPORT_URL_STRIPE_BRL: &str = "https://donate.stripe.com/test_5kQ4gydHc5tO6Cf5Ko3VC05";

/// Renderiza o diálogo modal de apoio e doações caso esteja ativo.
pub fn render_support_dialog(app: &mut VisorApp, ctx: &Context) {
    if !app.show_support {
        return;
    }

    let tr = *app.tr();
    let palette = ThemePalette::dark();
    let mut should_close = false;

    let screen = ctx.input(|i| {
        i.raw.screen_rect.unwrap_or(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(1024.0, 768.0),
        ))
    });
    let modal_width = (screen.width() - 48.0).clamp(380.0, 520.0);
    let modal_max_height = (screen.height() - 64.0).clamp(380.0, 600.0);

    #[allow(deprecated)]
    let modal_response = egui::Modal::new(egui::Id::new("support_dialog_modal"))
        .backdrop_color(Color32::from_rgba_unmultiplied(8, 8, 12, 225))
        .frame(
            egui::Frame::NONE
                .fill(palette.surface_panel)
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                ))
                .corner_radius(ThemeRounding::RADIUS_LG)
                .inner_margin(egui::Margin::same(ThemeSpacing::LG as i8))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: Color32::from_black_alpha(180),
                }),
        )
        .show(ctx, |ui| {
            ui.set_width(modal_width);
            ui.set_max_width(modal_width);
            ui.set_max_height(modal_max_height);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

            // -------------------------------------------------------------
            // 1. CABEÇALHO DO DIÁLOGO
            // -------------------------------------------------------------
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(palette.accent_muted)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("VISOR · APOIO / SUPPORT")
                                .size(11.0)
                                .strong()
                                .color(palette.accent_default),
                        );
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        RichText::new("✕")
                            .size(13.0)
                            .color(palette.text_secondary),
                    )
                    .fill(Color32::TRANSPARENT)
                    .frame(false);

                    if ui
                        .add(close_btn)
                        .on_hover_text(tr.close_tooltip)
                        .clicked()
                    {
                        should_close = true;
                    }
                });
            });

            ui.add_space(ThemeSpacing::SM);
            ui.label(
                RichText::new(tr.support_title)
                    .size(17.0)
                    .strong()
                    .color(palette.text_primary),
            );

            ui.add_space(4.0);
            ui.label(
                RichText::new(tr.support_subtitle)
                    .size(12.0)
                    .color(palette.text_secondary),
            );

            ui.add_space(ThemeSpacing::MD);

            // -------------------------------------------------------------
            // 2. OPÇÃO 1: BRASIL (INFINITEPAY / PIX & CARTÃO)
            // -------------------------------------------------------------
            let teal_accent = Color32::from_rgb(0, 201, 167);
            let teal_border = Color32::from_rgba_unmultiplied(0, 201, 167, 75);

            egui::Frame::NONE
                .fill(palette.surface_card)
                .stroke(Stroke::new(1.0, teal_border))
                .corner_radius(ThemeRounding::RADIUS_MD)
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(0, 201, 167, 36))
                            .corner_radius(ThemeRounding::RADIUS_XS)
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(tr.support_brasil_badge)
                                        .size(10.0)
                                        .strong()
                                        .color(teal_accent),
                                );
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("BRL · Instantâneo")
                                    .size(11.0)
                                    .color(palette.text_muted),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(tr.support_brasil_title)
                            .size(14.0)
                            .strong()
                            .color(palette.text_primary),
                    );

                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(tr.support_brasil_desc)
                            .size(11.5)
                            .color(palette.text_secondary),
                    );

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        let pix_btn = egui::Button::new(
                            RichText::new(tr.support_brasil_btn)
                                .size(12.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(0, 168, 140))
                        .corner_radius(ThemeRounding::RADIUS_SM);

                        if ui
                            .add(pix_btn)
                            .on_hover_text(SUPPORT_URL_INFINITEPAY)
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(SUPPORT_URL_INFINITEPAY));
                        }

                        let stripe_brl_btn = egui::Button::new(
                            RichText::new(tr.support_stripe_brl_btn)
                                .size(11.5)
                                .color(palette.text_secondary),
                        );

                        if ui
                            .add(stripe_brl_btn)
                            .on_hover_text(SUPPORT_URL_STRIPE_BRL)
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(SUPPORT_URL_STRIPE_BRL));
                        }
                    });
                });

            ui.add_space(ThemeSpacing::SM + 2.0);

            // -------------------------------------------------------------
            // 3. OPÇÃO 2: GLOBAL (STRIPE · INTERNACIONAL)
            // -------------------------------------------------------------
            let purple_accent = Color32::from_rgb(120, 110, 255);
            let purple_border = Color32::from_rgba_unmultiplied(99, 91, 255, 75);

            egui::Frame::NONE
                .fill(palette.surface_card)
                .stroke(Stroke::new(1.0, purple_border))
                .corner_radius(ThemeRounding::RADIUS_MD)
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(99, 91, 255, 36))
                            .corner_radius(ThemeRounding::RADIUS_XS)
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(tr.support_global_badge)
                                        .size(10.0)
                                        .strong()
                                        .color(purple_accent),
                                );
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("USD · Worldwide")
                                    .size(11.0)
                                    .color(palette.text_muted),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(tr.support_global_title)
                            .size(14.0)
                            .strong()
                            .color(palette.text_primary),
                    );

                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(tr.support_global_desc)
                            .size(11.5)
                            .color(palette.text_secondary),
                    );

                    ui.add_space(10.0);
                    let stripe_btn = egui::Button::new(
                        RichText::new(tr.support_global_btn)
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(99, 91, 255))
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui
                        .add(stripe_btn)
                        .on_hover_text(SUPPORT_URL_STRIPE_GLOBAL)
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(SUPPORT_URL_STRIPE_GLOBAL));
                    }
                });

            // -------------------------------------------------------------
            // 4. RODAPÉ DE AGRADECIMENTO & FECHAR
            // -------------------------------------------------------------
            ui.add_space(ThemeSpacing::MD);
            ui.separator();
            ui.add_space(ThemeSpacing::XS);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.support_thank_you)
                        .size(11.5)
                        .italics()
                        .color(palette.text_muted),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new(tr.close).size(12.0))
                        .clicked()
                    {
                        should_close = true;
                    }
                });
            });
        });

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        should_close = true;
    }

    if modal_response.should_close() || should_close {
        app.show_support = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::AppLanguage;
    use eframe::egui::Context;

    #[test]
    fn test_support_dialog_skips_when_closed() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_support = false;

        render_support_dialog(&mut app, &ctx);
        assert!(!app.show_support);
    }

    #[test]
    fn test_support_dialog_renders_when_open() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_support = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_support_dialog(&mut app, ui.ctx());
        });
        output.textures_delta.clear();

        assert!(app.show_support);
    }

    #[test]
    fn test_support_dialog_all_languages_render_cleanly() {
        let ctx = Context::default();
        let languages = [
            AppLanguage::PtBr,
            AppLanguage::EnUs,
            AppLanguage::EsEs,
            AppLanguage::ZhCn,
            AppLanguage::JaJp,
            AppLanguage::DeDe,
            AppLanguage::FrFr,
            AppLanguage::ItIt,
            AppLanguage::RuRu,
        ];

        for lang in languages {
            let mut app = VisorApp::new(&ctx, None);
            app.settings.language = lang;
            app.show_support = true;

            let mut output = ctx.run_ui(Default::default(), |ui| {
                render_support_dialog(&mut app, ui.ctx());
            });
            output.textures_delta.clear();

            assert!(app.show_support);
        }
    }
}
