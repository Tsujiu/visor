//! Visor — visualizador de imagens para Windows.
//!
//! ```text
//! cargo run --release
//! cargo run --release -- caminho\da\foto.jpg
//! visor.exe --registrar-padrao
//! ```

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Duration;

use eframe::egui::emath::GuiRounding as _;
use eframe::egui::{
    self, Align, Align2, Color32, ColorImage, Context, CursorIcon, FontId, Key, Layout, Modifiers,
    PointerButton, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
    TextureWrapMode, Vec2, ViewportCommand,
};

const BG: Color32 = Color32::from_rgb(22, 22, 24);
const PANEL: Color32 = Color32::from_rgb(30, 30, 34);

fn window_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/moldura.png"))
        .expect("ícone Moldura");
    let rgba = image.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result {
    if std::env::args_os().any(|arg| arg == "--registrar-padrao") {
        register_and_store(true);
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Visor")
            .with_icon(window_icon())
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([520.0, 360.0])
            .with_drag_and_drop(true),
        centered: true,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let initial = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .find(|path| path.as_os_str() != "--registrar-padrao");
    eframe::run_native(
        "Visor",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            cc.egui_ctx.all_styles_mut(|style| {
                style.spacing.button_padding = Vec2::new(10.0, 5.0);
                style.visuals.panel_fill = PANEL;
                style.visuals.window_fill = BG;
            });
            Ok(Box::new(Visor::new(&cc.egui_ctx, initial)))
        }),
    )
}

struct Picture {
    texture: TextureHandle,
    frames: Vec<visor::Frame>,
    frame_i: usize,
    info: visor::ImageInfo,
    playing: bool,
    shown_at: f64,
}

impl Picture {
    fn frame(&self) -> &visor::Frame {
        &self.frames[self.frame_i]
    }

    fn size(&self) -> Vec2 {
        let frame = self.frame();
        Vec2::new(frame.width as f32, frame.height as f32)
    }
}

struct LoadMsg {
    token: u64,
    result: Result<visor::Decoded, String>,
}

struct Inflight {
    token: u64,
    rx: Receiver<LoadMsg>,
}

struct ThumbMsg {
    index: usize,
    list_token: u64,
    image: Option<ColorImage>,
}

struct View {
    fit: bool,
    zoom: f32,
    offset: Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self {
            fit: true,
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
}

impl View {
    fn scale(&self, image_px: Vec2, canvas: Rect) -> f32 {
        if self.fit {
            fit_scale(image_px, canvas)
        } else {
            self.zoom
        }
    }

    fn set_fit(&mut self) {
        self.fit = true;
        self.offset = Vec2::ZERO;
    }

    fn set_actual(&mut self, pixels_per_point: f32) {
        self.fit = false;
        self.zoom = 1.0 / pixels_per_point.max(0.5);
        self.offset = Vec2::ZERO;
    }

    fn is_actual(&self, pixels_per_point: f32) -> bool {
        !self.fit && (self.zoom * pixels_per_point - 1.0).abs() < 0.04
    }
}

struct Visor {
    files: Vec<PathBuf>,
    index: usize,
    list_token: u64,
    picture: Option<Picture>,
    view: View,
    error: Option<String>,
    loading: bool,
    generation: u64,
    queued: Option<(u64, PathBuf)>,
    job: Option<Inflight>,
    last_dir: Option<PathBuf>,
    title: String,
    shown_percent: Option<i32>,
    checker: TextureHandle,
    want_open: bool,
    want_export: bool,
    want_batch_files: bool,
    want_batch_dir: bool,
    show_info: bool,
    show_formats: bool,
    show_batch: bool,
    batch_files: Vec<PathBuf>,
    batch_kind: visor::ExportKind,
    thumbs: HashMap<usize, TextureHandle>,
    thumb_rx: Option<Receiver<ThumbMsg>>,
    notice_tx: Sender<String>,
    notice_rx: Receiver<String>,
    notice: Option<String>,
}

impl Visor {
    fn new(ctx: &Context, initial: Option<PathBuf>) -> Self {
        let checker = ctx.load_texture(
            "checker",
            checker_image(),
            TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            },
        );
        let (notice_tx, notice_rx) = mpsc::channel();
        maybe_register_on_first_launch(notice_tx.clone());
        let mut app = Self {
            files: Vec::new(),
            index: 0,
            list_token: 1,
            picture: None,
            view: View::default(),
            error: None,
            loading: false,
            generation: 0,
            queued: None,
            job: None,
            last_dir: None,
            title: "Visor".to_owned(),
            shown_percent: None,
            checker,
            want_open: false,
            want_export: false,
            want_batch_files: false,
            want_batch_dir: false,
            show_info: false,
            show_formats: false,
            show_batch: false,
            batch_files: Vec::new(),
            batch_kind: visor::ExportKind::Png,
            thumbs: HashMap::new(),
            thumb_rx: None,
            notice_tx,
            notice_rx,
            notice: None,
        };
        if let Some(path) = initial {
            app.open_input(path);
        }
        app
    }

