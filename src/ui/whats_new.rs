//! Central do VISOR: Guia completo de recursos, formatos suportados, segurança e novidades.

use eframe::egui::{self, Color32, Context, RichText, Stroke, Vec2};

use crate::app::VisorApp;
use crate::system::i18n::ResolvedLanguage;
use crate::ui::theme::{ThemePalette, ThemeRounding, ThemeSpacing};
use crate::updater::whats_new::{
    EMBEDDED_CHANGELOG, get_current_version_notes, parse_all_versions, parse_bullet_item,
};

/// Abas de navegação da Central do VISOR.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CentralTab {
    Overview,
    Formats,
    Security,
    Changelog,
}

/// Renderiza o diálogo modal da Central do VISOR (Recursos, Suporte, Segurança & Novidades).
pub fn render_whats_new_window(app: &mut VisorApp, ctx: &Context) {
    if !app.updater.show_whats_new_window {
        return;
    }

    let tr = *app.tr();
    let lang = app.settings.language.resolve();
    let palette = ThemePalette::dark();
    let mut should_close = false;
    let mut open_formats_catalog = false;

    let notes = app
        .updater
        .whats_new_notes
        .clone()
        .unwrap_or_else(get_current_version_notes);

    // Enquadramento responsivo baseado na resolução da tela
    let screen = ctx.input(|i| {
        i.raw.screen_rect.unwrap_or(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(1024.0, 768.0),
        ))
    });
    let modal_width = (screen.width() - 48.0).clamp(420.0, 680.0);
    let modal_max_height = (screen.height() - 64.0).clamp(360.0, 620.0);

    #[allow(deprecated)]
    let modal_response = egui::Modal::new(egui::Id::new("whats_new_modal"))
        .backdrop_color(Color32::from_rgba_unmultiplied(8, 8, 12, 220))
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
            // 1. CABEÇALHO DA CENTRAL DO VISOR
            // -------------------------------------------------------------
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(palette.accent_muted)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("VISOR v{}", notes.version))
                                .size(11.5)
                                .strong()
                                .color(palette.accent_default),
                        );
                    });

                let date_str = notes.date.as_deref().unwrap_or(tr.channel_stable);
                ui.label(
                    RichText::new(format!("•  {date_str}"))
                        .size(11.5)
                        .color(palette.text_muted),
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
                        should_close = true;
                    }
                });
            });

            ui.add_space(ThemeSpacing::XS);

            ui.label(
                RichText::new(tr.hub_title)
                    .size(18.0)
                    .strong()
                    .color(palette.text_primary),
            );

            ui.label(
                RichText::new(tr.hub_subtitle)
                    .size(12.0)
                    .color(palette.text_secondary),
            );

            ui.add_space(ThemeSpacing::SM);

            // -------------------------------------------------------------
            // 2. SELETOR DE ABAS PRINCIPAIS
            // -------------------------------------------------------------
            #[allow(deprecated)]
            let tab_id = egui::Id::new("central_visor_active_tab");
            let mut active_tab = ui
                .data_mut(|d| d.get_temp(tab_id))
                .unwrap_or(CentralTab::Overview);

            ui.horizontal(|ui| {
                if ui
                    .selectable_label(active_tab == CentralTab::Overview, tr.hub_tab_overview)
                    .on_hover_text(tr.hub_tooltip_overview)
                    .clicked()
                {
                    active_tab = CentralTab::Overview;
                }
                if ui
                    .selectable_label(active_tab == CentralTab::Formats, tr.hub_tab_formats)
                    .on_hover_text(tr.hub_tooltip_formats)
                    .clicked()
                {
                    active_tab = CentralTab::Formats;
                }
                if ui
                    .selectable_label(active_tab == CentralTab::Security, tr.hub_tab_security)
                    .on_hover_text(tr.hub_tooltip_security)
                    .clicked()
                {
                    active_tab = CentralTab::Security;
                }
                if ui
                    .selectable_label(active_tab == CentralTab::Changelog, tr.hub_tab_changelog)
                    .on_hover_text(tr.hub_tooltip_changelog)
                    .clicked()
                {
                    active_tab = CentralTab::Changelog;
                }
            });
            ui.data_mut(|d| d.insert_temp(tab_id, active_tab));

            ui.add_space(ThemeSpacing::XS);
            ui.separator();
            ui.add_space(ThemeSpacing::XS);

            // -------------------------------------------------------------
            // 3. CONTEÚDO ROLÁVEL POR ABA
            // -------------------------------------------------------------
            let scroll_height = (modal_max_height - 185.0).max(140.0);
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(ThemeSpacing::SM, ThemeSpacing::MD);

                    match active_tab {
                        // =====================================================
                        // ABA 1: VISÃO GERAL & RECURSOS
                        // =====================================================
                        CentralTab::Overview => {
                            let cards = get_overview_cards(lang);
                            let colors = [
                                palette.accent_default,
                                palette.accent_hover,
                                palette.status_success,
                                palette.status_warning,
                                palette.accent_default,
                                palette.text_secondary,
                            ];
                            for (card, color) in cards.iter().zip(colors.iter()) {
                                render_feature_card(
                                    ui, card.title, card.desc, card.tag, *color, &palette,
                                );
                            }
                        }

                        // =====================================================
                        // ABA 2: FORMATOS SUPORTADOS (75+ EXTENSÕES)
                        // =====================================================
                        CentralTab::Formats => {
                            egui::Frame::NONE
                                .fill(palette.surface_active)
                                .corner_radius(ThemeRounding::RADIUS_MD)
                                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(tr.hub_formats_honest_title)
                                                .strong()
                                                .size(13.0)
                                                .color(palette.text_primary),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .button(
                                                        RichText::new(tr.hub_open_catalog)
                                                            .size(11.5)
                                                            .strong(),
                                                    )
                                                    .clicked()
                                                {
                                                    open_formats_catalog = true;
                                                }
                                            },
                                        );
                                    });
                                    ui.add_space(ThemeSpacing::XXS);
                                    ui.label(
                                        RichText::new(tr.hub_formats_honest_desc)
                                            .size(11.5)
                                            .color(palette.text_secondary),
                                    );
                                });

                            let cats = get_format_categories(lang);
                            for cat in &cats {
                                render_format_category_card(ui, cat.title, cat.list, &palette);
                            }
                        }

                        // =====================================================
                        // ABA 3: SEGURANÇA & PRIVACIDADE
                        // =====================================================
                        CentralTab::Security => {
                            let cards = get_security_cards(lang);
                            for card in &cards {
                                render_security_card(ui, card.tag, card.title, card.desc, &palette);
                            }
                        }

                        // =====================================================
                        // ABA 4: NOVIDADES & VERSÃO 1.0
                        // =====================================================
                        CentralTab::Changelog => {
                            // Manifesto Rumo à Versão 1.0
                            egui::Frame::NONE
                                .fill(palette.surface_active)
                                .corner_radius(ThemeRounding::RADIUS_MD)
                                .stroke(palette.border_default)
                                .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(tr.hub_roadmap_title)
                                                .strong()
                                                .size(13.5)
                                                .color(palette.accent_default),
                                        );
                                    });
                                    ui.add_space(ThemeSpacing::XXS);
                                    ui.label(
                                        RichText::new(tr.hub_roadmap_desc)
                                            .size(12.0)
                                            .color(palette.text_primary),
                                    );
                                });

                            ui.add_space(ThemeSpacing::XS);

                            // Categorias da versão atual
                            if !notes.added.is_empty() {
                                render_category_card(
                                    ui,
                                    tr.hub_tag_new,
                                    &format!("{} v{}", tr.hub_title_new, notes.version),
                                    &notes.added,
                                    palette.accent_default,
                                    &palette,
                                );
                            }

                            if !notes.changed.is_empty() {
                                render_category_card(
                                    ui,
                                    tr.hub_tag_improvement,
                                    tr.hub_title_improvement,
                                    &notes.changed,
                                    palette.accent_hover,
                                    &palette,
                                );
                            }

                            if !notes.fixed.is_empty() {
                                render_category_card(
                                    ui,
                                    tr.hub_tag_fix,
                                    tr.hub_title_fix,
                                    &notes.fixed,
                                    palette.status_success,
                                    &palette,
                                );
                            }

                            // Histórico de versões
                            let all_versions = parse_all_versions(EMBEDDED_CHANGELOG);
                            if all_versions.len() > 1 {
                                ui.add_space(ThemeSpacing::XS);
                                ui.label(
                                    RichText::new(tr.hub_history_title)
                                        .size(13.0)
                                        .strong()
                                        .color(palette.text_secondary),
                                );

                                for v in all_versions.iter().skip(1) {
                                    let date_str =
                                        v.date.as_deref().unwrap_or(tr.hub_history_date_unrecorded);

                                    egui::Frame::NONE
                                        .fill(palette.surface_card)
                                        .corner_radius(ThemeRounding::RADIUS_MD)
                                        .stroke(palette.border_subtle)
                                        .inner_margin(egui::Margin::same(ThemeSpacing::SM as i8))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let version_prefix = format!(
                                                    "{} {}",
                                                    tr.hub_history_version_prefix, v.version
                                                );
                                                ui.label(
                                                    RichText::new(version_prefix)
                                                        .size(12.5)
                                                        .strong()
                                                        .color(palette.text_primary),
                                                );
                                                ui.label(
                                                    RichText::new(format!("• {date_str}"))
                                                        .size(11.0)
                                                        .color(palette.text_muted),
                                                );
                                            });

                                            let count = v.added.len()
                                                + v.changed.len()
                                                + v.fixed.len()
                                                + v.security.len();
                                            let count_desc =
                                                format!("{count} {}", tr.hub_history_count_desc);
                                            ui.label(
                                                RichText::new(count_desc)
                                                    .size(11.0)
                                                    .color(palette.text_muted),
                                            );
                                        });
                                }
                            }
                        }
                    }
                });

            // -------------------------------------------------------------
            // 4. RODAPÉ COM AÇÕES
            // -------------------------------------------------------------
            ui.add_space(ThemeSpacing::SM);
            ui.separator();
            ui.add_space(ThemeSpacing::XS);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new(
                        RichText::new(tr.close)
                            .size(13.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    )
                    .fill(palette.accent_default)
                    .corner_radius(ThemeRounding::RADIUS_SM)
                    .min_size(Vec2::new(96.0, 30.0));

                    if ui.add(close_btn).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if open_formats_catalog {
        app.show_formats = true;
        app.updater.show_whats_new_window = false;
    } else if modal_response.should_close() || should_close {
        app.updater.show_whats_new_window = false;
    }
}

