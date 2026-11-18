//! Tema visual, tokens de design e fundação do Design System inspirado no Windows 11.
//!
//! Centraliza a paleta de cores (Dark/Light), superfícies, bordas translúcidas,
//! cantos arredondados, espaçamentos, tipografia e hierarquia de botões do VISOR.

#![allow(dead_code)]

use eframe::egui::{self, Color32, ColorImage, CornerRadius, Stroke, Vec2};

// =============================================================================
// 1. CONSTANTES DE COMPATIBILIDADE RETROATIVA
// =============================================================================

/// Cor de fundo base do visualizador (canvas escuro neutro para destaque da foto).
pub const BG: Color32 = Color32::from_rgb(20, 20, 22);

/// Cor de fundo de painéis encaixados (painel lateral e barras).
pub const PANEL: Color32 = Color32::from_rgb(30, 30, 34);

// =============================================================================
// 2. TOKENS DE ESPAÇAMENTO E DIMENSÕES (SPACING & SIZING TOKENS)
// =============================================================================

/// Métrica de espaçamento contínuo do sistema visual.
pub struct ThemeSpacing;

impl ThemeSpacing {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

/// Métrica de cantos arredondados inspirados no Fluent Design do Windows 11.
pub struct ThemeRounding;

impl ThemeRounding {
    /// 3px: Usado para badges, tags pequenas e indicadores sutis.
    pub const RADIUS_XS: CornerRadius = CornerRadius::same(3);

    /// 5px: Usado para botões padrão, caixas de seleção e campos de texto.
    pub const RADIUS_SM: CornerRadius = CornerRadius::same(5);

    /// 8px: Usado para cards de metadados, menus de contexto e popovers.
    pub const RADIUS_MD: CornerRadius = CornerRadius::same(8);

    /// 12px: Usado para drawers laterais, diálogos e painéis modais.
    pub const RADIUS_LG: CornerRadius = CornerRadius::same(12);

    /// 24px: Cápsula completa (Pill) para a Toolbar flutuante centralizada.
    pub const RADIUS_PILL: CornerRadius = CornerRadius::same(24);
}

/// Dimensões padrão para controles interativos.
pub struct ThemeDimensions;

impl ThemeDimensions {
    /// Padding interno para botões padrão.
    pub const BUTTON_PADDING: Vec2 = Vec2::new(10.0, 6.0);

    /// Padding interno para botões compactos da toolbar em cápsula.
    pub const TOOLBAR_BUTTON_PADDING: Vec2 = Vec2::new(8.0, 5.0);

    /// Tamanho fixo de botão de ícone quadrado (32×32 px).
    pub const ICON_BUTTON_SIZE: Vec2 = Vec2::new(32.0, 32.0);

    /// Tamanho padrão de miniaturas no filmstrip inferior (76×56 px).
    pub const FILMSTRIP_THUMB_SIZE: Vec2 = Vec2::new(76.0, 56.0);

    /// Tamanho compacto de miniaturas em telas estreitas (58×44 px).
    pub const FILMSTRIP_THUMB_SIZE_COMPACT: Vec2 = Vec2::new(58.0, 44.0);

    /// Altura padrão do painel do filmstrip inferior.
    pub const FILMSTRIP_HEIGHT: f32 = 72.0;

    /// Altura compacta do painel do filmstrip em telas estreitas.
    pub const FILMSTRIP_HEIGHT_COMPACT: f32 = 58.0;

    /// Altura da barra de status inferior.
    pub const STATUS_BAR_HEIGHT: f32 = 26.0;

    /// Largura padrão do Drawer de Informações técnicas.
    pub const INFO_PANEL_WIDTH: f32 = 280.0;
}

// =============================================================================
// 3. PALETA DE CORES E SUPERFÍCIES (THEME PALETTE)
// =============================================================================

/// Conjunto estruturado de cores e superfícies para um tema (Dark ou Light).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemePalette {
    pub is_dark: bool,

    // Superfícies e Camadas
    pub surface_base: Color32,
    pub surface_panel: Color32,
    pub surface_card: Color32,
    pub surface_flyout: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,

    // Bordas e Delimitações
    pub border_subtle: Stroke,
    pub border_default: Stroke,
    pub border_strong: Stroke,
    pub border_focus: Stroke,

    // Cores de Destaque (Accent Windows 11 Blue)
    pub accent_default: Color32,
    pub accent_hover: Color32,
    pub accent_active: Color32,
    pub accent_muted: Color32,

    // Tipografia
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_accent: Color32,

    // Status Semânticos
    pub status_success: Color32,
    pub status_warning: Color32,
    pub status_error: Color32,
}

