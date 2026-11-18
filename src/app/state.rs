//! Modelos de estado da aplicação, visualização, carregamento e miniaturas.

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};

use eframe::egui::{Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2};

use crate::core::ImageCache;
use crate::core::geometry::fit_scale;
use crate::workers::{CancellationToken, ThreadPool};

/// Modos de interpolação na renderização da imagem.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMode {
    /// Automático: Nearest para ampliações (>= 100%), Linear para reduções (< 100%).
    #[default]
    Auto,
    /// Suave: Bilinear sempre, ideal para fotografias de alta resolução.
    Linear,
    /// Pixel Perfect: Nearest sempre, preserva pixels nítidos sem borrão em pixel art.
    Nearest,
}

impl InterpolationMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "Automático (Auto)",
            Self::Linear => "Suave (Bilinear)",
            Self::Nearest => "Pixel Perfect (Nearest)",
        }
    }

    pub fn localized_label(&self, tr: &crate::system::i18n::I18nStrings) -> &'static str {
        match self {
            Self::Auto => tr.interp_auto,
            Self::Linear => tr.interp_linear,
            Self::Nearest => tr.interp_nearest,
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Linear => "Linear",
            Self::Nearest => "Nearest",
        }
    }

    /// Resolve as opções de textura para a escala efetiva atual (escala * ppp).
    pub fn resolve_options(&self, effective_scale: f32) -> TextureOptions {
        match self {
            Self::Auto => {
                if effective_scale >= 0.999 {
                    TextureOptions::NEAREST
                } else {
                    TextureOptions::LINEAR
                }
            }
            Self::Linear => TextureOptions::LINEAR,
            Self::Nearest => TextureOptions::NEAREST,
        }
    }
}

/// Modos de fundo para áreas transparentes da imagem (canal alfa).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CanvasBackground {
    /// Xadrez escuro (#303034 / #222226), ideal para imagens e vetores claros.
    DarkChecker,
    /// Xadrez claro (#F0F0F4 / #DCDCE2), ideal para silhuetas e logos escuros.
    LightChecker,
    /// Preto sólido (#0C0C0E).
    Black,
    /// Branco sólido (#FAFAFC).
    White,
    /// Cor personalizada escolhida pelo usuário.
    Custom(Color32),
}

impl Default for CanvasBackground {
    fn default() -> Self {
        Self::DarkChecker
    }
}