    fn open_input(&mut self, path: PathBuf) {
        let path = normalize_path(&path);
        if path.is_dir() {
            let files = list_images(&path);
            if files.is_empty() {
                self.error = Some("Essa pasta não tem imagens com decodificador neste programa.".to_owned());
                return;
            }
            self.last_dir = Some(path);
            self.set_files(files, 0);
            self.request(self.files[0].clone());
            return;
        }
        if !path.is_file() {
            self.error = Some("Arquivo não encontrado.".to_owned());
            return;
        }

        let parent = path.parent().map(Path::to_path_buf);
        let mut files = parent.as_deref().map(list_images).unwrap_or_default();
        if !files.iter().any(|candidate| same_path(candidate, &path)) {
            files.push(path.clone());
            files.sort_by(|a, b| cmp_name(a, b));
        }
        let index = files
            .iter()
            .position(|candidate| same_path(candidate, &path))
            .unwrap_or(0);
        self.last_dir = parent;
        self.set_files(files, index);
        self.request(path);
    }

    fn set_files(&mut self, files: Vec<PathBuf>, index: usize) {
        self.files = files;
        self.index = index;
        self.list_token = self.list_token.wrapping_add(1);
        self.thumbs.clear();
        self.thumb_rx = None;
    }

    fn request(&mut self, path: PathBuf) {
        self.generation = self.generation.wrapping_add(1);
        self.queued = Some((self.generation, path));
        self.loading = true;
        self.error = None;
    }

    fn step(&mut self, delta: isize) {
        let len = self.files.len() as isize;
        if len < 2 {
            return;
        }
        let next = (self.index as isize + delta).rem_euclid(len) as usize;
        self.index = next;
        self.request(self.files[next].clone());
    }

    fn go_to(&mut self, index: usize) {
        if self.files.is_empty() || index >= self.files.len() || index == self.index {
            return;
        }
        self.index = index;
        self.request(self.files[index].clone());
    }

