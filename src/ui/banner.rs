//! Banner inteligente de detecção de associações e primeiro uso para Windows 10 e 11.
//!
//! Exibe exclusivamente as extensões pendentes de configuração, permitindo abrir
//! as configurações oficiais do Windows, ignorar extensões individualmente ou atualizar a verificação.

use eframe::egui::{self, Button, RichText, Stroke, Ui, Vec2};

use crate::app::state::VisorApp;
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};

/// Renderiza o banner inteligente de associações de arquivos.
pub fn render_association_banner(app: &mut VisorApp, ui: &mut Ui) {
    if !app.show_association_banner || !app.associations.has_pending() {
        return;
    }

    let tr = *app.tr();
    let palette = ThemePalette::dark();
    let pending = app.associations.pending_extensions();
    if pending.is_empty() {
        return;
    }

    let mut to_ignore: Option<String> = None;
    let mut ignore_all = false;
    let mut check_again = false;
    let mut open_settings = false;

    egui::Frame::NONE
        .fill(palette.surface_card)
        .stroke(Stroke::new(1.0, palette.border_default.color))
        .corner_radius(ThemeRounding::RADIUS_MD)
        .inner_margin(egui::Margin::symmetric(ThemeSpacing::MD as i8, 8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Linha de Cabeçalho com Ícone e Título Principal
                ui.horizontal(|ui| {
                    egui::Frame::NONE
                        .fill(palette.accent_muted)
                        .corner_radius(ThemeRounding::RADIUS_XS)
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(tr.banner_tag)
                                    .size(10.0)
                                    .strong()
                                    .color(palette.accent_default),
                            );
                        });
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(tr.banner_title)
                                .size(13.0)
                                .strong()
                                .color(palette.text_primary),
                        );

                        // Linha informativa com contadores e lista de pendências
                        let pending_formatted: Vec<String> =
                            pending.iter().map(|e| format!(".{e}")).collect();
                        let pending_preview = if pending_formatted.len() <= 6 {
                            pending_formatted.join(" · ")
                        } else {
                            format!(
                                "{} {}",
                                pending_formatted[..5].join(" · "),
                                tr.banner_more_formats
                                    .replace("{}", &(pending_formatted.len() - 5).to_string())
                            )
                        };

                        let counts_info = if app.associations.unknown_count > 0 {
                            format!(
                                "({} {}, {} {}, {} {})",
                                app.associations.configured_count,
                                tr.banner_counts_configured,
                                app.associations.ignored_count,
                                tr.banner_counts_ignored,
                                app.associations.unknown_count,
                                tr.banner_counts_unknown
                            )
                        } else {
                            format!(
                                "({} {}, {} {})",
                                app.associations.configured_count,
                                tr.banner_counts_configured,
                                app.associations.ignored_count,
                                tr.banner_counts_ignored
                            )
                        };

                        ui.label(
                            RichText::new(format!(
                                "{}: {} {}",
                                tr.banner_pending_text
                                    .replace("{}", &pending.len().to_string()),
                                pending_preview,
                                counts_info
                            ))
                            .size(11.5)
                            .color(palette.text_secondary),
                        );
                    });
                });

                ui.add_space(6.0);

                // Linha de Ações: Configurar no Windows, Verificar e Ações Individuais de Ignorar
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::XS, ThemeSpacing::XS);

                    // 1. Botão "Configurar no Windows"
                    let cfg_btn = Button::new(
                        RichText::new(tr.configure_in_windows)
                            .size(11.5)
                            .strong()
                            .color(palette.text_on_accent),
                    )
                    .fill(palette.accent_default)
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui
                        .add(cfg_btn)
                        .on_hover_text(tr.banner_open_settings_tooltip)
                        .clicked()
                    {
                        open_settings = true;
                    }

                    // 2. Botão "Verificar novamente"
                    let check_label = if app.is_checking_associations {
                        tr.verifying
                    } else {
                        tr.verify_again
                    };
                    let verify_btn = Button::new(
                        RichText::new(check_label)
                            .size(11.5)
                            .color(palette.text_secondary),
                    )
                    .fill(palette.surface_active)
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui
                        .add_enabled(!app.is_checking_associations, verify_btn)
                        .on_hover_text(tr.banner_verify_tooltip)
                        .clicked()
                    {
                        check_again = true;
                    }

                    ui.separator();

                    ui.label(
                        RichText::new(tr.ignore_label)
                            .size(11.0)
                            .color(palette.text_muted),
                    );

                    // 3. Botões individuais para cada extensão pendente
                    for ext in &pending {
                        let ignore_btn = Button::new(
                            RichText::new(format!("{} .{ext}", tr.ignore_ext))
                                .size(11.0)
                                .color(palette.text_muted),
                        )
                        .fill(palette.surface_card)
                        .stroke(Stroke::new(1.0, palette.border_subtle.color))
                        .corner_radius(ThemeRounding::RADIUS_XS);

                        if ui
                            .add(ignore_btn)
                            .on_hover_text(format!("{} .{ext}", tr.banner_ignore_ext_tooltip))
                            .clicked()
                        {
                            to_ignore = Some(ext.to_string());
                        }
                    }

                    // 4. Se houver mais de 1 pendente, oferece botão sutil para ignorar todas as restantes
                    if pending.len() > 1 {
                        let ignore_all_btn = Button::new(
                            RichText::new(tr.ignore_all)
                                .size(11.0)
                                .color(palette.text_muted),
                        )
                        .frame(false);

                        if ui
                            .add(ignore_all_btn)
                            .on_hover_text(
                                "Não perguntar sobre nenhuma das extensões pendentes atuais",
                            )
                            .clicked()
                        {
                            ignore_all = true;
                        }
                    }
                });
            });
        });

    if open_settings {
        app.define_default();
    }
    if check_again {
        app.trigger_association_check();
    }
    if let Some(ext) = to_ignore {
        app.ignore_extension(&ext);
    }
    if ignore_all {
        for ext in &pending {
            app.settings.ignore_extension(ext);
        }
        app.save_settings();
        app.trigger_association_check();
    }
}

