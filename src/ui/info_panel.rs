//! Painel lateral moderno de informações técnicas, metadados fotográficos (EXIF) e perfil de cor.
//!
//! Inspirado na linguagem visual do macOS StillView e Windows 11 Fluent Design:
//! organização em cartões discretos, tipografia com forte hierarquia visual,
//! extração canônica de metadados de câmera/exposição e botão de cópia formatada.

use std::fs;
use std::path::Path;

use eframe::egui::{self, Align, Layout, Margin, RichText, Stroke, StrokeKind, Vec2};

use super::theme::{ThemePalette, ThemeRounding, ThemeSpacing};
use crate::app::state::VisorApp;
use crate::system::i18n::CategoryExt;

/// Renderiza o painel lateral de informações da imagem ativa.
pub fn render_info_panel(app: &mut VisorApp, ui: &mut egui::Ui) {
    let palette = ThemePalette::dark();
    let tr = *app.tr();

    // 1. Cabeçalho do Painel
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tr.info)
                .strong()
                .size(15.0)
                .color(palette.text_primary),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Botão Fechar [x]
            if ui
                .button(RichText::new("x").size(13.0).color(palette.text_secondary))
                .on_hover_text(tr.info_close_tooltip)
                .clicked()
            {
                app.show_info = false;
            }

            // Botão Copiar Resumo
            if app.picture.is_some() {
                if ui
                    .button(
                        RichText::new(tr.info_copy_btn)
                            .size(12.0)
                            .color(palette.text_secondary),
                    )
                    .on_hover_text(tr.info_copy_tooltip)
                    .clicked()
                {
                    let summary = generate_info_summary(app);
                    ui.ctx().copy_text(summary);
                    app.notice = Some(tr.info_copied_notice.to_owned());
                }
            }
        });
    });

    ui.add_space(ThemeSpacing::XS);

    // 2. Se nenhuma imagem estiver aberta, exibe mensagem limpa
    let Some(picture) = &app.picture else {
        render_card(ui, &palette, |ui| {
            ui.label(
                RichText::new(tr.info_no_image)
                    .color(palette.text_muted)
                    .size(13.0),
            );
            ui.label(
                RichText::new(tr.info_no_image_desc)
                    .color(palette.text_secondary)
                    .size(12.0),
            );
        });
        return;
    };

    let info = &picture.info;
    let frame = picture.frame();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, ThemeSpacing::SM);

            // =================================================================
            // SEÇÃO 1: INFORMAÇÕES PRINCIPAIS (GERAL)
            // =================================================================
            render_card(ui, &palette, |ui| {
                ui.label(
                    RichText::new(tr.info_section_general)
                        .size(11.0)
                        .strong()
                        .color(palette.text_secondary),
                );
                ui.add_space(2.0);

                // Nome do arquivo com truncamento elegante e tooltip
                let file_path = app.current_path();
                let display_name = file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| info.name.clone());

                let file_label = ui.add(
                    egui::Label::new(
                        RichText::new(&display_name)
                            .strong()
                            .size(13.0)
                            .color(palette.text_primary),
                    )
                    .truncate(),
                );
                if let Some(path) = &file_path {
                    file_label.on_hover_text(path.display().to_string());
                }

                ui.add_space(ThemeSpacing::XS);

                // Formato e Categoria
                render_kv_row(
                    ui,
                    &palette,
                    tr.format,
                    &format!("{} ({})", info.name, info.category.localized_label(&tr)),
                );

                // Dimensões em pixels e megapixels
                let mp = (info.original_w as f64 * info.original_h as f64) / 1_000_000.0;
                render_kv_row(
                    ui,
                    &palette,
                    tr.dimensions,
                    &format!(
                        "{} × {} px ({:.1} MP)",
                        info.original_w, info.original_h, mp
                    ),
                );

                if info.reduced {
                    render_kv_row(
                        ui,
                        &palette,
                        tr.info_display,
                        &format!(
                            "{} × {} px ({})",
                            frame.width, frame.height, tr.info_reduced
                        ),
                    );
                }

                // Proporção (Aspect Ratio)
                let ratio = calculate_aspect_ratio(info.original_w, info.original_h);
                render_kv_row(ui, &palette, tr.info_aspect_ratio, &ratio);

                // Tamanho em disco
                if let Some(path) = &file_path {
                    if let Ok(meta) = fs::metadata(path) {
                        let size_formatted = format_file_size(meta.len());
                        render_kv_row(ui, &palette, tr.file_size, &size_formatted);
                    }
                }
            });

            // =================================================================
            // SEÇÃO 2: INFORMAÇÕES FOTOGRÁFICAS (EXIF)
            // =================================================================
            render_card(ui, &palette, |ui| {
                ui.label(
                    RichText::new(tr.info_section_photo)
                        .size(11.0)
                        .strong()
                        .color(palette.text_secondary),
                );
                ui.add_space(2.0);

                let photo = extract_photographic_info(&info.exif);

                if photo.is_empty() {
                    ui.label(
                        RichText::new(tr.info_no_exif)
                            .color(palette.text_muted)
                            .size(12.0),
                    );
                } else {
                    if let Some(camera) = &photo.camera {
                        render_kv_row(ui, &palette, tr.info_camera, camera);
                    }
                    if let Some(lens) = photo.lens {
                        render_kv_row(ui, &palette, tr.info_lens, lens);
                    }
                    if let Some(focal) = photo.focal_length {
                        render_kv_row(ui, &palette, tr.info_focal_length, focal);
                    }
                    if let Some(aperture) = photo.aperture {
                        render_kv_row(ui, &palette, tr.info_aperture, aperture);
                    }
                    if let Some(shutter) = photo.shutter {
                        render_kv_row(ui, &palette, tr.info_shutter, shutter);
                    }
                    if let Some(iso) = photo.iso {
                        render_kv_row(ui, &palette, "ISO", iso);
                    }
                    if let Some(dt) = photo.datetime {
                        render_kv_row(ui, &palette, tr.info_captured, dt);
                    }

                    // Se houver tags adicionais, exibe em lista colapsável
                    if !photo.other_tags.is_empty() {
                        ui.add_space(ThemeSpacing::XS);
                        let more_label =
                            format!("{} ({})", tr.info_more_metadata, photo.other_tags.len());
                        ui.collapsing(
                            RichText::new(more_label)
                                .size(11.0)
                                .color(palette.text_secondary),
                            |ui| {
                                for (k, v) in photo.other_tags {
                                    render_kv_row(ui, &palette, k, v);
                                }
                            },
                        );
                    }
                }
            });

            // =================================================================
            // SEÇÃO 3: INFORMAÇÕES TÉCNICAS
            // =================================================================
            render_card(ui, &palette, |ui| {
                ui.label(
                    RichText::new(tr.info_section_technical)
                        .size(11.0)
                        .strong()
                        .color(palette.text_secondary),
                );
                ui.add_space(2.0);

                render_kv_row(
                    ui,
                    &palette,
                    tr.info_colors,
                    &format!("{} bits · {}", info.bit_depth, info.color),
                );

                let alpha_val = if info.has_alpha {
                    tr.info_alpha_yes
                } else {
                    tr.info_alpha_no
                };
                render_kv_row(ui, &palette, tr.info_alpha, alpha_val);

                let icc_val = info.icc.as_deref().unwrap_or(tr.info_icc_none);
                render_kv_row(ui, &palette, tr.info_icc, icc_val);

                if info.frame_count > 1 {
                    let anim_val = format!(
                        "{} ({} · {} ms)",
                        info.frame_count,
                        picture.frame_i + 1,
                        frame.delay_ms
                    );
                    render_kv_row(ui, &palette, tr.info_animation, &anim_val);
                }

                if info.mipmaps > 1 {
                    let mip_val = format!("{} {}", info.mipmaps, tr.info_resolution_levels);
                    render_kv_row(ui, &palette, "Mipmaps", &mip_val);
                }

                if !info.notes.is_empty() {
                    ui.add_space(2.0);
                    for note in &info.notes {
                        ui.label(RichText::new(note).size(11.0).color(palette.text_muted));
                    }
                }
            });
        });
}