    fn pump(&mut self, ctx: &Context) {
        let finished = match &self.job {
            Some(job) => match job.rx.try_recv() {
                Ok(msg) => Some(msg),
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(16));
                    None
                }
                Err(TryRecvError::Disconnected) => Some(LoadMsg {
                    token: job.token,
                    result: Err("A leitura da imagem foi interrompida.".to_owned()),
                }),
            },
            None => None,
        };

        if let Some(msg) = finished {
            let current = msg.token == self.generation;
            self.job = None;
            if current {
                self.loading = false;
                match msg.result {
                    Ok(decoded) => self.install(ctx, decoded),
                    Err(err) => {
                        self.picture = None;
                        self.shown_percent = None;
                        self.error = Some(err);
                    }
                }
            }
        }

        if self.job.is_none() {
            if let Some((token, path)) = self.queued.take() {
                self.spawn(ctx, token, path);
            }
        }

        if let Ok(text) = self.notice_rx.try_recv() {
            self.notice = Some(text);
        }
        self.poll_thumbs(ctx);
    }

    fn spawn(&mut self, ctx: &Context, token: u64, path: PathBuf) {
        let (tx, rx) = mpsc::channel();
        self.job = Some(Inflight { token, rx });
        self.loading = true;
        ctx.request_repaint();

        let built = std::thread::Builder::new()
            .name("visor-decode".to_owned())
            .spawn(move || {
                let result = std::panic::catch_unwind(AssertUnwindSafe(|| visor::open_image(&path)));
                let result = match result {
                    Ok(decoded) => decoded,
                    Err(_) => Err("Esse arquivo não pôde ser lido.".to_owned()),
                };
                let _ = tx.send(LoadMsg { token, result });
            });

        if built.is_err() {
            self.job = None;
            self.loading = false;
            self.error = Some("Não foi possível iniciar a leitura.".to_owned());
        }
    }

    fn install(&mut self, ctx: &Context, decoded: visor::Decoded) {
        if decoded.frames.is_empty() {
            self.picture = None;
            self.error = Some("A imagem não tem pixels.".to_owned());
            return;
        }
        let now = ctx.input(|input| input.time);
        let frame = &decoded.frames[0];
        let color = ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        let texture = ctx.load_texture(
            format!("picture-{}", self.generation),
            color,
            TextureOptions::LINEAR,
        );
        let playing = decoded.frames.len() > 1;
        self.picture = Some(Picture {
            texture,
            frames: decoded.frames,
            frame_i: 0,
            info: decoded.info,
            playing,
            shown_at: now,
        });
        self.view = View::default();
        self.error = None;
    }

    fn upload_frame(&mut self, ctx: &Context) {
        let Some(picture) = self.picture.as_mut() else {
            return;
        };
        let frame = picture.frame();
        let color = ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        picture.texture.set(color, TextureOptions::LINEAR);
        picture.shown_at = ctx.input(|input| input.time);
    }

    fn advance_animation(&mut self, ctx: &Context) {
        let Some(picture) = self.picture.as_ref() else {
            return;
        };
        if !picture.playing || picture.frames.len() < 2 {
            return;
        }
        let now = ctx.input(|input| input.time);
        let delay = picture.frames[picture.frame_i].delay_ms.max(1) as f64 / 1000.0;
        if now - picture.shown_at < delay {
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        if let Some(picture) = self.picture.as_mut() {
            picture.frame_i = (picture.frame_i + 1) % picture.frames.len();
        }
        self.upload_frame(ctx);
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn rotate(&mut self, ctx: &Context, clockwise: bool) {
        let Some(picture) = self.picture.as_mut() else {
            return;
        };
        for frame in &mut picture.frames {
            let (pixels, width, height) =
                rotate_pixels(&frame.rgba, frame.width, frame.height, clockwise);
            frame.rgba = pixels;
            frame.width = width;
            frame.height = height;
        }
        self.view.offset = Vec2::ZERO;
        self.upload_frame(ctx);
    }

    fn browse(&mut self, frame: &eframe::Frame) {
        let verified = visor::verified_extensions();
        let partial = visor::partial_extensions();
        let external = visor::external_extensions();
        let mut dialog = rfd::FileDialog::new()
            .set_title("Abrir imagem")
            .set_parent(frame)
            .add_filter("Suporte completo ou somente leitura", &verified)
            .add_filter("Suporte parcial", &partial);
        if visor::magick_available() {
            dialog = dialog.add_filter("Conversor externo (ImageMagick)", &external);
        }
        dialog = dialog.add_filter("Todos os arquivos", &["*"]);
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.pick_file() {
            self.open_input(path);
        }
    }

    fn export_current(&mut self, frame: &eframe::Frame) {
        let Some(src) = self.current_path().map(Path::to_path_buf) else {
            return;
        };
        let stem = src
            .file_stem()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "imagem".to_owned());
        let mut dialog = rfd::FileDialog::new()
            .set_title("Exportar")
            .set_parent(frame)
            .set_file_name(format!("{stem}.png"));
        for kind in visor::ExportKind::all() {
            dialog = dialog.add_filter(kind.label(), &[kind.extension()]);
        }
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        let Some(dst) = dialog.save_file() else {
            return;
        };
        let kind = kind_from_path(&dst);
        let tx = self.notice_tx.clone();
        self.notice = Some("Exportando…".to_owned());
        std::thread::spawn(move || {
            let text = match visor::export_image(&src, &dst, kind) {
                Ok(()) => format!("Exportado: {}", dst.display()),
                Err(err) => format!("Exportação falhou: {err}"),
            };
            let _ = tx.send(text);
        });
    }

    fn pick_batch_files(&mut self, frame: &eframe::Frame) {
        let verified = visor::verified_extensions();
        let partial = visor::partial_extensions();
        let mut dialog = rfd::FileDialog::new()
            .set_title("Arquivos do lote")
            .set_parent(frame)
            .add_filter("Imagens que este programa lê", &verified)
            .add_filter("Suporte parcial", &partial)
            .add_filter("Todos os arquivos", &["*"]);
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(files) = dialog.pick_files() {
            self.batch_files = files;
            self.show_batch = true;
        }
    }

    fn run_batch(&mut self, frame: &eframe::Frame) {
        if self.batch_files.is_empty() {
            self.notice = Some("Escolha os arquivos do lote.".to_owned());
            return;
        }
        let mut dialog = rfd::FileDialog::new()
            .set_title("Pasta de saída")
            .set_parent(frame);
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        let Some(dir) = dialog.pick_folder() else {
            return;
        };
        let files = self.batch_files.clone();
        let kind = self.batch_kind;
        let tx = self.notice_tx.clone();
        self.notice = Some("Convertendo o lote…".to_owned());
        std::thread::spawn(move || {
            let results = visor::export_many(&files, &dir, kind);
            let ok = results.iter().filter(|(_, result)| result.is_ok()).count();
            let failed: Vec<_> = results
                .iter()
                .filter_map(|(_, result)| result.as_ref().err())
                .take(2)
                .cloned()
                .collect();
            let mut text = format!("Lote: {ok} de {} em {}", results.len(), dir.display());
            if !failed.is_empty() {
                text.push_str(". ");
                text.push_str(&failed.join(" "));
            }
            let _ = tx.send(text);
        });
    }

    fn define_default(&mut self) {
        let tx = self.notice_tx.clone();
        self.notice = Some("Registrando o Visor como visualizador…".to_owned());
        std::thread::spawn(move || {
            let _ = tx.send(register_and_store(true));
        });
    }

    fn take_drops(&mut self, ctx: &Context) {
        let mut dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if dropped.len() == 1 {
            self.open_input(dropped.remove(0));
            return;
        }
        let mut images: Vec<PathBuf> = dropped
            .into_iter()
            .filter(|path| path.is_file() && is_image_path(path))
            .collect();
        if images.len() > 1 {
            images.sort_by(|a, b| cmp_name(a, b));
            let first = images[0].clone();
            self.last_dir = first.parent().map(Path::to_path_buf);
            self.set_files(images, 0);
            self.request(first);
        } else if let Some(path) = images.pop() {
            self.open_input(path);
        }
    }

    fn handle_keys(&mut self, ui: &mut egui::Ui) {
        let open = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::O));
        if open {
            self.want_open = true;
        }
        let export = ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::S));
        if export {
            self.want_export = true;
        }

        let (plain_left, plain_right, home, end, fit, actual, rotate, shift, fullscreen, info, space) =
            ui.input(|input| {
                let plain = !input.modifiers.command && !input.modifiers.alt && !input.modifiers.shift;
                (
                    plain && input.key_pressed(Key::ArrowLeft),
                    plain && input.key_pressed(Key::ArrowRight),
                    plain && input.key_pressed(Key::Home),
                    plain && input.key_pressed(Key::End),
                    plain && input.key_pressed(Key::F),
                    plain && (input.key_pressed(Key::Num1) || input.key_pressed(Key::Num0)),
                    !input.modifiers.command && !input.modifiers.alt && input.key_pressed(Key::R),
                    input.modifiers.shift,
                    plain && input.key_pressed(Key::F11),
                    plain && input.key_pressed(Key::I),
                    plain && input.key_pressed(Key::Space),
                )
            });

        if plain_left {
            self.step(-1);
        }
        if plain_right {
            self.step(1);
        }
        if home {
            self.go_to(0);
        }
        if end {
            self.go_to(self.files.len().saturating_sub(1));
        }
        if fit {
            self.view.set_fit();
        }
        if actual {
            let ppp = ui.pixels_per_point();
            self.view.set_actual(ppp);
        }
        if rotate {
            self.rotate(ui.ctx(), !shift);
        }
        if info {
            self.show_info = !self.show_info;
        }
        if space {
            if let Some(picture) = self.picture.as_mut() {
                if picture.frames.len() > 1 {
                    picture.playing = !picture.playing;
                    picture.shown_at = ui.input(|input| input.time);
                }
            }
        }
        if fullscreen {
            let on = ui.ctx().input(|input| input.viewport().fullscreen.unwrap_or(false));
            ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(!on));
        }

        let escape_fullscreen = ui.input(|input| {
            input.key_pressed(Key::Escape)
                && input.viewport().fullscreen.unwrap_or(false)
                && !input.modifiers.command
        });
        if escape_fullscreen {
            ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(false));
        }

        let (back, forward) = ui.input(|input| {
            (
                input.pointer.button_pressed(PointerButton::Extra1),
                input.pointer.button_pressed(PointerButton::Extra2),
            )
        });
        if back {
            self.step(-1);
        }
        if forward {
            self.step(1);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Abrir").on_hover_text("Ctrl+O").clicked() {
                self.want_open = true;
            }
            if ui
                .add_enabled(self.picture.is_some(), egui::Button::new("Exportar"))
                .on_hover_text("Ctrl+S")
                .clicked()
            {
                self.want_export = true;
            }
            if ui.button("Lote").on_hover_text("Converter vários arquivos").clicked() {
                self.show_batch = true;
            }
            if ui.button("Formatos").clicked() {
                self.show_formats = !self.show_formats;
            }
            if ui
                .button("Padrão")
                .on_hover_text("Definir o Visor como visualizador deste usuário")
                .clicked()
            {
                self.define_default();
            }
            ui.separator();

            let can_step = self.files.len() > 1;
            if ui
                .add_enabled(can_step, egui::Button::new("◀"))
                .on_hover_text("Imagem anterior  ·  ←")
                .clicked()
            {
                self.step(-1);
            }
            if ui
                .add_enabled(can_step, egui::Button::new("▶"))
                .on_hover_text("Próxima imagem  ·  →")
                .clicked()
            {
                self.step(1);
            }
            ui.separator();

            let ppp = ui.pixels_per_point();
            if ui
                .selectable_label(self.view.fit, "Ajustar")
                .on_hover_text("F")
                .clicked()
            {
                self.view.set_fit();
            }
            if ui
                .selectable_label(self.view.is_actual(ppp), "100%")
                .on_hover_text("1  ·  um pixel da imagem por pixel da tela")
                .clicked()
            {
                self.view.set_actual(ppp);
            }
            if ui
                .add_enabled(self.picture.is_some(), egui::Button::new("Girar"))
                .on_hover_text("R  ·  Shift+R")
                .clicked()
            {
                self.rotate(ui.ctx(), true);
            }
            let animated = self
                .picture
                .as_ref()
                .is_some_and(|picture| picture.frames.len() > 1);
            if animated {
                let playing = self.picture.as_ref().is_some_and(|picture| picture.playing);
                let label = if playing { "Pausa" } else { "Reproduzir" };
                if ui.button(label).on_hover_text("Espaço").clicked() {
                    if let Some(picture) = self.picture.as_mut() {
                        picture.playing = !picture.playing;
                        picture.shown_at = ui.input(|input| input.time);
                    }
                }
            }
            if ui
                .selectable_label(self.show_info, "Info")
                .on_hover_text("I")
                .clicked()
            {
                self.show_info = !self.show_info;
            }

            if let Some(name) = self.current_name() {
                ui.separator();
                ui.label(RichText::new(name).strong());
            }
            if self.loading {
                ui.label(RichText::new("carregando…").weak());
            }
        });
    }

    fn info_panel(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let Some(picture) = &self.picture else {
                ui.label("Nenhuma imagem aberta.");
                return;
            };
            let info = &picture.info;
            ui.label(RichText::new(&info.name).strong());
            ui.label(format!("{} · {}", info.category.label(), info.support.label()));
            ui.separator();
            ui.label(format!(
                "{} × {} px na tela",
                picture.frame().width,
                picture.frame().height
            ));
            ui.label(format!("original {} × {}", info.original_w, info.original_h));
            ui.label(format!("{} bits · {}", info.bit_depth, info.color));
            ui.label(if info.has_alpha {
                "canal alfa presente"
            } else {
                "sem transparência"
            });
            if info.mipmaps > 1 {
                ui.label(format!("{} mipmaps; a tela mostra o nível de base", info.mipmaps));
            }
            if info.frame_count > 1 {
                ui.label(format!(
                    "quadro {} de {} · {} ms",
                    picture.frame_i + 1,
                    info.frame_count,
                    picture.frame().delay_ms
                ));
            }
            if let Some(icc) = &info.icc {
                ui.separator();
                ui.label(icc);
            }
            if !info.notes.is_empty() {
                ui.separator();
                for note in &info.notes {
                    ui.label(RichText::new(note).weak());
                }
            }
            if !info.exif.is_empty() {
                ui.separator();
                ui.label(RichText::new("EXIF").strong());
                for (tag, value) in &info.exif {
                    ui.label(format!("{tag}: {value}"));
                }
            }
        });
    }

    fn formats_window(&mut self, ctx: &Context) {
        let mut open = self.show_formats;
        egui::Window::new("Formatos")
            .open(&mut open)
            .default_size([520.0, 460.0])
            .show(ctx, |ui| {
                ui.label("O filtro de abertura só lista o que um teste de decodificação exercitou.");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for category in [
                        visor::Category::Raster,
                        visor::Category::Texture,
                        visor::Category::Vector,
                        visor::Category::Raw,
                        visor::Category::Animation,
                    ] {
                        ui.add_space(8.0);
                        ui.label(RichText::new(category.label()).strong());
                        for spec in visor::formats().iter().filter(|spec| spec.category == category) {
                            let exts = spec.extensions.join(", ");
                            ui.label(format!("{} · {} · {exts}", spec.name, spec.support.label()));
                            ui.label(RichText::new(spec.note).weak().size(12.0));
                        }
                    }
                    ui.add_space(8.0);
                    ui.label(RichText::new("Bibliotecas").strong());
                    for (name, role) in visor::libraries() {
                        ui.label(format!("{name}: {role}"));
                    }
                });
            });
        self.show_formats = open;
    }

    fn batch_window(&mut self, ctx: &Context) {
        let mut open = self.show_batch;
        let mut pick = false;
        let mut run = false;
        egui::Window::new("Conversão em lote")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("{} arquivo(s)", self.batch_files.len()));
                egui::ComboBox::from_id_salt("lote-formato")
                    .selected_text(self.batch_kind.label())
                    .show_ui(ui, |ui| {
                        for kind in visor::ExportKind::all() {
                            ui.selectable_value(&mut self.batch_kind, *kind, kind.label());
                        }
                    });
                if ui.button("Escolher arquivos").clicked() {
                    pick = true;
                }
                if ui.button("Exportar para uma pasta").clicked() {
                    run = true;
                }
                ui.label("A exportação relê cada arquivo. PNG e TIFF de 16 bits continuam em 16 bits.");
            });
        self.show_batch = open;
        if pick {
            self.want_batch_files = true;
        }
        if run {
            self.want_batch_dir = true;
        }
    }

    fn filmstrip(&mut self, ui: &mut egui::Ui) {
        let start = self.index.saturating_sub(3);
        let end = (self.index + 4).min(self.files.len());
        let mut clicked = None;
        ui.horizontal(|ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for index in start..end {
                        let selected = index == self.index;
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(104.0, 78.0), Sense::click());
                        let fill = if selected {
                            Color32::from_rgb(48, 72, 110)
                        } else {
                            Color32::from_rgb(36, 36, 40)
                        };
                        ui.painter().rect_filled(rect, 4.0, fill);
                        if let Some(texture) = self.thumbs.get(&index) {
                            let image = texture.size_vec2();
                            let fitted = fit_rect(rect.shrink(4.0), image);
                            ui.painter().image(
                                texture.id(),
                                fitted,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else if let Some(name) = self.files.get(index).and_then(|path| path.file_name())
                        {
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                name.to_string_lossy(),
                                FontId::proportional(11.0),
                                Color32::from_gray(180),
                            );
                        }
                        if response.clicked() {
                            clicked = Some(index);
                        }
                    }
                });
            });
        });
        if let Some(index) = clicked {
            self.go_to(index);
        }
    }

    fn poll_thumbs(&mut self, ctx: &Context) {
        let ready = self
            .thumb_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok());
        if let Some(msg) = ready {
            self.thumb_rx = None;
            if msg.list_token == self.list_token {
                if let Some(image) = msg.image {
                    let texture = ctx.load_texture(
                        format!("thumb-{}-{}", msg.list_token, msg.index),
                        image,
                        TextureOptions::LINEAR,
                    );
                    self.thumbs.insert(msg.index, texture);
                }
            }
        }
        if self.thumb_rx.is_some() || self.files.is_empty() {
            if self.thumb_rx.is_some() {
                ctx.request_repaint_after(Duration::from_millis(40));
            }
            return;
        }
        let start = self.index.saturating_sub(3);
        let end = (self.index + 4).min(self.files.len());
        let Some(index) = (start..end).find(|index| !self.thumbs.contains_key(index)) else {
            return;
        };
        let path = self.files[index].clone();
        let list_token = self.list_token;
        let (tx, rx) = mpsc::channel();
        self.thumb_rx = Some(rx);
        ctx.request_repaint();
        let spawned = std::thread::Builder::new()
            .name("visor-thumb".to_owned())
            .spawn(move || {
                let image = visor::thumbnail(&path, 96).ok().map(|frame| {
                    ColorImage::from_rgba_unmultiplied(
                        [frame.width as usize, frame.height as usize],
                        &frame.rgba,
                    )
                });
                let _ = tx.send(ThumbMsg {
                    index,
                    list_token,
                    image,
                });
            });
        if spawned.is_err() {
            self.thumb_rx = None;
        }
    }

    fn status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(picture) = &self.picture {
                let frame = picture.frame();
                ui.label(format!("{} × {}", frame.width, frame.height));
                ui.label(picture.info.name.as_str());
                ui.label(RichText::new(picture.info.support.label()).weak());
                if picture.info.reduced {
                    ui.label(
                        RichText::new(format!(
                            "original {} × {}",
                            picture.info.original_w, picture.info.original_h
                        ))
                        .weak(),
                    );
                }
                if picture.frames.len() > 1 {
                    ui.label(format!("quadro {}/{}", picture.frame_i + 1, picture.frames.len()));
                }
            }
            if !self.files.is_empty() {
                ui.label(format!("{} / {}", self.index + 1, self.files.len()));
            }
            if let Some(percent) = self.shown_percent {
                ui.label(format!("{percent}%"));
            }
            if let Some(notice) = &self.notice {
                ui.label(RichText::new(notice).weak());
            }
            if let Some(path) = self.current_path() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(egui::Label::new(path.display().to_string()).truncate());
                });
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let canvas = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(canvas, Sense::click_and_drag());
        let painter = ui.painter().clone();
        painter.rect_filled(canvas, 0.0, BG);

        let hovering_file = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
        if hovering_file {
            painter.rect_stroke(
                canvas.shrink(6.0),
                8.0,
                Stroke::new(2.0, Color32::from_rgb(120, 176, 255)),
                StrokeKind::Inside,
            );
        }

        let Some((image_px, has_alpha, texture_id)) = self.picture.as_ref().map(|picture| {
            (picture.size(), picture.info.has_alpha, picture.texture.id())
        }) else {
            self.draw_empty(&painter, canvas);
            return;
        };

        let ppp = ui.pixels_per_point();
        let (min_scale, max_scale) = scale_limits(ppp);

        let zoom_in = ui.input(|input| input.key_pressed(Key::Plus) || input.key_pressed(Key::Equals));
        let zoom_out = ui.input(|input| input.key_pressed(Key::Minus));
        if zoom_in || zoom_out {
            let factor = if zoom_in { 1.15 } else { 1.0 / 1.15 };
            let anchor = ui
                .ctx()
                .input(|input| input.pointer.hover_pos())
                .unwrap_or_else(|| canvas.center());
            self.apply_zoom(canvas, image_px, anchor, factor, min_scale, max_scale);
        }

        if response.double_clicked() {
            if self.view.fit {
                self.view.set_actual(ppp);
            } else {
                self.view.set_fit();
            }
        }

        let scale_before_drag = self.view.scale(image_px, canvas);
        if response.dragged_by(PointerButton::Primary) {
            if self.view.fit {
                self.view.fit = false;
                self.view.zoom = scale_before_drag;
                self.view.offset = Vec2::ZERO;
            }
            self.view.offset += response.drag_delta();
        }

        if response.hovered() {
            let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll_y.abs() > 0.1 {
                let anchor = response.hover_pos().unwrap_or_else(|| canvas.center());
                let factor = (scroll_y * 0.0016).exp();
                self.apply_zoom(canvas, image_px, anchor, factor, min_scale, max_scale);
            }
        }

        if !self.view.fit {
            let scale = self.view.zoom;
            self.view.offset = clamp_offset(canvas, image_px, scale, self.view.offset);
        }

        let scale = self.view.scale(image_px, canvas);
        let rect = image_rect(canvas, image_px, scale, self.view.offset).round_to_pixels(ppp);
        self.shown_percent = Some((scale * ppp * 100.0).round() as i32);

        if has_alpha {
            let cell = 12.0;
            let repeats = rect.size() / (cell * 2.0);
            painter.image(
                self.checker.id(),
                rect,
                Rect::from_min_size(Pos2::ZERO, repeats),
                Color32::WHITE,
            );
        }
        painter.image(
            texture_id,
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else if response.hovered() {
            response.clone().on_hover_cursor(CursorIcon::Grab);
        }

        if self.loading {
            painter.text(
                canvas.center_top() + Vec2::new(0.0, 18.0),
                Align2::CENTER_TOP,
                "Carregando…",
                FontId::proportional(16.0),
                Color32::from_gray(210),
            );
        }
    }

    fn draw_empty(&self, painter: &egui::Painter, canvas: Rect) {
        let color = Color32::from_gray(168);
        let muted = Color32::from_gray(120);
        if let Some(error) = &self.error {
            let galley = painter.layout(
                error.clone(),
                FontId::proportional(16.0),
                Color32::from_rgb(255, 140, 130),
                (canvas.width() - 64.0).max(120.0),
            );
            painter.galley(
                canvas.center() - galley.size() * 0.5,
                galley,
                Color32::WHITE,
            );
            return;
        }
        if self.loading {
            painter.text(
                canvas.center(),
                Align2::CENTER_CENTER,
                "Carregando…",
                FontId::proportional(22.0),
                color,
            );
            return;
        }
        painter.text(
            canvas.center() + Vec2::new(0.0, -36.0),
            Align2::CENTER_CENTER,
            "Arraste uma imagem para cá",
            FontId::proportional(24.0),
            color,
        );
        painter.text(
            canvas.center() + Vec2::new(0.0, 0.0),
            Align2::CENTER_CENTER,
            "ou clique em Abrir    ·    Ctrl+O",
            FontId::proportional(16.0),
            muted,
        );
        painter.text(
            canvas.center() + Vec2::new(0.0, 32.0),
            Align2::CENTER_CENTER,
            "← → pasta    ·    roda: zoom    ·    arrastar: mover    ·    F ajusta    ·    1 tamanho real    ·    R gira",
            FontId::proportional(14.0),
            muted,
        );
        painter.text(
            canvas.center() + Vec2::new(0.0, 56.0),
            Align2::CENTER_CENTER,
            "I informações    ·    Espaço animação    ·    Ctrl+S exportar",
            FontId::proportional(14.0),
            muted,
        );
    }

    fn apply_zoom(
        &mut self,
        canvas: Rect,
        image_px: Vec2,
        anchor: Pos2,
        factor: f32,
        min_scale: f32,
        max_scale: f32,
    ) {
        let scale = self.view.scale(image_px, canvas);
        let (zoom, offset) = zoom_about(
            canvas, image_px, scale, self.view.offset, anchor, factor, min_scale, max_scale,
        );
        self.view.fit = false;
        self.view.zoom = zoom;
        self.view.offset = offset;
    }

    fn current_path(&self) -> Option<&Path> {
        self.files.get(self.index).map(PathBuf::as_path)
    }

    fn current_name(&self) -> Option<String> {
        self.current_path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
    }

    fn sync_title(&mut self, ctx: &Context) {
        let title = match self.current_name() {
            Some(name) => format!("{name} — Visor"),
            None if self.loading => "Carregando… — Visor".to_owned(),
            None => "Visor".to_owned(),
        };
        if self.title != title {
            self.title.clone_from(&title);
            ctx.send_viewport_cmd(ViewportCommand::Title(title));
        }
    }
}

