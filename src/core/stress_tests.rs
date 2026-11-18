//! Testes de estresse, memória, concorrência e desempenho para o Passo 5.
//!
//! Exercita cargas pesadas (100, 500, 1000 requisições), navegação sob pressão,
//! animações multi-quadro, auditoria de cópia de memória e medições de latência (média, mediana, p95).

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::core::image_cache::{CacheKey, ImageCache, estimate_decoded_bytes};
use crate::workers::{CancellationToken, Priority, TaskQueue, ThreadPool};

fn make_test_image(w: u32, h: u32, frames_count: usize, id: usize) -> visor::Decoded {
    let frame_bytes = (w as usize) * (h as usize) * 4;
    let mut frames = Vec::with_capacity(frames_count);
    for f in 0..frames_count {
        // Padrão de cores determinístico
        let mut rgba = vec![0u8; frame_bytes];
        if !rgba.is_empty() {
            rgba[0] = (id % 255) as u8;
            rgba[1] = (f % 255) as u8;
        }
        frames.push(visor::Frame {
            rgba,
            width: w,
            height: h,
            delay_ms: 40,
        });
    }

    visor::Decoded {
        frames,
        info: visor::ImageInfo {
            id: "stress",
            name: format!("stress_img_{id}.png"),
            category: visor::Category::Raster,
            support: visor::Support::Full,
            bit_depth: 8,
            color: "RGBA8".to_owned(),
            original_w: w,
            original_h: h,
            display_w: w,
            display_h: h,
            reduced: false,
            has_alpha: true,
            frame_count: frames_count,
            loop_forever: true,
            mipmaps: 0,
            exif: vec![("TestTag".to_owned(), format!("Val_{id}"))],
            icc: Some("sRGB".to_owned()),
            notes: vec![format!("Nota de estresse {id}")],
        },
    }
}

fn calculate_stats(mut samples: Vec<Duration>) -> (Duration, Duration, Duration) {
    if samples.is_empty() {
        return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    }
    samples.sort();
    let count = samples.len();
    let sum: Duration = samples.iter().sum();
    let mean = sum / (count as u32);
    let median = samples[count / 2];
    let p95_idx = ((count as f64 * 0.95) as usize).min(count - 1);
    let p95 = samples[p95_idx];
    (mean, median, p95)
}

// =============================================================================
// 1. TESTES DE ESTRESSE DE CONCORRÊNCIA E REQUISIÇÕES CONSECUTIVAS
// =============================================================================

#[test]
fn test_stress_consecutive_requests_scaling() {
    for &total_requests in &[100, 500, 1000] {
        let pool = Arc::new(ThreadPool::new(4));
        let cache = ImageCache::with_capacity(32 * 1024 * 1024);
        let (tx, rx) = mpsc::channel::<(u64, usize)>();

        let mut current_gen = 0u64;
        let mut active_token: Option<CancellationToken> = None;
        let mut installed_id = None;

        let start = Instant::now();

        for req_id in 1..=total_requests {
            if let Some(t) = active_token.take() {
                t.cancel();
            }
            current_gen += 1;
            let generation_id = current_gen;
            let cancel = CancellationToken::new();
            let c = cancel.clone();
            let tx_clone = tx.clone();
            let cache_clone = cache.clone();

            pool.spawn_with_priority(Priority::Viewport, Some(c.clone()), move || {
                if c.is_cancelled() {
                    return;
                }
                let key = CacheKey::new(
                    PathBuf::from(format!(r"C:\stress\{req_id}.png")),
                    1000,
                    Some(SystemTime::UNIX_EPOCH),
                );
                let img = make_test_image(30, 30, 1, req_id);
                cache_clone.insert(key, img);

                if !c.is_cancelled() {
                    let _ = tx_clone.send((generation_id, req_id));
                }
            });
            active_token = Some(cancel);
        }

        let target_gen = current_gen;

        // Drena até processar o alvo ou bater timeout defensivo de 5 segundos
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            while let Ok((msg_gen, id)) = rx.try_recv() {
                if msg_gen == target_gen {
                    installed_id = Some(id);
                }
            }
            if installed_id == Some(total_requests) && pool.is_idle() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }

        let elapsed = start.elapsed();
        assert_eq!(
            installed_id,
            Some(total_requests),
            "Para {total_requests} requisições, o estado final deve assentar na última imagem"
        );
        assert!(
            cache.current_bytes() <= cache.max_bytes(),
            "Orçamento não pode ser estourado"
        );
        println!(
            "[Estresse] {} requisições concluídas em {:?} com workers saudáveis",
            total_requests, elapsed
        );
    }
}