/// Renderiza um container de card com superfícies Fluent Design.
fn render_card<R>(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let frame = egui::Frame::NONE
        .fill(palette.surface_card)
        .stroke(palette.border_subtle)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .inner_margin(Margin::same(10));

    frame.show(ui, add_contents).inner
}

/// Renderiza uma linha de chave-valor alinhada e hierárquica.
fn render_kv_row(ui: &mut egui::Ui, palette: &ThemePalette, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).size(12.0).color(palette.text_secondary));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add(
                egui::Label::new(RichText::new(value).size(12.0).color(palette.text_primary))
                    .truncate(),
            )
            .on_hover_text(value);
        });
    });
}

/// Calcula a proporção geométrica da imagem em forma de fração amigável ou decimal.
pub fn calculate_aspect_ratio(w: u32, h: u32) -> String {
    if w == 0 || h == 0 {
        return "0:0".to_owned();
    }
    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }
    let g = gcd(w, h);
    let rw = w / g;
    let rh = h / g;
    if rw <= 32 && rh <= 32 {
        format!("{rw}:{rh}")
    } else {
        format!("{:.2}:1", w as f32 / h.max(1) as f32)
    }
}

/// Formata bytes em unidades legíveis (B, KB, MB, GB).
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Estrutura de metadados fotográficos canônicos extraídos do EXIF.
#[derive(Default)]
pub struct PhotographicInfo<'a> {
    pub camera: Option<String>,
    pub lens: Option<&'a str>,
    pub aperture: Option<&'a str>,
    pub shutter: Option<&'a str>,
    pub iso: Option<&'a str>,
    pub focal_length: Option<&'a str>,
    pub datetime: Option<&'a str>,
    pub other_tags: Vec<(&'a str, &'a str)>,
}

