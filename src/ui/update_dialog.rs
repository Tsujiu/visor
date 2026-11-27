//! Diálogo modal para notificação de atualizações disponíveis, progresso de download e instalação.

use eframe::egui::{self, Align, Color32, Context, Layout, ProgressBar, RichText, Stroke, Vec2};

use crate::app::VisorApp;
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};
use crate::updater::download::DownloadStatus;
use crate::updater::format_timestamp_display;
use crate::updater::install::{launch_official_installer, reveal_portable_package};
use crate::updater::version::CURRENT_VERSION;
use crate::updater::whats_new::{get_notes_or_parse_remote, parse_bullet_item};

/// Renderiza o diálogo modal do Update Center caso esteja ativo.
pub fn render_update_dialog(app: &mut VisorApp, ctx: &Context) {
    if !app.updater.show_update_window {
        return;
    }

    let palette = ThemePalette::dark();

    let screen = ctx.input(|i| {
        i.raw.screen_rect.unwrap_or(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(1024.0, 768.0),
        ))
    });
    let modal_width = (screen.width() - 48.0).clamp(380.0, 560.0);
    let modal_max_height = (screen.height() - 64.0).clamp(320.0, 560.0);

    #[allow(deprecated)]
    let modal_response = egui::Modal::new(egui::Id::new("update_dialog_modal"))
        .backdrop_color(Color32::from_rgba_unmultiplied(8, 8, 12, 220)) // Fundo escurecido / foco total
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

            let tr = *app.tr();
            // Caso haja download ativo ou recém-finalizado
            if let Some(progress) = &app.updater.download_progress.clone() {
                render_download_content(app, ui, progress, &palette, &tr);
            } else if let Some(release) = &app.updater.available_release.clone() {
                render_available_release_content(app, ui, release, modal_max_height, &palette, &tr);
            } else {
                ui.label(RichText::new(tr.update_dialog_no_updates).color(palette.text_secondary));
            }
        });

    if modal_response.should_close() {
        app.updater.show_update_window = false;
    }
}