impl eframe::App for Visor {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.pump(ui.ctx());
        self.take_drops(ui.ctx());
        self.handle_keys(ui);
        self.advance_animation(ui.ctx());

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        if self.show_info {
            egui::Panel::right("info")
                .exact_size(300.0)
                .show(ui, |ui| self.info_panel(ui));
        }
        egui::Panel::bottom("status").show(ui, |ui| self.status(ui));
        if self.files.len() > 1 {
            egui::Panel::bottom("film").show(ui, |ui| self.filmstrip(ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG))
            .show(ui, |ui| self.canvas(ui));

        self.formats_window(ui.ctx());
        self.batch_window(ui.ctx());

        if self.want_open {
            self.want_open = false;
            self.browse(frame);
        }
        if self.want_export {
            self.want_export = false;
            self.export_current(frame);
        }
        if self.want_batch_files {
            self.want_batch_files = false;
            self.pick_batch_files(frame);
        }
        if self.want_batch_dir {
            self.want_batch_dir = false;
            self.run_batch(frame);
        }
        self.pump(ui.ctx());
        self.sync_title(ui.ctx());
    }
}

fn checker_image() -> ColorImage {
    ColorImage::from_rgba_unmultiplied(
        [2, 2],
        &[
            58, 58, 62, 255, 42, 42, 46, 255, 42, 42, 46, 255, 58, 58, 62, 255,
        ],
    )
}