impl<'a> PhotographicInfo<'a> {
    pub fn is_empty(&self) -> bool {
        self.camera.is_none()
            && self.lens.is_none()
            && self.aperture.is_none()
            && self.shutter.is_none()
            && self.iso.is_none()
            && self.focal_length.is_none()
            && self.datetime.is_none()
            && self.other_tags.is_empty()
    }
}

/// Extrai e padroniza metadados fotográficos do vetor bruto de tags EXIF.
pub fn extract_photographic_info<'a>(exif: &'a [(String, String)]) -> PhotographicInfo<'a> {
    let mut photo = PhotographicInfo::default();
    let mut make = None;
    let mut model = None;

    for (k, v) in exif {
        let tag = k.trim();
        let val = v.trim();
        match tag {
            "Make" => make = Some(val),
            "Model" => model = Some(val),
            "LensModel" | "Lens" => photo.lens = Some(val),
            "FNumber" | "ApertureValue" => photo.aperture = Some(val),
            "ExposureTime" | "ShutterSpeedValue" => photo.shutter = Some(val),
            "ISOSpeedRatings" | "PhotographicSensitivity" | "ISO" => photo.iso = Some(val),
            "FocalLength" | "FocalLengthIn35mmFilm" => photo.focal_length = Some(val),
            "DateTimeOriginal" | "DateTime" | "CreateDate" => photo.datetime = Some(val),
            _ => photo.other_tags.push((tag, val)),
        }
    }

    photo.camera = match (make, model) {
        (Some(mk), Some(md)) => {
            if md.starts_with(mk) {
                Some(md.to_owned())
            } else {
                Some(format!("{mk} {md}"))
            }
        }
        (Some(mk), None) => Some(mk.to_owned()),
        (None, Some(md)) => Some(md.to_owned()),
        (None, None) => None,
    };

    photo
}

