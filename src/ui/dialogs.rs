//! Janelas de diálogo modernas inspiradas no Fluent Design do Windows 11.
//!
//! Contém:
//! 1. Janela de Formatos Suportados: catálogo honesto com busca em tempo real,
//!    categorização (Raster, Texturas, Vetoriais, RAW, Animações) e distinção
//!    clara entre Leitura (Full/Partial/External) e Exportação real.
//! 2. Janela de Conversão em Lote: cards modernos de arquivos, configurações
//!    de formato (os 11 formatos suportados) e status/progresso.

use std::path::PathBuf;

use eframe::egui::{self, Color32, Context, RichText, Vec2};

use crate::app::state::VisorApp;
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};

/// Determina se um formato possui capacidade real de exportação/gravação no VISOR.
///
/// Baseado estritamente nos 11 formatos suportados por [`visor::ExportKind::all`].
/// NUNCA apresenta suporte a escrita inexistente (ex.: SVG, JXL, AVIF, RAW, DDS).
pub fn is_format_exportable(id: &str) -> bool {
    matches!(
        id,
        "png" | "jpeg" | "webp" | "bmp" | "tiff" | "gif" | "qoi" | "tga" | "ico" | "pnm" | "hdr"
    )
}

/// Verifica se um `FormatSpec` satisfaz o termo de pesquisa (busca por nome, id,
/// categoria, extensões ou nota técnica).
pub fn format_matches_query(spec: &visor::FormatSpec, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    spec.name.to_lowercase().contains(&q)
        || spec.id.to_lowercase().contains(&q)
        || spec.category.label().to_lowercase().contains(&q)
        || spec
            .extensions
            .iter()
            .any(|ext| ext.to_lowercase().contains(&q))
        || spec.note.to_lowercase().contains(&q)
}

/// Renderiza um badge estilizado com fundo translúcido e cantos arredondados.
fn render_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(ThemeRounding::RADIUS_XS)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(fg).strong());
        });
}

/// Renderiza o badge correspondente ao nível de suporte de leitura.
fn render_reading_badge(
    ui: &mut egui::Ui,
    support: visor::Support,
    palette: &ThemePalette,
    tr: &crate::system::i18n::I18nStrings,
) {
    match support {
        visor::Support::Full => {
            render_badge(
                ui,
                tr.badge_full,
                Color32::from_rgba_unmultiplied(108, 203, 95, 36),
                palette.status_success,
            );
        }
        visor::Support::ReadOnly => {
            render_badge(
                ui,
                tr.badge_read_only,
                Color32::from_rgba_unmultiplied(96, 205, 255, 36),
                palette.accent_default,
            );
        }
        visor::Support::Partial => {
            render_badge(
                ui,
                tr.badge_partial,
                Color32::from_rgba_unmultiplied(252, 225, 0, 36),
                palette.status_warning,
            );
        }
        visor::Support::External => {
            render_badge(
                ui,
                tr.badge_external,
                Color32::from_rgba_unmultiplied(180, 130, 255, 36),
                Color32::from_rgb(190, 140, 255),
            );
        }
        visor::Support::Unsupported => {
            render_badge(
                ui,
                tr.badge_unsupported,
                Color32::from_rgba_unmultiplied(255, 100, 100, 36),
                palette.status_error,
            );
        }
    }
}

/// Renderiza o badge correspondente à capacidade de exportação/gravação.
fn render_export_badge(
    ui: &mut egui::Ui,
    spec_id: &str,
    palette: &ThemePalette,
    tr: &crate::system::i18n::I18nStrings,
) {
    if is_format_exportable(spec_id) {
        render_badge(
            ui,
            tr.badge_exportable,
            Color32::from_rgba_unmultiplied(108, 203, 95, 36),
            palette.status_success,
        );
    } else {
        render_badge(
            ui,
            tr.badge_read_only,
            Color32::from_rgba_unmultiplied(160, 160, 168, 25),
            palette.text_muted,
        );
    }
}