fn rotate_pixels(pixels: &[u8], width: u32, height: u32, clockwise: bool) -> (Vec<u8>, u32, u32) {
    let Some(buffer) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels.to_vec())
    else {
        return (pixels.to_vec(), width, height);
    };
    let rotated = if clockwise {
        image::imageops::rotate90(&buffer)
    } else {
        image::imageops::rotate270(&buffer)
    };
    let (width, height) = rotated.dimensions();
    (rotated.into_raw(), width, height)
}

fn list_images(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_image_path(path))
        .collect();
    files.sort_by(|a, b| cmp_name(a, b));
    files
}

fn is_image_path(path: &Path) -> bool {
    visor::browsable(path)
}

fn kind_from_path(path: &Path) -> visor::ExportKind {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "jpe" | "jfif" => visor::ExportKind::Jpeg,
        "webp" => visor::ExportKind::Webp,
        "bmp" => visor::ExportKind::Bmp,
        "tif" | "tiff" => visor::ExportKind::Tiff,
        "gif" => visor::ExportKind::Gif,
        "qoi" => visor::ExportKind::Qoi,
        "tga" => visor::ExportKind::Tga,
        "ico" => visor::ExportKind::Ico,
        "ppm" | "pgm" | "pbm" | "pnm" => visor::ExportKind::Pnm,
        "hdr" => visor::ExportKind::Hdr,
        _ => visor::ExportKind::Png,
    }
}

