//! Janela modal de configurações e preferências persistentes do VISOR.
//!
//! Reorganizada para máxima clareza e utilidade: concentra apenas preferências
//! reais de sistema e comportamento (roda do mouse, layout inicial, aplicativo
//! padrão e central de atualizações), eliminando redundâncias com o menu principal.

#![allow(unused_imports)]

use eframe::egui::{self, Align2, Color32, Context, FontId, RichText, Sense, StrokeKind, Vec2};

use crate::app::state::{CanvasBackground, InterpolationMode, VisorApp};
use crate::system::config::{AppSettings, DefaultZoomMode, MouseWheelAction};
use crate::ui::theme::{ThemeDimensions, ThemePalette, ThemeRounding, ThemeSpacing};

/// Renderiza a janela modal de configurações persistentes.
pub fn render_settings_window(app: &mut VisorApp, ctx: &Context) {
    if !app.show_settings {
        return;
    }

    let tr = *app.tr();
    let mut open = app.show_settings;
    let palette = ThemePalette::dark();
    let mut changed = false;
    let mut should_close = false;

    egui::Window::new(tr.settings_title)
        .open(&mut open)
        .default_size([500.0, 520.0])
        .min_size([420.0, 380.0])
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.input(|i| {
            i.raw
                .screen_rect
                .map(|r| egui::pos2(r.center().x, r.center().y - 20.0))
                .unwrap_or(egui::pos2(420.0, 300.0))
        }))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::SM, ThemeSpacing::MD);

                    // ---------------------------------------------------------
                    // 0. SEÇÃO: IDIOMA / LANGUAGE
                    // ---------------------------------------------------------
                    render_section_card(ui, tr.section_language, &palette, |ui| {
                        ui.label(
                            RichText::new(tr.language_label)
                                .size(13.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        let current_lang = app.settings.language;
                        egui::ComboBox::from_id_salt("settings_app_language_selector")
                            .selected_text(current_lang.display_name())
                            .width(260.0)
                            .show_ui(ui, |ui| {
                                for &lang in crate::system::i18n::AppLanguage::ALL {
                                    if ui
                                        .selectable_value(
                                            &mut app.settings.language,
                                            lang,
                                            lang.display_name(),
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                }
                            });

                        if app.settings.language == crate::system::i18n::AppLanguage::System {
                            ui.add_space(ThemeSpacing::XXS);
                            ui.label(
                                RichText::new(tr.language_auto_desc)
                                    .size(11.5)
                                    .color(palette.text_muted),
                            );
                        }
                    });

                    // ---------------------------------------------------------
                    // 1. SEÇÃO: NAVEGAÇÃO & RODA DO MOUSE
                    // ---------------------------------------------------------
                    render_section_card(ui, tr.section_navigation, &palette, |ui| {
                        ui.label(
                            RichText::new(tr.mouse_wheel_action_label)
                                .size(13.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .radio_value(
                                &mut app.settings.mouse_wheel_action,
                                MouseWheelAction::Zoom,
                                RichText::new(tr.mouse_wheel_zoom).size(14.0).strong(),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.mouse_wheel_zoom_desc))
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .radio_value(
                                &mut app.settings.mouse_wheel_action,
                                MouseWheelAction::Navigate,
                                RichText::new(tr.mouse_wheel_nav).size(14.0).strong(),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.mouse_wheel_nav_desc))
                                .size(12.0)
                                .color(palette.text_muted),
                        );

                        // Card de dica de usabilidade Fluent
                        ui.add_space(ThemeSpacing::SM);
                        egui::Frame::NONE
                            .fill(palette.surface_active)
                            .corner_radius(ThemeRounding::RADIUS_SM)
                            .inner_margin(egui::Margin::symmetric(ThemeSpacing::SM as i8, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(tr.tip_prefix)
                                            .size(12.0)
                                            .color(palette.accent_default)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(tr.mouse_wheel_tip)
                                            .size(12.0)
                                            .color(palette.text_secondary),
                                    );
                                });
                            });
                    });

                    // ---------------------------------------------------------
                    // 2. SEÇÃO: INTERFACE & INICIALIZAÇÃO
                    // ---------------------------------------------------------
                    render_section_card(ui, tr.section_interface, &palette, |ui| {
                        ui.label(
                            RichText::new(tr.default_zoom_label)
                                .size(13.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .radio_value(
                                &mut app.settings.default_zoom,
                                DefaultZoomMode::Actual100,
                                RichText::new(tr.default_zoom_100).size(13.5).strong(),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.default_zoom_100_desc))
                                .size(11.5)
                                .color(palette.text_muted),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .radio_value(
                                &mut app.settings.default_zoom,
                                DefaultZoomMode::Fit,
                                RichText::new(tr.default_zoom_fit).size(13.5).strong(),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.default_zoom_fit_desc))
                                .size(11.5)
                                .color(palette.text_muted),
                        );

                        ui.add_space(ThemeSpacing::MD);

                        ui.label(
                            RichText::new(tr.interface_panels_desc)
                                .size(13.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .checkbox(
                                &mut app.settings.show_filmstrip,
                                RichText::new(tr.show_filmstrip_label).size(13.5),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.show_filmstrip_desc))
                                .size(11.5)
                                .color(palette.text_muted),
                        );

                        ui.add_space(ThemeSpacing::XS);

                        if ui
                            .checkbox(
                                &mut app.settings.show_info,
                                RichText::new(tr.show_info_label).size(13.5),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label(
                            RichText::new(format!("     {}", tr.show_info_desc))
                                .size(11.5)
                                .color(palette.text_muted),
                        );
                    });

                    // ---------------------------------------------------------
                    // 3. SEÇÃO: APLICATIVO PADRÃO DO WINDOWS (ASSOCIAÇÕES)
                    // ---------------------------------------------------------
                    render_section_card(ui, tr.section_defaults, &palette, |ui| {
                        ui.label(
                            RichText::new(tr.default_app_desc)
                                .size(13.0)
                                .color(palette.text_secondary),
                        );
                        ui.add_space(ThemeSpacing::XS);

                        let mut unignore_target: Option<String> = None;
                        let mut trigger_check = false;
                        let mut open_settings = false;

                        // Resumo de contadores com indicador de status
                        ui.horizontal(|ui| {
                            let (badge_text, badge_color) = if app.is_checking_associations {
                                (tr.verifying, palette.text_muted)
                            } else if app.associations.has_pending() {
                                (tr.default_pending, palette.status_warning)
                            } else {
                                (tr.default_configured, palette.status_success)
                            };

                            ui.label(RichText::new("•").size(16.0).color(badge_color));
                            ui.label(
                                RichText::new(badge_text)
                                    .size(13.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                        });

                        ui.label(
                            RichText::new(format!(
                                "{} {} · {} {} · {} {}",
                                app.associations.configured_count,
                                tr.formats_configured_word,
                                app.associations.pending_count,
                                tr.formats_pending_word,
                                app.associations.ignored_count,
                                tr.formats_ignored_word,
                            ))
                            .size(12.0)
                            .color(palette.text_muted),
                        );

                        ui.add_space(ThemeSpacing::SM);

                        // Botões de Ação
                        ui.horizontal(|ui| {
                            let cfg_btn = egui::Button::new(
                                RichText::new(tr.configure_in_windows)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_on_accent),
                            )
                            .fill(palette.accent_default)
                            .corner_radius(ThemeRounding::RADIUS_SM);

                            if ui
                                .add(cfg_btn)
                                .on_hover_text(tr.settings_open_assoc_tooltip)
                                .clicked()
                            {
                                open_settings = true;
                            }

                            let verify_label = if app.is_checking_associations {
                                tr.verifying
                            } else {
                                tr.verify_again
                            };
                            let verify_btn = egui::Button::new(
                                RichText::new(verify_label)
                                    .size(12.0)
                                    .color(palette.text_secondary),
                            )
                            .fill(palette.surface_active)
                            .corner_radius(ThemeRounding::RADIUS_SM);

                            if ui
                                .add_enabled(!app.is_checking_associations, verify_btn)
                                .on_hover_text(tr.settings_refresh_assoc_tooltip)
                                .clicked()
                            {
                                trigger_check = true;
                            }
                        });

                        // Bloco de Extensões Ignoradas (se houver alguma)
                        let ignored = app.associations.ignored_extensions();
                        if !ignored.is_empty() {
                            ui.add_space(ThemeSpacing::SM);
                            ui.label(
                                RichText::new(tr.ignored_exts_label)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_secondary),
                            );

                            ui.horizontal_wrapped(|ui| {
                                for ext in ignored {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!(".{ext}"))
                                                .size(12.0)
                                                .color(palette.text_secondary),
                                        );
                                        let btn = egui::Button::new(
                                            RichText::new(tr.unignore_btn)
                                                .size(11.0)
                                                .color(palette.accent_default),
                                        )
                                        .frame(false);
                                        if ui
                                            .add(btn)
                                            .on_hover_text(tr.settings_unignore_tooltip)
                                            .clicked()
                                        {
                                            unignore_target = Some(ext.to_string());
                                        }
                                    });
                                }
                            });
                        }

                        // Extensões pendentes (se houver alguma)
                        if app.associations.pending_count > 0 {
                            ui.add_space(ThemeSpacing::XS);
                            ui.collapsing(
                                format!(
                                    "{} ({})",
                                    tr.pending_exts_title, app.associations.pending_count
                                ),
                                |ui| {
                                    let pending = app.associations.pending_extensions();
                                    let list_str = pending
                                        .iter()
                                        .map(|e| format!(".{e}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    ui.label(
                                        RichText::new(list_str)
                                            .size(11.5)
                                            .color(palette.status_warning),
                                    );
                                },
                            );
                        }

                        if open_settings {
                            app.define_default();
                        }
                        if trigger_check {
                            app.trigger_association_check();
                        }
                        if let Some(ext) = unignore_target {
                            app.unignore_extension(&ext);
                            changed = true;
                        }
                    });

                    // ---------------------------------------------------------
                    // 4. SEÇÃO: ATUALIZAÇÕES & SOBRE O APLICATIVO
                    // ---------------------------------------------------------
                    render_section_card(ui, tr.section_updates_about, &palette, |ui| {
                        use crate::updater::format_timestamp_display;
                        use crate::updater::version::CURRENT_VERSION;

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr.version)
                                    .size(12.5)
                                    .color(palette.text_secondary),
                            );
                            ui.label(
                                RichText::new(format!("v{CURRENT_VERSION}"))
                                    .strong()
                                    .size(13.0)
                                    .color(palette.text_primary),
                            );
                            ui.label(RichText::new("·").color(palette.text_muted));
                            ui.label(
                                RichText::new(tr.channel)
                                    .size(12.5)
                                    .color(palette.text_secondary),
                            );
                            ui.label(
                                RichText::new(tr.channel_stable)
                                    .size(12.5)
                                    .color(palette.accent_default),
                            );
                        });

                        ui.add_space(ThemeSpacing::XXS);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr.license)
                                    .size(12.5)
                                    .color(palette.text_secondary),
                            );
                            ui.label(
                                RichText::new(tr.license_free)
                                    .strong()
                                    .size(12.5)
                                    .color(palette.text_primary),
                            );
                            ui.label(RichText::new("·").color(palette.text_muted));
                            ui.label(
                                RichText::new(tr.protection)
                                    .size(12.5)
                                    .color(palette.text_secondary),
                            );
                            ui.label(
                                RichText::new(tr.anti_tamper_active)
                                    .size(12.5)
                                    .color(palette.status_success),
                            );
                        });

                        ui.add_space(ThemeSpacing::XS);
                        // Estado da última verificação
                        let last_check_text = if let Some(last) = &app.settings.updates.last_check {
                            format_timestamp_display(last)
                        } else {
                            tr.never_checked.to_string()
                        };

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr.last_check)
                                    .size(12.0)
                                    .color(palette.text_secondary),
                            );
                            ui.label(
                                RichText::new(&last_check_text)
                                    .size(12.0)
                                    .color(palette.text_muted),
                            );
                        });

                        ui.add_space(ThemeSpacing::XS);

                        // Status reativo
                        if app.updater.is_checking {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("•").size(14.0).color(palette.accent_default),
                                );
                                ui.label(
                                    RichText::new(tr.checking_updates)
                                        .size(12.0)
                                        .color(palette.accent_default),
                                );
                            });
                        } else if let Some(rel) = &app.updater.available_release {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("•").size(14.0).color(palette.status_warning),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}",
                                        tr.update_available, rel.tag_name
                                    ))
                                    .strong()
                                    .size(12.5)
                                    .color(palette.status_warning),
                                );
                                if ui
                                    .button(RichText::new(tr.view_update).size(11.5).strong())
                                    .clicked()
                                {
                                    app.updater.show_update_window = true;
                                }
                            });
                        } else if let Some(err) = &app.updater.last_error {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("•").size(14.0).color(palette.status_error));
                                ui.label(
                                    RichText::new(format!("{}: {err}", tr.check_error))
                                        .size(11.5)
                                        .color(palette.status_error),
                                );
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("•").size(14.0).color(palette.status_success),
                                );
                                ui.label(
                                    RichText::new(tr.up_to_date)
                                        .size(12.0)
                                        .color(palette.status_success),
                                );
                            });
                        }

                        ui.add_space(ThemeSpacing::SM);

                        // Configurações automáticas
                        let mut auto_check = app.settings.updates.automatic_check;
                        if ui
                            .checkbox(
                                &mut auto_check,
                                RichText::new(tr.auto_check_updates)
                                    .size(12.5)
                                    .color(palette.text_primary),
                            )
                            .changed()
                        {
                            app.settings.updates.automatic_check = auto_check;
                            changed = true;
                        }

                        ui.add_space(ThemeSpacing::SM);

                        // Botões de ação manual
                        ui.horizontal_wrapped(|ui| {
                            let check_btn =
                                egui::Button::new(RichText::new(tr.check_now).size(12.0).strong())
                                    .corner_radius(ThemeRounding::RADIUS_SM);

                            if ui
                                .add_enabled(!app.updater.is_checking, check_btn)
                                .clicked()
                            {
                                app.trigger_update_check(true);
                            }

                            if ui
                                .button(RichText::new(tr.visor_center_btn).size(12.0))
                                .clicked()
                            {
                                app.updater.whats_new_notes =
                                    Some(crate::updater::whats_new::get_current_version_notes());
                                app.updater.show_whats_new_window = true;
                            }

                            if ui
                                .button(RichText::new("GitHub").size(12.0))
                                .on_hover_text("https://github.com/Tsujiu/visor")
                                .clicked()
                            {
                                ui.ctx().open_url(egui::OpenUrl::new_tab(
                                    "https://github.com/Tsujiu/visor",
                                ));
                            }

                            if ui
                                .button(
                                    RichText::new(tr.support_btn)
                                        .size(12.0)
                                        .strong()
                                        .color(palette.accent_default),
                                )
                                .on_hover_text(tr.support_title)
                                .clicked()
                            {
                                app.show_support = true;
                            }
                        });
                    });

                    // ---------------------------------------------------------
                    // 5. BARRA INFERIOR DE AÇÕES
                    // ---------------------------------------------------------
                    ui.add_space(ThemeSpacing::SM);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new(tr.restore_defaults)
                                    .size(12.5)
                                    .color(palette.status_warning),
                            )
                            .on_hover_text(tr.restore_defaults_desc)
                            .clicked()
                        {
                            app.restore_default_settings();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = egui::Button::new(
                                RichText::new(tr.close)
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
        });

    if should_close {
        open = false;
    }

    if changed {
        app.save_settings();
        ctx.request_repaint();
    }

    app.show_settings = open;
}

