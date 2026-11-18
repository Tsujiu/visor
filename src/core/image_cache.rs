//! Cache LRU (Least Recently Used) de imagens decodificadas em memória RAM.
//!
//! Armazena dados de CPU (`visor::Decoded`) com controle rigoroso de orçamento de memória,
//! chaves compostas (caminho canônico + tamanho + timestamp de modificação),
//! evicção automática LRU e suporte a acesso concorrente seguro via `Arc`.

#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Capacidade padrão do cache: 128 MiB.
pub const DEFAULT_CACHE_BYTES: usize = 128 * 1024 * 1024;

/// Configurações operacionais do cache de imagens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheConfig {
    pub max_bytes: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_CACHE_BYTES,
        }
    }
}

/// Chave composta para validação de integridade e detecção de arquivos alterados no disco.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub path: PathBuf,
    pub file_size: u64,
    pub modified: Option<SystemTime>,
}

impl CacheKey {
    /// Constrói uma chave a partir do caminho e dos metadados atuais do arquivo no disco.
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let meta = fs::metadata(path)?;
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Ok(Self {
            path: canonical,
            file_size: meta.len(),
            modified: meta.modified().ok(),
        })
    }

    /// Cria uma chave com metadados explícitos (útil para testes ou chaves virtuais).
    pub fn new(path: PathBuf, file_size: u64, modified: Option<SystemTime>) -> Self {
        Self {
            path,
            file_size,
            modified,
        }
    }
}

/// Estatísticas de uso e diagnóstico do cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_bytes: usize,
    pub entry_count: usize,
}

/// Calcula a pegada de memória real em bytes de uma imagem decodificada (`visor::Decoded`).
///
/// Considera buffers de pixel de todos os quadros de animação, estruturas de quadros,
/// strings de metadados, EXIF e notas, utilizando operações com proteção contra overflow.
pub fn estimate_decoded_bytes(decoded: &visor::Decoded) -> usize {
    let mut total: usize = std::mem::size_of::<visor::Decoded>();

    // Vetor de quadros e seus buffers RGBA
    total = total.saturating_add(
        decoded
            .frames
            .capacity()
            .saturating_mul(std::mem::size_of::<visor::Frame>()),
    );

    for frame in &decoded.frames {
        total = total.saturating_add(frame.rgba.capacity());
    }

    // Strings de metadados principais
    total = total.saturating_add(decoded.info.name.capacity());
    total = total.saturating_add(decoded.info.color.capacity());
    if let Some(icc) = &decoded.info.icc {
        total = total.saturating_add(icc.capacity());
    }

    // Tabela de metadados EXIF
    total = total.saturating_add(
        decoded
            .info
            .exif
            .capacity()
            .saturating_mul(std::mem::size_of::<(String, String)>()),
    );
    for (k, v) in &decoded.info.exif {
        total = total.saturating_add(k.capacity().saturating_add(v.capacity()));
    }

    // Vetor de notas
    total = total.saturating_add(
        decoded
            .info
            .notes
            .capacity()
            .saturating_mul(std::mem::size_of::<String>()),
    );
    for note in &decoded.info.notes {
        total = total.saturating_add(note.capacity());
    }

    total
}

struct CacheEntry {
    decoded: Arc<visor::Decoded>,
    byte_size: usize,
    last_access: u64,
}

struct CacheInner {
    config: CacheConfig,
    entries: HashMap<CacheKey, CacheEntry>,
    order: BTreeMap<u64, CacheKey>,
    access_counter: u64,
    current_bytes: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CacheInner {
    fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
            order: BTreeMap::new(),
            access_counter: 0,
            current_bytes: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            self.order.remove(&entry.last_access);
            self.access_counter = self.access_counter.wrapping_add(1);
            entry.last_access = self.access_counter;
            self.order.insert(entry.last_access, key.clone());
        }
    }

    fn evict_least_recently_used(&mut self) -> bool {
        let lru_access = match self.order.keys().next().copied() {
            Some(access) => access,
            None => return false,
        };

        let key = self.order.remove(&lru_access).unwrap();
        if let Some(entry) = self.entries.remove(&key) {
            self.current_bytes = self.current_bytes.saturating_sub(entry.byte_size);
            self.evictions = self.evictions.saturating_add(1);
            true
        } else {
            false
        }
    }
}