#[test]
fn test_stress_rapid_navigation_large_images() {
    let pool = Arc::new(ThreadPool::new(4));
    // Cache limitado a 16 MiB
    let cache = ImageCache::with_capacity(16 * 1024 * 1024);
    let (tx, rx) = mpsc::channel::<(u64, usize)>();

    let mut current_gen = 0u64;
    let mut active_token: Option<CancellationToken> = None;

    // Dispara 30 requisições de imagens de 1000x1000 pixels (cada uma consome ~4 MB)
    for req_id in 1..=30 {
        if let Some(t) = active_token.take() {
            t.cancel();
        }
        current_gen += 1;
        let generation_id = current_gen;
        let cancel = CancellationToken::new();
        let c = cancel.clone();
        let tx_clone = tx.clone();
        let cache_clone = cache.clone();

        pool.spawn_with_priority(Priority::Viewport, Some(c.clone()), move || {
            if c.is_cancelled() {
                return;
            }
            let key = CacheKey::new(
                PathBuf::from(format!(r"C:\large\{req_id}.png")),
                4_000_000,
                Some(SystemTime::UNIX_EPOCH),
            );
            let img = make_test_image(1000, 1000, 1, req_id);
            cache_clone.insert(key, img);

            if !c.is_cancelled() {
                let _ = tx_clone.send((generation_id, req_id));
            }
        });
        active_token = Some(cancel);
    }

    let target_gen = current_gen;
    let mut final_id = None;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        while let Ok((msg_gen, id)) = rx.try_recv() {
            if msg_gen == target_gen {
                final_id = Some(id);
            }
        }
        if final_id == Some(30) && pool.is_idle() {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    assert_eq!(final_id, Some(30));
    assert!(
        cache.current_bytes() <= cache.max_bytes(),
        "Consumo ({} bytes) deve respeitar 16 MiB",
        cache.current_bytes()
    );

    // Em seguida, navega por 6 imagens grandes completas para exercitar a evicção do teto de 16 MiB
    for i in 31..=36 {
        let key = CacheKey::new(
            PathBuf::from(format!(r"C:\large\{i}.png")),
            4_000_000,
            Some(SystemTime::UNIX_EPOCH),
        );
        let img = make_test_image(1000, 1000, 1, i);
        cache.insert(key, img);
    }

    assert!(
        cache.stats().evictions > 0,
        "Inserir 6 imagens de ~4 MiB em um cache de 16 MiB deve acionar evicções automáticas LRU"
    );
    assert!(
        cache.current_bytes() <= cache.max_bytes(),
        "Orçamento de 16 MiB deve ser mantido após evicções"
    );
}

#[test]
fn test_stress_multi_frame_animations() {
    let cache = ImageCache::with_capacity(32 * 1024 * 1024);

    // Cria animações com 10, 25 e 50 frames
    for &(frames, id) in &[(10, 1), (25, 2), (50, 3)] {
        let key = CacheKey::new(
            PathBuf::from(format!(r"C:\anim\{id}.gif")),
            200_000,
            Some(SystemTime::UNIX_EPOCH),
        );
        let anim = make_test_image(200, 200, frames, id);
        let estimated = estimate_decoded_bytes(&anim);

        assert!(
            estimated >= frames * 200 * 200 * 4,
            "Estimativa deve englobar a alocação de todos os quadros"
        );
        assert!(cache.insert(key.clone(), anim));

        let hit = cache.get(&key).unwrap();
        assert_eq!(hit.frames.len(), frames);
    }
}

#[test]
fn test_stress_concurrent_viewport_and_thumbnails() {
    let queue = Arc::new(TaskQueue::default());
    let pool = ThreadPool::with_queue(2, Arc::clone(&queue));
    let served_order = Arc::new(Mutex::new(Vec::new()));

    // Pausa workers inicialmente
    let barrier = Arc::new(Barrier::new(3));
    for _ in 0..2 {
        let b = Arc::clone(&barrier);
        pool.spawn_with_priority(Priority::Viewport, None, move || {
            b.wait();
        });
    }

    // Enfileira 30 thumbnails
    for i in 0..30 {
        let order = Arc::clone(&served_order);
        pool.spawn_with_priority(Priority::Thumbnail, None, move || {
            order.lock().unwrap().push(format!("thumb_{i}"));
        });
    }

    // Enfileira 5 viewports intercaladas
    for i in 0..5 {
        let order = Arc::clone(&served_order);
        pool.spawn_with_priority(Priority::Viewport, None, move || {
            order.lock().unwrap().push(format!("viewport_{i}"));
        });
    }

    // Libera os workers
    barrier.wait();

    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        if pool.is_idle() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let results = served_order.lock().unwrap().clone();
    assert_eq!(
        results.len(),
        35,
        "Todas as 35 tarefas devem ser executadas"
    );

    // Verifica que as primeiras tarefas processadas foram prioritariamente viewports
    let first_five_viewports = results[0..5]
        .iter()
        .filter(|name| name.starts_with("viewport"))
        .count();
    assert!(
        first_five_viewports >= 3,
        "Prioridade Viewport deve prevalecer sobre miniaturas pendentes"
    );
}

#[test]
fn test_stress_shutdown_during_heavy_load() {
    let (done_tx, done_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut pool = ThreadPool::new(4);
        let cancel_all = CancellationToken::new();

        // Enfileira 100 tarefas concorrentes pesadas
        for _ in 0..100 {
            let c = cancel_all.clone();
            pool.spawn_with_priority(Priority::Thumbnail, Some(c), || {
                thread::sleep(Duration::from_millis(2));
            });
        }

        // Interrompe imediatamente durante execução
        cancel_all.cancel();
        let t0 = Instant::now();
        pool.shutdown();
        let shutdown_duration = t0.elapsed();

        assert!(
            shutdown_duration < Duration::from_secs(2),
            "Shutdown com cancelamento deve ser rápido (< 2s, foi {:?})",
            shutdown_duration
        );
        done_tx.send(true).unwrap();
    });

    let res = done_rx.recv_timeout(Duration::from_secs(3));
    assert!(
        res.is_ok(),
        "Não pode ocorrer deadlock no encerramento sob estresse"
    );
}