fn cmp_name(a: &Path, b: &Path) -> Ordering {
    let left = a
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let right = b
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    left.cmp(&right).then_with(|| a.cmp(b))
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
}

fn normalize_path(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

fn local_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Visor")
}

fn open_default_apps_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", "ms-settings:defaultapps"])
        .spawn();
}

fn register_and_store(open_settings_on_failure: bool) -> String {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("visor.exe"));
    let report = visor::register_as_default(&exe);
    let dir = local_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("assoc-report.txt"), report.summary());
    let _ = fs::write(dir.join("default-attempted.txt"), b"1");
    if open_settings_on_failure && !report.png_ok {
        open_default_apps_settings();
    }
    report.summary()
}

fn maybe_register_on_first_launch(tx: Sender<String>) {
    if std::env::var_os("VISOR_SKIP_REGISTER").is_some() {
        return;
    }
    if visor::current_png_progid().as_deref() == Some("Visor.Image") {
        return;
    }
    let marker = local_dir().join("default-attempted.txt");
    if marker.exists() {
        return;
    }
    if let Some(parent) = marker.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&marker, b"1");
    std::thread::spawn(move || {
        let _ = tx.send(register_and_store(true));
    });
}

fn fit_scale(image_px: Vec2, canvas: Rect) -> f32 {
    let available = (canvas.size() - Vec2::splat(32.0)).max(Vec2::splat(32.0));
    let width = image_px.x.max(1.0);
    let height = image_px.y.max(1.0);
    (available.x / width).min(available.y / height).max(0.000_1)
}

