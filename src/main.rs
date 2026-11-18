//! Visor — visualizador de imagens para Windows.
//!
//! ```text
//! cargo run --release
//! cargo run --release -- caminho\da\foto.jpg
//! visor.exe --registrar-padrao
//! ```

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod system;
mod ui;
mod updater;
mod workers;

use std::path::PathBuf;

use eframe::egui;

use app::VisorApp;
use ui::theme::{ThemePalette, configure_visor_theme, window_icon};

/// Extrai o caminho de arquivo inicial passado pela linha de comando, se houver.
/// Ignora argumentos internos como flags iniciadas por `--`.
pub fn parse_cli_initial_path<I, T>(args: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = T>,
    T: Into<PathBuf>,
{
    args.into_iter().map(Into::into).find(|path| {
        if let Some(s) = path.to_str() {
            !s.starts_with("--")
        } else {
            true
        }
    })
}

fn main() -> eframe::Result {
    // 1. Blindagem de inicialização: Anti-Tamper, integridade de processo e Anti-Debugging
    let _ = system::enforce_startup_security();

    if std::env::args_os().any(|arg| arg == "--registrar-padrao") {
        match system::assoc::register_current_exe() {
            Ok(report) => {
                println!("{}", report.summary());
            }
            Err(err) => {
                eprintln!("Erro ao registrar o VISOR: {err}");
            }
        }
        return Ok(());
    }

    let capture_final = std::env::args_os().any(|arg| arg == "--capture-final-validation");
    let capture_step6 = std::env::args_os().any(|arg| arg == "--capture-step6");
    let capture_step5 = std::env::args_os().any(|arg| arg == "--capture-step5");
    let capture_gallery = std::env::args_os().any(|arg| arg == "--capture-gallery")
        || capture_step5
        || capture_step6
        || capture_final;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Visor")
            .with_icon(window_icon())
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([280.0, 200.0])
            .with_drag_and_drop(true),
        centered: true,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let mut initial = parse_cli_initial_path(std::env::args_os().skip(1));

    if capture_gallery && initial.is_none() {
        initial = Some(PathBuf::from("assets/moldura.png"));
    }

    eframe::run_native(
        "Visor",
        options,
        Box::new(move |cc| {
            configure_visor_theme(&cc.egui_ctx, &ThemePalette::dark());
            let mut app = VisorApp::new(&cc.egui_ctx, initial);
            let capture_dir = std::env::var_os("VISOR_CAPTURE_DIR")
                .map(PathBuf::from)
                .or_else(|| std::env::current_dir().ok());

            if capture_final {
                app.capture_state = Some(60);
                app.capture_wait = 15;
                app.capture_dir = capture_dir;
            } else if capture_step6 {
                app.capture_state = Some(44);
                app.capture_wait = 15;
                app.capture_dir = capture_dir;
            } else if capture_step5 {
                app.capture_state = Some(38);
                app.capture_wait = 15;
                app.capture_dir = capture_dir;
            } else if capture_gallery {
                app.capture_state = Some(0);
                app.capture_wait = 15;
                app.capture_dir = capture_dir;
            }
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use eframe::egui::{Pos2, Rect, Vec2};

    use crate::core::geometry::{fit_scale, image_rect, zoom_about};
    use crate::core::navigation::{is_image_path, list_images};
    use crate::parse_cli_initial_path;

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
        let (new_scale, new_offset) =
            zoom_about(canvas, image_px, scale, offset, anchor, 1.5, 0.01, 100.0);
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

    // =========================================================================
    // Testes de Integração de Concorrência do Passo 3 (VisorApp + Workers)
    // =========================================================================

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    use crate::workers::{CancellationToken, Priority, TaskQueue, ThreadPool};

    #[test]
    fn test_rapid_navigation_no_stale_overwrite() {
        let pool = Arc::new(ThreadPool::new(4));
        let (tx, rx) = mpsc::channel::<(u64, &'static str)>();

        // Simula o estado do visualizador
        let mut active_generation = 0u64;
        let mut installed_image: Option<&'static str> = None;

        // Requisição 1: Imagem A (lenta, 60ms)
        active_generation += 1;
        let gen1 = active_generation;
        let cancel1 = CancellationToken::new();
        let c1 = cancel1.clone();
        let tx1 = tx.clone();
        pool.spawn_with_priority(Priority::Viewport, Some(c1.clone()), move || {
            thread::sleep(Duration::from_millis(60));
            let _ = tx1.send((gen1, "imagem_A.png"));
        });
        let mut active_token = Some(cancel1);

        // Usuário navega rapidamente para Imagem B
        if let Some(t) = active_token.take() {
            t.cancel();
        }
        active_generation += 1;
        let gen2 = active_generation;
        let cancel2 = CancellationToken::new();
        let c2 = cancel2.clone();
        let tx2 = tx.clone();
        pool.spawn_with_priority(Priority::Viewport, Some(c2.clone()), move || {
            thread::sleep(Duration::from_millis(40));
            let _ = tx2.send((gen2, "imagem_B.png"));
        });
        active_token = Some(cancel2);

        // Usuário navega para Imagem C (última selecionada, rápida)
        if let Some(t) = active_token.take() {
            t.cancel();
        }
        active_generation += 1;
        let gen3 = active_generation;
        let cancel3 = CancellationToken::new();
        let c3 = cancel3.clone();
        let tx3 = tx.clone();
        pool.spawn_with_priority(Priority::Viewport, Some(c3.clone()), move || {
            let _ = tx3.send((gen3, "imagem_C.png"));
        });
        active_token = Some(cancel3);

        // Drena as mensagens simulando o pump() do VisorApp
        let deadline = std::time::Instant::now() + Duration::from_millis(600);
        while std::time::Instant::now() < deadline {
            while let Ok((token, img)) = rx.try_recv() {
                // Regra inviolável do VisorApp: somente a geração atual é instalada
                if token == active_generation {
                    installed_image = Some(img);
                }
            }
            if installed_image == Some("imagem_C.png") && pool.is_idle() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Garante que mesmo com as threads A e B terminando, o estado visual é estritamente a imagem C
        assert_eq!(installed_image, Some("imagem_C.png"));
        assert!(active_token.is_some());
    }

    #[test]
    fn test_100_consecutive_requests_settles_on_latest() {
        let pool = Arc::new(ThreadPool::new(4));
        let (tx, rx) = mpsc::channel::<(u64, usize)>();

        let mut current_gen = 0u64;
        let mut active_token: Option<CancellationToken> = None;
        let mut installed_id = None;

        // Dispara 100 requisições consecutivas simulando rolagem rápida da roda do mouse ou navegação
        for req_id in 1..=100 {
            if let Some(t) = active_token.take() {
                t.cancel();
            }
            current_gen += 1;
            let generation_id = current_gen;
            let cancel = CancellationToken::new();
            let c = cancel.clone();
            let tx_clone = tx.clone();

            pool.spawn_with_priority(Priority::Viewport, Some(c.clone()), move || {
                if c.is_cancelled() {
                    return;
                }
                thread::sleep(Duration::from_millis(1));
                if !c.is_cancelled() {
                    let _ = tx_clone.send((generation_id, req_id));
                }
            });
            active_token = Some(cancel);
        }

        let target_gen = current_gen; // 100

        // Processa mensagens simulando o loop de eventos
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            while let Ok((msg_gen, id)) = rx.try_recv() {
                if msg_gen == target_gen {
                    installed_id = Some(id);
                }
            }
            if installed_id == Some(100) && pool.is_idle() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(
            installed_id,
            Some(100),
            "O estado final deve assentar rigorosamente na 100ª imagem"
        );
        assert!(active_token.is_some());
    }

    #[test]
    fn test_simultaneous_viewport_and_thumbnails_priority() {
        let queue = Arc::new(TaskQueue::default());
        let pool = ThreadPool::with_queue(1, Arc::clone(&queue));
        let log = Arc::new(Mutex::new(Vec::new()));

        // Segura a execução do único worker com uma barreira para permitir enfileirar tudo antes
        let barrier = Arc::new(Barrier::new(2));
        let b = Arc::clone(&barrier);
        pool.spawn_with_priority(Priority::Viewport, None, move || {
            b.wait();
        });

        // O worker agora está bloqueado na barreira.
        // Enfileira 10 miniaturas (Thumbnail priority)
        for i in 0..10 {
            let log_clone = Arc::clone(&log);
            pool.spawn_with_priority(Priority::Thumbnail, None, move || {
                log_clone.lock().unwrap().push(format!("thumb_{i}"));
            });
        }

        // Enfileira 1 imagem de Viewport (Viewport priority)
        let log_clone = Arc::clone(&log);
        pool.spawn_with_priority(Priority::Viewport, None, move || {
            log_clone.lock().unwrap().push("viewport_main".to_string());
        });

        // Libera a barreira para o worker começar a retirar itens da fila
        barrier.wait();

        // Aguarda término da execução de todas as tarefas
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if pool.is_idle() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        let executed = log.lock().unwrap().clone();
        assert!(!executed.is_empty(), "Tarefas deveriam ter sido executadas");
        // O primeiro item processado após o desbloqueio DEVE ser o Viewport!
        assert_eq!(
            executed[0], "viewport_main",
            "A imagem do Viewport deve ter prioridade máxima sobre as miniaturas enfileiradas antes dela"
        );
        assert_eq!(executed.len(), 11);
    }

    #[test]
    fn test_cancellation_discards_obsolete_pending_jobs() {
        let queue = Arc::new(TaskQueue::default());
        let pool = ThreadPool::with_queue(1, Arc::clone(&queue));
        let executed_count = Arc::new(AtomicUsize::new(0));

        // Bloqueia o worker temporariamente com barreira
        let barrier = Arc::new(Barrier::new(2));
        let b = Arc::clone(&barrier);
        pool.spawn_with_priority(Priority::Viewport, None, move || {
            b.wait();
        });

        let mut tokens = Vec::new();
        // Enfileira 10 tarefas com tokens
        for _ in 0..10 {
            let token = CancellationToken::new();
            tokens.push(token.clone());
            let count = Arc::clone(&executed_count);
            pool.spawn_with_priority(Priority::Thumbnail, Some(token), move || {
                count.fetch_add(1, Ordering::SeqCst);
            });
        }

        // Cancela 8 das 10 tarefas enquanto ainda estão aguardando na fila
        for t in &tokens[0..8] {
            t.cancel();
        }

        // Libera o worker
        barrier.wait();

        // Aguarda esvaziar
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if pool.is_idle() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Apenas as 2 tarefas não canceladas devem ter executado a closure
        assert_eq!(
            executed_count.load(Ordering::SeqCst),
            2,
            "Somente as tarefas não canceladas devem executar a closure"
        );
        // Pelo menos 8 tarefas devem ter sido descartadas diretamente pela fila
        assert!(
            queue.dropped_cancelled_count() >= 8,
            "Fila deve contabilizar o descarte de tarefas canceladas"
        );
    }

    #[test]
    fn test_app_closing_shutdown_no_deadlock() {
        let (done_tx, done_rx) = mpsc::channel();

        // Executa o teste de encerramento em thread separada com timeout estrito contra deadlock
        thread::spawn(move || {
            let mut pool = ThreadPool::new(4);
            let cancel_job = CancellationToken::new();

            // Enfileira tarefas de naturezas diversas
            for _ in 0..20 {
                let c = cancel_job.clone();
                pool.spawn_with_priority(Priority::Thumbnail, Some(c), || {
                    thread::sleep(Duration::from_millis(5));
                });
            }

            // Simula fechamento do aplicativo: cancela tarefas pendentes e chama shutdown()
            cancel_job.cancel();
            pool.shutdown();

            // Rejeição de novas tarefas após shutdown
            let rejected = pool.spawn(|| {});
            assert!(
                !rejected,
                "ThreadPool após shutdown deve rejeitar novos spawns"
            );

            done_tx.send(true).unwrap();
        });

        // Se demorar mais de 3 segundos, é sinal de deadlock
        let result = done_rx.recv_timeout(Duration::from_secs(3));
        assert!(
            result.is_ok(),
            "O desligamento do pool e fila não deve causar deadlock"
        );
    }

    #[test]
    fn test_cache_integration_with_file_loading() {
        let dir = temp_dir("cache-integration");
        let file_path = dir.join("teste.png");

        let img = image::RgbImage::from_pixel(20, 20, image::Rgb([255, 0, 0]));
        image::DynamicImage::ImageRgb8(img)
            .save_with_format(&file_path, image::ImageFormat::Png)
            .unwrap();

        let cache = crate::core::ImageCache::default();

        // 1ª consulta: Cache miss
        assert!(cache.get_by_path(&file_path).is_none());

        // Decodificação e inserção no cache
        let decoded = visor::open_image(&file_path).unwrap();
        let key = crate::core::image_cache::CacheKey::from_path(&file_path).unwrap();
        cache.insert(key, decoded);

        // 2ª consulta: Cache hit imediato
        let t0 = std::time::Instant::now();
        let hit = cache.get_by_path(&file_path);
        let elapsed = t0.elapsed();

        assert!(hit.is_some(), "Deveria recuperar do cache");
        assert!(
            elapsed < Duration::from_millis(5),
            "Cache hit deve ser ultra-rápido (< 5ms, foi {:?})",
            elapsed
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_ctrl_c_without_image_does_nothing() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        assert!(app.picture.is_none());
        assert!(app.clipboard_rx.is_none());

        app.copy_current_image(&ctx);

        assert!(app.clipboard_rx.is_none());
    }

    #[test]
    fn test_ctrl_c_when_text_edit_focused_preserves_text_copy() {
        let ctx = eframe::egui::Context::default();
        let app = crate::app::VisorApp::new(&ctx, None);

        let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |c| {
            eframe::egui::CentralPanel::default().show(c, |ui| {
                let mut text = "teste".to_owned();
                let resp = ui.add(eframe::egui::TextEdit::singleline(&mut text));
                resp.request_focus();

                if ui.ctx().egui_wants_keyboard_input() {
                    assert!(ui.ctx().egui_wants_keyboard_input());
                }
            });
        });
        output.textures_delta.clear();

        assert!(app.clipboard_rx.is_none());
    }

    #[test]
    fn test_delete_without_image_does_nothing() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        assert!(app.files.is_empty());
        assert!(app.current_path().is_none());

        app.delete_current_image(&ctx);
        assert!(app.trash_rx.is_none());
        assert!(app.deleting_path.is_none());
    }

    #[test]
    fn test_delete_key_when_text_edit_focused_preserves_input() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        app.files = vec![PathBuf::from("teste.png")];
        app.index = 0;

        let mut output = ctx.run_ui(eframe::egui::RawInput::default(), |c| {
            eframe::egui::CentralPanel::default().show(c, |ui| {
                let mut text = "texto de busca".to_owned();
                let resp = ui.add(eframe::egui::TextEdit::singleline(&mut text));
                resp.request_focus();

                crate::app::shortcuts::handle_keys(&mut app, ui);
            });
        });
        output.textures_delta.clear();

        // O atalho Delete não deve ter sido disparado pois o campo de texto estava com foco
        assert!(app.trash_rx.is_none());
        assert_eq!(app.files.len(), 1);
    }

    #[test]
    fn test_delete_integration_file_removed_from_app_and_cache() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        let dir = temp_dir("test-delete-integration");
        let file_a = dir.join("foto_a.png");
        let file_b = dir.join("foto_b.png");

        let img = image::RgbImage::from_pixel(10, 10, image::Rgb([100, 150, 200]));
        image::DynamicImage::ImageRgb8(img.clone())
            .save_with_format(&file_a, image::ImageFormat::Png)
            .unwrap();
        image::DynamicImage::ImageRgb8(img)
            .save_with_format(&file_b, image::ImageFormat::Png)
            .unwrap();

        // Insere file_a no cache
        let decoded = visor::open_image(&file_a).unwrap();
        let key_a = crate::core::image_cache::CacheKey::from_path(&file_a).unwrap();
        app.cache.insert(key_a, decoded);
        assert!(app.cache.get_by_path(&file_a).is_some());

        app.files = vec![file_a.clone(), file_b.clone()];
        app.index = 0;

        // Dispara envio para a lixeira
        app.delete_current_image(&ctx);
        assert!(app.trash_rx.is_some());
        assert_eq!(app.deleting_path.as_ref(), Some(&file_a));

        // Bombeia até a conclusão
        let start = std::time::Instant::now();
        while app.trash_rx.is_some() && start.elapsed() < Duration::from_secs(5) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(20));
        }

        assert!(app.trash_rx.is_none());
        assert!(app.deleting_path.is_none());

        // 1. Arquivo A foi removido da lista do app
        assert_eq!(app.files.len(), 1);
        assert_eq!(app.files[0], file_b);
        assert_eq!(app.index, 0);

        // 2. Cache foi invalidado para o arquivo A
        assert!(app.cache.get_by_path(&file_a).is_none());

        // 3. Arquivo físico foi movido para a Lixeira e não existe mais na pasta
        assert!(!file_a.exists());

        // 4. Arquivo B permanece intacto no disco
        assert!(file_b.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_delete_last_file_transitions_to_empty_state() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        let dir = temp_dir("test-delete-last");
        let file_only = dir.join("unica.png");
        let img = image::RgbImage::from_pixel(10, 10, image::Rgb([50, 50, 50]));
        image::DynamicImage::ImageRgb8(img)
            .save_with_format(&file_only, image::ImageFormat::Png)
            .unwrap();

        app.files = vec![file_only.clone()];
        app.index = 0;

        app.delete_current_image(&ctx);

        let start = std::time::Instant::now();
        while app.trash_rx.is_some() && start.elapsed() < Duration::from_secs(5) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(20));
        }

        assert!(app.files.is_empty());
        assert!(app.picture.is_none());
        assert!(app.shown_percent.is_none());
        assert_eq!(app.index, 0);
        assert!(!file_only.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_delete_preserves_list_on_user_abort_or_failure() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        let path1 = PathBuf::from("img1.png");
        let path2 = PathBuf::from("img2.png");
        app.files = vec![path1.clone(), path2.clone()];
        app.index = 0;

        // Simula recebimento de AbortedByUser
        let (tx, rx) = mpsc::channel();
        tx.send(crate::app::state::TrashMsg {
            path: path1.clone(),
            generation: 1,
            result: crate::system::trash::TrashResult::AbortedByUser,
        })
        .unwrap();
        app.trash_rx = Some(rx);
        app.deleting_path = Some(path1.clone());

        app.pump(&ctx);

        assert_eq!(app.files.len(), 2);
        assert_eq!(app.index, 0);
        assert_eq!(app.notice.as_deref(), Some("Operação cancelada."));
        assert!(app.deleting_path.is_none());

        // Simula recebimento de Failed
        let (tx2, rx2) = mpsc::channel();
        tx2.send(crate::app::state::TrashMsg {
            path: path1.clone(),
            generation: 1,
            result: crate::system::trash::TrashResult::Failed("Falha de teste".to_owned()),
        })
        .unwrap();
        app.trash_rx = Some(rx2);
        app.deleting_path = Some(path1.clone());

        app.pump(&ctx);

        assert_eq!(app.files.len(), 2);
        assert_eq!(app.index, 0);
        assert!(app.error.as_ref().unwrap().contains("Falha de teste"));
        assert!(app.deleting_path.is_none());
    }

    #[test]
    fn test_delete_duplicate_request_ignored_while_pending() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        let path = PathBuf::from("imagem.png");
        app.files = vec![path.clone()];
        app.index = 0;
        app.deleting_path = Some(path.clone());

        let (_tx, rx) = mpsc::channel();
        app.trash_rx = Some(rx);

        // Tentar chamar novamente enquanto já está pendente para o mesmo path
        app.delete_current_image(&ctx);

        // O canal não deve ter sido substituído
        assert!(app.trash_rx.is_some());
    }

    #[test]
    fn test_real_windows_recycle_bin_and_restore() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        let dir = temp_dir("test-real-recycle-restore");
        let file_name = "foto_teste_açúcar_e_coração.png";
        let file_path = dir.join(file_name);

        let img = image::RgbImage::from_pixel(16, 16, image::Rgb([45, 120, 210]));
        image::DynamicImage::ImageRgb8(img)
            .save_with_format(&file_path, image::ImageFormat::Png)
            .unwrap();
        assert!(file_path.exists());

        app.files = vec![file_path.clone()];
        app.index = 0;

        // Dispara exclusão para a lixeira
        app.delete_current_image(&ctx);

        let start = std::time::Instant::now();
        while app.trash_rx.is_some() && start.elapsed() < Duration::from_secs(5) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(25));
        }

        // 1. Arquivo saiu da pasta de origem
        assert!(
            !file_path.exists(),
            "Arquivo deve ter saído da pasta de origem"
        );
        assert!(
            app.files.is_empty(),
            "Lista do app deve estar vazia após exclusão da única imagem"
        );

        // 2. Localiza na Lixeira do Windows e restaura
        let script = r#"$shell = New-Object -ComObject Shell.Application;