impl ThemePalette {
    /// Paleta Dark Mode refinada (padrão oficial do VISOR).
    ///
    /// Desenvolvida para manter a foto como protagonista, com superfícies discretas
    /// e bordas translúcidas de 1px com sutil alpha.
    pub fn dark() -> Self {
        let accent = Color32::from_rgb(96, 205, 255); // Fluent Blue Light in Dark Mode
        Self {
            is_dark: true,

            // Superfícies
            surface_base: Color32::from_rgb(20, 20, 22), // Quase preto neutro
            surface_panel: Color32::from_rgb(30, 30, 34), // Superfície sólida de docking
            surface_card: Color32::from_rgb(38, 38, 44), // Cards e seções
            surface_flyout: Color32::from_rgba_unmultiplied(32, 32, 36, 240), // 94% opaco fosco
            surface_hover: Color32::from_rgba_unmultiplied(255, 255, 255, 18), // Hover sutil
            surface_active: Color32::from_rgba_unmultiplied(255, 255, 255, 30), // Pressionado

            // Bordas translúcidas Fluent (1px)
            border_subtle: Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 22)),
            border_default: Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 38)),
            border_strong: Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 65)),
            border_focus: Stroke::new(2.0, accent),

            // Accent
            accent_default: accent,
            accent_hover: Color32::from_rgb(128, 218, 255),
            accent_active: Color32::from_rgb(64, 192, 255),
            accent_muted: Color32::from_rgba_unmultiplied(96, 205, 255, 38),

            // Tipografia
            text_primary: Color32::from_rgb(244, 244, 246),
            text_secondary: Color32::from_rgb(160, 160, 168),
            text_muted: Color32::from_rgb(105, 105, 115),
            text_on_accent: Color32::from_rgb(16, 16, 20),

            // Feedback
            status_success: Color32::from_rgb(108, 203, 95),
            status_warning: Color32::from_rgb(252, 225, 0),
            status_error: Color32::from_rgb(255, 100, 100),
        }
    }

    /// Paleta Light Mode preparada para expansão futura.
    pub fn light() -> Self {
        let accent = Color32::from_rgb(0, 103, 192); // Windows 11 Blue in Light Mode
        Self {
            is_dark: false,

            // Superfícies
            surface_base: Color32::from_rgb(243, 243, 243),
            surface_panel: Color32::from_rgb(249, 249, 249),
            surface_card: Color32::from_rgb(255, 255, 255),
            surface_flyout: Color32::from_rgba_unmultiplied(255, 255, 255, 242),
            surface_hover: Color32::from_rgba_unmultiplied(0, 0, 0, 15),
            surface_active: Color32::from_rgba_unmultiplied(0, 0, 0, 25),

            // Bordas
            border_subtle: Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 18)),
            border_default: Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 32)),
            border_strong: Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 50)),
            border_focus: Stroke::new(2.0, accent),

            // Accent
            accent_default: accent,
            accent_hover: Color32::from_rgb(24, 120, 204),
            accent_active: Color32::from_rgb(0, 88, 166),
            accent_muted: Color32::from_rgba_unmultiplied(0, 103, 192, 35),

            // Tipografia
            text_primary: Color32::from_rgb(28, 28, 30),
            text_secondary: Color32::from_rgb(90, 90, 96),
            text_muted: Color32::from_rgb(140, 140, 146),
            text_on_accent: Color32::from_rgb(255, 255, 255),

            // Feedback
            status_success: Color32::from_rgb(15, 123, 15),
            status_warning: Color32::from_rgb(157, 93, 0),
            status_error: Color32::from_rgb(196, 43, 28),
        }
    }
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::dark()
    }
}

// =============================================================================
// 4. CONFIGURAÇÃO GLOBAL DO EGUIDESIGN SYSTEM
// =============================================================================

/// Injeta a paleta do Design System nas propriedades visuais globais do `egui::Context`.
pub fn configure_visor_theme(ctx: &egui::Context, palette: &ThemePalette) {
    ctx.set_theme(if palette.is_dark {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });

    // Injeta fonte fallback para CJK (chinês simplificado, japonês) se disponível no Windows
    let yahei_path = std::path::Path::new(r"C:\Windows\Fonts\msyh.ttc");
    if yahei_path.is_file() {
        if let Ok(font_bytes) = std::fs::read(yahei_path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "ms_yahei".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_bytes)),
            );
            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.push("ms_yahei".to_owned());
            }
            if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                mono.push("ms_yahei".to_owned());
            }
            ctx.set_fonts(fonts);
        }
    }

    ctx.all_styles_mut(|style| {
        // Paddings e espaçamentos globais
        style.spacing.button_padding = ThemeDimensions::BUTTON_PADDING;
        style.spacing.item_spacing = Vec2::new(ThemeSpacing::SM, ThemeSpacing::SM);
        style.spacing.window_margin = egui::Margin::same(ThemeSpacing::MD as i8);

        // Visuals
        let mut visuals = if palette.is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.dark_mode = palette.is_dark;
        visuals.panel_fill = palette.surface_panel;
        visuals.window_fill = palette.surface_base;
        visuals.window_stroke = palette.border_default;
        visuals.window_corner_radius = ThemeRounding::RADIUS_LG;
        visuals.menu_corner_radius = ThemeRounding::RADIUS_MD;

        // Widgets Inactive (botões e controles em repouso)
        visuals.widgets.inactive.bg_fill = palette.surface_card;
        visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = palette.border_subtle;
        visuals.widgets.inactive.corner_radius = ThemeRounding::RADIUS_SM;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text_primary);

        // Widgets Hovered
        visuals.widgets.hovered.bg_fill = palette.surface_hover;
        visuals.widgets.hovered.weak_bg_fill = palette.surface_hover;
        visuals.widgets.hovered.bg_stroke = palette.border_default;
        visuals.widgets.hovered.corner_radius = ThemeRounding::RADIUS_SM;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text_primary);

        // Widgets Active (pressionado)
        visuals.widgets.active.bg_fill = palette.surface_active;
        visuals.widgets.active.weak_bg_fill = palette.surface_active;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent_default);
        visuals.widgets.active.corner_radius = ThemeRounding::RADIUS_SM;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text_primary);

        // Widgets Noninteractive (labels, painéis estáticos)
        visuals.widgets.noninteractive.bg_fill = palette.surface_panel;
        visuals.widgets.noninteractive.bg_stroke = palette.border_subtle;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text_secondary);

        // Seleção
        visuals.selection.bg_fill = palette.accent_muted;
        visuals.selection.stroke = Stroke::new(1.0, palette.accent_default);

        style.visuals = visuals;
    });
}