/// Renderiza a janela de Formatos Suportados com busca, filtros e catálogo técnico.
pub fn render_formats_window(app: &mut VisorApp, ctx: &Context) {
    let mut open = app.show_formats;
    let tr = *app.tr();
    let palette = ThemePalette::dark();

    egui::Window::new(tr.formats_catalog)
        .open(&mut open)
        .default_size([600.0, 520.0])
        .min_size([420.0, 360.0])
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.input(|i| {
            i.raw
                .screen_rect
                .map(|r| egui::pos2(r.center().x, r.center().y - 30.0))
                .unwrap_or(egui::pos2(400.0, 270.0))
        }))
        .show(ctx, |ui| {
            // Cabeçalho da janela com barra de pesquisa
            egui::Frame::NONE
                .fill(palette.surface_card)
                .corner_radius(ThemeRounding::RADIUS_MD)
                .stroke(palette.border_subtle)
                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let search_edit = egui::TextEdit::singleline(&mut app.format_search)
                            .hint_text(tr.dialog_formats_search_hint)
                            .desired_width(ui.available_width() - 36.0);
                        ui.add(search_edit);

                        if !app.format_search.is_empty() && ui.button("x").clicked() {
                            app.format_search.clear();
                        }
                    });

                    ui.add_space(ThemeSpacing::XS);
                    ui.label(
                        RichText::new(tr.dialog_formats_subtitle)
                            .size(11.5)
                            .color(palette.text_secondary),
                    );
                });

            ui.add_space(ThemeSpacing::SM);

            // Lista rolável de categorias e formatos
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let all_formats = visor::formats();
                    let categories = [
                        (visor::Category::Raster, tr.cat_raster),
                        (visor::Category::Animation, tr.cat_animation),
                        (visor::Category::Vector, tr.cat_vector),
                        (visor::Category::Raw, tr.cat_raw),
                        (visor::Category::Texture, tr.cat_texture),
                    ];

                    let mut total_matches = 0;

                    for (category, cat_label) in categories {
                        let matching_specs: Vec<&visor::FormatSpec> = all_formats
                            .iter()
                            .filter(|s| {
                                s.category == category
                                    && format_matches_query(s, &app.format_search)
                            })
                            .collect();

                        if matching_specs.is_empty() {
                            continue;
                        }

                        total_matches += matching_specs.len();

                        ui.add_space(ThemeSpacing::SM);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(cat_label)
                                    .size(14.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            ui.label(
                                RichText::new(format!("({})", matching_specs.len()))
                                    .size(12.0)
                                    .color(palette.text_muted),
                            );
                        });

                        ui.add_space(ThemeSpacing::XXS);

                        for spec in matching_specs {
                            egui::Frame::NONE
                                .fill(palette.surface_card)
                                .corner_radius(ThemeRounding::RADIUS_MD)
                                .stroke(palette.border_subtle)
                                .inner_margin(egui::Margin::symmetric(
                                    ThemeSpacing::MD as i8,
                                    ThemeSpacing::SM as i8,
                                ))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(spec.name)
                                                .strong()
                                                .size(13.0)
                                                .color(palette.text_primary),
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                render_export_badge(ui, spec.id, &palette, &tr);
                                                render_reading_badge(
                                                    ui,
                                                    spec.support,
                                                    &palette,
                                                    &tr,
                                                );
                                            },
                                        );
                                    });

                                    ui.add_space(ThemeSpacing::XXS);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "· {}",
                                                spec.extensions.join(", ")
                                            ))
                                            .size(12.0)
                                            .color(palette.text_secondary),
                                        )
                                        .wrap(),
                                    );

                                    ui.add_space(ThemeSpacing::XXS);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(spec.note)
                                                .size(11.5)
                                                .color(palette.text_secondary),
                                        )
                                        .wrap(),
                                    );
                                });

                            ui.add_space(ThemeSpacing::XS);
                        }
                    }

                    if total_matches == 0 {
                        ui.add_space(ThemeSpacing::XL);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} \"{}\".",
                                    tr.dialog_no_formats_found, app.format_search
                                ))
                                .color(palette.text_secondary),
                            );
                        });
                        ui.add_space(ThemeSpacing::XL);
                    }

                    // Seção de Bibliotecas do Ecossistema
                    let q = app.format_search.trim().to_lowercase();
                    let show_libraries = q.is_empty()
                        || "bibliotecas".contains(&q)
                        || "crates".contains(&q)
                        || visor::libraries().iter().any(|(name, role)| {
                            name.to_lowercase().contains(&q) || role.to_lowercase().contains(&q)
                        });

                    if show_libraries {
                        ui.add_space(ThemeSpacing::MD);
                        ui.label(
                            RichText::new(tr.dialog_libraries_title)
                                .size(14.0)
                                .strong()
                                .color(palette.text_primary),
                        );
                        ui.add_space(ThemeSpacing::XXS);

                        egui::Frame::NONE
                            .fill(palette.surface_card)
                            .corner_radius(ThemeRounding::RADIUS_MD)
                            .stroke(palette.border_subtle)
                            .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                            .show(ui, |ui| {
                                for (name, role) in visor::libraries() {
                                    if !q.is_empty()
                                        && !name.to_lowercase().contains(&q)
                                        && !role.to_lowercase().contains(&q)
                                        && !"bibliotecas".contains(&q)
                                    {
                                        continue;
                                    }
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(*name)
                                                .strong()
                                                .color(palette.accent_default),
                                        );
                                        ui.label(
                                            RichText::new(format!("· {role}"))
                                                .size(12.0)
                                                .color(palette.text_secondary),
                                        );
                                    });
                                    ui.add_space(ThemeSpacing::XXS);
                                }
                            });
                    }

                    ui.add_space(ThemeSpacing::MD);
                });
        });

    app.show_formats = open;
}