fn scale_limits(pixels_per_point: f32) -> (f32, f32) {
    let pixels_per_point = pixels_per_point.max(0.5);
    (0.02 / pixels_per_point, 32.0 / pixels_per_point)
}

fn image_rect(canvas: Rect, image_px: Vec2, scale: f32, offset: Vec2) -> Rect {
    Rect::from_center_size(canvas.center() + offset, image_px * scale)
}

fn fit_rect(bounds: Rect, image: Vec2) -> Rect {
    let scale = (bounds.width() / image.x.max(1.0))
        .min(bounds.height() / image.y.max(1.0))
        .max(0.000_1);
    Rect::from_center_size(bounds.center(), image * scale)
}

fn zoom_about(
    canvas: Rect,
    image_px: Vec2,
    scale: f32,
    offset: Vec2,
    anchor: Pos2,
    factor: f32,
    min_scale: f32,
    max_scale: f32,
) -> (f32, Vec2) {
    let safe_scale = scale.max(0.000_1);
    let new_scale = (safe_scale * factor).clamp(min_scale, max_scale);
    let old = image_rect(canvas, image_px, safe_scale, offset);
    let point = if old.contains(anchor) { anchor } else { old.center() };
    let relative = (point - old.min) / safe_scale;
    let new_min = point - relative * new_scale;
    let new_center = new_min + image_px * new_scale * 0.5;
    (new_scale, new_center - canvas.center())
}