impl CanvasBackground {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DarkChecker => "Xadrez Escuro (Padrão)",
            Self::LightChecker => "Xadrez Claro",
            Self::Black => "Preto Sólido",
            Self::White => "Branco Sólido",
            Self::Custom(_) => "Cor Personalizada",
        }
    }

    pub fn localized_label(&self, tr: &crate::system::i18n::I18nStrings) -> &'static str {
        match self {
            Self::DarkChecker => tr.bg_dark_checker,
            Self::LightChecker => tr.bg_light_checker,
            Self::Black => tr.bg_solid_black,
            Self::White => tr.bg_solid_white,
            Self::Custom(_) => tr.bg_custom,
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::DarkChecker => Self::LightChecker,
            Self::LightChecker => Self::Black,
            Self::Black => Self::White,
            Self::White => Self::DarkChecker,
            Self::Custom(_) => Self::DarkChecker,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum CanvasBackgroundDef {
    DarkChecker,
    LightChecker,
    Black,
    White,
    Custom([u8; 4]),
}

impl serde::Serialize for CanvasBackground {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let def = match *self {
            Self::DarkChecker => CanvasBackgroundDef::DarkChecker,
            Self::LightChecker => CanvasBackgroundDef::LightChecker,
            Self::Black => CanvasBackgroundDef::Black,
            Self::White => CanvasBackgroundDef::White,
            Self::Custom(c) => CanvasBackgroundDef::Custom(c.to_srgba_unmultiplied()),
        };
        def.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for CanvasBackground {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let def = CanvasBackgroundDef::deserialize(deserializer)?;
        Ok(match def {
            CanvasBackgroundDef::DarkChecker => Self::DarkChecker,
            CanvasBackgroundDef::LightChecker => Self::LightChecker,
            CanvasBackgroundDef::Black => Self::Black,
            CanvasBackgroundDef::White => Self::White,
            CanvasBackgroundDef::Custom([r, g, b, a]) => {
                Self::Custom(Color32::from_rgba_unmultiplied(r, g, b, a))
            }
        })
    }
}

pub struct Picture {
    pub texture: TextureHandle,
    pub decoded: Arc<visor::Decoded>,
    pub frame_i: usize,
    pub playing: bool,
    pub shown_at: f64,
    pub texture_options: TextureOptions,
}

impl Deref for Picture {
    type Target = visor::Decoded;

    fn deref(&self) -> &Self::Target {
        &self.decoded
    }
}

impl Picture {
    pub fn frame(&self) -> &visor::Frame {
        &self.decoded.frames[self.frame_i]
    }

    pub fn size(&self) -> Vec2 {
        let frame = self.frame();
        Vec2::new(frame.width as f32, frame.height as f32)
    }
}

pub struct LoadMsg {
    pub token: u64,
    pub result: Result<Arc<visor::Decoded>, String>,
}

pub struct Inflight {
    pub token: u64,
    pub cancel_token: CancellationToken,
    pub rx: Receiver<LoadMsg>,
}

pub struct ThumbMsg {
    pub index: usize,
    pub list_token: u64,
    pub image: Option<ColorImage>,
}

pub struct ClipboardMsg {
    pub token: u64,
    pub png: Option<Vec<u8>>,
    pub dib: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct TrashMsg {
    pub path: PathBuf,
    pub generation: u64,
    pub result: crate::system::trash::TrashResult,
}

pub struct View {
    pub fit: bool,
    pub zoom: f32,
    pub offset: Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self {
            fit: false,
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

impl View {
    pub fn actual(pixels_per_point: f32) -> Self {
        Self {
            fit: false,
            zoom: 1.0 / pixels_per_point.max(0.5),
            offset: Vec2::ZERO,
        }
    }

    pub fn fit() -> Self {
        Self {
            fit: true,
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
    pub fn scale(&self, image_px: Vec2, canvas: Rect) -> f32 {
        if self.fit {
            fit_scale(image_px, canvas)
        } else {
            self.zoom
        }
    }

    pub fn set_fit(&mut self) {
        self.fit = true;
        self.offset = Vec2::ZERO;
    }

    pub fn set_actual(&mut self, pixels_per_point: f32) {
        self.fit = false;
        self.zoom = 1.0 / pixels_per_point.max(0.5);
        self.offset = Vec2::ZERO;
    }

    pub fn is_actual(&self, pixels_per_point: f32) -> bool {
        !self.fit && (self.zoom * pixels_per_point - 1.0).abs() < 0.04
    }
}

pub struct VisorApp {
    pub pool: Arc<ThreadPool>,
    pub cache: ImageCache,
    pub files: Vec<PathBuf>,
    pub index: usize,
    pub list_token: u64,
    pub picture: Option<Picture>,
    pub view: View,
    pub error: Option<String>,
    pub loading: bool,
    pub generation: u64,
    pub queued: Option<(u64, PathBuf)>,
    pub job: Option<Inflight>,
    pub last_dir: Option<PathBuf>,
    pub title: String,
    pub shown_percent: Option<i32>,
    pub checker: TextureHandle,
    pub checker_light: TextureHandle,
    pub interpolation: InterpolationMode,
    pub background: CanvasBackground,
    pub want_open: bool,
    pub want_export: bool,
    pub want_batch_files: bool,
    pub want_batch_dir: bool,
    pub show_info: bool,
    pub show_formats: bool,
    pub show_batch: bool,
    pub show_filmstrip: bool,
    pub filmstrip_scroll_target: Option<usize>,
    pub show_settings: bool,
    pub show_color_picker: bool,
    pub color_picker_hex_input: String,
    pub show_association_banner: bool,
    pub associations: crate::system::assoc::OverallAssociationState,
    pub assoc_rx: Option<Receiver<crate::system::assoc::OverallAssociationState>>,
    pub is_checking_associations: bool,
    pub was_settings_opened_for_assoc: bool,
    pub was_window_focused: bool,
    pub settings: crate::system::config::AppSettings,
    pub settings_writer: Option<crate::system::config::SettingsWriter>,
    pub wheel_nav_accumulator: f32,
    pub format_search: String,
    pub batch_files: Vec<PathBuf>,
    pub batch_kind: visor::ExportKind,
    pub thumbs: HashMap<usize, TextureHandle>,
    pub failed_thumbs: HashSet<usize>,
    pub thumb_rx: Option<Receiver<ThumbMsg>>,
    pub thumb_cancel: Option<CancellationToken>,
    pub notice_tx: Sender<String>,
    pub notice_rx: Receiver<String>,
    pub notice: Option<String>,
    pub clipboard_rx: Option<Receiver<ClipboardMsg>>,
    pub clipboard_token: u64,
    pub shell_sta: Option<Arc<crate::system::shell_sta::ShellStaExecutor>>,
    pub trash_rx: Option<Receiver<TrashMsg>>,
    pub deleting_path: Option<PathBuf>,
    pub updater: crate::updater::UpdaterState,
    pub updater_rx: Option<Receiver<crate::updater::UpdateEvent>>,
    pub updater_tx: Sender<crate::updater::UpdateEvent>,
    pub capture_state: Option<usize>,
    pub capture_wait: usize,
    pub capture_dir: Option<PathBuf>,
}

impl VisorApp {
    /// Retorna o conjunto de strings traduzidas para o idioma ativo nas configurações.
    #[inline]
    pub fn tr(&self) -> &'static crate::system::i18n::I18nStrings {
        crate::system::i18n::strings(self.settings.language)
    }
}

impl Drop for VisorApp {
    fn drop(&mut self) {
        if let Some(job) = &self.job {
            job.cancel_token.cancel();
        }
        if let Some(t) = &self.thumb_cancel {
            t.cancel();
        }
        self.settings.interpolation = self.interpolation;
        self.settings.background = self.background;
        if let CanvasBackground::Custom(c) = self.background {
            self.settings.custom_color = c.to_srgba_unmultiplied();
        }
        self.settings.show_filmstrip = self.show_filmstrip;
        self.settings.show_info = self.show_info;

        if let Some(mut writer) = self.settings_writer.take() {
            writer.schedule_save(self.settings.clone());
            writer.shutdown_and_wait();
        } else {
            let _ = self.settings.save();
        }
    }
}