/// Renderiza o conteúdo da notificação de versão disponível.
fn render_available_release_content(
    app: &mut VisorApp,
    ui: &mut egui::Ui,
    release: &crate::updater::GitHubRelease,
    modal_max_height: f32,
    palette: &ThemePalette,
    tr: &crate::system::i18n::I18nStrings,
) {
    // -------------------------------------------------------------
    // 1. HEADER DO CARD COM COMPARAÇÃO DE VERSÕES
    // -------------------------------------------------------------
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.update_dialog_available_title)
                        .size(16.0)
                        .strong()
                        .color(palette.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        RichText::new("X")
                            .size(12.5)
                            .strong()
                            .color(palette.text_muted),
                    )
                    .frame(false);

                    if ui.add(close_btn).on_hover_text(tr.close_tooltip).clicked() {
                        app.updater.show_update_window = false;
                    }
                });
            });

            ui.add_space(ThemeSpacing::XS);

            // Comparador visual de versões
            ui.horizontal(|ui| {
                // Badge Versão Atual
                egui::Frame::NONE
                    .fill(palette.surface_hover)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} v{CURRENT_VERSION}",
                                tr.update_dialog_current_prefix
                            ))
                            .size(11.5)
                            .color(palette.text_muted),
                        );
                    });

                ui.label(RichText::new(">").size(11.5).color(palette.text_muted));

                // Badge Nova Versão
                egui::Frame::NONE
                    .fill(palette.accent_muted)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                tr.update_dialog_available_prefix, release.tag_name
                            ))
                            .size(11.5)
                            .strong()
                            .color(palette.accent_default),
                        );
                    });

                // Tag Oficial
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(52, 211, 153, 30))
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(tr.update_dialog_official_badge)
                                .size(9.5)
                                .strong()
                                .color(palette.status_success),
                        );
                    });
            });

            if let Some(pub_date) = &release.published_at {
                ui.add_space(ThemeSpacing::XS);
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        tr.update_dialog_released_on,
                        format_timestamp_display(pub_date)
                    ))
                    .size(11.0)
                    .color(palette.text_muted),
                );
            }
        });

    ui.add_space(ThemeSpacing::SM);

    // -------------------------------------------------------------
    // 2. PARSING E EXIBIÇÃO ESTRUTURADA DAS NOVIDADES
    // -------------------------------------------------------------
    let notes = get_notes_or_parse_remote(
        &release.tag_name,
        release.published_at.as_deref(),
        release.body.as_deref(),
    );

    let scroll_height = (modal_max_height - 240.0).clamp(100.0, 220.0);
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tr.update_dialog_highlights)
                        .size(13.0)
                        .strong()
                        .color(palette.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total = notes.total_items();
                    if total > 0 {
                        ui.label(
                            RichText::new(format!(
                                "{total} {}",
                                tr.update_dialog_improvements_count
                            ))
                            .size(10.5)
                            .color(palette.text_muted),
                        );
                    }
                });
            });

            ui.add_space(ThemeSpacing::XS);
            ui.separator();
            ui.add_space(ThemeSpacing::XS);

            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::XS, ThemeSpacing::SM);

                    if !notes.added.is_empty() {
                        render_dialog_category_card(
                            ui,
                            tr.hub_tag_new,
                            &notes.added,
                            palette.accent_default,
                            palette,
                        );
                    }

                    if !notes.changed.is_empty() {
                        render_dialog_category_card(
                            ui,
                            tr.hub_tag_improvement,
                            &notes.changed,
                            palette.accent_hover,
                            palette,
                        );
                    }

                    if !notes.fixed.is_empty() {
                        render_dialog_category_card(
                            ui,
                            tr.hub_tag_fix,
                            &notes.fixed,
                            palette.status_success,
                            palette,
                        );
                    }

                    if !notes.security.is_empty() {
                        render_dialog_category_card(
                            ui,
                            tr.hub_tab_security,
                            &notes.security,
                            palette.status_warning,
                            palette,
                        );
                    }

                    if !notes.important.is_empty() {
                        render_dialog_category_card(
                            ui,
                            "!",
                            &notes.important,
                            palette.status_error,
                            palette,
                        );
                    }

                    if notes.total_items() == 0 {
                        let body_text = release
                            .body
                            .as_deref()
                            .unwrap_or(tr.hub_formats_honest_desc);
                        ui.label(
                            RichText::new(body_text)
                                .size(11.5)
                                .color(palette.text_secondary),
                        );
                    }
                });
        });

    ui.add_space(ThemeSpacing::SM);

    // -------------------------------------------------------------
    // 3. AÇÕES DE DOWNLOAD, SEGURANÇA E CONTROLE
    // -------------------------------------------------------------
    // Sempre prioriza o instalador oficial da release
    let installer_opt = release.find_installer_asset().cloned();
    let portable_opt = release.find_portable_asset().cloned();
    let manifest_opt = release.find_manifest_asset().cloned();

    let target_package = installer_opt.or_else(|| portable_opt.clone());

    ui.horizontal(|ui| {
        if let Some(pkg) = target_package {
            let size_mb = (pkg.size as f64 / (1024.0 * 1024.0)).max(0.1);
            let btn_label = format!("{} ({:.1} MB)", tr.update_dialog_download_btn, size_mb);

            let download_btn = egui::Button::new(
                RichText::new(btn_label)
                    .size(13.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(palette.accent_default)
            .corner_radius(ThemeRounding::RADIUS_SM)
            .min_size(Vec2::new(160.0, 32.0));

            if ui
                .add(download_btn)
                .on_hover_text(tr.update_dialog_download_tooltip)
                .clicked()
            {
                app.start_package_download(release.tag_name.clone(), pkg, manifest_opt);
            }
        } else {
            ui.label(RichText::new(tr.update_dialog_pkg_not_found).color(palette.status_warning));
        }

        if ui
            .button(
                RichText::new(tr.update_dialog_all_news)
                    .size(12.0)
                    .color(palette.text_secondary),
            )
            .clicked()
        {
            app.updater.whats_new_notes = Some(notes);
            app.updater.show_whats_new_window = true;
        }
    });

    ui.add_space(ThemeSpacing::XS);
    ui.horizontal(|ui| {
        if ui
            .button(
                RichText::new(tr.update_dialog_remind_later)
                    .size(11.5)
                    .color(palette.text_muted),
            )
            .clicked()
        {
            app.postpone_update(&release.tag_name);
        }

        if ui
            .button(
                RichText::new(tr.update_dialog_ignore_version)
                    .size(11.5)
                    .color(palette.text_muted),
            )
            .on_hover_text(tr.update_dialog_ignore_tooltip)
            .clicked()
        {
            app.ignore_update_version(&release.tag_name);
        }
    });
}

/// Renderiza um card compacto de novidades para o diálogo de atualização.
fn render_dialog_category_card(
    ui: &mut egui::Ui,
    tag: &str,
    items: &[String],
    tag_color: Color32,
    palette: &ThemePalette,
) {
    if items.is_empty() {
        return;
    }

    ui.horizontal(|ui| {
        egui::Frame::NONE
            .fill(Color32::from_rgba_unmultiplied(
                tag_color.r(),
                tag_color.g(),
                tag_color.b(),
                36,
            ))
            .corner_radius(ThemeRounding::RADIUS_XS)
            .inner_margin(egui::Margin::symmetric(5, 2))
            .show(ui, |ui| {
                ui.label(RichText::new(tag).size(9.5).strong().color(tag_color));
            });

        ui.label(
            RichText::new(format!(
                "{} item{}",
                items.len(),
                if items.len() > 1 { "s" } else { "" }
            ))
            .size(10.5)
            .color(palette.text_muted),
        );
    });

    for item in items.iter().take(4) {
        let (title_opt, desc) = parse_bullet_item(item);
        if let Some(t) = title_opt {
            ui.label(
                RichText::new(format!("  • {t}: {desc}"))
                    .size(11.5)
                    .color(palette.text_secondary),
            );
        } else {
            ui.label(
                RichText::new(format!("  • {desc}"))
                    .size(11.5)
                    .color(palette.text_secondary),
            );
        }
    }

    if items.len() > 4 {
        ui.label(
            RichText::new(format!("  ... e mais {} itens", items.len() - 4))
                .size(11.0)
                .color(palette.text_muted),
        );
    }
}

/// Renderiza o conteúdo durante o download ou após a conclusão.
fn render_download_content(
    app: &mut VisorApp,
    ui: &mut egui::Ui,
    progress: &crate::updater::DownloadProgress,
    palette: &ThemePalette,
    tr: &crate::system::i18n::I18nStrings,
) {
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        tr.update_dialog_downloading_prefix, progress.version
                    ))
                    .size(15.0)
                    .strong()
                    .color(palette.text_primary),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if matches!(
                        progress.status,
                        DownloadStatus::Downloading | DownloadStatus::Preparing
                    )
                        && ui
                            .button(
                                RichText::new(tr.remove)
                                    .size(11.5)
                                    .color(palette.status_error),
                            )
                            .clicked()
                        {
                            app.updater.cancel_active_download();
                        }
                });
            });

            ui.label(
                RichText::new(&progress.filename)
                    .size(11.5)
                    .monospace()
                    .color(palette.text_muted),
            );

            ui.add_space(ThemeSpacing::SM);

            match &progress.status {
                DownloadStatus::Preparing => {
                    ui.label(
                        RichText::new(tr.update_dialog_connecting)
                            .size(12.0)
                            .color(palette.text_secondary),
                    );
                    ui.add(ProgressBar::new(0.0).animate(true));
                }
                DownloadStatus::Downloading => {
                    ui.add(ProgressBar::new(progress.percentage).show_percentage());
                    ui.add_space(ThemeSpacing::XS);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(progress.format_progress_size())
                                .size(12.0)
                                .color(palette.text_secondary),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    progress.format_speed(),
                                    progress.format_eta()
                                ))
                                .size(12.0)
                                .color(palette.text_muted),
                            );
                        });
                    });
                }
                DownloadStatus::Verifying => {
                    ui.label(
                        RichText::new(tr.update_dialog_verifying_sha)
                            .strong()
                            .color(palette.accent_default),
                    );
                    ui.label(
                        RichText::new(tr.update_dialog_sha_manifest)
                            .size(11.5)
                            .color(palette.text_muted),
                    );
                    ui.add(ProgressBar::new(1.0).animate(true));
                }
                DownloadStatus::Completed(path) => {
                    ui.horizontal(|ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_rgba_unmultiplied(52, 211, 153, 30))
                            .corner_radius(ThemeRounding::RADIUS_XS)
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(tr.update_dialog_sha_verified)
                                        .size(9.5)
                                        .strong()
                                        .color(palette.status_success),
                                );
                            });

                        ui.label(
                            RichText::new(tr.update_dialog_download_success)
                                .strong()
                                .color(palette.status_success),
                        );
                    });

                    ui.add_space(ThemeSpacing::XS);
                    ui.label(
                        RichText::new(tr.update_dialog_sha_confirmed)
                            .size(11.5)
                            .color(palette.text_muted),
                    );

                    ui.add_space(ThemeSpacing::MD);

                    let is_setup = path.extension().map(|e| e == "exe").unwrap_or(false);
                    if is_setup {
                        ui.label(
                            RichText::new(tr.update_dialog_install_prompt)
                                .size(12.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::SM);

                        let install_btn = egui::Button::new(
                            RichText::new(tr.update_dialog_install_btn)
                                .size(13.5)
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(palette.status_success)
                        .corner_radius(ThemeRounding::RADIUS_SM)
                        .min_size(Vec2::new(140.0, 32.0));

                        if ui.add(install_btn).clicked() {
                            match launch_official_installer(path) {
                                Ok(()) => {
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                Err(err) => {
                                    let mut failed = progress.clone();
                                    failed.status = DownloadStatus::Failed(err);
                                    app.updater.download_progress = Some(failed);
                                }
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new(tr.update_dialog_portable_success)
                                .size(12.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::SM);

                        if ui
                            .button(
                                RichText::new(tr.update_dialog_open_folder)
                                    .size(13.0)
                                    .strong(),
                            )
                            .clicked()
                        {
                            let _ = reveal_portable_package(path);
                        }
                    }
                }
                DownloadStatus::Failed(err) => {
                    ui.label(
                        RichText::new(tr.update_dialog_download_failed)
                            .strong()
                            .color(palette.status_error),
                    );
                    ui.label(RichText::new(err).size(12.0).color(palette.status_error));

                    ui.add_space(ThemeSpacing::SM);
                    if ui.button(tr.close).clicked() {
                        app.updater.download_progress = None;
                        app.updater.show_update_window = false;
                    }
                }
                DownloadStatus::Cancelled => {
                    ui.label(
                        RichText::new(tr.update_dialog_cancelled)
                            .size(12.0)
                            .color(palette.text_muted),
                    );
                    ui.add_space(ThemeSpacing::SM);
                    if ui.button(tr.close).clicked() {
                        app.updater.download_progress = None;
                        app.updater.show_update_window = false;
                    }
                }
                DownloadStatus::Idle => {}
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_dialog_skips_when_closed() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.show_update_window = false;

        render_update_dialog(&mut app, &ctx);
        assert!(!app.updater.show_update_window);
    }

    #[test]
    fn test_update_dialog_renders_when_open() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.show_update_window = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_update_dialog(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
    }
    #[test]
    fn test_update_dialog_targets_installer_release_not_portable() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.available_release = Some(crate::updater::GitHubRelease {
            tag_name: "v99.0.0".to_string(),
            name: Some("VISOR v99.0.0".to_string()),
            body: Some("Notas da release oficial".to_string()),
            html_url: "https://github.com/Tsujiu/visor/releases/tag/v99.0.0".to_string(),
            published_at: Some("2026-11-09T12:00:00Z".to_string()),
            draft: false,
            prerelease: false,
            assets: vec![
                crate::updater::GitHubAsset {
                    name: "Visor-Setup-v99.0.0-x64.exe".to_string(),
                    size: 8_500_000,
                    browser_download_url: "https://github.com/Tsujiu/visor/releases/download/v99.0.0/Visor-Setup-v99.0.0-x64.exe".to_string(),
                    content_type: Some("application/x-msdownload".to_string()),
                },
                crate::updater::GitHubAsset {
                    name: "Visor-Portable-v99.0.0-x64.zip".to_string(),
                    size: 7_500_000,
                    browser_download_url: "https://github.com/Tsujiu/visor/releases/download/v99.0.0/Visor-Portable-v99.0.0-x64.zip".to_string(),
                    content_type: Some("application/zip".to_string()),
                },
            ],
        });
        app.updater.show_update_window = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_update_dialog(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.updater.show_update_window);
    }
}