fn clamp_offset(canvas: Rect, image_px: Vec2, scale: f32, offset: Vec2) -> Vec2 {
    let rect = image_rect(canvas, image_px, scale, offset);
    let margin = 48.0;
    let mut delta = Vec2::ZERO;
    if rect.right() < canvas.left() + margin {
        delta.x = canvas.left() + margin - rect.right();
    } else if rect.left() > canvas.right() - margin {
        delta.x = canvas.right() - margin - rect.left();
    }
    if rect.bottom() < canvas.top() + margin {
        delta.y = canvas.top() + margin - rect.bottom();
    } else if rect.top() > canvas.bottom() - margin {
        delta.y = canvas.bottom() - margin - rect.top();
    }
    offset + delta
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("visor-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn recognizes_extensions() {
        assert!(is_image_path(Path::new("foto.JPEG")));
        assert!(is_image_path(Path::new(r"C:\imgs\a.WebP")));
        assert!(!is_image_path(Path::new("notas.txt")));
    }

    #[test]
    fn lists_images_case_insensitive() {
        let dir = temp_dir("list");
        fs::write(dir.join("b.PNG"), b"").unwrap();
        fs::write(dir.join("a.jpg"), b"").unwrap();
        fs::write(dir.join("nota.txt"), b"oi").unwrap();
        let listed = list_images(&dir);
        let names: Vec<_> = listed
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.jpg", "b.PNG"]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let canvas = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let image_px = Vec2::new(100.0, 50.0);
        let scale = 2.0;
        let offset = Vec2::new(10.0, -4.0);
        let old = image_rect(canvas, image_px, scale, offset);
        let anchor = old.min + Vec2::new(30.0, 12.0);
        let (new_scale, new_offset) = zoom_about(
            canvas, image_px, scale, offset, anchor, 1.5, 0.01, 100.0,
        );
        let new_rect = image_rect(canvas, image_px, new_scale, new_offset);
        let relative = (anchor - new_rect.min) / new_scale;
        assert!((relative.x - 30.0 / scale).abs() < 0.01);
        assert!((relative.y - 12.0 / scale).abs() < 0.01);
    }

    #[test]
    fn fit_uses_the_smaller_axis() {
        let canvas = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0));
        let scale = fit_scale(Vec2::new(100.0, 100.0), canvas);
        assert!(scale < 1.0);
        assert!(scale > 0.5);
    }
}