#[test]
fn test_stress_cache_thrashing_fits_and_exceeds() {
    // Cache com teto de 500 KB
    let cache = ImageCache::with_capacity(500 * 1024);

    for cycle in 0..50 {
        // Imagem pequena (cabe)
        let key_small = CacheKey::new(
            PathBuf::from(format!(r"C:\cycle\{cycle}_small.png")),
            1000,
            Some(SystemTime::UNIX_EPOCH),
        );
        let small = make_test_image(30, 30, 1, cycle);
        assert!(cache.insert(key_small.clone(), small));
        assert!(cache.get(&key_small).is_some());

        // Imagem gigantesca (800 KB, maior que o teto total de 500 KB)
        let key_huge = CacheKey::new(
            PathBuf::from(format!(r"C:\cycle\{cycle}_huge.png")),
            800_000,
            Some(SystemTime::UNIX_EPOCH),
        );
        let huge = make_test_image(500, 500, 1, cycle);
        let accepted = cache.insert(key_huge.clone(), huge);
        assert!(!accepted, "Imagem excedente deve ser rejeitada");
        assert!(cache.get(&key_huge).is_none());

        assert!(
            cache.current_bytes() <= cache.max_bytes(),
            "Orçamento deve ser rigorosamente mantido durante thrashing"
        );
    }
}