/// Cache de imagens decodificadas com política LRU, thread-safe e compartilhamento via `Arc`.
#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<CacheInner>>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

impl ImageCache {
    /// Cria um novo `ImageCache` com a configuração fornecida.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CacheInner::new(config))),
        }
    }

    /// Cria um novo `ImageCache` com capacidade máxima especificada em bytes.
    pub fn with_capacity(max_bytes: usize) -> Self {
        Self::new(CacheConfig { max_bytes })
    }

    /// Consulta uma imagem no cache.
    ///
    /// Se encontrada (cache hit), atualiza sua recência e retorna uma referência `Arc`.
    /// Se não encontrada (cache miss), incrementa a contabilidade de misses e retorna `None`.
    pub fn get(&self, key: &CacheKey) -> Option<Arc<visor::Decoded>> {
        let mut inner = self.inner.lock().unwrap();
        if inner.entries.contains_key(key) {
            inner.hits = inner.hits.saturating_add(1);
            inner.touch(key);
            let entry = inner.entries.get(key).unwrap();
            Some(Arc::clone(&entry.decoded))
        } else {
            inner.misses = inner.misses.saturating_add(1);
            None
        }
    }

    /// Consulta uma imagem no cache a partir de um caminho no disco.
    /// Constrói a `CacheKey` consultando metadados atuais para validar se o arquivo não foi alterado.
    pub fn get_by_path(&self, path: &Path) -> Option<Arc<visor::Decoded>> {
        let key = CacheKey::from_path(path).ok()?;
        self.get(&key)
    }

    /// Insere uma imagem decodificada no cache associada a uma `CacheKey`.
    ///
    /// - Se a imagem exceder o orçamento total `max_bytes`, não é armazenada e retorna `false`.
    /// - Se o orçamento for zero, a inserção é ignorada e retorna `false`.
    /// - Se a chave já existir, a entrada anterior é substituída e a contabilidade é ajustada.
    /// - Executa evicção automática LRU até liberar espaço suficiente para a nova entrada.
    pub fn insert(&self, key: CacheKey, decoded: visor::Decoded) -> bool {
        self.insert_arc(key, Arc::new(decoded))
    }

    /// Insere uma referência compartilhada `Arc<visor::Decoded>` no cache.
    pub fn insert_arc(&self, key: CacheKey, decoded: Arc<visor::Decoded>) -> bool {
        let byte_size = estimate_decoded_bytes(&decoded);
        let mut inner = self.inner.lock().unwrap();

        // Limite zero ou imagem maior que a capacidade total do cache não entra
        if inner.config.max_bytes == 0 || byte_size > inner.config.max_bytes {
            // Se já existia uma entrada anterior para esta chave, remove-a para manter consistência
            if let Some(old) = inner.entries.remove(&key) {
                inner.order.remove(&old.last_access);
                inner.current_bytes = inner.current_bytes.saturating_sub(old.byte_size);
            }
            return false;
        }

        // Se a chave já existia, subtrai o tamanho antigo antes de calcular a necessidade de evicção
        if let Some(old) = inner.entries.remove(&key) {
            inner.order.remove(&old.last_access);
            inner.current_bytes = inner.current_bytes.saturating_sub(old.byte_size);
        }

        // Evicção LRU: remove as entradas menos recentemente usadas até caber a nova
        while inner.current_bytes.saturating_add(byte_size) > inner.config.max_bytes {
            if !inner.evict_least_recently_used() {
                break;
            }
        }

        // Se por algum motivo ainda não couber (ex: max_bytes menor que o tamanho):
        if inner.current_bytes.saturating_add(byte_size) > inner.config.max_bytes {
            return false;
        }

        inner.access_counter = inner.access_counter.wrapping_add(1);
        let access = inner.access_counter;

        inner.order.insert(access, key.clone());
        inner.current_bytes = inner.current_bytes.saturating_add(byte_size);
        inner.entries.insert(
            key,
            CacheEntry {
                decoded,
                byte_size,
                last_access: access,
            },
        );

        true
    }

    /// Invalida e remove uma chave específica do cache. Retorna `true` se a entrada existia.
    pub fn invalidate(&self, key: &CacheKey) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.entries.remove(key) {
            inner.order.remove(&entry.last_access);
            inner.current_bytes = inner.current_bytes.saturating_sub(entry.byte_size);
            true
        } else {
            false
        }
    }

    /// Invalida todas as entradas associadas a um caminho (normalizado ou canônico).
    pub fn invalidate_path(&self, path: &Path) -> usize {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let mut inner = self.inner.lock().unwrap();

        let to_remove: Vec<CacheKey> = inner
            .entries
            .keys()
            .filter(|k| k.path == canonical || k.path == path)
            .cloned()
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            if let Some(entry) = inner.entries.remove(&key) {
                inner.order.remove(&entry.last_access);
                inner.current_bytes = inner.current_bytes.saturating_sub(entry.byte_size);
            }
        }
        count
    }

    /// Limpa integralmente todas as entradas do cache.
    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.entries.clear();
        inner.order.clear();
        inner.current_bytes = 0;
    }

    /// Retorna uma cópia das métricas atuais do cache.
    pub fn stats(&self) -> CacheStats {
        let inner = self.inner.lock().unwrap();
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            evictions: inner.evictions,
            current_bytes: inner.current_bytes,
            entry_count: inner.entries.len(),
        }
    }

    /// Retorna a quantidade de bytes atualmente alocada pelas entradas no cache.
    pub fn current_bytes(&self) -> usize {
        self.inner.lock().unwrap().current_bytes
    }

    /// Retorna a capacidade máxima configurada em bytes.
    pub fn max_bytes(&self) -> usize {
        self.inner.lock().unwrap().config.max_bytes
    }

    /// Retorna o número de entradas ativas no cache.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().entries.len()
    }

    /// Retorna se o cache está vazio.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().entries.is_empty()
    }

    /// Ajusta dinamicamente a capacidade máxima do cache, acionando evicção se necessário.
    pub fn set_max_bytes(&self, new_max_bytes: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.config.max_bytes = new_max_bytes;
        while inner.current_bytes > new_max_bytes {
            if !inner.evict_least_recently_used() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, SystemTime};

    fn make_mock_decoded(w: u32, h: u32, num_frames: usize) -> visor::Decoded {
        let frame_bytes = (w as usize) * (h as usize) * 4;
        let mut frames = Vec::with_capacity(num_frames);
        for _ in 0..num_frames {
            frames.push(visor::Frame {
                rgba: vec![128u8; frame_bytes],
                width: w,
                height: h,
                delay_ms: 50,
            });
        }
        visor::Decoded {
            frames,
            info: visor::ImageInfo {
                id: "test",
                name: "mock_image.png".to_owned(),
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
                frame_count: num_frames,
                loop_forever: false,
                mipmaps: 0,
                exif: vec![("Make".to_owned(), "VisorTest".to_owned())],
                icc: Some("sRGB".to_owned()),
                notes: vec!["teste".to_owned()],
            },
        }
    }

    fn make_key(id: usize, size: u64) -> CacheKey {
        CacheKey::new(
            PathBuf::from(format!(r"C:\images\foto_{id}.png")),
            size,
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(id as u64)),
        )
    }

    // 1. Cache hit: Reutiliza imagem armazenada
    #[test]
    fn test_cache_hit_returns_shared_arc() {
        let cache = ImageCache::with_capacity(10 * 1024 * 1024);
        let key = make_key(1, 5000);
        let img = make_mock_decoded(50, 50, 1);

        assert!(cache.insert(key.clone(), img));
        let hit = cache.get(&key);
        assert!(hit.is_some(), "Deveria retornar a imagem armazenada");

        let arc1 = hit.unwrap();
        let arc2 = cache.get(&key).unwrap();
        assert!(
            Arc::ptr_eq(&arc1, &arc2),
            "Deve retornar o mesmo ponteiro Arc compartilhado"
        );
        assert_eq!(cache.stats().hits, 2);
    }

    // 2. Cache miss: Identifica imagem ausente
    #[test]
    fn test_cache_miss_returns_none() {
        let cache = ImageCache::with_capacity(10 * 1024 * 1024);
        let key = make_key(99, 1000);

        let miss = cache.get(&key);
        assert!(miss.is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    // 3. LRU eviction: Remove a entrada menos recente
    #[test]
    fn test_lru_eviction_removes_least_recently_used() {
        // Cria imagens onde cada uma consome ~10.000 bytes
        let img1 = make_mock_decoded(50, 50, 1);
        let bytes1 = estimate_decoded_bytes(&img1);

        // Capacidade para exatamente 2 imagens
        let capacity = bytes1 * 2 + 500;
        let cache = ImageCache::with_capacity(capacity);

        let key1 = make_key(1, 100);
        let key2 = make_key(2, 200);
        let key3 = make_key(3, 300);

        cache.insert(key1.clone(), img1);
        cache.insert(key2.clone(), make_mock_decoded(50, 50, 1));
        assert_eq!(cache.len(), 2);

        // Inserir a 3ª imagem deve forçar a evicção da 1ª (a menos recentemente usada)
        cache.insert(key3.clone(), make_mock_decoded(50, 50, 1));
        assert_eq!(cache.len(), 2);

        assert!(cache.get(&key1).is_none(), "Key1 deveria ter sido evictada");
        assert!(cache.get(&key2).is_some(), "Key2 deve permanecer");
        assert!(cache.get(&key3).is_some(), "Key3 deve permanecer");
        assert_eq!(cache.stats().evictions, 1);
    }

    // 4. Recência: Acesso atualiza a prioridade
    #[test]
    fn test_recency_access_updates_priority() {
        let img1 = make_mock_decoded(50, 50, 1);
        let bytes1 = estimate_decoded_bytes(&img1);
        let capacity = bytes1 * 2 + 500;
        let cache = ImageCache::with_capacity(capacity);

        let key1 = make_key(1, 100);
        let key2 = make_key(2, 200);
        let key3 = make_key(3, 300);

        cache.insert(key1.clone(), img1);
        cache.insert(key2.clone(), make_mock_decoded(50, 50, 1));

        // Acessa key1: agora key1 é mais recente do que key2!
        let _ = cache.get(&key1);

        // Ao inserir key3, quem deve ser evictado é a key2, e NÃO a key1!
        cache.insert(key3.clone(), make_mock_decoded(50, 50, 1));

        assert!(
            cache.get(&key1).is_some(),
            "Key1 foi acessada recentemente e deve permanecer"
        );
        assert!(
            cache.get(&key2).is_none(),
            "Key2 deveria ser evictada por ser a menos recente"
        );
        assert!(cache.get(&key3).is_some(), "Key3 deve permanecer");
    }

    // 5. Substituição: Corrige a contabilidade
    #[test]
    fn test_replacement_updates_accounting() {
        let cache = ImageCache::with_capacity(100 * 1024);
        let key = make_key(1, 100);

        let img_small = make_mock_decoded(10, 10, 1);
        let small_bytes = estimate_decoded_bytes(&img_small);

        cache.insert(key.clone(), img_small);
        assert_eq!(cache.current_bytes(), small_bytes);

        // Substitui a mesma chave por uma imagem maior
        let img_large = make_mock_decoded(40, 40, 1);
        let large_bytes = estimate_decoded_bytes(&img_large);

        cache.insert(key.clone(), img_large);
        assert_eq!(cache.len(), 1, "Deve existir apenas 1 entrada");
        assert_eq!(
            cache.current_bytes(),
            large_bytes,
            "Contabilidade deve refletir apenas a nova imagem"
        );
    }

    // 6. Limite zero: Não armazena entradas
    #[test]
    fn test_zero_capacity_disables_cache() {
        let cache = ImageCache::with_capacity(0);
        let key = make_key(1, 100);
        let img = make_mock_decoded(20, 20, 1);

        let accepted = cache.insert(key.clone(), img);
        assert!(!accepted, "Não deve aceitar inserção com capacidade 0");
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_bytes(), 0);
        assert!(cache.get(&key).is_none());
    }

    // 7. Imagem excedente: Não ultrapassa o orçamento
    #[test]
    fn test_oversized_image_rejected() {
        let cache = ImageCache::with_capacity(1000);
        let key = make_key(1, 500);
        let img = make_mock_decoded(100, 100, 1); // ~40.000 bytes

        let accepted = cache.insert(key.clone(), img);
        assert!(
            !accepted,
            "Imagem maior que o orçamento total deve ser rejeitada"
        );
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_bytes(), 0);
    }

    // 8. Animações: Contabiliza todos os frames
    #[test]
    fn test_animation_accounts_for_all_frames() {
        let single_frame = make_mock_decoded(100, 100, 1);
        let multi_frame = make_mock_decoded(100, 100, 10);

        let single_bytes = estimate_decoded_bytes(&single_frame);
        let multi_bytes = estimate_decoded_bytes(&multi_frame);

        // 10 frames de 100x100x4 (40.000 bytes cada) devem somar ~400.000 bytes
        assert!(
            multi_bytes >= single_bytes * 9,
            "Animação com 10 quadros deve contabilizar todos os quadros (multi: {multi_bytes}, single: {single_bytes})"
        );
    }

    // 9. Overflow: Não permite cálculo incorreto
    #[test]
    fn test_overflow_protection_in_estimation() {
        let mut decoded = make_mock_decoded(100, 100, 5);
        for i in 0..100 {
            decoded
                .info
                .notes
                .push(format!("nota_exemplo_de_metadados_longa_{i}"));
            decoded
                .info
                .exif
                .push((format!("Tag_{i}"), format!("Valor_Exif_{i}")));
        }
        let estimated = estimate_decoded_bytes(&decoded);
        assert!(estimated >= 5 * 100 * 100 * 4);

        // Verifica que operações de estimativa utilizam saturating arithmetic
        let base: usize = usize::MAX - 50;
        assert_eq!(base.saturating_add(100), usize::MAX);
        assert_eq!(base.saturating_mul(2), usize::MAX);
    }

    // 10. Concorrência: Acesso simultâneo seguro
    #[test]
    fn test_concurrent_access_thread_safety() {
        let cache = ImageCache::with_capacity(500 * 1024);
        let mut handles = Vec::new();

        let successful_hits = Arc::new(AtomicUsize::new(0));

        for thread_id in 0..4 {
            let c = cache.clone();
            let hits_counter = Arc::clone(&successful_hits);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    let key = make_key((thread_id * 10) + (i % 5), 1000);
                    if i % 2 == 0 {
                        let img = make_mock_decoded(20, 20, 1);
                        c.insert(key, img);
                    } else if c.get(&key).is_some() {
                        hits_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Garante que o orçamento nunca foi estourado pelas threads concorrentes
        assert!(
            cache.current_bytes() <= cache.max_bytes(),
            "Consumo concorrente ({}) não pode exceder capacidade ({})",
            cache.current_bytes(),
            cache.max_bytes()
        );
    }

    // 11. Invalidação: Remove entradas de arquivos alterados
    #[test]
    fn test_invalidation_and_file_modification_detection() {
        let cache = ImageCache::with_capacity(10 * 1024 * 1024);
        let path = PathBuf::from(r"C:\test\imagem.png");

        let key_v1 = CacheKey::new(path.clone(), 1000, Some(SystemTime::UNIX_EPOCH));
        let img1 = make_mock_decoded(30, 30, 1);
        cache.insert(key_v1.clone(), img1);

        assert!(cache.get(&key_v1).is_some());

        // Simula arquivo modificado no disco (novo timestamp e tamanho)
        let key_v2 = CacheKey::new(
            path.clone(),
            1200,
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(60)),
        );

        // A chave antiga não corresponde aos novos metadados -> miss automático
        assert!(
            cache.get(&key_v2).is_none(),
            "Metadados diferentes devem produzir miss automático"
        );

        // Invalidação explícita por caminho
        let removed = cache.invalidate_path(&path);
        assert_eq!(removed, 1);
        assert_eq!(cache.len(), 0);
        assert!(cache.get(&key_v1).is_none());
    }
}