// =============================================================================
// 5. RECURSOS GRÁFICOS COMPARTILHADOS (ÍCONES E TEXTURAS)
// =============================================================================

/// Retorna os dados do ícone nativo da janela (moldura).
pub fn window_icon() -> egui::IconData {
    let image =
        image::load_from_memory(include_bytes!("../../assets/moldura.png")).expect("ícone Moldura");
    let rgba = image.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
}

/// Textura de fundo quadriculada para exibição de transparência (canal alfa).
pub fn checker_image() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(
        [2, 2],
        &[
            48, 48, 52, 255, 34, 34, 38, 255, 34, 34, 38, 255, 48, 48, 52, 255,
        ],
    )
}

/// Textura de fundo quadriculada clara para exibição de transparência sobre fundos claros ou silhuetas escuras.
pub fn light_checker_image() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(
        [2, 2],
        &[
            240, 240, 244, 255, 220, 220, 226, 255, 220, 220, 226, 255, 240, 240, 244, 255,
        ],
    )
}

// =============================================================================
// 6. TESTES UNITÁRIOS DO DESIGN SYSTEM
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_palette_tokens_integrity() {
        let dark = ThemePalette::dark();
        assert!(dark.is_dark);
        assert_eq!(dark.surface_base, Color32::from_rgb(20, 20, 22));
        assert_eq!(dark.accent_default, Color32::from_rgb(96, 205, 255));
        assert_eq!(dark.border_subtle.width, 1.0);
    }

    #[test]
    fn test_light_palette_tokens_integrity() {
        let light = ThemePalette::light();
        assert!(!light.is_dark);
        assert_eq!(light.surface_base, Color32::from_rgb(243, 243, 243));
        assert_eq!(light.accent_default, Color32::from_rgb(0, 103, 192));
    }

    #[test]
    fn test_rounding_metrics_scale() {
        assert_eq!(ThemeRounding::RADIUS_XS, CornerRadius::same(3));
        assert_eq!(ThemeRounding::RADIUS_SM, CornerRadius::same(5));
        assert_eq!(ThemeRounding::RADIUS_MD, CornerRadius::same(8));
        assert_eq!(ThemeRounding::RADIUS_LG, CornerRadius::same(12));
        assert_eq!(ThemeRounding::RADIUS_PILL, CornerRadius::same(24));
    }

    #[test]
    fn test_spacing_metrics_scale() {
        assert!(ThemeSpacing::XXS < ThemeSpacing::XS);
        assert!(ThemeSpacing::XS < ThemeSpacing::SM);
        assert!(ThemeSpacing::SM < ThemeSpacing::MD);
        assert!(ThemeSpacing::MD < ThemeSpacing::LG);
        assert!(ThemeSpacing::LG < ThemeSpacing::XL);
        assert!(ThemeSpacing::XL < ThemeSpacing::XXL);
    }

    #[test]
    fn test_checker_images_dimensions() {
        let dark = checker_image();
        assert_eq!(dark.size, [2, 2]);
        assert_eq!(dark.pixels.len(), 4);

        let light = light_checker_image();
        assert_eq!(light.size, [2, 2]);
        assert_eq!(light.pixels.len(), 4);
    }

    #[test]
    fn test_configure_theme_applies_without_panic() {
        let ctx = egui::Context::default();
        let palette = ThemePalette::dark();
        configure_visor_theme(&ctx, &palette);

        let style = ctx.style_of(egui::Theme::Dark);
        assert_eq!(style.visuals.panel_fill, palette.surface_panel);
        assert_eq!(style.visuals.window_fill, palette.surface_base);
        assert_eq!(style.visuals.window_corner_radius, ThemeRounding::RADIUS_LG);
    }
}