/// Renderiza a janela de Conversão em Lote moderna com cards organizados,
/// seleção de formatos suportados e status de exportação.
pub fn render_batch_window(app: &mut VisorApp, ctx: &Context) {
    let mut open = app.show_batch;
    let tr = *app.tr();
    let mut pick = false;
    let mut run = false;
    let mut clear = false;
    let palette = ThemePalette::dark();

    egui::Window::new(tr.batch_export)
        .open(&mut open)
        .default_size([520.0, 460.0])
        .min_size([440.0, 400.0])
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.input(|i| {
            i.raw
                .screen_rect
                .map(|r| egui::pos2(r.center().x, r.center().y - 30.0))
                .unwrap_or(egui::pos2(400.0, 270.0))
        }))
        .show(ctx, |ui| {
            // Card 1: Arquivos de Entrada
            egui::Frame::NONE
                .fill(palette.surface_card)
                .corner_radius(ThemeRounding::RADIUS_MD)
                .stroke(palette.border_subtle)
                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(tr.batch_input_files)
                                .strong()
                                .color(palette.text_primary),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let count_text =
                                format!("{} {}", app.batch_files.len(), tr.batch_file_word);
                            render_badge(
                                ui,
                                &count_text,
                                palette.accent_muted,
                                palette.accent_default,
                            );
                        });
                    });

                    ui.add_space(ThemeSpacing::SM);

                    if app.batch_files.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(ThemeSpacing::SM);
                            ui.label(RichText::new(tr.batch_empty).color(palette.text_muted));
                            ui.add_space(ThemeSpacing::SM);
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(120.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (i, path) in app.batch_files.iter().enumerate() {
                                    let filename = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(tr.batch_file_word);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{}.", i + 1))
                                                .size(11.0)
                                                .color(palette.text_muted),
                                        );
                                        let label = ui.label(
                                            RichText::new(filename)
                                                .size(12.0)
                                                .color(palette.text_secondary),
                                        );
                                        label.on_hover_text(path.display().to_string());
                                    });
                                }
                            });
                    }

                    ui.add_space(ThemeSpacing::SM);
                    ui.horizontal(|ui| {
                        if ui.button(tr.batch_add_files).clicked() {
                            pick = true;
                        }
                        if !app.batch_files.is_empty() && ui.button(tr.batch_clear_list).clicked() {
                            clear = true;
                        }
                    });
                });

            ui.add_space(ThemeSpacing::SM);

            // Card 2: Configurações de Formato de Saída
            egui::Frame::NONE
                .fill(palette.surface_card)
                .corner_radius(ThemeRounding::RADIUS_MD)
                .stroke(palette.border_subtle)
                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(tr.batch_target_format)
                            .strong()
                            .color(palette.text_primary),
                    );
                    ui.add_space(ThemeSpacing::SM);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(tr.batch_convert_to).color(palette.text_secondary));
                        egui::ComboBox::from_id_salt("batch_export_format")
                            .selected_text(RichText::new(app.batch_kind.label()).strong())
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for kind in visor::ExportKind::all() {
                                    ui.selectable_value(&mut app.batch_kind, *kind, kind.label());
                                }
                            });
                    });

                    ui.add_space(ThemeSpacing::SM);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("•").color(palette.accent_default).strong());
                        ui.label(
                            RichText::new(tr.batch_depth_note)
                                .size(11.5)
                                .color(palette.text_secondary),
                        );
                    });
                });

            ui.add_space(ThemeSpacing::SM);

            // Card 3: Status e Ação de Exportação
            egui::Frame::NONE
                .fill(palette.surface_card)
                .corner_radius(ThemeRounding::RADIUS_MD)
                .stroke(palette.border_subtle)
                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(tr.batch_dest_folder)
                            .strong()
                            .color(palette.text_primary),
                    );
                    ui.add_space(ThemeSpacing::SM);

                    if let Some(dir) = &app.last_dir {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(tr.batch_recent_folder)
                                    .size(12.0)
                                    .color(palette.text_secondary),
                            );
                            let lbl = ui.label(
                                RichText::new(dir.display().to_string())
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            lbl.on_hover_text(dir.display().to_string());
                        });
                    } else {
                        ui.label(
                            RichText::new(tr.batch_folder_prompt)
                                .size(12.0)
                                .color(palette.text_muted),
                        );
                    }

                    if let Some(notice) = &app.notice {
                        ui.add_space(ThemeSpacing::SM);
                        let is_converting =
                            notice.contains("Convertendo") || notice.contains("Converting");
                        let is_success = notice.contains("Lote:") || notice.contains("Batch:");
                        let banner_bg = if is_success {
                            Color32::from_rgba_unmultiplied(108, 203, 95, 25)
                        } else if is_converting {
                            palette.accent_muted
                        } else {
                            Color32::from_rgba_unmultiplied(255, 100, 100, 25)
                        };

                        let banner_border = if is_success {
                            palette.status_success
                        } else if is_converting {
                            palette.accent_default
                        } else {
                            palette.status_error
                        };

                        egui::Frame::NONE
                            .fill(banner_bg)
                            .corner_radius(ThemeRounding::RADIUS_SM)
                            .stroke(egui::Stroke::new(1.0, banner_border))
                            .inner_margin(egui::Margin::same(ThemeSpacing::SM as i8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let icon = if is_success {
                                        "OK:"
                                    } else if is_converting {
                                        "..."
                                    } else {
                                        "Note:"
                                    };
                                    ui.label(RichText::new(icon).strong().color(banner_border));
                                    ui.label(
                                        RichText::new(notice)
                                            .size(12.0)
                                            .color(palette.text_primary),
                                    );
                                });
                            });
                    }

                    ui.add_space(ThemeSpacing::MD);

                    // Botão primário CTA com estilo Accent Windows 11
                    let has_files = !app.batch_files.is_empty();
                    let btn = egui::Button::new(
                        RichText::new(tr.batch_export_button)
                            .color(if has_files {
                                palette.text_on_accent
                            } else {
                                palette.text_muted
                            })
                            .size(13.0)
                            .strong(),
                    )
                    .fill(if has_files {
                        palette.accent_default
                    } else {
                        palette.surface_hover
                    })
                    .min_size(Vec2::new(ui.available_width(), 34.0))
                    .corner_radius(ThemeRounding::RADIUS_SM);

                    if ui.add_enabled(has_files, btn).clicked() {
                        run = true;
                    }
                });
        });

    app.show_batch = open;
    if pick {
        app.want_batch_files = true;
    }
    if run {
        app.want_batch_dir = true;
    }
    if clear {
        app.batch_files.clear();
    }
}