/// Renderiza um cartão estilizado de funcionalidade ou conceito do VISOR.
fn render_feature_card(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    tag: &str,
    tag_color: Color32,
    palette: &ThemePalette,
) {
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(
                        tag_color.r(),
                        tag_color.g(),
                        tag_color.b(),
                        36,
                    ))
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(RichText::new(tag).size(9.5).strong().color(tag_color));
                    });

                ui.label(
                    RichText::new(title)
                        .size(13.5)
                        .strong()
                        .color(palette.text_primary),
                );
            });

            ui.add_space(ThemeSpacing::XS);
            ui.label(
                RichText::new(description)
                    .size(11.5)
                    .color(palette.text_secondary),
            );
        });
}

/// Renderiza um cartão de categoria de formato suportado.
fn render_format_category_card(
    ui: &mut egui::Ui,
    title: &str,
    formats_list: &str,
    palette: &ThemePalette,
) {
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .size(13.0)
                    .strong()
                    .color(palette.text_primary),
            );
            ui.add_space(ThemeSpacing::XXS);
            ui.label(
                RichText::new(formats_list)
                    .size(11.5)
                    .color(palette.text_secondary),
            );
        });
}

/// Renderiza um card explicativo de segurança e privacidade.
fn render_security_card(
    ui: &mut egui::Ui,
    tag: &str,
    title: &str,
    description: &str,
    palette: &ThemePalette,
) {
    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(palette.accent_muted)
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(tag)
                                .size(9.5)
                                .strong()
                                .color(palette.accent_default),
                        );
                    });

                ui.label(
                    RichText::new(title)
                        .size(13.0)
                        .strong()
                        .color(palette.text_primary),
                );
            });

            ui.add_space(ThemeSpacing::XS);
            ui.label(
                RichText::new(description)
                    .size(11.5)
                    .color(palette.text_secondary),
            );
        });
}

/// Renderiza um card moderno categorizado de novidades com tag badge colorida.
fn render_category_card(
    ui: &mut egui::Ui,
    tag: &str,
    title: &str,
    items: &[String],
    tag_color: egui::Color32,
    palette: &ThemePalette,
) {
    if items.is_empty() {
        return;
    }

    egui::Frame::NONE
        .fill(palette.surface_card)
        .corner_radius(ThemeRounding::RADIUS_MD)
        .stroke(palette.border_subtle)
        .inner_margin(egui::Margin::same(ThemeSpacing::MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(
                        tag_color.r(),
                        tag_color.g(),
                        tag_color.b(),
                        36,
                    ))
                    .corner_radius(ThemeRounding::RADIUS_XS)
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(RichText::new(tag).size(10.0).strong().color(tag_color));
                    });

                ui.label(
                    RichText::new(title)
                        .size(13.5)
                        .strong()
                        .color(palette.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let count_text = format!(
                        "{} item{}",
                        items.len(),
                        if items.len() > 1 { "s" } else { "" }
                    );
                    egui::Frame::NONE
                        .fill(palette.surface_hover)
                        .corner_radius(ThemeRounding::RADIUS_XS)
                        .inner_margin(egui::Margin::symmetric(6, 2))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(count_text)
                                    .size(10.5)
                                    .color(palette.text_muted),
                            );
                        });
                });
            });

            ui.add_space(ThemeSpacing::XS);
            ui.separator();
            ui.add_space(ThemeSpacing::XS);

            for item in items {
                let (title_opt, desc) = parse_bullet_item(item);
                if let Some(t) = title_opt {
                    if desc.is_empty() {
                        ui.add_space(ThemeSpacing::XS);
                        ui.label(
                            RichText::new(format!("•  {t}"))
                                .size(12.5)
                                .strong()
                                .color(palette.text_primary),
                        );
                    } else {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(format!("•  {t}: {desc}"))
                                .size(12.0)
                                .color(palette.text_secondary),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new(format!("   - {desc}"))
                            .size(11.5)
                            .color(palette.text_secondary),
                    );
                }
            }
        });
}

// =============================================================================
// ESTRUTURAS DE CONTEÚDO LOCALIZADO (CENTRAL DO VISOR)
// =============================================================================

struct ContentCard {
    tag: &'static str,
    title: &'static str,
    desc: &'static str,
}

struct FormatCatCard {
    title: &'static str,
    list: &'static str,
}

