//! Ações, transições de estado, I/O assíncrono e controle do aplicativo.

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use eframe::egui::{
    self, ColorImage, Context, TextureOptions, TextureWrapMode, Vec2, ViewportCommand,
};

use super::state::{ClipboardMsg, Inflight, LoadMsg, Picture, TrashMsg, View, VisorApp};
use crate::core::image_cache::{CacheKey, ImageCache};
use crate::core::image_ops::rotate_pixels;
use crate::core::navigation::{cmp_name, is_image_path, list_images};
#[cfg(not(test))]
use crate::system::assoc::ensure_registered_in_background;
use crate::system::clipboard::{
    ClipboardPublishResult, get_valid_window_handle, prepare_dibv5, prepare_png,
    publish_to_clipboard,
};
use crate::system::paths::{kind_from_path, normalize_path, same_path};
use crate::system::shell_sta::ShellStaExecutor;
use crate::system::trash::{TrashResult, send_to_trash};
use crate::ui::filmstrip::poll_thumbs;
use crate::ui::theme::{checker_image, light_checker_image};
use crate::workers::{CancellationToken, Priority, ThreadPool};

impl VisorApp {
    pub fn new(ctx: &Context, initial: Option<PathBuf>) -> Self {
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
        let checker_light = ctx.load_texture(
            "checker_light",
            light_checker_image(),
            TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: TextureWrapMode::Repeat,
                mipmap_mode: None,
            },
        );
        let (notice_tx, notice_rx) = mpsc::channel();
        let (updater_tx, updater_rx) = mpsc::channel();
        #[cfg(not(test))]
        ensure_registered_in_background();
        let (settings, settings_notice) = crate::system::config::AppSettings::load();
        if let Some(msg) = settings_notice {
            let _ = notice_tx.send(msg);
        }
        let pool = Arc::new(ThreadPool::with_default_workers());
        let shell_sta = ShellStaExecutor::new().ok().map(Arc::new);
        let cache = ImageCache::default();
        let show_association_banner = false;
        let initial_view = match settings.default_zoom {
            crate::system::config::DefaultZoomMode::Actual100 => {
                View::actual(ctx.pixels_per_point())
            }
            crate::system::config::DefaultZoomMode::Fit => View::fit(),
        };
        let mut app = Self {
            pool,
            cache,
            files: Vec::new(),
            index: 0,
            list_token: 1,
            picture: None,
            view: initial_view,
            interpolation: settings.interpolation,
            background: settings.background,
            error: None,
            loading: false,
            generation: 0,
            queued: None,
            job: None,
            last_dir: None,
            title: "Visor".to_owned(),
            shown_percent: None,
            checker,
            checker_light,
            want_open: false,
            want_export: false,
            want_batch_files: false,
            want_batch_dir: false,
            show_info: settings.show_info,
            show_formats: false,
            show_batch: false,
            show_filmstrip: settings.show_filmstrip,
            filmstrip_scroll_target: Some(0),
            show_settings: false,
            show_color_picker: false,
            color_picker_hex_input: String::new(),
            show_association_banner,
            associations: crate::system::assoc::OverallAssociationState::default(),
            assoc_rx: None,
            is_checking_associations: false,
            was_settings_opened_for_assoc: false,
            was_window_focused: true,
            settings,
            settings_writer: Some(crate::system::config::SettingsWriter::new()),
            wheel_nav_accumulator: 0.0,
            format_search: String::new(),
            batch_files: Vec::new(),
            batch_kind: visor::ExportKind::Png,
            thumbs: HashMap::new(),
            failed_thumbs: std::collections::HashSet::new(),
            thumb_rx: None,
            thumb_cancel: None,
            notice_tx,
            notice_rx,
            notice: None,
            clipboard_rx: None,
            clipboard_token: 0,
            shell_sta,
            trash_rx: None,
            deleting_path: None,
            updater: crate::updater::UpdaterState::default(),
            updater_rx: Some(updater_rx),
            updater_tx,
            capture_state: None,
            capture_wait: 0,
            capture_dir: None,
        };
        app.trigger_association_check();
        app.check_post_update_whats_new();