// =============================================================================
// 2. AUDITORIA DE CÓPIA DE MEMÓRIA E COMPARTILHAMENTO DE ARC
// =============================================================================

#[test]
fn test_audit_zero_copy_buffer_sharing() {
    let cache = ImageCache::with_capacity(10 * 1024 * 1024);
    let key = CacheKey::new(
        PathBuf::from(r"C:\audit\shared.png"),
        5000,
        Some(SystemTime::UNIX_EPOCH),
    );

    let original = make_test_image(100, 100, 1, 42);
    let original_ptr = original.frames[0].rgba.as_ptr();

    cache.insert(key.clone(), original);

    // Consulta do Cache Hit
    let arc1 = cache.get(&key).expect("Cache Hit esperado");
    let arc2 = cache.get(&key).expect("Cache Hit esperado");

    // 1. Comprova que são exatamente a mesma instância Arc
    assert!(Arc::ptr_eq(&arc1, &arc2));
    assert!(Arc::strong_count(&arc1) >= 3); // Cache + arc1 + arc2

    // 2. Comprova que o buffer de pixels no heap NÃO foi copiado (mesmo endereço de memória)
    let ptr1 = arc1.frames[0].rgba.as_ptr();
    let ptr2 = arc2.frames[0].rgba.as_ptr();
    assert_eq!(
        ptr1, ptr2,
        "Buffer de pixel no heap deve ser compartilhado sem cópia"
    );
    assert_eq!(
        ptr1, original_ptr,
        "Ponteiro deve ser idêntico ao original alocado"
    );

    // 3. Comprova que Arc::make_mut desacopla (Copy-on-Write) apenas sob mutação
    let mut modified_arc = Arc::clone(&arc1);
    let make_mut_ptr = {
        let unique = Arc::make_mut(&mut modified_arc);
        unique.frames[0].rgba[0] = 255;
        unique.frames[0].rgba.as_ptr()
    };

    assert_ne!(
        make_mut_ptr, ptr1,
        "Após mutação via make_mut, o buffer deve ser clonado de forma isolada"
    );
    // O cache permanece com o dado original intacto
    let cached_again = cache.get(&key).unwrap();
    assert_eq!(cached_again.frames[0].rgba[0], 42);
}

// =============================================================================
// 3. MEDIÇÃO REAL DE DESEMPENHO E BENCHMARKS (Ryzen 5 5500)
// =============================================================================