/// Gera um sumário em texto limpo com todos os metadados para cópia.
pub fn generate_info_summary(app: &VisorApp) -> String {
    let Some(picture) = &app.picture else {
        return "Nenhuma imagem aberta.".to_owned();
    };
    let info = &picture.info;
    let mut out = String::new();

    out.push_str(&format!("Arquivo: {}\n", info.name));
    out.push_str(&format!(
        "Formato: {} ({})\n",
        info.name,
        info.category.label()
    ));
    let mp = (info.original_w as f64 * info.original_h as f64) / 1_000_000.0;
    out.push_str(&format!(
        "Dimensões: {} × {} px ({:.1} MP)\n",
        info.original_w, info.original_h, mp
    ));
    out.push_str(&format!(
        "Proporção: {}\n",
        calculate_aspect_ratio(info.original_w, info.original_h)
    ));
    out.push_str(&format!(
        "Profundidade: {} bits · {}\n",
        info.bit_depth, info.color
    ));
    out.push_str(&format!(
        "Transparência: {}\n",
        if info.has_alpha { "Sim" } else { "Não" }
    ));
    if let Some(icc) = &info.icc {
        out.push_str(&format!("Perfil ICC: {icc}\n"));
    }
    if info.frame_count > 1 {
        out.push_str(&format!("Quadros de animação: {}\n", info.frame_count));
    }

    let photo = extract_photographic_info(&info.exif);
    if !photo.is_empty() {
        out.push_str("\n--- Metadados EXIF ---\n");
        if let Some(c) = &photo.camera {
            out.push_str(&format!("Câmera: {c}\n"));
        }
        if let Some(l) = photo.lens {
            out.push_str(&format!("Lente: {l}\n"));
        }
        if let Some(f) = photo.focal_length {
            out.push_str(&format!("Distância Focal: {f}\n"));
        }
        if let Some(a) = photo.aperture {
            out.push_str(&format!("Abertura: {a}\n"));
        }
        if let Some(s) = photo.shutter {
            out.push_str(&format!("Obturador: {s}\n"));
        }
        if let Some(i) = photo.iso {
            out.push_str(&format!("ISO: {i}\n"));
        }
        if let Some(d) = photo.datetime {
            out.push_str(&format!("Data da Captura: {d}\n"));
        }
        for (k, v) in photo.other_tags {
            out.push_str(&format!("{k}: {v}\n"));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_test_picture(
        name: &str,
        w: u32,
        h: u32,
        exif: Vec<(String, String)>,
        icc: Option<String>,
        frames: usize,
    ) -> Arc<visor::Decoded> {
        let frame_bytes = (w as usize) * (h as usize) * 4;
        let frames_vec = (0..frames)
            .map(|_| visor::Frame {
                rgba: vec![128u8; frame_bytes],
                width: w,
                height: h,
                delay_ms: 100,
            })
            .collect();

        Arc::new(visor::Decoded {
            frames: frames_vec,
            info: visor::ImageInfo {
                id: "test",
                name: name.to_owned(),
                category: visor::Category::Raster,
                support: visor::Support::Full,
                bit_depth: 8,
                color: "RGB".to_owned(),
                original_w: w,
                original_h: h,
                display_w: w,
                display_h: h,
                reduced: false,
                has_alpha: true,
                frame_count: frames,
                loop_forever: false,
                mipmaps: 0,
                exif,
                icc,
                notes: Vec::new(),
            },
        })
    }

    #[test]
    fn test_calculate_aspect_ratio_values() {
        assert_eq!(calculate_aspect_ratio(1920, 1080), "16:9");
        assert_eq!(calculate_aspect_ratio(800, 600), "4:3");
        assert_eq!(calculate_aspect_ratio(1000, 1000), "1:1");
        assert_eq!(calculate_aspect_ratio(1200, 800), "3:2");
        assert_eq!(calculate_aspect_ratio(0, 100), "0:0");
    }

    #[test]
    fn test_format_file_size_scales() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2.0 KB");
        assert_eq!(format_file_size(1024 * 1024 * 3), "3.00 MB");
    }

    #[test]
    fn test_extract_photographic_info_complete() {
        let exif = vec![
            ("Make".to_owned(), "Sony".to_owned()),
            ("Model".to_owned(), "ILCE-7M4".to_owned()),
            ("FNumber".to_owned(), "f/2.8".to_owned()),
            ("ExposureTime".to_owned(), "1/500s".to_owned()),
            ("ISO".to_owned(), "100".to_owned()),
            ("FocalLength".to_owned(), "50mm".to_owned()),
            ("Software".to_owned(), "Adobe Lightroom".to_owned()),
        ];

        let photo = extract_photographic_info(&exif);
        assert!(!photo.is_empty());
        assert_eq!(photo.camera.as_deref(), Some("Sony ILCE-7M4"));
        assert_eq!(photo.aperture, Some("f/2.8"));
        assert_eq!(photo.shutter, Some("1/500s"));
        assert_eq!(photo.iso, Some("100"));
        assert_eq!(photo.focal_length, Some("50mm"));
        assert_eq!(photo.other_tags.len(), 1);
        assert_eq!(photo.other_tags[0], ("Software", "Adobe Lightroom"));
    }

    #[test]
    fn test_extract_photographic_info_empty() {
        let exif = Vec::new();
        let photo = extract_photographic_info(&exif);
        assert!(photo.is_empty());
    }

    #[test]
    fn test_info_panel_renders_empty_state_without_panic() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.picture = None;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_info_panel(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_info_panel_renders_with_complete_exif_and_icc() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let exif = vec![
            ("Make".to_owned(), "Canon".to_owned()),
            ("Model".to_owned(), "Canon EOS R5".to_owned()),
            ("FNumber".to_owned(), "f/1.8".to_owned()),
            ("ISO".to_owned(), "400".to_owned()),
        ];
        let decoded = make_test_picture(
            "sample.jpg",
            1920,
            1080,
            exif,
            Some("Display P3".to_owned()),
            1,
        );
        app.install(&ctx, decoded);

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_info_panel(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_info_panel_renders_animated_frames_and_long_name() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let long_name = "foto_com_nome_extremamente_longo_para_testar_truncamento_e_responsividade_no_painel_lateral_sem_quebrar_layout.png";
        let decoded = make_test_picture(long_name, 800, 600, Vec::new(), None, 12);
        app.install(&ctx, decoded);

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_info_panel(&mut app, ui);
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_generate_info_summary_contains_key_fields() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        let exif = vec![
            ("Make".to_owned(), "Nikon".to_owned()),
            ("Model".to_owned(), "Z8".to_owned()),
        ];
        let decoded = make_test_picture("nikon.jpg", 1920, 1080, exif, Some("sRGB".to_owned()), 1);
        app.install(&ctx, decoded);

        let summary = generate_info_summary(&app);
        assert!(summary.contains("Arquivo: nikon.jpg"));
        assert!(summary.contains("1920 × 1080 px"));
        assert!(summary.contains("16:9"));
        assert!(summary.contains("Câmera: Nikon Z8"));
        assert!(summary.contains("Perfil ICC: sRGB"));
    }
}