/// Renderiza um container de seção tipo Card com cantos arredondados e borda sutil.
fn render_section_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    palette: &ThemePalette,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .size(15.0)
                    .strong()
                    .color(palette.text_primary),
            );
            ui.add_space(ThemeSpacing::XS);
            add_contents(ui)
        })
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_window_renders_when_closed() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_settings = false;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_settings_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(!app.show_settings);
    }

    #[test]
    fn test_settings_window_renders_when_open() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_settings = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_settings_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.show_settings);
    }

    #[test]
    fn test_restore_defaults_action() {
        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);

        // Altera para valores não-padrão
        app.settings.mouse_wheel_action = MouseWheelAction::Navigate;
        app.settings.interpolation = InterpolationMode::Nearest;
        app.settings.background = CanvasBackground::White;
        app.settings.show_filmstrip = false;
        app.settings.show_info = true;
        app.interpolation = InterpolationMode::Nearest;
        app.background = CanvasBackground::White;
        app.show_filmstrip = false;
        app.show_info = true;

        app.restore_default_settings();

        assert_eq!(app.settings.mouse_wheel_action, MouseWheelAction::Zoom);
        assert_eq!(app.settings.interpolation, InterpolationMode::Auto);
        assert_eq!(app.settings.background, CanvasBackground::DarkChecker);
        assert!(app.settings.show_filmstrip);
        assert!(!app.settings.show_info);
        assert_eq!(app.interpolation, InterpolationMode::Auto);
        assert_eq!(app.background, CanvasBackground::DarkChecker);
        assert!(app.show_filmstrip);
        assert!(!app.show_info);
    }
}