#[test]
fn test_performance_benchmarks_real_measurements() {
    let temp_dir = std::env::temp_dir().join(format!("visor-bench-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let test_file = temp_dir.join("bench.png");
    let dyn_img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        200,
        200,
        image::Rgb([100, 150, 200]),
    ));
    dyn_img
        .save_with_format(&test_file, image::ImageFormat::Png)
        .unwrap();

    // 1. Cache Miss: Tempo de leitura e decodificação do disco
    let mut miss_samples = Vec::new();
    for _ in 0..30 {
        let t0 = Instant::now();
        let decoded = visor::open_image(&test_file).unwrap();
        miss_samples.push(t0.elapsed());
        assert_eq!(decoded.frames[0].width, 200);
    }
    let (miss_mean, miss_median, miss_p95) = calculate_stats(miss_samples);

    // Configura o cache para testes de Hit
    let cache = ImageCache::with_capacity(32 * 1024 * 1024);
    let key = CacheKey::from_path(&test_file).unwrap();
    let initial_decoded = visor::open_image(&test_file).unwrap();
    cache.insert(key.clone(), initial_decoded);

    // 2. Cache Hit: Tempo de recuperação na memória RAM (Zero-Copy)
    let mut hit_samples = Vec::new();
    for _ in 0..100 {
        let t0 = Instant::now();
        let hit = cache.get(&key).unwrap();
        hit_samples.push(t0.elapsed());
        assert_eq!(hit.frames[0].width, 200);
    }
    let (hit_mean, hit_median, hit_p95) = calculate_stats(hit_samples);

    // 3. Cache sob pressão: Tempo de inserção com evicção forçada
    let pressure_cache = ImageCache::with_capacity(100 * 1024);
    let mut eviction_samples = Vec::new();
    for i in 0..50 {
        let k = CacheKey::new(
            PathBuf::from(format!(r"C:\pressure\{i}.png")),
            10_000,
            Some(SystemTime::UNIX_EPOCH),
        );
        let img = make_test_image(80, 80, 1, i);
        let t0 = Instant::now();
        pressure_cache.insert(k, img);
        eviction_samples.push(t0.elapsed());
    }
    let (evict_mean, evict_median, evict_p95) = calculate_stats(eviction_samples);

    println!("\n=== RESULTADOS REAIS DE DESEMPENHO (Ryzen 5 5500) ===");
    println!(
        "Cache Miss (Leitura + Decode PNG): Média = {:?}, Mediana = {:?}, P95 = {:?}",
        miss_mean, miss_median, miss_p95
    );
    println!(
        "Cache Hit (Zero-Copy RAM):        Média = {:?}, Mediana = {:?}, P95 = {:?}",
        hit_mean, hit_median, hit_p95
    );
    println!(
        "Cache sob Pressão (Evicção LRU): Média = {:?}, Mediana = {:?}, P95 = {:?}",
        evict_mean, evict_median, evict_p95
    );

    let _ = fs::remove_dir_all(temp_dir);

    // Validações rigorosas
    assert!(
        hit_mean < Duration::from_micros(50),
        "Cache Hit deve ser sub-milissegundo (< 50µs, foi {:?})",
        hit_mean
    );
    assert!(
        miss_mean > hit_mean,
        "Cache Hit deve ser significativamente mais rápido que Miss"
    );
}

#[test]
fn test_performance_roundtrip_navigation_hit_rate() {
    let cache = ImageCache::with_capacity(32 * 1024 * 1024);
    let total_images = 20;

    // Ida (navegação para frente 1..=20): Todas são inéditas -> 100% Miss
    let mut miss_count = 0;
    for i in 1..=total_images {
        let key = CacheKey::new(
            PathBuf::from(format!(r"C:\galeria\foto_{i}.png")),
            5000,
            Some(SystemTime::UNIX_EPOCH),
        );
        if cache.get(&key).is_none() {
            miss_count += 1;
            let decoded = make_test_image(50, 50, 1, i);
            cache.insert(key, decoded);
        }
    }
    assert_eq!(miss_count, 20);

    // Volta (navegação para trás 20..=1): Todas estão em cache -> 100% Hit!
    let mut hit_count = 0;
    let t0 = Instant::now();
    for i in (1..=total_images).rev() {
        let key = CacheKey::new(
            PathBuf::from(format!(r"C:\galeria\foto_{i}.png")),
            5000,
            Some(SystemTime::UNIX_EPOCH),
        );
        if cache.get(&key).is_some() {
            hit_count += 1;
        }
    }
    let elapsed_roundtrip_hits = t0.elapsed();

    assert_eq!(
        hit_count, 20,
        "Todas as 20 imagens na volta devem resultar em Cache Hit"
    );
    println!(
        "\n[Navegação Ida e Volta] 20 imagens: 20 hits na volta em {:?} (média de {:?} por imagem)",
        elapsed_roundtrip_hits,
        elapsed_roundtrip_hits / 20
    );
}