fn get_overview_cards(lang: ResolvedLanguage) -> [ContentCard; 6] {
    match lang {
        ResolvedLanguage::ZhCn => [
            ContentCard {
                tag: "理念",
                title: "什么是 VISOR？",
                desc: "VISOR 专为解决传统图像查看器启动缓慢和臃肿遥测而打造。基于 Rust 开发，采用 GPU 硬件加速渲染与 Windows 11 Fluent 现代美学设计。VISOR 秒速启动、静默休眠时零内存占用，并始终将您的图像文件安全保留在本地设备上。",
            },
            ContentCard {
                tag: "性能",
                title: "超快渲染引擎与连续平滑缩放",
                desc: "• 以光标为锚点的无级缩放（1% 至 6400%），缩放过程始终保持焦点清晰。\n• 智能双击：在“窗口适应”与“100% 原始尺寸”之间无缝切换。\n• 动态 Pixel Perfect 算法：摄影照片采用双线性平滑插值，像素艺术与游戏纹理采用锐利无模糊的邻近采样。\n• 单击拖拽平移，支持深色/浅色网格透明背景、纯黑/纯白背景及 2D 取色器自定义背景。",
            },
            ContentCard {
                tag: "导航",
                title: "胶卷式缩略图预览条与预加载",
                desc: "• 屏幕底部可收缩的缩略图胶卷条，精准保持各图像长宽比。\n• 协作式后台线程池平滑加载缩略图，绝不卡顿操作界面。\n• 解耦滚动浏览：鼠标在文件夹中自由翻看，视口不会被强行拉回当前图片。",
            },
            ContentCard {
                tag: "摄影",
                title: "技术元数据侧边面板 (EXIF)",
                desc: "• 深度提取专业摄影参数：相机型号、镜头、光圈 (f-stop)、ISO、快门速度与焦距。\n• 识别嵌入式 ICC 色彩配置文件与 GPS 地理坐标。\n• 一键“复制”按钮：直接将格式化技术摘要导出到系统剪贴板。",
            },
            ContentCard {
                tag: "集成",
                title: "高保真系统剪贴板",
                desc: "• 按下 Ctrl+C 时，以最高保真度完整保留透明通道复制图片，即刻粘贴至任意图像编辑器、浏览器或即时通讯软件。",
            },
            ContentCard {
                tag: "工具",
                title: "内置批量转换器 (Batch Export)",
                desc: "• 将整个文件夹或选定文件一键批量转换为 11 种支持的光栅格式（PNG、JPEG、WebP、BMP、TIFF、GIF、QOI、TGA、ICO、PNM、HDR），在对应的输出格式中完整保留 16 位深度。",
            },
        ],
        ResolvedLanguage::JaJp => [
            ContentCard {
                tag: "コンセプト",
                title: "VISOR とは？",
                desc: "VISOR は、従来の画像ビューアーの起動の遅さや過度なテレメトリを解決するために開発されました。Rust 言語で構築され、GPU ハードウェアアクセラレーションと Windows 11 Fluent デザインを採用。即座に起動し、待機時のメモリ消費はゼロ。ファイルは常にPC上で安全に保たれます。",
            },
            ContentCard {
                tag: "パフォーマンス",
                title: "超高速レンダリングエンジン＆連続ズーム",
                desc: "• カーソル位置を基準にした 1%〜6400% の連続ズーム（フォーカスがブレません）。\n• スマートダブルクリック：画面フィットと 100%（原寸大）を瞬時に切り替え。\n• ダイナミック Pixel Perfect アルゴリズム：写真には滑らかな補間、ドット絵やテクスチャには滲みのない Nearest Neighbor。\n• ドラッグによる直感的なパン操作。明暗チェッカー、単色黒/白、2Dカラーピッカーによるカスタム背景に対応。",
            },
            ContentCard {
                tag: "ナビゲーション",
                title: "フィルムストリップ＆先読みキャッシュ",
                desc: "• 各画像のアスペクト比を忠実に維持する、画面下部の格納式フィルムストリップ。\n• UIをフリーズさせずにサムネイルを読み込む協調型バックグラウンドスレッドプール。\n• デカップリングスクロール：フォルダー内をマウスで自由に巡回しても、表示が強制的に引き戻されません。",
            },
            ContentCard {
                tag: "撮影データ",
                title: "技術情報サイドパネル (EXIF)",
                desc: "• 撮影パラメータの詳細抽出：カメラ、レンズ、絞り (F値)、ISO、シャッター速度、焦点距離。\n• 埋め込み ICC カラープロファイルおよび GPS 位置情報の検出。\n• ワンクリック「コピー」ボタン：整形された技術サマリーをクリップボードへ直接エクスポート。",
            },
            ContentCard {
                tag: "システム統合",
                title: "高忠実度クリップボード",
                desc: "• Ctrl+C を押すと、透過情報を完全保持した最高画質で画像をコピーし、各種画像編集ソフトやブラウザに即座に貼り付け可能です。",
            },
            ContentCard {
                tag: "ユーティリティ",
                title: "一括バッチ変換機能 (Batch Export)",
                desc: "• フォルダー全体または選択したファイル群を、11 種類の対応ラスター形式（PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR）に一括変換。対応形式では 16ビット深度を保持します。",
            },
        ],
        ResolvedLanguage::PtBr => [
            ContentCard {
                tag: "CONCEITO",
                title: "O que é o VISOR?",
                desc: "O VISOR foi criado para resolver a lentidão e o excesso de telemetria dos visualizadores convencionais. Desenvolvido em Rust com renderização acelerada por GPU e estética inspirada no Windows 11 Fluent Design, o VISOR inicia instantaneamente, não consome memória em repouso e mantém seus arquivos sempre seguros no seu computador.",
            },
            ContentCard {
                tag: "PERFORMANCE",
                title: "Motor de Renderização & Zoom Ultrarrápido",
                desc: "• Zoom contínuo ancorado no cursor de 1% a 6400% sem perda de foco.\n• Duplo clique inteligente: alterna entre Ajustar à Tela (Fit) e 100% (Tamanho Real).\n• Algoritmo dinâmico Pixel Perfect: interpolação suave para fotografias e 'Nearest Neighbor' nítido sem borrões para pixel art ou inspeção de texturas.\n• Pan com clique e arrasto e suporte a fundos com xadrez claro/escuro, cores sólidas ou personalizadas com seletor 2D.",
            },
            ContentCard {
                tag: "NAVEGAÇÃO",
                title: "Tira de Miniaturas (Filmstrip) & Pré-carregamento",
                desc: "• Tira retrátil na base da tela com proporções fiéis de cada imagem.\n• Thread pool cooperativo em segundo plano que carrega miniaturas sem travar a interface.\n• Scroll desacoplado: navegue livremente pela pasta com o mouse sem que a visualização seja puxada à força de volta à foto atual.",
            },
            ContentCard {
                tag: "FOTOGRAFIA",
                title: "Painel Lateral de Metadados Técnicos (EXIF)",
                desc: "• Extração profunda de parâmetros fotográficos: Câmera, Lente, Abertura (f-stop), ISO, Velocidade do Obturador e Distância Focal.\n• Detecção de perfis de cor ICC incorporados e coordenadas geográficas GPS.\n• Botão 'Copiar': exporta todo o resumo técnico formatado diretamente para a Área de Transferência.",
            },
            ContentCard {
                tag: "INTEGRAÇÃO",
                title: "Área de Transferência com Alta Fidelidade",
                desc: "• Ao pressionar Ctrl+C, a imagem é transferida com máxima fidelidade e preservação de transparência, pronta para colar diretamente em qualquer editor de imagens, navegador ou mensageiro.",
            },
            ContentCard {
                tag: "UTILITÁRIO",
                title: "Conversor em Lote Integrado (Batch Export)",
                desc: "• Converte pastas inteiras ou conjuntos de arquivos selecionados para 11 formatos raster suportados (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) preservando profundidades de 16 bits na saída correspondente.",
            },
        ],
        ResolvedLanguage::EsEs => [
            ContentCard {
                tag: "CONCEPTO",
                title: "¿Qué es VISOR?",
                desc: "VISOR fue creado para solucionar la lentitud y el exceso de telemetría de los visores convencionales. Desarrollado en Rust con aceleración por GPU y diseño Fluent de Windows 11, VISOR inicia al instante, no consume memoria en reposo y mantiene sus archivos seguros en su equipo.",
            },
            ContentCard {
                tag: "RENDIMIENTO",
                title: "Motor de Renderizado y Zoom Ultrarrápido",
                desc: "• Zoom continuo anclado al cursor del 1% al 6400% sin perder nitidez.\n• Doble clic inteligente: alterna entre Ajustar a Pantalla y 100% (Tamaño Real).\n• Algoritmo Pixel Perfect: interpolación suave para fotos y Nearest Neighbor nítido para pixel art y texturas.\n• Desplazamiento por arrastre y fondos con cuadrícula clara/oscura, colores sólidos o personalizados.",
            },
            ContentCard {
                tag: "NAVEGACIÓN",
                title: "Tira de Miniaturas (Filmstrip) y Precarga",
                desc: "• Tira retráctil en la base con proporciones fieles de cada imagen.\n• Carga en segundo plano mediante hilos cooperativos sin bloquear la interfaz.\n• Desplazamiento desacoplado: explore la carpeta con el ratón sin saltos forzados a la imagen actual.",
            },
            ContentCard {
                tag: "FOTOGRAFÍA",
                title: "Panel Lateral de Metadatos Técnicos (EXIF)",
                desc: "• Extracción profunda de parámetros fotográficos: Cámara, Lente, Apertura, ISO, Obturador y Distancia Focal.\n• Detección de perfiles de color ICC incrustados y coordenadas GPS.\n• Botón 'Copiar': exporta el resumen técnico formateado directamente al Portapapeles.",
            },
            ContentCard {
                tag: "INTEGRACIÓN",
                title: "Portapapeles de Alta Fidelidad",
                desc: "• Al pulsar Ctrl+C, la imagen se copia con máxima fidelidad y preservación de transparencia, lista para pegar en cualquier editor, navegador o aplicación.",
            },
            ContentCard {
                tag: "UTILIDAD",
                title: "Conversor por Lotes Integrado (Batch Export)",
                desc: "• Convierta carpetas o selecciones a 11 formatos raster (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) conservando la profundidad de 16 bits en la salida.",
            },
        ],
        ResolvedLanguage::DeDe => [
            ContentCard {
                tag: "KONZEPT",
                title: "Was ist VISOR?",
                desc: "VISOR wurde entwickelt, um die Trägheit und übermäßige Telemetrie herkömmlicher Bildbetrachter zu beseitigen. Geschrieben in purem Rust mit GPU-Hardwarebeschleunigung und modernem Windows 11 Fluent Design startet VISOR blitzschnell, verbraucht im Leerlauf 0% CPU und schützt Ihre Privatsphäre zu 100% offline.",
            },
            ContentCard {
                tag: "LEISTUNG",
                title: "Ultraschnelle Render-Engine & stufenloser Zoom",
                desc: "• Cursor-verankerter stufenloser Zoom von 1% bis 6400% ohne Schärfeverlust.\n• Intelligenter Doppelklick: wechselt sofort zwischen Fensteranpassung und 100% (Originalgröße).\n• Dynamischer Pixel-Perfect-Modus: sanfte bilineare Filterung für Fotos, scharfes Nearest Neighbor für Pixel-Art und Texturen.\n• Intuitives Verschieben per Mauszug und Schachbrett-/Einfarbenhintergründe.",
            },
            ContentCard {
                tag: "NAVIGATION",
                title: "Miniatur-Filmleiste & asynchrones Vorladen",
                desc: "• Einziehbare Filmleiste am unteren Bildschirmrand mit originalgetreuen Seitenverhältnissen.\n• Asynchroner Hintergrund-Thread-Pool lädt Miniaturansichten ohne UI-Verzögerung.\n• Entkoppeltes Scrollen: bequemes Durchsuchen des Ordners ohne erzwungenes Zurückspringen.",
            },
            ContentCard {
                tag: "FOTOGRAFIE",
                title: "Technisches EXIF-Metadaten-Seitenpanel",
                desc: "• Tiefgehende Extraktion: Kamera, Objektiv, Blende (f-stop), ISO, Belichtungszeit und Brennweite.\n• Erkennung eingebetteter ICC-Farbprofile und GPS-Koordinaten.\n• 'Kopieren'-Schaltfläche: formatiert und exportiert alle Daten direkt in die Zwischenablage.",
            },
            ContentCard {
                tag: "INTEGRATION",
                title: "Verlustfreie Zwischenablage",
                desc: "• Drücken von Strg+C kopiert das Bild in maximaler Qualität inklusive Alphakanal-Transparenz zum direkten Einfügen in Editoren und Browser.",
            },
            ContentCard {
                tag: "WERKZEUGE",
                title: "Integrierter Stapelkonverter (Batch Export)",
                desc: "• Konvertieren Sie ganze Verzeichnisse in 11 Rasterformate (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) mit bis zu 16-Bit Farbtiefe.",
            },
        ],
        ResolvedLanguage::FrFr => [
            ContentCard {
                tag: "CONCEPT",
                title: "Qu'est-ce que VISOR ?",
                desc: "VISOR a été conçu pour éliminer la lenteur et la télémétrie excessive des visionneuses d'images traditionnelles. Développé en Rust natif avec accélération matérielle GPU et esthétique Fluent Windows 11, VISOR démarre instantanément, ne consomme aucune mémoire au repos et conserve vos fichiers strictement privés.",
            },
            ContentCard {
                tag: "PERFORMANCE",
                title: "Moteur de rendu ultrarapide & zoom continu",
                desc: "• Zoom fluide ancré au curseur de 1% à 6400% sans perte de mise au point.\n• Double-clic intelligent : bascule instantanément entre ajustement à l'écran et 100% (taille réelle).\n• Algorithme Pixel Perfect : interpolation douce pour les photos, échantillonnage net au plus proche pour le pixel art.\n• Déplacement panoramique par glisser-déposer et arrière-plans personnalisables.",
            },
            ContentCard {
                tag: "NAVIGATION",
                title: "Bandeau de miniatures & préchargement",
                desc: "• Bandeau inférieur rétractable préservant fidèlement les proportions de chaque image.\n• Chargement multitâche en arrière-plan sans aucun blocage de l'interface.\n• Défilement découplé : parcourez le dossier à la molette sans recentrage forcé.",
            },
            ContentCard {
                tag: "PHOTOGRAPHIE",
                title: "Panneau latéral de métadonnées techniques (EXIF)",
                desc: "• Analyse approfondie : appareil photo, objectif, ouverture, ISO, temps de pose et focale.\n• Détection des profils colorimétriques ICC intégrés et coordonnées géographiques GPS.\n• Bouton 'Copier' : exporte le résumé technique formaté directement dans le Presse-papiers.",
            },
            ContentCard {
                tag: "INTÉGRATION",
                title: "Presse-papiers haute fidélité",
                desc: "• La combinaison Ctrl+C copie l'image avec préservation absolue de la transparence et des détails, prête à coller dans vos logiciels d'édition.",
            },
            ContentCard {
                tag: "UTILITAIRE",
                title: "Convertisseur par lots intégré (Batch Export)",
                desc: "• Convertissez des dossiers entiers vers 11 formats raster pris en charge (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) jusqu'à 16 bits de profondeur.",
            },
        ],
        ResolvedLanguage::ItIt => [
            ContentCard {
                tag: "CONCETTO",
                title: "Cos'è VISOR?",
                desc: "VISOR è stato creato per eliminare la lentezza e la telemetria invasiva dei visualizzatori tradizionali. Sviluppato in puro Rust con accelerazione GPU e design Fluent per Windows 11, VISOR si avvia all'istante, non consuma memoria inattivo e mantiene i tuoi file 100% al sicuro offline.",
            },
            ContentCard {
                tag: "PRESTAZIONI",
                title: "Motore di rendering ultrarapido & zoom continuo",
                desc: "• Zoom continuo ancorato al cursore dall'1% al 6400% senza perdere nitidezza.\n• Doppio clic intelligente: alterna tra Adatta alla Finestra e 100% (Dimensioni Reali).\n• Algoritmo dinamico Pixel Perfect: campionamento bilineare per le foto e Nearest Neighbor nitido per pixel art.\n• Panoramica fluida trascinando il mouse e sfondi personalizzabili.",
            },
            ContentCard {
                tag: "NAVIGAZIONE",
                title: "Pellicola di anteprime & precaricamento asincrono",
                desc: "• Pellicola inferiore a scomparsa che rispetta le proporzioni originali delle immagini.\n• Pool di thread in background che carica le miniature senza bloccare l'interfaccia utente.\n• Scorrimento disaccoppiato: naviga liberamente nella cartella senza salti forzati.",
            },
            ContentCard {
                tag: "FOTOGRAFIA",
                title: "Pannello laterale metadati tecnici (EXIF)",
                desc: "• Estrazione profonda dei parametri fotografici: fotocamera, obiettivo, apertura, ISO, otturatore.\n• Rilevamento dei profili colore ICC incorporati e coordinate GPS.\n• Pulsante 'Copia': esporta il riassunto tecnico direttamente negli Appunti di sistema.",
            },
            ContentCard {
                tag: "INTEGRAZIONE",
                title: "Appunti di sistema ad alta fedeltà",
                desc: "• Premendo Ctrl+C l'immagine viene copiata con piena fedeltà e trasparenza, pronta per essere incollata in qualsiasi app o browser.",
            },
            ContentCard {
                tag: "STRUMENTI",
                title: "Convertitore in batch integrato (Batch Export)",
                desc: "• Converti intere cartelle o file selezionati in 11 formati raster supportati preservando la profondità fino a 16 bit in output.",
            },
        ],
        ResolvedLanguage::RuRu => [
            ContentCard {
                tag: "КОНЦЕПЦИЯ",
                title: "Что такое VISOR?",
                desc: "VISOR создан для решения проблем медлительности и навязчивой телеметрии стандартных программ просмотра. Разработан на Rust с аппаратным ускорением GPU и эстетикой Fluent Design Windows 11. Открывается мгновенно, не тратит память в простое и гарантирует 100% офлайн-приватность.",
            },
            ContentCard {
                tag: "СКОРОСТЬ",
                title: "Сверхбыстрый рендеринг и плавный зум",
                desc: "• Плавное масштабирование от 1% до 6400% с привязкой к позиции курсора.\n• Умный двойной клик: мгновенное переключение между «По размеру окна» и 100% (1:1).\n• Режим Pixel Perfect: гладкая фильтрация для фото и сверхчеткий Nearest Neighbor для пиксель-арта.\n• Перетаскивание зажатой мышью и выбор фона (шахматный, черный, белый или произвольный).",
            },
            ContentCard {
                tag: "НАВИГАЦИЯ",
                title: "Лента миниатюр и фоновый кэш",
                desc: "• Выдвижная нижняя панель с точным сохранением пропорций каждого кадра.\n• Фоновый пул потоков плавно готовит миниатюры без зависания интерфейса.\n• Независимая прокрутка: свободный просмотр каталога без принудительного возврата фокуса.",
            },
            ContentCard {
                tag: "ФОТОГРАФИЯ",
                title: "Техническая панель метаданных (EXIF)",
                desc: "• Детальный анализ: камера, объектив, диафрагма (f-число), ISO, выдержка и фокусное расстояние.\n• Чтение цветовых профилей ICC и географических координат GPS.\n• Кнопка «Копировать»: быстрый экспорт технического отчета в буфер обмена.",
            },
            ContentCard {
                tag: "ИНТЕГРАЦИЯ",
                title: "Высокоточный буфер обмена",
                desc: "• По нажатию Ctrl+C изображение копируется с сохранением прозрачности и максимального качества для вставки в любые редакторы и мессенджеры.",
            },
            ContentCard {
                tag: "ИНСТРУМЕНТЫ",
                title: "Встроенный пакетный конвертер",
                desc: "• Преобразование целых папок в 11 растровых форматов (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) с сохранением до 16 бит глубины цвета.",
            },
        ],
        _ => [
            ContentCard {
                tag: "CONCEPT",
                title: "What is VISOR?",
                desc: "VISOR was built to solve the slowness and excessive telemetry of conventional image viewers. Developed in Rust with GPU-accelerated rendering and aesthetics inspired by Windows 11 Fluent Design, VISOR launches instantly, consumes zero memory at idle, and keeps your files strictly secure on your PC.",
            },
            ContentCard {
                tag: "PERFORMANCE",
                title: "Ultra-Fast Rendering Engine & Continuous Zoom",
                desc: "• Smooth cursor-anchored zoom from 1% to 6400% without losing focus.\n• Smart double-click: instantly toggles between Fit to Screen and 100% (Actual Size).\n• Dynamic Pixel Perfect algorithm: smooth interpolation for photos, crisp Nearest Neighbor for pixel art and textures.\n• Click & drag panning with support for dark/light checkerboards, solid black/white, or custom colors via 2D picker.",
            },
            ContentCard {
                tag: "NAVIGATION",
                title: "Filmstrip Carousel & Predictive Preloading",
                desc: "• Retractable bottom filmstrip preserving faithful aspect ratios.\n• Cooperative background thread pool loading thumbnails smoothly without freezing the UI.\n• Decoupled scrolling: freely browse folder contents without the viewport snapping back.",
            },
            ContentCard {
                tag: "PHOTOGRAPHY",
                title: "Technical Metadata Side Panel (EXIF)",
                desc: "• Deep photographic extraction: Camera, Lens, Aperture (f-stop), ISO, Shutter Speed, and Focal Length.\n• Embedded ICC color profiles and GPS geographic coordinates.\n• One-click 'Copy' button: exports the formatted technical summary directly to your Clipboard.",
            },
            ContentCard {
                tag: "INTEGRATION",
                title: "High-Fidelity Clipboard",
                desc: "• Pressing Ctrl+C copies images with maximum fidelity and transparency preservation, ready to paste into any image editor, browser, or messaging app.",
            },
            ContentCard {
                tag: "UTILITY",
                title: "Integrated Batch Converter (Batch Export)",
                desc: "• Convert entire folders or selected files to 11 supported raster formats (PNG, JPEG, WebP, BMP, TIFF, GIF, QOI, TGA, ICO, PNM, HDR) preserving 16-bit depths on output.",
            },
        ],
    }
}