        // Se uma versÃ£o desatualizada jÃ¡ foi identificada anteriormente, restaura a notificaÃ§Ã£o imediatamente
        if let Some(tag) = &app.settings.updates.last_known_release_tag {
            if crate::updater::version::is_newer_than_current(tag) {
                if let Some(json_str) = &app.settings.updates.last_known_release_json {
                    if let Ok(release) =
                        serde_json::from_str::<crate::updater::GitHubRelease>(json_str)
                    {
                        let is_ignored = app
                            .settings
                            .updates
                            .ignored_version
                            .as_deref()
                            .map(|ign| {
                                crate::updater::version::clean_version_str(ign)
                                    == crate::updater::version::clean_version_str(&release.tag_name)
                            })
                            .unwrap_or(false);

                        app.updater.available_release = Some(release);
                        if !is_ignored {
                            app.updater.show_update_window = true;
                        }
                    }
                }
            }
        }
        app.trigger_background_update_check_if_needed();
        if let Some(path) = initial {
            app.open_input(path);
        }
        app
    }

    pub fn open_input(&mut self, path: PathBuf) {
        let path = normalize_path(&path);
        if path.is_dir() {
            let files = list_images(&path);
            if files.is_empty() {
                self.error =
                    Some("Essa pasta não tem imagens com decodificador neste programa.".to_owned());
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

    pub fn set_files(&mut self, files: Vec<PathBuf>, index: usize) {
        if let Some(t) = self.thumb_cancel.take() {
            t.cancel();
        }
        self.files = files;
        self.index = index;
        self.filmstrip_scroll_target = Some(index);
        self.list_token = self.list_token.wrapping_add(1);
        self.thumbs.clear();
        self.failed_thumbs.clear();
        self.thumb_rx = None;
    }

    pub fn request(&mut self, path: PathBuf) {
        // Cancela cooperativamente a tarefa anterior se ainda estiver rodando
        if let Some(job) = &self.job {
            job.cancel_token.cancel();
        }
        self.generation = self.generation.wrapping_add(1);
        self.queued = Some((self.generation, path));
        self.loading = true;
        self.error = None;
    }

    pub fn step(&mut self, delta: isize) {
        let len = self.files.len() as isize;
        if len < 2 {
            return;
        }

        let next = ((self.index as isize + delta).rem_euclid(len)) as usize;
        self.index = next;
        self.filmstrip_scroll_target = Some(next);
        if let Some(path) = self.files.get(next).cloned() {
            self.request(path);
        }
    }

    pub fn go_to(&mut self, index: usize) {
        if self.files.is_empty() || index >= self.files.len() || index == self.index {
            return;
        }
        self.index = index;
        self.filmstrip_scroll_target = Some(index);
        self.request(self.files[index].clone());
    }

    pub fn pump(&mut self, ctx: &Context) {
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
                    Ok(decoded) => {
                        self.install(ctx, decoded);
                    }
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

        if let Some(rx) = &self.clipboard_rx {
            match rx.try_recv() {
                Ok(msg) => {
                    self.clipboard_rx = None;
                    if msg.token == self.clipboard_token {
                        if msg.png.is_none() && msg.dib.is_none() {
                            let err_msg = msg
                                .error
                                .unwrap_or_else(|| "falha na preparação dos dados".to_owned());
                            self.error =
                                Some(format!("Não foi possível copiar a imagem: {err_msg}"));
                        } else {
                            let hwnd = get_valid_window_handle();
                            if let Some(h) = hwnd {
                                match publish_to_clipboard(
                                    h,
                                    msg.png.as_deref(),
                                    msg.dib.as_deref(),
                                ) {
                                    ClipboardPublishResult::Success => {
                                        self.notice = Some(
                                            "Imagem copiada para a área de transferência."
                                                .to_owned(),
                                        );
                                    }
                                    ClipboardPublishResult::PartialPngOnly => {
                                        self.notice = Some(
                                            "Imagem copiada para a área de transferência (apenas PNG)."
                                                .to_owned(),
                                        );
                                    }
                                    ClipboardPublishResult::PartialDibOnly => {
                                        self.notice = Some(
                                            "Imagem copiada para a área de transferência (apenas DIB)."
                                                .to_owned(),
                                        );
                                    }
                                    ClipboardPublishResult::Failed(err) => {
                                        self.error = Some(format!(
                                            "Não foi possível copiar a imagem: {err}"
                                        ));
                                    }
                                }
                            } else {
                                self.error = Some(
                                    "Não foi possível copiar a imagem: janela ativa não encontrada."
                                        .to_owned(),
                                );
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(16));
                }
                Err(TryRecvError::Disconnected) => {
                    self.clipboard_rx = None;
                }
            }
        }

        if let Some(rx) = &self.trash_rx {
            match rx.try_recv() {
                Ok(msg) => {
                    self.trash_rx = None;
                    let _generation = msg.generation;
                    if self.deleting_path.as_ref() == Some(&msg.path) {
                        self.deleting_path = None;
                    }
                    match msg.result {
                        TrashResult::Success => {
                            self.cache.invalidate_path(&msg.path);
                            let current_viewed = self.files.get(self.index).cloned();
                            let removed_pos = self.files.iter().position(|p| p == &msg.path);
                            if let Some(idx) = removed_pos {
                                self.files.remove(idx);
                                if let Some(cancel) = &self.thumb_cancel {
                                    cancel.cancel();
                                }
                                self.thumb_rx = None;
                                self.thumb_cancel = None;
                                self.thumbs.clear();
                                self.failed_thumbs.clear();
                                self.list_token = self.list_token.wrapping_add(1);

                                let file_name = msg
                                    .path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "Imagem".to_owned());
                                self.notice =
                                    Some(format!("\"{file_name}\" enviada para a Lixeira."));

                                if self.files.is_empty() {
                                    self.picture = None;
                                    self.shown_percent = None;
                                    self.index = 0;
                                    self.error = None;
                                    self.loading = false;
                                    self.sync_title(ctx);
                                } else if removed_pos == Some(self.index)
                                    || current_viewed.as_ref() == Some(&msg.path)
                                {
                                    self.index = self.index.min(self.files.len() - 1);
                                    self.filmstrip_scroll_target = Some(self.index);
                                    let next_path = self.files[self.index].clone();
                                    self.request(next_path);
                                } else if let Some(viewed) = current_viewed {
                                    if let Some(new_idx) =
                                        self.files.iter().position(|p| p == &viewed)
                                    {
                                        self.index = new_idx;
                                    } else {
                                        self.index = self.index.min(self.files.len() - 1);
                                    }
                                }
                            } else {
                                self.notice = Some("Imagem enviada para a Lixeira.".to_owned());
                            }
                            ctx.request_repaint();
                        }
                        TrashResult::AbortedByUser => {
                            self.notice = Some("Operação cancelada.".to_owned());
                            ctx.request_repaint();
                        }
                        TrashResult::Failed(err) => {
                            self.error = Some(format!(
                                "Não foi possível enviar a imagem para a Lixeira: {err}"
                            ));
                            ctx.request_repaint();
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(16));
                }
                Err(TryRecvError::Disconnected) => {
                    self.trash_rx = None;
                    self.deleting_path = None;
                }
            }
        }

        if let Some(rx) = &self.assoc_rx {
            match rx.try_recv() {
                Ok(state) => {
                    self.associations = state;
                    self.is_checking_associations = false;
                    self.assoc_rx = None;
                    self.recalculate_association_banner();
                    ctx.request_repaint();
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.is_checking_associations = false;
                    self.assoc_rx = None;
                }
            }
        }

        let mut update_events = Vec::new();
        if let Some(rx) = &self.updater_rx {
            while let Ok(event) = rx.try_recv() {
                update_events.push(event);
            }
        }
        for event in update_events {
            self.handle_update_event(ctx, event);
        }

        poll_thumbs(self, ctx);
    }

    pub fn spawn(&mut self, ctx: &Context, token: u64, path: PathBuf) {
        // Consulta o cache antes de iniciar uma nova decodificação (Zero-Copy: compartilha Arc)
        if let Some(cached) = self.cache.get_by_path(&path) {
            self.job = None;
            self.loading = false;
            self.install(ctx, cached);
            return;
        }

        let (tx, rx) = mpsc::channel();
        let cancel_token = CancellationToken::new();
        let task_cancel = cancel_token.clone();

        self.job = Some(Inflight {
            token,
            cancel_token,
            rx,
        });
        self.loading = true;
        ctx.request_repaint();

        let cache = self.cache.clone();
        let path_clone = path.clone();

        let accepted = self.pool.spawn_with_priority(
            Priority::Viewport,
            Some(task_cancel.clone()),
            move || {
                if task_cancel.is_cancelled() {
                    return;
                }
                // Consulta o cache em background caso tenha sido inserido por outra tarefa
                if let Some(cached) = cache.get_by_path(&path_clone) {
                    if !task_cancel.is_cancelled() {
                        let _ = tx.send(LoadMsg {
                            token,
                            result: Ok(cached),
                        });
                    }
                    return;
                }

                let result =
                    std::panic::catch_unwind(AssertUnwindSafe(|| visor::open_image(&path_clone)));
                if task_cancel.is_cancelled() {
                    return;
                }
                let result = match result {
                    Ok(Ok(decoded)) => {
                        let arc = Arc::new(decoded);
                        if !task_cancel.is_cancelled() {
                            if let Ok(key) = CacheKey::from_path(&path_clone) {
                                cache.insert_arc(key, Arc::clone(&arc));
                            }
                        }
                        Ok(arc)
                    }
                    Ok(Err(err)) => Err(err),
                    Err(_) => Err("Esse arquivo não pôde ser lido.".to_owned()),
                };

                if !task_cancel.is_cancelled() {
                    let _ = tx.send(LoadMsg { token, result });
                }
            },
        );

        if !accepted {
            self.job = None;
            self.loading = false;
            self.error = Some("Não foi possível iniciar a leitura.".to_owned());
        }
    }

    pub fn install(&mut self, ctx: &Context, decoded: Arc<visor::Decoded>) {
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
        let initial_options = self.interpolation.resolve_options(1.0);
        let texture = ctx.load_texture(
            format!("picture-{}", self.generation),
            color,
            initial_options,
        );
        let playing = decoded.frames.len() > 1;
        self.picture = Some(Picture {
            texture,
            decoded,
            frame_i: 0,
            playing,
            shown_at: now,
            texture_options: initial_options,
        });
        self.view = match self.settings.default_zoom {
            crate::system::config::DefaultZoomMode::Actual100 => {
                View::actual(ctx.pixels_per_point())
            }
            crate::system::config::DefaultZoomMode::Fit => View::fit(),
        };
        self.error = None;
        self.prefetch_adjacent();
    }

    /// Dispara o pré-carregamento assíncrono em background das imagens adjacentes (próxima e anterior).
    ///
    /// Utiliza `Priority::Prefetch` (prioridade mínima) para nunca interferir na renderização
    /// ou no carregamento da imagem ativa. As imagens decodificadas são inseridas no `ImageCache` LRU,
    /// garantindo que a navegação com as setas do teclado ocorra instantaneamente (0 ms de espera).
    pub fn prefetch_adjacent(&self) {
        if self.files.len() <= 1 {
            return;
        }

        let len = self.files.len();
        let next_idx = (self.index + 1) % len;
        let prev_idx = self.index.checked_sub(1).unwrap_or(len - 1);

        for &target_idx in &[next_idx, prev_idx] {
            if target_idx == self.index || target_idx >= len {
                continue;
            }
            let target_path = self.files[target_idx].clone();

            // Evita enfileirar caso já esteja no cache de memória
            if self.cache.get_by_path(&target_path).is_some() {
                continue;
            }

            let cache = self.cache.clone();
            let _ = self
                .pool
                .spawn_with_priority(Priority::Prefetch, None, move || {
                    if cache.get_by_path(&target_path).is_some() {
                        return;
                    }
                    if let Ok(Ok(decoded)) = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        visor::open_image(&target_path)
                    })) {
                        let arc = Arc::new(decoded);
                        if let Ok(key) = CacheKey::from_path(&target_path) {
                            cache.insert_arc(key, arc);
                        }
                    }
                });
        }
    }

    pub fn upload_frame(&mut self, ctx: &Context) {
        let Some(picture) = self.picture.as_mut() else {
            return;
        };
        let frame = picture.frame();
        let color = ColorImage::from_rgba_unmultiplied(
            [frame.width as usize, frame.height as usize],
            &frame.rgba,
        );
        picture.texture.set(color, picture.texture_options);
        picture.shown_at = ctx.input(|input| input.time);
    }

    pub fn advance_animation(&mut self, ctx: &Context) {
        let Some(picture) = self.picture.as_ref() else {
            return;
        };
        if !picture.playing || picture.frames.len() < 2 {
            return;
        }
        let now = ctx.input(|input| input.time);
        let delay = picture.frames[picture.frame_i].delay_ms.max(1) as f64 / 1000.0;
        let elapsed = now - picture.shown_at;
        if elapsed < delay {
            let remaining = (delay - elapsed).max(0.005);
            ctx.request_repaint_after(Duration::from_secs_f64(remaining));
            return;
        }
        if let Some(picture) = self.picture.as_mut() {
            picture.frame_i = (picture.frame_i + 1) % picture.frames.len();
        }
        self.upload_frame(ctx);
        if let Some(picture) = self.picture.as_ref() {
            let next_delay = picture.frames[picture.frame_i].delay_ms.max(1) as f64 / 1000.0;
            ctx.request_repaint_after(Duration::from_secs_f64(next_delay));
        }
    }

    pub fn rotate(&mut self, ctx: &Context, clockwise: bool) {
        let Some(picture) = self.picture.as_mut() else {
            return;
        };
        // Isolamento Copy-on-Write: separa do cache apenas se o Arc for compartilhado
        let decoded = Arc::make_mut(&mut picture.decoded);
        for frame in &mut decoded.frames {
            let (pixels, width, height) =
                rotate_pixels(&frame.rgba, frame.width, frame.height, clockwise);
            frame.rgba = pixels;
            frame.width = width;
            frame.height = height;
        }
        self.view.offset = Vec2::ZERO;
        self.upload_frame(ctx);
    }

    pub fn copy_current_image(&mut self, ctx: &Context) {
        let Some(picture) = &self.picture else {
            return;
        };

        let decoded = Arc::clone(&picture.decoded);
        let frame_i = picture.frame_i;

        self.clipboard_token = self.clipboard_token.wrapping_add(1);
        let token = self.clipboard_token;

        let (tx, rx) = mpsc::channel();
        self.clipboard_rx = Some(rx);
        self.notice = Some("Copiando imagem para a área de transferência…".to_owned());
        ctx.request_repaint();

        self.pool
            .spawn_with_priority(Priority::Viewport, None, move || {
                if frame_i >= decoded.frames.len() {
                    let _ = tx.send(ClipboardMsg {
                        token,
                        png: None,
                        dib: None,
                        error: Some("quadro não encontrado".to_owned()),
                    });
                    return;
                }
                let frame = &decoded.frames[frame_i];
                let png_res = prepare_png(&frame.rgba, frame.width, frame.height);
                let dib_res = prepare_dibv5(&frame.rgba, frame.width, frame.height);

                let (png, dib, error) = match (png_res, dib_res) {
                    (Ok(p), Ok(d)) => (Some(p), Some(d), None),
                    (Ok(p), Err(e)) => (Some(p), None, Some(format!("DIB falhou: {e}"))),
                    (Err(e), Ok(d)) => (None, Some(d), Some(format!("PNG falhou: {e}"))),
                    (Err(e1), Err(e2)) => (None, None, Some(format!("PNG: {e1}; DIB: {e2}"))),
                };
                let _ = tx.send(ClipboardMsg {
                    token,
                    png,
                    dib,
                    error,
                });
            });
    }

    pub fn delete_image_at_path(&mut self, path: PathBuf, ctx: &Context) {
        if self.deleting_path.as_ref() == Some(&path) {
            return;
        }
        self.deleting_path = Some(path.clone());

        self.notice = Some("Enviando imagem para a Lixeira…".to_owned());
        ctx.request_repaint();

        let (tx, rx) = mpsc::channel();
        self.trash_rx = Some(rx);

        // Captura o HWND real da janela do VISOR na thread da interface gráfica
        let hwnd_raw = crate::system::trash::get_visor_main_window_handle().map(|h| h as usize);
        let generation = self.generation;
        let path_clone = path.clone();

        if let Some(sta) = self.shell_sta.clone() {
            let spawn_res = sta.spawn(move || {
                let hwnd = hwnd_raw.map(|h| h as *mut std::ffi::c_void);
                let res = send_to_trash(&path_clone, hwnd);
                let _ = tx.send(TrashMsg {
                    path: path_clone,
                    generation,
                    result: res,
                });
            });

            if let Err(err) = spawn_res {
                self.trash_rx = None;
                self.deleting_path = None;
                self.error = Some(format!("Falha ao despachar exclusão para a Lixeira: {err}"));
            }
        } else {
            // Se o executor STA não estiver disponível (ex: falha de modo COM ou restrição de sistema),
            // executa fallback controlado e estritamente seguro para SHFileOperationW no ThreadPool.
            // NUNCA recorre à exclusão permanente.
            self.pool
                .spawn_with_priority(Priority::Viewport, None, move || {
                    let hwnd = hwnd_raw.map(|h| h as *mut std::ffi::c_void);
                    let clean = match crate::system::trash::validate_file_path(&path_clone) {
                        Ok(p) => p,
                        Err(res) => {
                            let _ = tx.send(TrashMsg {
                                path: path_clone,
                                generation,
                                result: res,
                            });
                            return;
                        }
                    };
                    let res = unsafe { crate::system::trash::delete_via_shfileop(&clean, hwnd) };
                    let _ = tx.send(TrashMsg {
                        path: path_clone,
                        generation,
                        result: res,
                    });
                });
        }
    }

    pub fn delete_current_image(&mut self, ctx: &Context) {
        if let Some(path) = self.current_path().map(Path::to_path_buf) {
            self.delete_image_at_path(path, ctx);
        }
    }

    pub fn reveal_in_explorer(&mut self, target: Option<&Path>) -> Option<PathBuf> {
        let path = match target {
            Some(p) => Some(p.to_path_buf()),
            None => self.current_path().map(Path::to_path_buf),
        };
        let Some(path) = path else {
            return None;
        };

        if !path.exists() {
            self.error = Some(format!("Arquivo não encontrado: {}", path.display()));
            return Some(path);
        }

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Arquivo".to_owned());
        self.notice = Some(format!(
            "\"{file_name}\" revelado no Explorador de Arquivos."
        ));

        if cfg!(test) || std::env::var_os("VISOR_TEST_MODE").is_some() {
            return Some(path);
        }

        let path_clone = path.clone();
        if let Some(sta) = self.shell_sta.clone() {
            let _ = sta.spawn(move || {
                let _ = crate::system::reveal_in_explorer(&path_clone);
            });
        } else {
            std::thread::Builder::new()
                .name("visor-explorer".to_owned())
                .spawn(move || {
                    let _ = crate::system::reveal_in_explorer(&path_clone);
                })
                .ok();
        }

        Some(path)
    }

    pub fn open_with_dialog(&mut self, target: Option<&Path>) -> Option<PathBuf> {
        let path = match target {
            Some(p) => Some(p.to_path_buf()),
            None => self.current_path().map(Path::to_path_buf),
        };
        let Some(path) = path else {
            return None;
        };

        if !path.exists() {
            self.error = Some(format!("Arquivo não encontrado: {}", path.display()));
            return Some(path);
        }

        // Em testes automatizados, registra o alvo validado sem abrir janela modal
        // que bloquearia o runner e travaria arquivos temporários
        if cfg!(test) || std::env::var_os("VISOR_TEST_MODE").is_some() {
            self.notice = Some(format!("Abrir com: {}", path.display()));
            return Some(path);
        }

        let hwnd_raw = crate::system::trash::get_visor_main_window_handle().map(|h| h as usize);
        let path_clone = path.clone();

        // O diálogo nativo 'Abrir com' é modal e interativo com o usuário.
        // Executá-lo em uma thread STA independente dedicada (com ScopedComSta)
        // garante que o ShellStaExecutor permaneça 100% desimpedido para operações
        // concorrentes de exclusão e IFileOperation.
        std::thread::Builder::new()
            .name("visor-open-with".to_owned())
            .spawn(move || {
                let hwnd = hwnd_raw.map(|h| h as *mut std::ffi::c_void);
                let _ = crate::system::open_with_dialog(&path_clone, hwnd);
            })
            .ok();

        Some(path)
    }

    pub fn copy_path(&mut self, ctx: &Context, target: Option<&Path>) {
        let path = match target {
            Some(p) => Some(p.to_path_buf()),
            None => self.current_path().map(Path::to_path_buf),
        };
        if let Some(path) = path {
            let abs = std::path::absolute(&path).unwrap_or(path);
            let clean = abs
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string();
            ctx.copy_text(clean);
            self.notice = Some("Caminho copiado para a área de transferência.".to_owned());
        }
    }

    pub fn browse(&mut self, frame: &eframe::Frame) {
        let verified = visor::verified_extensions();
        let partial = visor::partial_extensions();
        let external = visor::external_extensions();

        let mut all_supported = Vec::with_capacity(verified.len() + partial.len() + external.len());
        all_supported.extend_from_slice(&verified);
        all_supported.extend_from_slice(&partial);
        if visor::magick_available() {
            all_supported.extend_from_slice(&external);
        }
        all_supported.sort_unstable();
        all_supported.dedup();

        let mut dialog = rfd::FileDialog::new()
            .set_title("Abrir Imagem")
            .set_parent(frame)
            .add_filter("Todos os formatos suportados", &all_supported)
            .add_filter("Suporte completo ou somente leitura", &verified)
            .add_filter("Suporte parcial (RAW, DDS, etc.)", &partial);
        if visor::magick_available() {
            dialog = dialog.add_filter("Conversor externo (ImageMagick)", &external);
        }
        dialog = dialog.add_filter("Todos os arquivos (*.*)", &["*"]);
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(path) = dialog.pick_file() {
            self.open_input(path);
        }
    }

    pub fn export_current(&mut self, frame: &eframe::Frame) {
        let Some(src) = self.current_path().map(Path::to_path_buf) else {
            return;
        };
        let tr = *self.tr();
        let stem = src
            .file_stem()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "imagem".to_owned());
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr.export_btn)
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
        let cache = self.cache.clone();
        let dst_clone = dst.clone();
        self.notice = Some("Exportando…".to_owned());
        std::thread::spawn(move || {
            let text = match visor::export_image(&src, &dst, kind) {
                Ok(()) => {
                    cache.invalidate_path(&dst_clone);
                    format!("Exportado: {}", dst.display())
                }
                Err(err) => format!("Exportação falhou: {err}"),
            };
            let _ = tx.send(text);
        });
    }

    pub fn pick_batch_files(&mut self, frame: &eframe::Frame) {
        let verified = visor::verified_extensions();
        let partial = visor::partial_extensions();
        let external = visor::external_extensions();

        let mut all_supported = Vec::with_capacity(verified.len() + partial.len() + external.len());
        all_supported.extend_from_slice(&verified);
        all_supported.extend_from_slice(&partial);
        if visor::magick_available() {
            all_supported.extend_from_slice(&external);
        }
        all_supported.sort_unstable();
        all_supported.dedup();

        let tr = *self.tr();
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr.batch_export)
            .set_parent(frame)
            .add_filter(tr.dialog_formats_subtitle, &all_supported)
            .add_filter(tr.badge_full, &verified)
            .add_filter(tr.badge_partial, &partial);
        if visor::magick_available() {
            dialog = dialog.add_filter(tr.badge_external, &external);
        }
        dialog = dialog.add_filter("*.*", &["*"]);
        if let Some(dir) = &self.last_dir {
            dialog = dialog.set_directory(dir);
        }
        if let Some(files) = dialog.pick_files() {
            self.batch_files = files;
            self.show_batch = true;
        }
    }

    pub fn run_batch(&mut self, frame: &eframe::Frame) {
        let tr = *self.tr();
        if self.batch_files.is_empty() {
            self.notice = Some(tr.batch_choose_files.to_owned());
            return;
        }
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr.batch_dest_folder)
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
        self.notice = Some(tr.batch_converting.to_owned());
        let results_template = tr.batch_results.to_owned();
        std::thread::spawn(move || {
            let results = visor::export_many(&files, &dir, kind);
            let ok = results.iter().filter(|(_, result)| result.is_ok()).count();
            let failed: Vec<_> = results
                .iter()
                .filter_map(|(_, result)| result.as_ref().err())
                .take(2)
                .cloned()
                .collect();
            let mut text = results_template
                .replace("{ok}", &ok.to_string())
                .replace("{total}", &results.len().to_string());
            if !failed.is_empty() {
                text.push_str(". ");
                text.push_str(&failed.join(" "));
            }
            let _ = tx.send(text);
        });
    }

    pub fn define_default(&mut self) {
        self.was_settings_opened_for_assoc = true;
        let tr = *self.tr();
        let tx = self.notice_tx.clone();
        self.notice = Some(tr.settings_refresh_assoc_tooltip.to_owned());
        let success_notice = tr.set_default_notice.to_owned();
        std::thread::spawn(move || {
            let _ = crate::system::assoc::register_current_exe();
            match crate::system::assoc::open_default_apps_settings() {
                Ok(()) => {
                    let _ = tx.send(success_notice);
                }
                Err(err) => {
                    let _ = tx.send(format!("Error: {err}"));
                }
            }
        });
    }

    /// Dispara a verificação assíncrona das associações de arquivo no Windows sem bloquear a UI.
    pub fn trigger_association_check(&mut self) {
        if self.is_checking_associations {
            return;
        }
        self.is_checking_associations = true;
        let (tx, rx) = mpsc::channel();
        self.assoc_rx = Some(rx);
        let ignored = self.settings.ignored_default_extensions.clone();

        self.pool
            .spawn_with_priority(crate::workers::Priority::Prefetch, None, move || {
                let state = crate::system::assoc::check_all_associations(&ignored);
                let _ = tx.send(state);
            });
    }

    /// Ignora permanentemente uma extensão para não perguntar novamente.
    pub fn ignore_extension(&mut self, ext: &str) {
        let clean = crate::system::assoc::normalize_extension(ext);
        self.settings.ignore_extension(&clean);
        self.save_settings();

        if let Some(item) = self
            .associations
            .assessments
            .iter_mut()
            .find(|a| a.extension == clean)
        {
            if item.status == crate::system::assoc::ExtensionStatus::Pending {
                item.status = crate::system::assoc::ExtensionStatus::Ignored;
                self.associations.pending_count = self.associations.pending_count.saturating_sub(1);
                self.associations.ignored_count += 1;
            }
        }
        self.recalculate_association_banner();
    }

    /// Desfaz o ignoramento de uma extensão ("Voltar a perguntar").
    pub fn unignore_extension(&mut self, ext: &str) {
        let clean = crate::system::assoc::normalize_extension(ext);
        self.settings.unignore_extension(&clean);
        self.save_settings();
        self.trigger_association_check();
    }

    /// Recalcula a visibilidade do banner baseado nas pendências e estado da verificação.
    pub fn recalculate_association_banner(&mut self) {
        if self.is_checking_associations {
            self.show_association_banner = false;
        } else {
            self.show_association_banner = self.associations.has_pending();
        }
    }

    pub fn save_settings(&mut self) {
        self.settings.interpolation = self.interpolation;
        self.settings.background = self.background;
        if let super::state::CanvasBackground::Custom(c) = self.background {
            self.settings.custom_color = c.to_srgba_unmultiplied();
        }
        self.settings.show_filmstrip = self.show_filmstrip;
        self.settings.show_info = self.show_info;

        if let Some(writer) = &self.settings_writer {
            writer.schedule_save(self.settings.clone());
        } else {
            let settings = self.settings.clone();
            let tx = self.notice_tx.clone();
            std::thread::spawn(move || {
                if let Err(err) = settings.save() {
                    let _ = tx.send(format!("Erro ao salvar configurações: {err}"));
                }
            });
        }
    }

    #[allow(dead_code)]
    pub fn sync_and_save_settings_sync(&mut self) -> Result<(), String> {
        self.settings.interpolation = self.interpolation;
        self.settings.background = self.background;
        if let super::state::CanvasBackground::Custom(c) = self.background {
            self.settings.custom_color = c.to_srgba_unmultiplied();
        }
        self.settings.show_filmstrip = self.show_filmstrip;
        self.settings.show_info = self.show_info;
        self.settings.save()
    }

    pub fn restore_default_settings(&mut self) {
        self.settings = crate::system::config::AppSettings::default();
        self.interpolation = self.settings.interpolation;
        self.background = self.settings.background;
        self.show_filmstrip = self.settings.show_filmstrip;
        self.show_info = self.settings.show_info;
        self.save_settings();
        self.trigger_association_check();
        self.notice = Some("Configurações restauradas para os padrões.".to_owned());
    }

    /// Abre a janela interativa de seleção de cor de fundo personalizada, aplicando o modo imediatamente.
    pub fn open_color_picker(&mut self) {
        let [r, g, b, a] = self.settings.custom_color;
        let color = eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a);
        self.background = super::state::CanvasBackground::Custom(color);
        self.settings.background = super::state::CanvasBackground::Custom(color);
        if a < 255 {
            self.color_picker_hex_input = format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a);
        } else {
            self.color_picker_hex_input = format!("#{:02X}{:02X}{:02X}", r, g, b);
        }
        self.show_color_picker = true;
        self.save_settings();
    }

    pub fn take_drops(&mut self, ctx: &Context) {
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

    pub fn current_path(&self) -> Option<&Path> {
        self.files.get(self.index).map(PathBuf::as_path)
    }

    pub fn current_name(&self) -> Option<String> {
        self.current_path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
    }

    pub fn sync_title(&mut self, ctx: &Context) {
        let title = match self.current_name() {
            Some(name) => format!("{name} — Visor"),
            None if self.loading => format!("{} — Visor", self.tr().loading),
            None => "Visor".to_owned(),
        };
        if self.title != title {
            self.title.clone_from(&title);
            ctx.send_viewport_cmd(ViewportCommand::Title(title));
        }
    }

    // -------------------------------------------------------------------------
    // UPDATE CENTER & GERENCIAMENTO DE ATUALIZAÇÕES
    // -------------------------------------------------------------------------

    /// Verifica se as novidades pós-atualização devem ser apresentadas automaticamente.
    pub fn check_post_update_whats_new(&mut self) {
        let current_v = crate::updater::version::CURRENT_VERSION;
        let last_shown_v = self.settings.updates.post_update_notes_version.as_deref();

        if last_shown_v != Some(current_v) {
            // Nova versão detectada após atualização
            let notes = crate::updater::whats_new::get_current_version_notes();
            self.updater.whats_new_notes = Some(notes);
            self.updater.show_whats_new_window = true;

            // Marca a versão atual como apresentada para não reaparecer a cada abertura
            self.settings.updates.post_update_notes_version = Some(current_v.to_string());
            self.save_settings();
        }
    }

    /// Dispara a verificação automática de atualizações se o intervalo configurado tiver expirado.
    pub fn trigger_background_update_check_if_needed(&mut self) {
        if self.settings.updates.automatic_check {
            self.trigger_update_check(false);
        }
    }

    /// Inicia uma verificação de novas versões no GitHub.
    ///
    /// Se `is_manual` for verdadeiro, ignora ETag e limpa erros anteriores para feedback imediato.
    pub fn trigger_update_check(&mut self, is_manual: bool) {
        if self.updater.is_checking {
            return;
        }

        self.updater.is_checking = true;
        self.updater.last_error = None;

        let etag = if is_manual {
            None
        } else {
            self.settings.updates.last_etag.clone()
        };

        crate::updater::trigger_background_update_check(&self.pool, etag, self.updater_tx.clone());
    }

    /// Processa os eventos assíncronos recebidos das tarefas de atualização.
    pub fn handle_update_event(&mut self, ctx: &Context, event: crate::updater::UpdateEvent) {
        match event {
            crate::updater::UpdateEvent::CheckStarted => {
                self.updater.is_checking = true;
            }
            crate::updater::UpdateEvent::CheckFinished(result) => {
                self.updater.is_checking = false;
                match result {
                    Ok(outcome) => {
                        match &outcome {
                            crate::updater::CheckOutcome::NotModified { checked_at } => {
                                self.settings.updates.last_check = Some(checked_at.clone());
                                if self.updater.available_release.is_none() {
                                    if let Some(json_str) =
                                        &self.settings.updates.last_known_release_json
                                    {
                                        if let Ok(release) =
                                            serde_json::from_str::<crate::updater::GitHubRelease>(
                                                json_str,
                                            )
                                        {
                                            if crate::updater::version::is_newer_than_current(
                                                &release.tag_name,
                                            ) {
                                                let is_ignored = self.settings.updates.ignored_version.as_deref().map(|ign| {
                                                    crate::updater::version::clean_version_str(ign) == crate::updater::version::clean_version_str(&release.tag_name)
                                                }).unwrap_or(false);
                                                let is_postponed = self.updater.postponed_tag.as_deref().map(|p| {
                                                    crate::updater::version::clean_version_str(p) == crate::updater::version::clean_version_str(&release.tag_name)
                                                }).unwrap_or(false);
                                                self.updater.available_release = Some(release);
                                                if !is_ignored && !is_postponed {
                                                    self.updater.show_update_window = true;
                                                }
                                            }
                                        }
                                    }
                                }
                                self.save_settings();
                            }
                            crate::updater::CheckOutcome::UpToDate { checked_at } => {
                                self.settings.updates.last_check = Some(checked_at.clone());
                                self.updater.available_release = None;
                                self.settings.updates.last_known_release_tag = None;
                                self.settings.updates.last_known_release_json = None;
                                self.save_settings();
                            }
                            crate::updater::CheckOutcome::UpdateAvailable {
                                release,
                                checked_at,
                            } => {
                                self.settings.updates.last_check = Some(checked_at.clone());
                                self.settings.updates.last_known_release_tag =
                                    Some(release.tag_name.clone());
                                self.settings.updates.last_known_release_json =
                                    serde_json::to_string(release).ok();
                                self.save_settings();

                                let tag = &release.tag_name;
                                let is_ignored = self
                                    .settings
                                    .updates
                                    .ignored_version
                                    .as_deref()
                                    .map(|ign| {
                                        crate::updater::version::clean_version_str(ign)
                                            == crate::updater::version::clean_version_str(tag)
                                    })
                                    .unwrap_or(false);

                                let is_postponed = self
                                    .updater
                                    .postponed_tag
                                    .as_deref()
                                    .map(|post| {
                                        crate::updater::version::clean_version_str(post)
                                            == crate::updater::version::clean_version_str(tag)
                                    })
                                    .unwrap_or(false);

                                self.updater.available_release = Some(release.clone());

                                if !is_ignored && !is_postponed {
                                    self.updater.show_update_window = true;
                                }
                            }
                        }
                        self.updater.last_outcome = Some(outcome);
                    }
                    Err(err) => {
                        self.updater.last_error = Some(err.to_string());
                    }
                }
                ctx.request_repaint();
            }
            crate::updater::UpdateEvent::DownloadProgress(progress) => {
                self.updater.download_progress = Some(progress);
                ctx.request_repaint();
            }
            crate::updater::UpdateEvent::DownloadCompleted(path) => {
                if let Some(prog) = &mut self.updater.download_progress {
                    prog.status = crate::updater::DownloadStatus::Completed(path);
                }
                ctx.request_repaint();
            }
            crate::updater::UpdateEvent::DownloadFailed(err) => {
                if let Some(prog) = &mut self.updater.download_progress {
                    prog.status = crate::updater::DownloadStatus::Failed(err);
                }
                ctx.request_repaint();
            }
        }
    }

    /// Ignora uma versão específica para nunca mais notificar sobre ela.
    pub fn ignore_update_version(&mut self, version: &str) {
        self.settings.updates.ignored_version = Some(version.to_string());
        self.updater.show_update_window = false;
        self.save_settings();
    }

    /// Adia a notificação da release atual para a próxima sessão ("Lembrar mais tarde").
    pub fn postpone_update(&mut self, version: &str) {
        self.updater.postponed_tag = Some(version.to_string());
        self.updater.show_update_window = false;
    }

    /// Determina se o banner de atualizaÃ§Ã£o pendente deve estar visÃ­vel no topo da interface.
    pub fn has_active_update_banner(&self) -> bool {
        let Some(release) = &self.updater.available_release else {
            return false;
        };
        let tag = &release.tag_name;
        let is_ignored = self
            .settings
            .updates
            .ignored_version
            .as_deref()
            .map(|ign| {
                crate::updater::version::clean_version_str(ign)
                    == crate::updater::version::clean_version_str(tag)
            })
            .unwrap_or(false);
        let is_postponed = self
            .updater
            .postponed_tag
            .as_deref()
            .map(|post| {
                crate::updater::version::clean_version_str(post)
                    == crate::updater::version::clean_version_str(tag)
            })
            .unwrap_or(false);
        !is_ignored && !is_postponed
    }

    /// Dispara o download de um pacote selecionado (instalador ou versão portátil).
    pub fn start_package_download(
        &mut self,
        version: String,
        package: crate::updater::GitHubAsset,
        manifest: Option<crate::updater::GitHubAsset>,
    ) {
        let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.updater.download_cancel_flag = Some(cancel_flag.clone());

        self.updater.download_progress = Some(crate::updater::DownloadProgress {
            version: version.clone(),
            filename: package.name.clone(),
            downloaded_bytes: 0,
            total_bytes: Some(package.size),
            percentage: 0.0,
            speed_bytes_per_sec: 0.0,
            eta_seconds: None,
            status: crate::updater::DownloadStatus::Preparing,
        });
        self.updater.show_update_window = true;

        crate::updater::trigger_background_download(
            &self.pool,
            version,
            package,
            manifest,
            self.updater_tx.clone(),
            cancel_flag,
        );
    }
}