// =============================================================================
// TESTES UNITÁRIOS DOS DIÁLOGOS MODERNOS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_format_exportable_honesty() {
        // Os 11 formatos reais suportados pelo VISOR
        let exportable_ids = [
            "png", "jpeg", "webp", "bmp", "tiff", "gif", "qoi", "tga", "ico", "pnm", "hdr",
        ];
        for id in exportable_ids {
            assert!(
                is_format_exportable(id),
                "Formato {id} deve ser marcado como exportável"
            );
        }

        // Formatos sem codificador no VISOR (devem ser estritamente false)
        let non_exportable_ids = [
            "svg", "emf", "jxl", "avif", "dds", "ktx", "ktx2", "exr", "pcx", "sgi", "ras", "wbmp",
            "cur", "jp2", "blp",
        ];
        for id in non_exportable_ids {
            assert!(
                !is_format_exportable(id),
                "Formato {id} NUNCA deve ser marcado como exportável"
            );
        }
    }

    #[test]
    fn test_format_matches_query_filtering() {
        let all_specs = visor::formats();
        let png_spec = all_specs.iter().find(|s| s.id == "png").expect("PNG spec");
        let dds_spec = all_specs.iter().find(|s| s.id == "dds").expect("DDS spec");

        // Busca vazia aceita qualquer formato
        assert!(format_matches_query(png_spec, ""));
        assert!(format_matches_query(dds_spec, "   "));

        // Busca por nome exato e parcial
        assert!(format_matches_query(png_spec, "PNG"));
        assert!(format_matches_query(png_spec, "apng"));
        assert!(!format_matches_query(png_spec, "DirectDraw"));

        // Busca por extensão
        assert!(format_matches_query(dds_spec, "dds"));
        assert!(!format_matches_query(dds_spec, "png"));

        // Busca por categoria
        assert!(format_matches_query(dds_spec, "Texturas"));
        assert!(format_matches_query(png_spec, "Raster"));
    }

    #[test]
    fn test_render_formats_window_mock_context() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_formats = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_formats_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.show_formats);

        // Testar com busca aplicada
        app.format_search = "webp".to_owned();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_formats_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert_eq!(app.format_search, "webp");
    }

    #[test]
    fn test_render_batch_window_mock_context() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_batch = true;

        // Estado inicial com zero arquivos
        assert!(app.batch_files.is_empty());
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_batch_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();

        // Adicionando arquivos fictícios
        app.batch_files.push(PathBuf::from("foto1.jpg"));
        app.batch_files.push(PathBuf::from("foto2.png"));
        app.notice = Some("Convertendo o lote…".to_owned());

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_batch_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert_eq!(app.batch_files.len(), 2);
    }

    #[test]
    fn test_render_formats_window_empty_and_no_match() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_formats = true;

        // Termo que não casa com nada
        app.format_search = "termo_completamente_inexistente_12345".to_owned();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_formats_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.show_formats);
    }

    #[test]
    fn test_render_formats_window_raw_long_extensions() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_formats = true;
        app.format_search = "raw".to_owned();

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_formats_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(app.show_formats);
    }

    #[test]
    fn test_dialogs_open_and_close_state() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.show_formats = false;
        app.show_batch = false;

        // Quando fechadas, renderizar não altera estado e não gera erro
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_formats_window(&mut app, ui.ctx());
            render_batch_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
        assert!(!app.show_formats);
        assert!(!app.show_batch);
    }
}