fn get_format_categories(lang: ResolvedLanguage) -> [FormatCatCard; 5] {
    match lang {
        ResolvedLanguage::ZhCn => [
            FormatCatCard {
                title: "专业摄影与高精度光栅图像 (24 种格式)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16位), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB 合并层), GIMP (XCF 可见层), OpenRaster (ORA), Farbfeld (每通道16位), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP 和 X11 Bitmap (XBM)。",
            },
            FormatCatCard {
                title: "矢量图、CAD 与设计绘图 (7 种格式)",
                list: "• CorelDRAW (.cdr)：原生提取现代 ZIP 容器 (X4 至 2024+) 及传统 RIFF 容器 (v3 至 X3，含 DISP/DIB 块) 中的高清预览。\n• SVG 与 SVGZ：基于 resvg/usvg 的原生安全矢量渲染。\n• AutoCAD DXF：原生矢量 CAD 图纸解析转换。\n• Windows EMF 与 WMF：通过 Windows 官方 GDI+ 引擎渲染图元文件。\n• PDF, EPS, PS 与 Adobe Illustrator (AI)：当系统 PATH 中存在 ImageMagick 时通过外部转换器支持。",
            },
            FormatCatCard {
                title: "原生动态图像 (6 种格式)",
                list: "• GIF 与 APNG：具备精准帧时序控制与 Alpha 帧处理。\n• Windows 动态光标 (ANI)：RIFF/ACON 容器原生解析，精确到 Jiffy 刷新率。\n• Multiple-image Network Graphics (MNG)。\n• Lottie 矢量动画 (.lottie, .json) 与 Telegram 动态贴纸 (.tgs)：通过 rasterlottie 高性能渲染。",
            },
            FormatCatCard {
                title: "专业相机 RAW 原图照片 (23+ 主流品牌)",
                list: "• 佳能 Canon (CR2, CRW), 尼康 Nikon (NEF, NRW), 索尼 Sony (ARW, SRF, SR2), 富士 Fujifilm (RAF), 奥林巴斯 Olympus (ORF), 松下 Panasonic (RW2), 宾得 Pentax (PEF), 哈苏 Hasselblad (3FR), 飞思 Phase One (IIQ), 爱普生 Epson (ERF), 柯达 Kodak (KDC, DCR), 玛米亚 Mamiya (MOS), 美能达 Minolta (MRW), 适马 Sigma (X3F), 卡西欧 Casio (BAY), Phase One (CAP) 以及 Adobe DNG。\n• 佳能 CR3 ISOBMFF/CRX：直接提取全分辨率高质原片预览。",
            },
            FormatCatCard {
                title: "游戏与图形引擎纹理 (11 种格式)",
                list: "• 微软 DirectDraw Surface (DDS)：RGBA8, BC1, BC2, BC3, BC4, BC5, BC6H, BC7 及 Mipmap 层级显示。\n• Khronos KTX 1 与 KTX 2。\n• Source 引擎游戏 Valve Texture Format (VTF)。\n• 暴雪《魔兽争霸III》与《魔兽世界》BLP1/BLP2。\n• GoldSrc / 半条命 Half-Life IDSP 精灵图 (.spr)。\n• RenderWare TXD 纹理字典 (GTA III, Vice City, San Andreas)。\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) 与 Klei KTEX (.tex)。",
            },
        ],
        ResolvedLanguage::JaJp => [
            FormatCatCard {
                title: "写真＆高精度ラスター形式 (24 形式)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16ビット), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB 統合画像), GIMP (XCF 表示レイヤー), OpenRaster (ORA), Farbfeld (16ビット/チャンネル), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP, X11 Bitmap (XBM)。",
            },
            FormatCatCard {
                title: "ベクター、CAD ＆ イラスト (7 形式)",
                list: "• CorelDRAW (.cdr)：現代の ZIP 形式 (X4〜2024+) およびレガシー RIFF 形式 (v3〜X3、DISP/DIB チャンク) の埋め込みプレビューをネイティブ抽出。\n• SVG ＆ SVGZ：resvg/usvg による安全かつ高速なベクター描画。\n• AutoCAD DXF：CAD 図面のネイティブベクター変換。\n• Windows EMF ＆ WMF：Windows 公式 GDI+ によるメタファイル描画。\n• PDF, EPS, PS, Adobe Illustrator (AI)：PATH に ImageMagick が存在する場合、外部コンバーター経由で対応。",
            },
            FormatCatCard {
                title: "アニメーション形式 (6 形式)",
                list: "• GIF および APNG：正確なフレーム時間制御とアルファ破棄処理。\n• Windows アニメーションカーソル (ANI)：RIFF/ACON 形式をネイティブ解析。\n• Multiple-image Network Graphics (MNG)。\n• Lottie ベクターアニメーション (.lottie, .json) および Telegram アニメーションスタンプ (.tgs)：rasterlottie による高速描画。",
            },
            FormatCatCard {
                title: "プロ用カメラ RAW 写真 (23+ 主要ブランド)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP), Adobe DNG。\n• Canon CR3 ISOBMFF/CRX：最大解像度のプレビューをダイレクト抽出。",
            },
            FormatCatCard {
                title: "ゲームテクスチャ ＆ グラフィックエンジン (11 形式)",
                list: "• Microsoft DirectDraw Surface (DDS)：RGBA8, BC1, BC2, BC3, BC4, BC5, BC6H, BC7 および Mipmap 階層に対応。\n• Khronos KTX 1 および KTX 2。\n• Source エンジンの Valve Texture Format (VTF)。\n• Blizzard BLP1/BLP2 (Warcraft III, World of Warcraft)。\n• GoldSrc / Half-Life IDSP スプライト (.spr)。\n• RenderWare TXD (GTA III, Vice City, San Andreas)。\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf), Klei KTEX (.tex)。",
            },
        ],
        ResolvedLanguage::PtBr => [
            FormatCatCard {
                title: "Fotografia & Raster de Alta Precisão (24 formatos)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16 bits), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB composição achatada), GIMP (XCF camadas visíveis), OpenRaster (ORA), Farbfeld (16 bits/canal), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP e X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vetoriais, CAD & Ilustração (7 formatos)",
                list: "• CorelDRAW (.cdr): Extração nativa de preview raster embutido tanto de arquivos modernos baseados em ZIP (X4 a 2024+) quanto de contêineres legados baseados em RIFF (v3 a X3 com chunks DISP e DIB).\n• SVG & SVGZ: Renderização vetorial nativa e segura via resvg/usvg.\n• AutoCAD DXF: Conversão vetorial nativa de desenhos CAD.\n• Windows EMF & WMF: Renderização de metafiles via GDI+ oficial do Windows.\n• PDF, EPS, PS e Adobe Illustrator (AI): Suporte por conversor externo quando o ImageMagick estiver disponível no PATH.",
            },
            FormatCatCard {
                title: "Animações Nativas (6 formatos)",
                list: "• GIF e APNG com controle de tempo de quadros e descarte alfa.\n• Cursor Animado do Windows (ANI) em contêiner RIFF/ACON com taxas em jiffies.\n• Multiple-image Network Graphics (MNG).\n• Animações vetoriais Lottie (.lottie, .json) e Telegram Animated Stickers (.tgs) via rasterlottie.",
            },
            FormatCatCard {
                title: "Fotografias RAW de Câmeras Profissionais (23+ marcas)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) e Adobe DNG.\n• Canon CR3 ISOBMFF/CRX com extração direta da pré-visualização em resolução máxima.",
            },
            FormatCatCard {
                title: "Texturas de Jogos & Engines Gráficas (11 formatos)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1, BC2, BC3, BC4, BC5, BC6H, BC7 e contagem de mipmaps.\n• Khronos KTX 1 e KTX 2.\n• Valve Texture Format (VTF) de jogos da Source Engine.\n• Blizzard BLP1/BLP2 de Warcraft III e World of Warcraft.\n• GoldSrc / Half-Life IDSP sprites (.spr).\n• RenderWare TXD Texture Dictionary (GTA III, Vice City, San Andreas).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) e Klei KTEX (.tex).",
            },
        ],
        ResolvedLanguage::EsEs => [
            FormatCatCard {
                title: "Fotografía y Raster de Alta Precisión (24 formatos)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16 bits), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB aplanado), GIMP (XCF capas visibles), OpenRaster (ORA), Farbfeld (16 bits/canal), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP y X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vectoriales, CAD e Ilustración (7 formatos)",
                list: "• CorelDRAW (.cdr): Extracción nativa de vista previa incrustada para archivos basados en ZIP (X4 a 2024+) y contenedores RIFF heredados (v3 a X3 con chunks DISP/DIB).\n• SVG y SVGZ: Renderizado vectorial nativo y seguro con resvg/usvg.\n• AutoCAD DXF: Conversión vectorial nativa de planos CAD.\n• Windows EMF y WMF: Renderizado de metarchivos con Windows GDI+.\n• PDF, EPS, PS y Adobe Illustrator (AI): Compatibles mediante conversor externo si ImageMagick está en PATH.",
            },
            FormatCatCard {
                title: "Animaciones Nativas (6 formatos)",
                list: "• GIF y APNG con control de tiempos de fotograma y descarte alfa.\n• Cursor animado de Windows (ANI) en contenedor RIFF/ACON con tasas en jiffies.\n• Multiple-image Network Graphics (MNG).\n• Animaciones vectoriales Lottie (.lottie, .json) y stickers animados de Telegram (.tgs) con rasterlottie.",
            },
            FormatCatCard {
                title: "Fotografías RAW de Cámaras Profesionales (23+ marcas)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) y Adobe DNG.\n• Canon CR3 ISOBMFF/CRX con extracción directa de la vista previa a resolución máxima.",
            },
            FormatCatCard {
                title: "Texturas de Videojuegos y Motores Gráficos (11 formatos)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1, BC2, BC3, BC4, BC5, BC6H, BC7 y conteo de mipmaps.\n• Khronos KTX 1 y KTX 2.\n• Valve Texture Format (VTF) de juegos Source Engine.\n• Blizzard BLP1/BLP2 de Warcraft III y World of Warcraft.\n• Sprites GoldSrc / Half-Life IDSP (.spr).\n• RenderWare TXD (GTA III, Vice City, San Andreas).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) y Klei KTEX (.tex).",
            },
        ],
        ResolvedLanguage::DeDe => [
            FormatCatCard {
                title: "Foto- & Hochpräzisions-Rasterformate (24 Formate)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-Bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld (16-Bit), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP und X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vektor, CAD & Grafikdesign (7 Formate)",
                list: "• CorelDRAW (.cdr): Native Vorschaubild-Extraktion für ZIP-basierte (X4 bis 2024+) und RIFF-Dateien (v3 bis X3 mit DISP/DIB-Chunks).\n• SVG & SVGZ: Native sichere Vektorwiedergabe über resvg/usvg.\n• AutoCAD DXF: Native Vektorkonvertierung von CAD-Zeichnungen.\n• Windows EMF & WMF: Rendering über Windows GDI+.\n• PDF, EPS, PS und Adobe Illustrator (AI): Über ImageMagick im PATH.",
            },
            FormatCatCard {
                title: "Native Animationen (6 Formate)",
                list: "• GIF und APNG mit präziser Frame-Steuerung.\n• Windows animierte Cursor (ANI) im RIFF/ACON-Container.\n• Multiple-image Network Graphics (MNG).\n• Lottie-Vektoranimationen (.lottie, .json) und Telegram-Sticker (.tgs) über rasterlottie.",
            },
            FormatCatCard {
                title: "Kamera-RAW-Fotografien (23+ Marken)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) und Adobe DNG.\n• Canon CR3 ISOBMFF/CRX: Direkte Vorschau-Extraktion in voller Auflösung.",
            },
            FormatCatCard {
                title: "Spiele-Texturen & Grafik-Engines (11 Formate)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1 bis BC7 und Mipmaps.\n• Khronos KTX 1 und KTX 2.\n• Source Engine Valve Texture Format (VTF).\n• Blizzard BLP1/BLP2 (Warcraft III, World of Warcraft).\n• GoldSrc / Half-Life Sprites (.spr).\n• RenderWare TXD (GTA III, VC, SA).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) und Klei KTEX (.tex).",
            },
        ],
        ResolvedLanguage::FrFr => [
            FormatCatCard {
                title: "Photographie & formats raster haute précision (24 formats)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16 bits), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld (16 bits), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP et X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vectoriel, CAO & Graphisme (7 formats)",
                list: "• CorelDRAW (.cdr) : Extraction native de l'aperçu pour les fichiers ZIP (X4 à 2024+) et RIFF (v3 à X3).\n• SVG & SVGZ : Rendu vectoriel natif sécurisé via resvg/usvg.\n• AutoCAD DXF : Conversion vectorielle native de plans CAO.\n• Windows EMF & WMF : Métafichiers via le moteur officiel Windows GDI+.\n• PDF, EPS, PS et Adobe Illustrator (AI) : Via ImageMagick dans le PATH.",
            },
            FormatCatCard {
                title: "Animations natives (6 formats)",
                list: "• GIF et APNG avec contrôle précis du minutage des images.\n• Curseurs animés Windows (ANI) en conteneur RIFF/ACON.\n• Multiple-image Network Graphics (MNG).\n• Animations vectorielles Lottie (.lottie, .json) et stickers Telegram (.tgs) via rasterlottie.",
            },
            FormatCatCard {
                title: "Photos RAW d'appareils professionnels (23+ marques)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) et Adobe DNG.\n• Canon CR3 ISOBMFF/CRX : Extraction directe en pleine résolution.",
            },
            FormatCatCard {
                title: "Textures de jeux & moteurs 3D (11 formats)",
                list: "• Microsoft DirectDraw Surface (DDS) : RGBA8, BC1 à BC7 et mipmaps.\n• Khronos KTX 1 et KTX 2.\n• Valve Texture Format (VTF) des jeux Source Engine.\n• Blizzard BLP1/BLP2 (Warcraft III, World of Warcraft).\n• Sprites GoldSrc / Half-Life (.spr).\n• RenderWare TXD (GTA III, VC, SA).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) et Klei KTEX (.tex).",
            },
        ],
        ResolvedLanguage::ItIt => [
            FormatCatCard {
                title: "Fotografia e raster ad alta precisione (24 formati)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16 bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld (16 bit), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP e X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vettoriali, CAD & Illustrazione (7 formati)",
                list: "• CorelDRAW (.cdr): Estrazione nativa dell'anteprima da file ZIP (X4 fino al 2024+) e RIFF (v3 a X3).\n• SVG & SVGZ: Rendering vettoriale nativo e sicuro tramite resvg/usvg.\n• AutoCAD DXF: Conversione vettoriale nativa per disegni CAD.\n• Windows EMF & WMF: Metafile tramite GDI+ nativo di Windows.\n• PDF, EPS, PS e Adobe Illustrator (AI): Supportati tramite ImageMagick nel PATH.",
            },
            FormatCatCard {
                title: "Animazioni native (6 formati)",
                list: "• GIF e APNG con controllo temporale accurato dei fotogrammi.\n• Cursore animato di Windows (ANI) in formato RIFF/ACON.\n• Multiple-image Network Graphics (MNG).\n• Animazioni vettoriali Lottie (.lottie, .json) e sticker Telegram (.tgs) con rasterlottie.",
            },
            FormatCatCard {
                title: "Foto RAW da fotocamere professionali (23+ marchi)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) e Adobe DNG.\n• Canon CR3 ISOBMFF/CRX: Estrazione diretta dell'anteprima a piena risoluzione.",
            },
            FormatCatCard {
                title: "Texture di videogiochi & motori grafici (11 formati)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1 a BC7 e mipmaps.\n• Khronos KTX 1 e KTX 2.\n• Valve Texture Format (VTF) per giochi Source Engine.\n• Blizzard BLP1/BLP2 (Warcraft III, World of Warcraft).\n• Sprite GoldSrc / Half-Life (.spr).\n• RenderWare TXD (GTA III, VC, SA).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) e Klei KTEX (.tex).",
            },
        ],
        ResolvedLanguage::RuRu => [
            FormatCatCard {
                title: "Фотография и высокоточные растровые форматы (24 формата)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16 бит), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB), GIMP (XCF), OpenRaster (ORA), Farbfeld (16 бит), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP и X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Векторные форматы, CAD и графика (7 форматов)",
                list: "• CorelDRAW (.cdr): Прямое извлечение превью из архивов ZIP (X4–2024+) и контейнеров RIFF (v3–X3).\n• SVG & SVGZ: Безопасный рендеринг векторной графики через resvg/usvg.\n• AutoCAD DXF: Нативная конвертация чертежей CAD.\n• Windows EMF & WMF: Рендеринг метафайлов через официальный Windows GDI+.\n• PDF, EPS, PS и Adobe Illustrator (AI): Через ImageMagick в системном PATH.",
            },
            FormatCatCard {
                title: "Анимации (6 форматов)",
                list: "• GIF и APNG с покадровым контролем времени.\n• Анимированные курсоры Windows (ANI) в формате RIFF/ACON.\n• Multiple-image Network Graphics (MNG).\n• Векторные анимации Lottie (.lottie, .json) и Telegram-стикеры (.tgs) через rasterlottie.",
            },
            FormatCatCard {
                title: "Профессиональные RAW-фотографии (23+ бренда)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP) и Adobe DNG.\n• Canon CR3 ISOBMFF/CRX: Прямое извлечение полноразмерного превью.",
            },
            FormatCatCard {
                title: "Текстуры видеоигр и игровых движков (11 форматов)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1–BC7 и мип-мапы.\n• Khronos KTX 1 и KTX 2.\n• Valve Texture Format (VTF) для Source Engine.\n• Blizzard BLP1/BLP2 (Warcraft III, World of Warcraft).\n• Спрайты GoldSrc / Half-Life (.spr).\n• RenderWare TXD (GTA III, VC, SA).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf) и Klei KTEX (.tex).",
            },
        ],
        _ => [
            FormatCatCard {
                title: "Photography & High-Precision Raster (24 formats)",
                list: "PNG, APNG, JPEG, JFIF, WebP, AVIF, JPEG XL (JXL), TIFF (16-bit), BMP, DIB, QOI, HDR Radiance, OpenEXR (exr), ICO, CUR, Adobe Photoshop (PSD, PSB flattened), GIMP (XCF visible layers), OpenRaster (ORA), Farbfeld (16-bit/channel), JPEG 2000 (JP2, J2K, J2C, JPX), Netpbm PNM (PPM, PGM, PBM, PAM), SGI, Sun RAS, WBMP, and X11 Bitmap (XBM).",
            },
            FormatCatCard {
                title: "Vectors, CAD & Illustration (7 formats)",
                list: "• CorelDRAW (.cdr): Native embedded preview extraction for modern ZIP-based files (X4 to 2024+) and legacy RIFF containers (v3 to X3 with DISP/DIB chunks).\n• SVG & SVGZ: Native secure vector rendering via resvg/usvg.\n• AutoCAD DXF: Native vector CAD conversion.\n• Windows EMF & WMF: Metafile rendering via Windows GDI+.\n• PDF, EPS, PS, and Adobe Illustrator (AI): Supported via external converter when ImageMagick is in PATH.",
            },
            FormatCatCard {
                title: "Native Animations (6 formats)",
                list: "• GIF and APNG with frame timing control and alpha disposal.\n• Windows Animated Cursor (ANI) in RIFF/ACON container with jiffies rates.\n• Multiple-image Network Graphics (MNG).\n• Lottie vector animations (.lottie, .json) and Telegram Animated Stickers (.tgs) via rasterlottie.",
            },
            FormatCatCard {
                title: "Professional Camera RAW Photos (23+ brands)",
                list: "• Canon (CR2, CRW), Nikon (NEF, NRW), Sony (ARW, SRF, SR2), Fujifilm (RAF), Olympus (ORF), Panasonic (RW2), Pentax (PEF), Hasselblad (3FR), Phase One (IIQ), Epson (ERF), Kodak (KDC, DCR), Mamiya (MOS), Minolta (MRW), Sigma (X3F), Casio (BAY), Phase One (CAP), and Adobe DNG.\n• Canon CR3 ISOBMFF/CRX with direct full-resolution preview extraction.",
            },
            FormatCatCard {
                title: "Game Textures & Graphic Engines (11 formats)",
                list: "• Microsoft DirectDraw Surface (DDS): RGBA8, BC1, BC2, BC3, BC4, BC5, BC6H, BC7 and mipmap counts.\n• Khronos KTX 1 and KTX 2.\n• Valve Texture Format (VTF) from Source Engine games.\n• Blizzard BLP1/BLP2 from Warcraft III and World of Warcraft.\n• GoldSrc / Half-Life IDSP sprites (.spr).\n• RenderWare TXD Texture Dictionary (GTA III, Vice City, San Andreas).\n• ASTC (.astc), Basis Universal (.basis), PowerVR PVR (.pvr), Adobe ATF (.atf), and Klei KTEX (.tex).",
            },
        ],
    }
}