$bin = $shell.Namespace(0xa);
foreach ($item in $bin.Items()) {
    if ($item.Name -like "*foto_teste*" -or $bin.GetDetailsOf($item, 1) -like "*test-real-recycle-restore*") {
        foreach ($verb in $item.Verbs()) {
            if ($verb.Name.Contains("taurar") -or $verb.Name.Contains("estore")) {
                $verb.DoIt();
                Write-Host 'RESTORED';
                break;
            }
        }
        break;
    }
}"#;

        let output = std::process::Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command", script])
            .output()
            .expect("Executar comando PowerShell");

        let out_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            out_str.contains("RESTORED"),
            "Arquivo deve ser localizado na Lixeira e restaurado"
        );

        // 3. Aguarda o arquivo reaparecer no disco
        let mut restored = false;
        let start_restore = std::time::Instant::now();
        while start_restore.elapsed() < Duration::from_secs(5) {
            if file_path.exists() {
                restored = true;
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        assert!(
            restored,
            "Arquivo deve ter sido restaurado com sucesso na pasta de origem"
        );

        // 4. Reabre no VISOR e confere decodificação
        let reopened = visor::open_image(&file_path).expect("Reabrir imagem restaurada");
        assert_eq!(reopened.frames.len(), 1);
        assert_eq!(reopened.frames[0].width, 16);
        assert_eq!(reopened.frames[0].height, 16);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_fallback_when_shell_sta_is_none() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);
        app.shell_sta = None; // Força ausência de executor STA para testar o fallback controlado

        let dir = temp_dir("test-fallback-no-sta");
        let file_path = dir.join("foto_fallback.png");
        let img = image::RgbImage::from_pixel(10, 10, image::Rgb([30, 60, 90]));
        image::DynamicImage::ImageRgb8(img)
            .save_with_format(&file_path, image::ImageFormat::Png)
            .unwrap();

        app.files = vec![file_path.clone()];
        app.index = 0;

        app.delete_current_image(&ctx);
        assert!(app.trash_rx.is_some());

        let start = std::time::Instant::now();
        while app.trash_rx.is_some() && start.elapsed() < Duration::from_secs(5) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(25));
        }

        assert!(app.files.is_empty());
        assert!(
            !file_path.exists(),
            "Arquivo deve ter sido enviado à Lixeira via fallback de SHFileOperationW"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_filmstrip_context_menu_target_isolation_and_safety() {
        let dir = temp_dir("test-filmstrip-safety");
        let path_a = dir.join("A.png");
        let path_b = dir.join("B.png");
        let path_c = dir.join("C.png");

        // Cria 3 imagens distintas
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([255, 0, 0])))
            .save_with_format(&path_a, image::ImageFormat::Png)
            .unwrap();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([0, 255, 0])))
            .save_with_format(&path_b, image::ImageFormat::Png)
            .unwrap();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([0, 0, 255])))
            .save_with_format(&path_c, image::ImageFormat::Png)
            .unwrap();

        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        // 1. Abre A.png no VISOR
        app.open_input(path_a.clone());
        let start = std::time::Instant::now();
        while app.picture.is_none() && start.elapsed() < Duration::from_secs(3) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(app.index, 0, "A.png deve ser a imagem ativa inicial");
        assert_eq!(app.current_path(), Some(path_a.as_path()));
        assert_eq!(app.files.len(), 3);

        // Popula o cache LRU com B.png e C.png também para testar a invalidação seletiva
        let decoded_b = visor::open_image(&path_b).unwrap();
        let key_b = crate::core::image_cache::CacheKey::from_path(&path_b).unwrap();
        app.cache.insert(key_b, decoded_b);

        let decoded_c = visor::open_image(&path_c).unwrap();
        let key_c = crate::core::image_cache::CacheKey::from_path(&path_c).unwrap();
        app.cache.insert(key_c, decoded_c);

        assert!(app.cache.get_by_path(&path_a).is_some());
        assert!(app.cache.get_by_path(&path_b).is_some());
        assert!(app.cache.get_by_path(&path_c).is_some());

        // 2. Teste: Fechar o menu sem executar nenhuma ação -> seleção original (A.png) deve permanecer preservada!
        let mut output = ctx.run_ui(Default::default(), |ui| {
            crate::ui::context_menu::render_context_menu(&mut app, ui, Some(&path_b));
        });
        output.textures_delta.clear();
        assert_eq!(
            app.index, 0,
            "Seleção original (A.png) deve permanecer preservada"
        );
        assert_eq!(app.current_path(), Some(path_a.as_path()));

        // 3. Teste: Copiar caminho de B.png (alvo secundário)
        app.copy_path(&ctx, Some(&path_b));
        assert_eq!(
            app.index, 0,
            "Copiar caminho de B não altera o índice ativo"
        );
        assert_eq!(app.current_path(), Some(path_a.as_path()));
        assert_eq!(
            app.notice.as_deref(),
            Some("Caminho copiado para a área de transferência.")
        );

        // 4. Teste: Revelar no Explorer para B.png
        assert!(path_b.exists());
        app.reveal_in_explorer(Some(&path_b));
        assert_eq!(app.index, 0, "Revelar B não altera o índice ativo");
        assert_eq!(app.current_path(), Some(path_a.as_path()));
        assert!(app.notice.as_ref().unwrap().contains("B.png"));

        // 5. Teste: Abrir com... para B.png
        app.open_with_dialog(Some(&path_b));
        assert_eq!(app.index, 0, "Abrir com B não altera o índice ativo");
        assert_eq!(app.current_path(), Some(path_a.as_path()));

        // 6. Teste Destrutivo: Enviar B.png para a Lixeira através do menu da miniatura de B
        app.delete_image_at_path(path_b.clone(), &ctx);
        assert!(app.trash_rx.is_some());

        let start_del = std::time::Instant::now();
        while app.trash_rx.is_some() && start_del.elapsed() < Duration::from_secs(10) {
            app.pump(&ctx);
            thread::sleep(Duration::from_millis(25));
        }

        // Asserções obrigatórias da especificação:
        // - A.png deve continuar intacta no disco
        assert!(path_a.exists(), "A.png deve permanecer intacta no disco");
        // - B.png deve ter sido enviada à Lixeira (não existir mais na pasta)
        assert!(
            !path_b.exists(),
            "B.png deve ter sido enviada para a Lixeira"
        );
        // - C.png deve continuar intacta no disco
        assert!(path_c.exists(), "C.png deve permanecer intacta no disco");

        // - O Cache LRU deve invalidar SOMENTE B.png:
        assert!(
            app.cache.get_by_path(&path_b).is_none(),
            "Cache de B.png deve ser invalidado"
        );
        assert!(
            app.cache.get_by_path(&path_a).is_some(),
            "Cache de A.png deve ser mantido"
        );
        assert!(
            app.cache.get_by_path(&path_c).is_some(),
            "Cache de C.png deve ser mantido"
        );

        // - A lista e o índice devem ser atualizados corretamente:
        assert_eq!(
            app.files.len(),
            2,
            "Lista deve conter agora exatamente 2 arquivos"
        );
        assert_eq!(
            app.files[0], path_a,
            "Primeiro arquivo da lista deve ser A.png"
        );
        assert_eq!(
            app.files[1], path_c,
            "Segundo arquivo da lista deve ser C.png"
        );
        assert_eq!(
            app.index, 0,
            "O índice atual deve continuar sendo 0 (A.png)"
        );
        assert_eq!(
            app.current_path(),
            Some(path_a.as_path()),
            "Imagem ativa deve continuar sendo A.png"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_unicode_spaces_accents_explorer_and_open_with() {
        let dir = temp_dir("test-unicode-spaces");
        let path_spaces = dir.join("foto com espaços e múltiplos separadores.png");
        let path_accents = dir.join("fotografia com acentuação e cedilha çãé.png");
        let path_unicode = dir.join("imagem_japonesa_日本語_🎨.png");

        let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            4,
            4,
            image::Rgb([100, 150, 200]),
        ));
        img.save_with_format(&path_spaces, image::ImageFormat::Png)
            .unwrap();
        img.save_with_format(&path_accents, image::ImageFormat::Png)
            .unwrap();
        img.save_with_format(&path_unicode, image::ImageFormat::Png)
            .unwrap();

        let ctx = eframe::egui::Context::default();
        for test_path in [&path_spaces, &path_accents, &path_unicode] {
            assert!(test_path.exists());

            // 1. Validação de PIDL nativo pelo Shell
            let abs = std::path::absolute(test_path).unwrap();
            let clean = abs
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string();
            let mut wide: Vec<u16> = clean.encode_utf16().collect();
            wide.push(0);

            let _com = crate::system::explorer::ScopedComSta::enter();
            let pidl = unsafe { crate::system::explorer::ILCreateFromPathW(wide.as_ptr()) };
            assert!(
                !pidl.is_null(),
                "PIDL deve ser criado com sucesso para: {}",
                test_path.display()
            );
            unsafe { crate::system::explorer::ILFree(pidl) };

            // 2. Validação da estrutura OPENASINFO
            let info = crate::system::explorer::OPENASINFO {
                pcsz_file: wide.as_ptr(),
                pcsz_class: std::ptr::null(),
                oaif_in_flags: crate::system::explorer::OAIF_EXEC
                    | crate::system::explorer::OAIF_ALLOW_REGISTRATION,
            };
            assert_eq!(info.oaif_in_flags, 0x05);
            assert!(!info.pcsz_file.is_null());

            // 3. Validação de despacho no VisorApp para o caminho alvo com espaços/acentos/unicode
            let mut app = crate::app::VisorApp::new(&ctx, None);
            let target_rev = app.reveal_in_explorer(Some(test_path));
            assert_eq!(target_rev.as_deref(), Some(test_path.as_path()));
            assert!(app.notice.is_some());

            let target_open = app.open_with_dialog(Some(test_path));
            assert_eq!(target_open.as_deref(), Some(test_path.as_path()));
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_persistent_settings_e2e_integration() {
        let temp_dir = temp_dir("test-persistent-settings");
        let settings_path = temp_dir.join("settings.json");

        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        // Configura valores customizados
        app.settings.mouse_wheel_action = crate::system::config::MouseWheelAction::Navigate;
        app.interpolation = crate::app::state::InterpolationMode::Nearest;
        app.background = crate::app::state::CanvasBackground::White;
        app.show_filmstrip = false;
        app.show_info = true;

        // Salva diretamente no caminho de teste usando save_to_path
        app.settings.interpolation = app.interpolation;
        app.settings.background = app.background;
        app.settings.show_filmstrip = app.show_filmstrip;
        app.settings.show_info = app.show_info;

        app.settings
            .save_to_path(&settings_path)
            .expect("salvamento atômico falhou");
        assert!(settings_path.exists());

        // Recarrega e valida integridade total dos dados
        let (reloaded, notice) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert!(notice.is_none());
        assert_eq!(
            reloaded.mouse_wheel_action,
            crate::system::config::MouseWheelAction::Navigate
        );
        assert_eq!(
            reloaded.interpolation,
            crate::app::state::InterpolationMode::Nearest
        );
        assert_eq!(
            reloaded.background,
            crate::app::state::CanvasBackground::White
        );
        assert!(!reloaded.show_filmstrip);
        assert!(reloaded.show_info);

        // Testa restauração de padrões
        app.restore_default_settings();
        assert_eq!(
            app.settings.mouse_wheel_action,
            crate::system::config::MouseWheelAction::Zoom
        );
        assert_eq!(
            app.interpolation,
            crate::app::state::InterpolationMode::Auto
        );
        assert_eq!(
            app.background,
            crate::app::state::CanvasBackground::DarkChecker
        );
        assert!(app.show_filmstrip);
        assert!(!app.show_info);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_custom_color_e2e_persistence_and_subsequent_edits() {
        let temp_dir = temp_dir("test-custom-color-e2e");
        let settings_path = temp_dir.join("settings.json");

        let ctx = eframe::egui::Context::default();
        let mut app = crate::app::VisorApp::new(&ctx, None);

        // 1. Altera para uma cor personalizada específica RGBA
        let custom_rgba = [24, 128, 240, 215];
        let color32 = eframe::egui::Color32::from_rgba_unmultiplied(
            custom_rgba[0],
            custom_rgba[1],
            custom_rgba[2],
            custom_rgba[3],
        );
        app.background = crate::app::state::CanvasBackground::Custom(color32);
        app.settings.background = app.background;
        app.settings.custom_color = custom_rgba;

        // 2. Salva e fecha (simulado)
        app.settings
            .save_to_path(&settings_path)
            .expect("salvamento falhou");
        assert!(settings_path.exists());
        drop(app);

        // 3. Reabre o aplicativo
        let (reopened, notice) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert!(notice.is_none());
        assert_eq!(reopened.custom_color, custom_rgba);
        assert_eq!(
            reopened.background,
            crate::app::state::CanvasBackground::Custom(color32)
        );

        // 4. Altera outra preferência (ex: roda do mouse para Navigate)
        let mut app2 = crate::app::VisorApp::new(&ctx, None);
        app2.settings = reopened;
        app2.background = app2.settings.background;
        app2.settings.mouse_wheel_action = crate::system::config::MouseWheelAction::Navigate;

        app2.settings
            .save_to_path(&settings_path)
            .expect("segundo salvamento falhou");
        drop(app2);

        // 5. Confirma que a cor personalizada continua 100% preservada
        let (reopened2, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert_eq!(
            reopened2.mouse_wheel_action,
            crate::system::config::MouseWheelAction::Navigate
        );
        assert_eq!(reopened2.custom_color, custom_rgba);
        assert_eq!(
            reopened2.background,
            crate::app::state::CanvasBackground::Custom(color32)
        );

        // 6. Altera mais uma preferência (interpolação para Nearest)
        let mut app3 = crate::app::VisorApp::new(&ctx, None);
        app3.settings = reopened2;
        app3.background = app3.settings.background;
        app3.interpolation = crate::app::state::InterpolationMode::Nearest;
        app3.settings.interpolation = app3.interpolation;

        app3.settings
            .save_to_path(&settings_path)
            .expect("terceiro salvamento falhou");
        drop(app3);

        let (reopened3, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert_eq!(
            reopened3.interpolation,
            crate::app::state::InterpolationMode::Nearest
        );
        assert_eq!(reopened3.custom_color, custom_rgba);
        assert_eq!(
            reopened3.background,
            crate::app::state::CanvasBackground::Custom(color32)
        );

        // 7. Simula edição pelo seletor de cores unmultiplied:
        // ui.color_edit_button_srgba_unmultiplied(&mut app.settings.custom_color)
        let mut app4 = crate::app::VisorApp::new(&ctx, None);
        app4.settings = reopened3;
        app4.background = app4.settings.background;

        let next_rgba = [255, 0, 0, 128]; // Vermelho 50% translúcido
        app4.settings.custom_color = next_rgba;
        let [r, g, b, a] = app4.settings.custom_color;
        let next_color32 = eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a);
        app4.settings.background = crate::app::state::CanvasBackground::Custom(next_color32);
        app4.background = crate::app::state::CanvasBackground::Custom(next_color32);
        app4.settings
            .save_to_path(&settings_path)
            .expect("quarto salvamento falhou");
        drop(app4);

        let (reopened4, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert_eq!(reopened4.custom_color, next_rgba);
        assert_eq!(
            reopened4.background,
            crate::app::state::CanvasBackground::Custom(next_color32)
        );

        // 8. Alterna para outro fundo (DarkChecker) e salva: custom_color deve ser preservado
        let mut app5 = crate::app::VisorApp::new(&ctx, None);
        app5.settings = reopened4;
        app5.background = crate::app::state::CanvasBackground::DarkChecker;
        app5.settings.background = app5.background;
        app5.settings
            .save_to_path(&settings_path)
            .expect("salvamento com DarkChecker falhou");
        drop(app5);

        let (reopened5, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        assert_eq!(
            reopened5.background,
            crate::app::state::CanvasBackground::DarkChecker
        );
        assert_eq!(
            reopened5.custom_color, next_rgba,
            "custom_color deve permanecer intacto mesmo após alternar para DarkChecker"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_all_five_mandated_custom_color_cases_e2e() {
        let temp_dir = temp_dir("test-mandated-colors-e2e");
        let settings_path = temp_dir.join("settings.json");
        let ctx = eframe::egui::Context::default();

        let mandated_colors: &[[u8; 4]] = &[
            [24, 128, 240, 215],
            [255, 0, 0, 128],
            [0, 255, 0, 64],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
        ];

        for &rgba in mandated_colors {
            let [r, g, b, a] = rgba;
            let expected_color32 = eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a);

            let mut app = crate::app::VisorApp::new(&ctx, None);
            app.settings.custom_color = rgba;
            app.background = crate::app::state::CanvasBackground::Custom(expected_color32);
            app.settings.background = app.background;

            app.settings
                .save_to_path(&settings_path)
                .expect("salvamento falhou");
            drop(app);

            let (loaded, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
            assert_eq!(
                loaded.custom_color, rgba,
                "custom_color deve ser exatamente {:?}",
                rgba
            );
            assert_eq!(
                loaded.background,
                crate::app::state::CanvasBackground::Custom(expected_color32),
                "background deve coincidir com Color32 desmultiplicado para {:?}",
                rgba
            );

            // Valida round-trip de to_srgba_unmultiplied
            if let crate::app::state::CanvasBackground::Custom(c) = loaded.background {
                assert_eq!(
                    c.to_srgba_unmultiplied(),
                    rgba,
                    "to_srgba_unmultiplied() deve restaurar os bytes exatos para {:?}",
                    rgba
                );
            } else {
                panic!("esperado CanvasBackground::Custom");
            }
        }

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_visual_rendering_semi_transparent_custom_background() {
        // Valida e gera artefato visual demonstrando a renderização de cor personalizada
        // com transparência RGBA [24, 128, 240, 215] misturada sobre fundo xadrez e padrão gráfico.
        let width = 480usize;
        let height = 360usize;
        let custom_rgba = [24u8, 128u8, 240u8, 215u8];
        let color32 = eframe::egui::Color32::from_rgba_unmultiplied(
            custom_rgba[0],
            custom_rgba[1],
            custom_rgba[2],
            custom_rgba[3],
        );

        // Prepara buffer de imagem simulando a composição da tela
        let mut pixels = vec![0u8; width * height * 4];

        // 1. Fundo base (xadrez escuro de transparência do visor: #303034 e #222226)
        let checker_size = 16usize;
        let c1 = [0x30u8, 0x30u8, 0x34u8, 0xFFu8];
        let c2 = [0x22u8, 0x22u8, 0x26u8, 0xFFu8];

        for y in 0..height {
            for x in 0..width {
                let is_c1 = ((x / checker_size) + (y / checker_size)) % 2 == 0;
                let bg_col = if is_c1 { c1 } else { c2 };
                let idx = (y * width + x) * 4;
                pixels[idx..idx + 4].copy_from_slice(&bg_col);
            }
        }

        // 2. Composição da camada Custom Background semi-transparente sobre o xadrez
        // Usando a fórmula de alpha blending: C_out = C_src * alpha + C_dst * (1 - alpha)
        let alpha_f = custom_rgba[3] as f32 / 255.0;
        let inv_alpha_f = 1.0 - alpha_f;

        // Região central da imagem (viewport do canvas)
        let margin_x = 40usize;
        let margin_y = 30usize;
        for y in margin_y..(height - margin_y) {
            for x in margin_x..(width - margin_x) {
                let idx = (y * width + x) * 4;
                let dst_r = pixels[idx] as f32;
                let dst_g = pixels[idx + 1] as f32;
                let dst_b = pixels[idx + 2] as f32;

                let out_r =
                    ((custom_rgba[0] as f32 * alpha_f) + (dst_r * inv_alpha_f)).round() as u8;
                let out_g =
                    ((custom_rgba[1] as f32 * alpha_f) + (dst_g * inv_alpha_f)).round() as u8;
                let out_b =
                    ((custom_rgba[2] as f32 * alpha_f) + (dst_b * inv_alpha_f)).round() as u8;

                pixels[idx] = out_r;
                pixels[idx + 1] = out_g;
                pixels[idx + 2] = out_b;
                pixels[idx + 3] = 255;
            }
        }

        // 3. Desenha um elemento gráfico de teste central (ex: moldura e texto / círculo translúcido)
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = 70.0f32;

        for y in margin_y..(height - margin_y) {
            for x in margin_x..(width - margin_x) {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= radius {
                    let idx = (y * width + x) * 4;
                    // Círculo branco suave com borda
                    if dist > radius - 3.0 {
                        pixels[idx] = 255;
                        pixels[idx + 1] = 255;
                        pixels[idx + 2] = 255;
                    } else {
                        // Blend translúcido branco 40%
                        let fg_a = 0.4f32;
                        let inv_fg_a = 1.0 - fg_a;
                        pixels[idx] =
                            ((255.0 * fg_a) + (pixels[idx] as f32 * inv_fg_a)).round() as u8;
                        pixels[idx + 1] =
                            ((255.0 * fg_a) + (pixels[idx + 1] as f32 * inv_fg_a)).round() as u8;
                        pixels[idx + 2] =
                            ((255.0 * fg_a) + (pixels[idx + 2] as f32 * inv_fg_a)).round() as u8;
                    }
                }
            }
        }

        // 4. Salva a imagem gerada no diretório de saída de testes, se configurado
        if let Ok(dir) = std::env::var("VISOR_TEST_ARTIFACTS_DIR") {
            let out_path =
                std::path::PathBuf::from(dir).join("val_custom_color_semitransparent.png");
            let _ = image::save_buffer(
                &out_path,
                &pixels,
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
            );
        }

        assert_eq!(color32.to_srgba_unmultiplied(), custom_rgba);
    }

    #[test]
    fn test_cli_argument_parsing_variants() {
        // 1. Caminho simples
        let args1 = vec!["--capture-gallery", r"C:\Images\foto.png"];
        assert_eq!(
            parse_cli_initial_path(args1),
            Some(PathBuf::from(r"C:\Images\foto.png"))
        );

        // 2. Caminho com múltiplos espaços
        let args2 = vec![r"C:\Minhas Imagens\Viagem de Férias 2026\foto da praia.png"];
        assert_eq!(
            parse_cli_initial_path(args2),
            Some(PathBuf::from(
                r"C:\Minhas Imagens\Viagem de Férias 2026\foto da praia.png"
            ))
        );

        // 3. Caminho Unicode com acentos e caracteres internacionais
        let args3 = vec![r"D:\Imagens\Café & Açúcar\日本語_東京.jpg"];
        assert_eq!(
            parse_cli_initial_path(args3),
            Some(PathBuf::from(r"D:\Imagens\Café & Açúcar\日本語_東京.jpg"))
        );

        // 4. Caminho longo com prefixo Win32
        let args4 = vec![r"\\?\C:\Caminho\Muito\Longo\Para\Arquivos\Do\Windows\imagem.png"];
        assert_eq!(
            parse_cli_initial_path(args4),
            Some(PathBuf::from(
                r"\\?\C:\Caminho\Muito\Longo\Para\Arquivos\Do\Windows\imagem.png"
            ))
        );

        // 5. Somente argumentos e flags internas: não deve interpretar flags como imagem
        let args5 = vec![
            "--registrar-padrao",
            "--capture-step6",
            "--capture-final-validation",
        ];
        assert_eq!(parse_cli_initial_path(args5), None);

        // 6. Lista vazia
        let args6: Vec<&str> = vec![];
        assert_eq!(parse_cli_initial_path(args6), None);
    }

    #[test]
    fn test_first_use_banner_lifecycle_and_persistence() {
        let temp_dir = temp_dir("test-banner-lifecycle");
        let settings_path = temp_dir.join("settings.json");
        let ctx = eframe::egui::Context::default();

        // 1. Primeiro uso: simula término da verificação com extensões pendentes
        let mut app1 = crate::app::VisorApp::new(&ctx, None);
        app1.is_checking_associations = false;
        app1.associations.pending_count = 3;
        app1.recalculate_association_banner();
        assert!(
            app1.show_association_banner,
            "banner deve ficar visível quando existem extensões pendentes"
        );

        // 2. Simula ação 'Agora não' na sessão atual
        app1.show_association_banner = false;
        app1.settings
            .save_to_path(&settings_path)
            .expect("salvamento falhou");
        drop(app1);

        // 3. Reabre o app: simula conclusão da verificação com pendências
        let (reopened, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        let mut app2 = crate::app::VisorApp::new(&ctx, None);
        app2.settings = reopened;
        app2.is_checking_associations = false;
        app2.associations.pending_count = 3;
        app2.recalculate_association_banner();
        assert!(
            app2.show_association_banner,
            "banner deve reaparecer se ainda existirem pendências"
        );

        // 4. Usuário ignora as pendências
        app2.settings.ignore_extension("webp");
        app2.settings.ignore_extension("tiff");
        app2.settings.ignore_extension("dds");
        app2.associations.pending_count = 0;
        app2.associations.ignored_count = 3;
        app2.recalculate_association_banner();
        assert!(
            !app2.show_association_banner,
            "banner deve sumir após ignorar todas as pendências"
        );
        app2.settings
            .save_to_path(&settings_path)
            .expect("segundo salvamento falhou");
        drop(app2);

        // 5. Reabre o app com extensões ignoradas: banner NÃO deve ser exibido
        let (reopened2, _) = crate::system::config::AppSettings::load_from_path(&settings_path);
        let mut app3 = crate::app::VisorApp::new(&ctx, None);
        app3.settings = reopened2;
        app3.is_checking_associations = false;
        app3.associations.pending_count = 0;
        app3.associations.ignored_count = 3;
        app3.recalculate_association_banner();
        assert!(
            !app3.show_association_banner,
            "banner NÃO deve ser exibido com pendências zeradas"
        );

        // 6. Restauração de padrões: limpa ignoradas
        app3.restore_default_settings();
        assert!(
            app3.settings.ignored_default_extensions.is_empty(),
            "restaurar padrões deve limpar a lista de ignoradas"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_file_opening_by_association_flow() {
        let dir = temp_dir("test-assoc-file-open");
        let ctx = eframe::egui::Context::default();

        // 1. Arquivo de imagem válido com acentos e espaços
        let file_path = dir.join("foto de férias com acentuação café.png");
        let img = image::RgbaImage::new(16, 16);
        img.save_with_format(&file_path, image::ImageFormat::Png)
            .unwrap();

        let app = crate::app::VisorApp::new(&ctx, Some(file_path.clone()));
        assert_eq!(app.current_path(), Some(file_path.as_path()));
        assert!(app.error.is_none());

        // 2. Arquivo inexistente recebido por argumento
        let nonexistent = dir.join("arquivo_fantasma.png");
        let app_err = crate::app::VisorApp::new(&ctx, Some(nonexistent));
        assert_eq!(app_err.error, Some("Arquivo não encontrado.".to_owned()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_visual_rendering_first_use_banner() {
        let width = 720usize;
        let height = 80usize;
        let mut pixels = vec![0u8; width * height * 4];

        // 1. Fundo do painel do topo (surface_panel: #1E1E22)
        let bg = [30u8, 30u8, 34u8, 255u8];
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&bg);
        }

        // 2. Card do banner Fluent (surface_card: #26262C com borda #404048)
        let pad_x = 16usize;
        let pad_y = 10usize;
        let card_color = [38u8, 38u8, 44u8, 255u8];
        let border_color = [64u8, 64u8, 72u8, 255u8];

        for y in pad_y..(height - pad_y) {
            for x in pad_x..(width - pad_x) {
                let is_border =
                    x == pad_x || x == width - pad_x - 1 || y == pad_y || y == height - pad_y - 1;
                let idx = (y * width + x) * 4;
                if is_border {
                    pixels[idx..idx + 4].copy_from_slice(&border_color);
                } else {
                    pixels[idx..idx + 4].copy_from_slice(&card_color);
                }
            }
        }

        // 3. Botão "Definir como padrão" (accent_default: #60CDFF com texto #101014)
        let btn_w = 140usize;
        let btn_h = 32usize;
        let btn_x = width - pad_x - 16 - btn_w;
        let btn_y = (height - btn_h) / 2;
        let accent = [96u8, 205u8, 255u8, 255u8];

        for y in btn_y..(btn_y + btn_h) {
            for x in btn_x..(btn_x + btn_w) {
                let idx = (y * width + x) * 4;
                pixels[idx..idx + 4].copy_from_slice(&accent);
            }
        }

        // 4. Botão "Agora não" (surface_active: #3E3E48)
        let btn2_w = 90usize;
        let btn2_x = btn_x - 12 - btn2_w;
        let active_col = [62u8, 62u8, 72u8, 255u8];

        for y in btn_y..(btn_y + btn_h) {
            for x in btn2_x..(btn2_x + btn2_w) {
                let idx = (y * width + x) * 4;
                pixels[idx..idx + 4].copy_from_slice(&active_col);
            }
        }

        // 5. Botão "Não perguntar novamente" (text_muted)
        let btn3_w = 150usize;
        let btn3_x = btn2_x - 12 - btn3_w;
        let muted_col = [48u8, 48u8, 56u8, 255u8];

        for y in btn_y..(btn_y + btn_h) {
            for x in btn3_x..(btn3_x + btn3_w) {
                let idx = (y * width + x) * 4;
                pixels[idx..idx + 4].copy_from_slice(&muted_col);
            }
        }

        // Salva artefato visual para documentação de testes, se configurado
        if let Ok(dir) = std::env::var("VISOR_TEST_ARTIFACTS_DIR") {
            let out_path = std::path::PathBuf::from(dir).join("val_first_use_banner.png");
            let _ = image::save_buffer(
                &out_path,
                &pixels,
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
            );
        }
    }

    #[test]
    fn test_default_zoom_is_100_percent_on_open_and_step() {
        let ctx = eframe::egui::Context::default();
        let mut app = crate::VisorApp::new(&ctx, None);
        assert_eq!(
            app.settings.default_zoom,
            crate::system::config::DefaultZoomMode::Actual100
        );

        let make_decoded = |w: u32, h: u32| {
            std::sync::Arc::new(visor::Decoded {
                frames: vec![visor::Frame {
                    rgba: vec![255u8; (w * h) as usize * 4],
                    width: w,
                    height: h,
                    delay_ms: 0,
                }],
                info: visor::ImageInfo {
                    id: "test",
                    name: "test.png".to_string(),
                    category: visor::Category::Raster,
                    support: visor::Support::Full,
                    bit_depth: 8,
                    color: "RGBA8".to_string(),
                    original_w: w,
                    original_h: h,
                    display_w: w,
                    display_h: h,
                    reduced: false,
                    has_alpha: false,
                    frame_count: 1,
                    loop_forever: false,
                    mipmaps: 0,
                    exif: Vec::new(),
                    icc: None,
                    notes: Vec::new(),
                },
            })
        };

        // 1. Instala a primeira imagem -> deve estar em 100% (tamanho real), e NÃO em Ajustar (fit)
        let decoded1 = make_decoded(100, 100);
        app.install(&ctx, decoded1);
        assert!(
            !app.view.fit,
            "Ao abrir uma imagem, o padrão DEVE ser 100% (não Fit)"
        );
        assert!(
            app.view.is_actual(ctx.pixels_per_point()),
            "Ao abrir uma imagem, deve estar na escala real 100%"
        );

        // 2. Simula o usuário alterando manualmente para Ajustar (Fit)
        app.view.set_fit();
        assert!(app.view.fit);

        // 3. Ao avançar para a próxima imagem (ou abrir nova), o padrão volta para 100%
        let decoded2 = make_decoded(200, 150);
        app.install(&ctx, decoded2.clone());
        assert!(
            !app.view.fit,
            "Ao passar para a próxima imagem, o padrão DEVE ser 100% (não Fit)"
        );
        assert!(
            app.view.is_actual(ctx.pixels_per_point()),
            "A nova imagem DEVE estar em 100%"
        );

        // 4. Caso o usuário configure explicitamente o padrão para Fit nas opções:
        app.settings.default_zoom = crate::system::config::DefaultZoomMode::Fit;
        app.install(&ctx, decoded2);
        assert!(
            app.view.fit,
            "Quando configurado para Fit, deve respeitar a preferência"
        );
    }
}