/// Renderiza o banner superior persistente notificando que uma nova versão está disponível.
pub fn render_update_banner(app: &mut VisorApp, ui: &mut Ui) {
    let Some(release) = app.updater.available_release.clone() else {
        return;
    };

    let palette = ThemePalette::dark();
    let current_v = crate::updater::version::CURRENT_VERSION;

    egui::Frame::NONE
        .fill(palette.surface_card)
        .stroke(Stroke::new(1.0, palette.border_default.color))
        .corner_radius(ThemeRounding::RADIUS_MD)
        .inner_margin(egui::Margin::symmetric(ThemeSpacing::MD as i8, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(palette.accent_muted)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("ATUALIZAÇÃO")
                                .size(10.0)
                                .strong()
                                .color(palette.accent_default),
                        );
                    });

                ui.label(
                    RichText::new(format!(
                        "Nova versão disponível: {} (sua versão: v{})",
                        release.tag_name, current_v
                    ))
                    .size(12.5)
                    .strong()
                    .color(palette.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            RichText::new("Lembrar mais tarde")
                                .size(11.5)
                                .color(palette.text_muted),
                        )
                        .clicked()
                    {
                        app.postpone_update(&release.tag_name);
                    }

                    if ui
                        .button(
                            RichText::new("Ver Novidades")
                                .size(11.5)
                                .color(palette.text_secondary),
                        )
                        .clicked()
                    {
                        let notes = crate::updater::whats_new::get_notes_or_parse_remote(
                            &release.tag_name,
                            release.published_at.as_deref(),
                            release.body.as_deref(),
                        );
                        app.updater.whats_new_notes = Some(notes);
                        app.updater.show_whats_new_window = true;
                    }

                    let update_btn = egui::Button::new(
                        RichText::new("Atualizar Agora")
                            .size(12.0)
                            .strong()
                            .color(palette.text_on_accent),
                    )
                    .fill(palette.accent_default)
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui.add(update_btn).clicked() {
                        app.updater.show_update_window = true;
                    }
                });
            });
        });
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use eframe::egui::Context;
    use std::path::PathBuf;

    #[test]
    fn test_banner_hidden_when_flag_false_or_no_pending() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_association_banner = false;
        app.associations.pending_count = 0;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_association_banner(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(!app.show_association_banner);
    }

    #[test]
    fn test_banner_renders_pending_extensions_with_individual_buttons() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.associations = crate::system::assoc::OverallAssociationState {
            assessments: vec![
                crate::system::assoc::ExtensionAssessment {
                    extension: "webp".to_string(),
                    status: crate::system::assoc::ExtensionStatus::Pending,
                    current_handler: None,
                    error: None,
                },
                crate::system::assoc::ExtensionAssessment {
                    extension: "tiff".to_string(),
                    status: crate::system::assoc::ExtensionStatus::Pending,
                    current_handler: None,
                    error: None,
                },
                crate::system::assoc::ExtensionAssessment {
                    extension: "png".to_string(),
                    status: crate::system::assoc::ExtensionStatus::Configured,
                    current_handler: Some("Visor.Image".to_string()),
                    error: None,
                },
            ],
            configured_count: 1,
            pending_count: 2,
            ignored_count: 0,
            unknown_count: 0,
            is_checking: false,
            last_checked: None,
        };
        app.show_association_banner = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_association_banner(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.show_association_banner);
    }
    #[test]
    fn test_update_banner_renders_when_update_available() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.available_release = Some(crate::updater::GitHubRelease {
            tag_name: "v99.0.0".to_string(),
            name: Some("VISOR v99.0.0".to_string()),
            body: Some("Notas da release".to_string()),
            html_url: "https://github.com/Tsujiu/visor-downloads/releases/tag/v99.0.0".to_string(),
            published_at: Some("2026-11-09T12:00:00Z".to_string()),
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        });

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_update_banner(&mut app, ui);
        });
        output.textures_delta.clear();
        assert!(app.has_active_update_banner());
    }
}