fn get_security_cards(lang: ResolvedLanguage) -> [ContentCard; 6] {
    match lang {
        ResolvedLanguage::ZhCn => [
            ContentCard {
                tag: "安全与隐私",
                title: "绝对隐私与零遥测",
                desc: "VISOR 遵循“默认隐私”的核心原则设计。软件完全离线运行。绝不收集、记录或向任何服务器上传任何打开的图片、浏览历史、本地文件路径或使用行为画像。",
            },
            ContentCard {
                tag: "安全与隐私",
                title: "安全隔离与稳定防护",
                desc: "采用独立安全沙箱解码架构。当遇到损坏、不完整或异常的图像文件时，错误会被瞬间安全隔离，绝不导致程序崩溃，确保系统始终安全稳定运行。",
            },
            ContentCard {
                tag: "安全与隐私",
                title: "安全矢量处理，阻绝恶意脚本",
                desc: "SVG、DXF 和工程矢量图在打开时完全剥离任何代码执行能力：所有潜在脚本、外部网络连接或隐藏指令均被自动阻断，仅展示安全纯粹的画面。",
            },
            ContentCard {
                tag: "安全与隐私",
                title: "完全遵从 Windows 现代系统标准",
                desc: "VISOR 采用干净绿色的安装与运行模式，无需管理员权限，完全尊重系统默认应用规则，与 Windows 10 和 11 深度融合、无缝协同。",
            },
            ContentCard {
                tag: "安全与隐私",
                title: "官方认证与正品保障",
                desc: "官方分发版本均通过数字签名认证与安全完整性校验，确保您安装和使用的软件纯净正版、未经任何第三方篡改。",
            },
            ContentCard {
                tag: "安全与隐私",
                title: "Rust 原生极致性能与可靠性",
                desc: "基于 Rust 语言底层精益构建，提供极速响应、极低内存占用与内存安全保证，在全离线运行的同时为您带来坚如磐石的长久可靠性。",
            },
        ],
        ResolvedLanguage::JaJp => [
            ContentCard {
                tag: "セキュリティ",
                title: "完全なプライバシーとテレメトリゼロ",
                desc: "VISOR は「プライバシー・バイ・デフォルト」の原則に基づいて設計されています。ソフトウェアは 100% オフラインで動作します。開いた画像、閲覧履歴、ローカルファイルなどの情報は一切収集・送信されません。",
            },
            ContentCard {
                tag: "セキュリティ",
                title: "破損ファイルの安全隔離とクラッシュ防止",
                desc: "独立した安全デコード保護層を搭載。破損したファイルや異常な画像を開いた場合でも、アプリをクラッシュさせることなくエラーを安全に封じ込め、システムの安定性を保ちます。",
            },
            ContentCard {
                tag: "セキュリティ",
                title: "安全なベクター描画（スクリプト排除）",
                desc: "SVG や CAD 図面などのベクターファイルを開く際、悪意あるスクリプトや外部ネット接続を自動的に遮断し、純粋な画像表示のみを安全に行います。",
            },
            ContentCard {
                tag: "セキュリティ",
                title: "Windows 公式標準への完全準拠",
                desc: "管理者権限を要求せず、Windows 10 および 11 の標準規格に準拠したクリーンで安全な設計。ユーザーの規定アプリ設定を完全に尊重します。",
            },
            ContentCard {
                tag: "セキュリティ",
                title: "公式真正性と信頼性の保証",
                desc: "すべての公式配布パッケージはデジタル署名と整合性チェックにより保護されており、改ざんのない安全な正規アプリであることが保証されています。",
            },
            ContentCard {
                tag: "セキュリティ",
                title: "Rust による最高水準の堅牢性と高速性",
                desc: "Rust 言語のネイティブパワーにより、瞬時の起動、極限まで抑えたメモリ消費、そして高いメモリ安全性を実現し、快適な使い心地を維持します。",
            },
        ],
        ResolvedLanguage::PtBr => [
            ContentCard {
                tag: "SEGURANÇA",
                title: "Privacidade Absoluta e Zero Telemetria",
                desc: "O VISOR foi desenhado sob o princípio de Privacidade por Padrão. O software opera 100% offline. Nenhuma informação sobre imagens abertas, histórico de navegação, arquivos locais ou perfil de uso é coletada, gravada ou transmitida.",
            },
            ContentCard {
                tag: "SEGURANÇA",
                title: "Proteção Contra Arquivos Danificados",
                desc: "Leitura isolada e protegida. Se uma imagem estiver corrompida ou com defeito, o erro é contido de forma segura sem fechar o aplicativo e sem comprometer a estabilidade do seu computador.",
            },
            ContentCard {
                tag: "SEGURANÇA",
                title: "Segurança Contra Códigos Maliciosos",
                desc: "Arquivos vetoriais e ilustrações técnicas são abertos sem execução de scripts, macros ou conexões externas com a internet, garantindo que nada indesejado seja executado.",
            },
            ContentCard {
                tag: "SEGURANÇA",
                title: "Compatibilidade Oficial com Windows 10 e 11",
                desc: "Instalação limpa e moderna que não exige privilégios de Administrador, opera com total segurança e respeita integralmente as preferências do usuário no sistema.",
            },
            ContentCard {
                tag: "SEGURANÇA",
                title: "Autenticidade e Confiança Garantidas",
                desc: "Aplicativo verificado e distribuído com certificação oficial, garantindo que você está utilizando um software autêntico, seguro e livre de qualquer adulteração.",
            },
            ContentCard {
                tag: "SEGURANÇA",
                title: "Desempenho e Confiabilidade Nativa em Rust",
                desc: "Construído do zero em Rust para oferecer inicialização instantânea, baixíssimo consumo de memória e confiabilidade inabalável para o seu dia a dia.",
            },
        ],
        ResolvedLanguage::EsEs => [
            ContentCard {
                tag: "SEGURIDAD",
                title: "Privacidad Absoluta y Cero Telemetría",
                desc: "VISOR está diseñado bajo el principio de Privacidad por Defecto. Funciona 100% fuera de línea. Nunca se recopila, registra ni transmite información sobre imágenes abiertas, historial ni archivos locales.",
            },
            ContentCard {
                tag: "SEGURIDAD",
                title: "Protección Contra Archivos Dañados",
                desc: "Lectura aislada y protegida. Si una imagen está corrupta o incompleta, el error se contiene de forma segura sin cerrar la aplicación y sin comprometer la estabilidad del sistema.",
            },
            ContentCard {
                tag: "SEGURIDAD",
                title: "Seguridad Contra Scripts Maliciosos",
                desc: "Los archivos vectoriales e ilustraciones se abren sin ejecutar scripts ocultos, macros ni conexiones externas a internet, garantizando máxima tranquilidad.",
            },
            ContentCard {
                tag: "SEGURIDAD",
                title: "Compatibilidad Oficial con Windows 10 y 11",
                desc: "Instalación limpia y moderna que no requiere permisos de Administrador, opera con total seguridad y respeta plenamente las preferencias del usuario en el sistema.",
            },
            ContentCard {
                tag: "SEGURIDAD",
                title: "Autenticidad y Confianza Garantizadas",
                desc: "Aplicación verificada y distribuida con certificación oficial, garantizando que utiliza un software auténtico, seguro y libre de manipulaciones.",
            },
            ContentCard {
                tag: "SEGURIDAD",
                title: "Rendimiento y Fiabilidad Nativa en Rust",
                desc: "Desarrollado desde cero en Rust para ofrecer inicio instantáneo, mínimo consumo de memoria y máxima robustez para el uso continuo.",
            },
        ],
        ResolvedLanguage::DeDe => [
            ContentCard {
                tag: "SICHERHEIT",
                title: "Vollständige Privatsphäre & Null Telemetrie",
                desc: "VISOR wurde nach dem Grundsatz 'Datenschutz durch Design' entwickelt. Die Software arbeitet zu 100% offline. Es werden niemals geöffnete Bilder, Verläufe oder Dateipfade gesammelt oder übertragen.",
            },
            ContentCard {
                tag: "SICHERHEIT",
                title: "Schutz vor beschädigten Dateien",
                desc: "Isolierte und sichere Dekodierung. Bei defekten oder manipulierten Bilddateien wird der Fehler sicher abgefangen, ohne dass die Anwendung abstürzt oder Ihr System gefährdet wird.",
            },
            ContentCard {
                tag: "SICHERHEIT",
                title: "Sicherheit vor bösartigen Skripten",
                desc: "Vektordateien (wie SVG und DXF) werden streng ohne Skriptausführung, Makros oder externe Internetverbindungen geöffnet.",
            },
            ContentCard {
                tag: "SICHERHEIT",
                title: "Konform mit Windows 10 & 11 Standards",
                desc: "Saubere Installation ohne Administratorrechte, vollständig kompatibel mit den Microsoft-Sicherheitsrichtlinien für Standard-Apps.",
            },
            ContentCard {
                tag: "SICHERHEIT",
                title: "Echtheit & Vertrauenswürdigkeit garantiert",
                desc: "Offizielle Pakete werden kryptografisch mit SHA-256 verifiziert, um sicherzustellen, dass Sie eine unveränderte Originalsoftware verwenden.",
            },
            ContentCard {
                tag: "SICHERHEIT",
                title: "Native Rust-Zuverlässigkeit",
                desc: "Von Grund auf in Rust entwickelt für maximale Speichersicherheit, minimale Latenz und dauerhafte Stabilität im täglichen Einsatz.",
            },
        ],
        ResolvedLanguage::FrFr => [
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Confidentialité absolue & Zéro télémétrie",
                desc: "VISOR est conçu selon le principe de 'Confidentialité par défaut'. Fonctionnement 100% hors ligne : aucune image, métadonnée ou historique n'est collecté ou envoyé sur Internet.",
            },
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Protection contre les fichiers corrompus",
                desc: "Décodage en bac à sable sécurisé. Si un fichier est endommagé ou malformé, l'erreur est isolée proprement sans faire planter le logiciel ni menacer votre PC.",
            },
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Protection contre les scripts malveillants",
                desc: "Les formats vectoriels (SVG, DXF) sont analysés sans exécuter de scripts masqués, de macros ou de requêtes réseau indésirables.",
            },
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Conformité officielle Windows 10 & 11",
                desc: "Installation propre par utilisateur ne nécessitant aucun privilège administrateur, respectant scrupuleusement vos choix d'applications par défaut.",
            },
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Authenticité & Intégrité garanties",
                desc: "Chaque distribution officielle est validée par empreinte cryptographique SHA-256 pour garantir un binaire authentique et sain.",
            },
            ContentCard {
                tag: "SÉCURITÉ",
                title: "Fiabilité et robustesse native Rust",
                desc: "Conçu en Rust pur pour une sécurité mémoire garantie, une réactivité fulgurante et une endurance totale au quotidien.",
            },
        ],
        ResolvedLanguage::ItIt => [
            ContentCard {
                tag: "SICUREZZA",
                title: "Privacy assoluta e Zero telemetria",
                desc: "VISOR è progettato sul principio di Privacy by Design. Funziona al 100% offline. Nessuna immagine aperta, cronologia o percorso locale viene raccolto o trasmesso.",
            },
            ContentCard {
                tag: "SICUREZZA",
                title: "Protezione contro file danneggiati",
                desc: "Decodifica sicura e isolata. Se un file immagine è corrotto, l'errore viene gestito in sicurezza senza causare crash o instabilità al sistema.",
            },
            ContentCard {
                tag: "SICUREZZA",
                title: "Sicurezza contro script dannosi",
                desc: "I file vettoriali (SVG, DXF) vengono visualizzati senza consentire l'esecuzione di script, macro o chiamate di rete esterne.",
            },
            ContentCard {
                tag: "SICUREZZA",
                title: "Conformità standard Windows 10 e 11",
                desc: "Installazione pulita senza permessi di amministratore obbligatori, nel pieno rispetto delle linee guida di sicurezza di Microsoft.",
            },
            ContentCard {
                tag: "SICUREZZA",
                title: "Autenticità e integrità verificate",
                desc: "Tutte le release ufficiali sono protette da checksum SHA-256 per garantire l'autenticità e l'assenza di manomissioni.",
            },
            ContentCard {
                tag: "SICUREZZA",
                title: "Robustezza e velocità native Rust",
                desc: "Costruito interamente in Rust per garantire sicurezza della memoria, consumi minimi e massima stabilità per l'uso quotidiano.",
            },
        ],
        ResolvedLanguage::RuRu => [
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Полная приватность и ноль телеметрии",
                desc: "VISOR спроектирован по принципу «Конфиденциальность по умолчанию». Программа работает 100% офлайн. Никакие данные об открытых изображениях или истории не отправляются в сеть.",
            },
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Защита от поврежденных файлов",
                desc: "Изолированное безопасное декодирование. При открытии поврежденного файла ошибка безопасно перехватывается, предотвращая сбой программы и системы.",
            },
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Защита от вредоносных скриптов",
                desc: "Векторные форматы (SVG, DXF) обрабатываются со строгим запретом на выполнение скриптов, макросов или сетевых соединений.",
            },
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Соответствие стандартам Windows 10 и 11",
                desc: "Чистая установка без прав администратора, полное уважение к настройкам приложений по умолчанию в операционной системе.",
            },
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Подлинность и проверка целостности",
                desc: "Официальные релизы проверяются криптографическими контрольными суммами SHA-256 для гарантии защиты от подделки.",
            },
            ContentCard {
                tag: "БЕЗОПАСНОСТЬ",
                title: "Нативная надежность на языке Rust",
                desc: "Разработано с нуля на Rust для абсолютной безопасности работы с памятью, мгновенного отклика и непревзойденной стабильности.",
            },
        ],
        _ => [
            ContentCard {
                tag: "SECURITY",
                title: "Absolute Privacy and Zero Telemetry",
                desc: "VISOR is designed under the Privacy by Default principle. It operates 100% offline. No information about opened images, browsing history, local files, or usage profiles is ever collected, logged, or transmitted.",
            },
            ContentCard {
                tag: "SECURITY",
                title: "Protection Against Damaged Files",
                desc: "Protected and isolated reading. If an image file is corrupted or incomplete, the error is safely contained without closing the app or compromising system stability.",
            },
            ContentCard {
                tag: "SECURITY",
                title: "Protection Against Malicious Scripts",
                desc: "Vector files and technical illustrations open cleanly without executing hidden scripts, macros, or external internet connections, keeping your system safe.",
            },
            ContentCard {
                tag: "SECURITY",
                title: "Official Windows 10 & 11 Compatibility",
                desc: "Clean and modern installation that never requires Administrator privileges, operates safely within standard user permissions, and fully respects your system preferences.",
            },
            ContentCard {
                tag: "SECURITY",
                title: "Authenticity & Trust Guaranteed",
                desc: "Officially certified and distributed package, ensuring you are using genuine, secure software that is completely free of unauthorized modifications.",
            },
            ContentCard {
                tag: "SECURITY",
                title: "Native Performance & Reliability in Rust",
                desc: "Built from scratch in Rust for instant startup, exceptionally low memory consumption, and rock-solid stability for continuous daily use.",
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whats_new_window_skips_when_closed() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.show_whats_new_window = false;

        render_whats_new_window(&mut app, &ctx);
        assert!(!app.updater.show_whats_new_window);
    }

    #[test]
    fn test_whats_new_window_renders_when_open() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.show_whats_new_window = true;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_whats_new_window(&mut app, ui.ctx());
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_central_tabs_all_render_cleanly() {
        let ctx = egui::Context::default();
        let mut app = VisorApp::new(&ctx, None);
        app.updater.show_whats_new_window = true;

        for tab in [
            CentralTab::Overview,
            CentralTab::Formats,
            CentralTab::Security,
            CentralTab::Changelog,
        ] {
            #[allow(deprecated)]
            let tab_id = egui::Id::new("central_visor_active_tab");
            ctx.data_mut(|d| d.insert_temp(tab_id, tab));

            let mut output = ctx.run_ui(Default::default(), |ui| {
                render_whats_new_window(&mut app, ui.ctx());
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn test_central_content_for_all_languages() {
        for lang in [
            ResolvedLanguage::PtBr,
            ResolvedLanguage::EnUs,
            ResolvedLanguage::EsEs,
            ResolvedLanguage::ZhCn,
            ResolvedLanguage::JaJp,
            ResolvedLanguage::DeDe,
            ResolvedLanguage::FrFr,
            ResolvedLanguage::ItIt,
            ResolvedLanguage::RuRu,
        ] {
            let cards = get_overview_cards(lang);
            assert_eq!(cards.len(), 6);
            for c in &cards {
                assert!(!c.title.is_empty());
                assert!(!c.desc.is_empty());
                assert!(!c.tag.is_empty());
            }

            let formats = get_format_categories(lang);
            assert_eq!(formats.len(), 5);
            for f in &formats {
                assert!(!f.title.is_empty());
                assert!(!f.list.is_empty());
            }

            let sec = get_security_cards(lang);
            assert_eq!(sec.len(), 6);
            for s in &sec {
                assert!(!s.title.is_empty());
                assert!(!s.desc.is_empty());
                assert!(!s.tag.is_empty());
            }
        }
    }
}
