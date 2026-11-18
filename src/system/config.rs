//! Sistema de configurações persistentes do VISOR.
//!
//! Armazena preferências do usuário em `%LOCALAPPDATA%\Visor\settings.json`.
//! Suporta escrita atômica no Windows, recuperação de arquivos corrompidos e versionamento de schema.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::app::state::{CanvasBackground, InterpolationMode};
use crate::system::paths::local_dir;

/// Versão corrente do schema de configurações.
pub const CURRENT_SCHEMA_VERSION: u32 = 4;

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

fn default_true() -> bool {
    true
}

fn default_custom_color() -> [u8; 4] {
    [32, 32, 36, 255]
}

fn default_prompt_version() -> u32 {
    1
}

fn default_check_interval() -> u32 {
    24
}

/// Configurações do Update Center e verificação de atualizações.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateConfig {
    /// Se a verificação automática de atualizações na inicialização está habilitada.
    #[serde(default = "default_true")]
    pub automatic_check: bool,

    /// Intervalo em horas entre verificações automáticas (padrão: 24h).
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u32,

    /// Timestamp da última verificação bem-sucedida (formato ISO-8601).
    #[serde(default)]
    pub last_check: Option<String>,

    /// ETag HTTP da última consulta para suporte a 304 Not Modified.
    #[serde(default)]
    pub last_etag: Option<String>,

    /// Versão que o usuário escolheu ignorar (não notificar novamente sobre esta versão).
    #[serde(default)]
    pub ignored_version: Option<String>,

    /// Versão cujas novidades pós-atualização já foram apresentadas ao usuário.
    #[serde(default)]
    pub post_update_notes_version: Option<String>,

    /// JSON serializado da última release oficial identificada pelo updater.
    #[serde(default)]
    pub last_known_release_json: Option<String>,

    /// Tag da última release oficial identificada (ex: "v0.4.3").
    #[serde(default)]
    pub last_known_release_tag: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            automatic_check: true,
            check_interval_hours: default_check_interval(),
            last_check: None,
            last_etag: None,
            ignored_version: None,
            post_update_notes_version: None,
            last_known_release_json: None,
            last_known_release_tag: None,
        }
    }
}

/// Converte timestamp ISO-8601 UTC (ex: "2026-11-09T01:15:00Z") em segundos de época POSIX.
#[allow(dead_code)]
pub fn parse_iso_to_epoch_secs(iso: &str) -> Option<u64> {
    if iso.len() < 19 {
        return None;
    }
    let parts: Vec<&str> = iso.split('T').collect();
    if parts.len() != 2 {
        return None;
    }
    let d: Vec<u64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
    let t: Vec<u64> = parts[1]
        .trim_end_matches('Z')
        .split(':')
        .filter_map(|s| s.parse().ok())
        .collect();
    if d.len() != 3 || t.len() != 3 {
        return None;
    }
    let (y, m, day) = (d[0], d[1], d[2]);
    let (h, min, s) = (t[0], t[1], t[2]);

    let y = if m <= 2 { y.saturating_sub(1) } else { y };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era as i64) * 146097 + (doe as i64) - 719468;
    if days < 0 {
        return None;
    }
    Some((days as u64) * 86400 + h * 3600 + min * 60 + s)
}

/// Ação executada ao rolar a roda do mouse sobre o canvas.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseWheelAction {
    /// Amplia ou reduz a imagem ancorada no cursor (padrão).
    #[default]
    Zoom,
    /// Avança para a próxima imagem ou retrocede para a anterior na pasta.
    Navigate,
}

impl MouseWheelAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Zoom => "Zoom na imagem (padrão)",
            Self::Navigate => "Navegar entre imagens",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Zoom => {
                "Girar a roda amplia ou reduz a imagem sob o cursor (Ctrl não é necessário)."
            }
            Self::Navigate => {
                "Girar a roda avança ou volta imagens na pasta (Ctrl+Roda sempre aplica zoom)."
            }
        }
    }
}

/// Modo de exibição e zoom padrão ao abrir ou navegar entre imagens.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultZoomMode {
    /// Abre a imagem em 100% (tamanho real 1:1, pixel por pixel).
    #[default]
    Actual100,
    /// Ajusta a imagem para caber inteiramente na janela.
    Fit,
}

/// Configurações persistentes do VISOR.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    /// Versão do schema para controle de migração e retrocompatibilidade.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Ação padrão da roda do mouse sobre o viewport.
    #[serde(default)]
    pub mouse_wheel_action: MouseWheelAction,

    /// Modo de zoom padrão ao abrir ou navegar entre imagens (100% ou Ajustar).
    #[serde(default)]
    pub default_zoom: DefaultZoomMode,

    /// Modo de interpolação inicial ao renderizar imagens.
    #[serde(default)]
    pub interpolation: InterpolationMode,

    /// Modo de fundo inicial para áreas transparentes.
    #[serde(default)]
    pub background: CanvasBackground,

    /// Cor personalizada de fundo (RGBA) caso o usuário selecione `CanvasBackground::Custom`.
    #[serde(default = "default_custom_color")]
    pub custom_color: [u8; 4],

    /// Se a tira de miniaturas inferior (Filmstrip) deve iniciar visível.
    #[serde(default = "default_true")]
    pub show_filmstrip: bool,

    /// Se o painel lateral de informações técnicas deve iniciar aberto.
    #[serde(default)]
    pub show_info: bool,

    /// Se o banner de primeiro uso para associações de arquivos foi dispensado permanentemente ("Não perguntar novamente").
    #[serde(default)]
    pub association_prompt_dismissed: bool,

    /// Versão do prompt de associação para controle de futuras reapresentações.
    #[serde(default = "default_prompt_version")]
    pub association_prompt_version: u32,

    /// Lista de extensões ignoradas pelo usuário (não perguntar para definir como padrão).
    #[serde(default)]
    pub ignored_default_extensions: Vec<String>,

    /// Configurações do Update Center e verificação de atualizações.
    #[serde(default)]
    pub updates: UpdateConfig,

    /// Idioma ativo da interface do VISOR.
    #[serde(default)]
    pub language: crate::system::i18n::AppLanguage,

    /// Indica se o arquivo carregado possui versão de schema futura.
    /// Quando verdadeiro, a sobrescrita automática do arquivo original é estritamente bloqueada.
    #[serde(skip)]
    pub is_future_version: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            mouse_wheel_action: MouseWheelAction::Zoom,
            default_zoom: DefaultZoomMode::Actual100,
            interpolation: InterpolationMode::Auto,
            background: CanvasBackground::DarkChecker,
            custom_color: default_custom_color(),
            show_filmstrip: true,
            show_info: false,
            association_prompt_dismissed: false,
            association_prompt_version: default_prompt_version(),
            ignored_default_extensions: Vec::new(),
            updates: UpdateConfig::default(),
            language: crate::system::i18n::AppLanguage::System,
            is_future_version: false,
        }
    }
}

impl AppSettings {
    /// Retorna se uma determinada extensão está marcada como ignorada nas preferências.
    pub fn is_extension_ignored(&self, ext: &str) -> bool {
        let clean = crate::system::assoc::normalize_extension(ext);
        self.ignored_default_extensions
            .iter()
            .any(|e| crate::system::assoc::normalize_extension(e) == clean)
    }

    /// Adiciona uma extensão à lista de ignoradas com normalização e sem duplicatas.
    pub fn ignore_extension(&mut self, ext: &str) {
        let clean = crate::system::assoc::normalize_extension(ext);
        if !clean.is_empty() && !self.is_extension_ignored(&clean) {
            self.ignored_default_extensions.push(clean);
        }
    }

    /// Remove uma extensão da lista de ignoradas ("Voltar a perguntar").
    pub fn unignore_extension(&mut self, ext: &str) {
        let clean = crate::system::assoc::normalize_extension(ext);
        self.ignored_default_extensions
            .retain(|e| crate::system::assoc::normalize_extension(e) != clean);
    }

    /// Determina se uma verificação automática de atualizações deve ser disparada.
    #[allow(dead_code)]
    pub fn should_check_for_updates(&self) -> bool {
        if !self.updates.automatic_check {
            return false;
        }
        let Some(last) = &self.updates.last_check else {
            return true;
        };

        if let Some(epoch) = parse_iso_to_epoch_secs(last) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let interval_secs = (self.updates.check_interval_hours as u64) * 3600;
            now >= epoch.saturating_add(interval_secs)
        } else {
            true
        }
    }

    /// Retorna o caminho canônico do arquivo de configurações (`%LOCALAPPDATA%\Visor\settings.json`).
    pub fn settings_path() -> PathBuf {
        local_dir().join("settings.json")
    }

    /// Carrega as configurações do caminho padrão do aplicativo.
    ///
    /// Se o arquivo não existir, retorna `(AppSettings::default(), None)`.
    /// Se o arquivo estiver corrompido, salva uma cópia como `.corrupted`,
    /// registra uma mensagem descritiva de recuperação e retorna os valores padrão sem sobrescrever o arquivo original.
    pub fn load() -> (Self, Option<String>) {
        let path = Self::settings_path();
        let is_first_run = !path.exists();
        let (mut settings, notice) = Self::load_from_path(&path);
        if is_first_run {
            // Em uma instalação limpa (primeiro uso), registra a versão atual para não exibir pop-up falso de atualização
            settings.updates.post_update_notes_version =
                Some(env!("CARGO_PKG_VERSION").to_string());
        }
        (settings, notice)
    }

    /// Carrega as configurações a partir de um caminho explícito (útil para testes isolados).
    pub fn load_from_path(path: &Path) -> (Self, Option<String>) {
        if !path.exists() {
            return (Self::default(), None);
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => {
                let msg = format!(
                    "Não foi possível ler as configurações de {}: {err}",
                    path.display()
                );
                return (Self::default(), Some(msg));
            }
        };

        match serde_json::from_str::<AppSettings>(&content) {
            Ok(mut settings) => {
                if settings.schema_version > CURRENT_SCHEMA_VERSION {
                    let future_v = settings.schema_version;
                    let mut safe_settings = AppSettings::default();
                    safe_settings.schema_version = future_v;
                    safe_settings.is_future_version = true;
                    let msg = format!(
                        "Versão futura de schema detectada (v{}, suportada: v{}). O arquivo original foi preservado com todos os dados futuros e padrões seguros foram adotados em memória.",
                        future_v, CURRENT_SCHEMA_VERSION
                    );
                    (safe_settings, Some(msg))
                } else {
                    if settings.schema_version < 3 {
                        // Migração de schema v1/v2 para v3:
                        // Se o usuário dispensou o prompt no passado, preserva essa escolha
                        // preenchendo as extensões como ignoradas para não reapresentar banners.
                        if settings.association_prompt_dismissed
                            && settings.ignored_default_extensions.is_empty()
                        {
                            settings.ignored_default_extensions =
                                crate::system::assoc::target_extensions()
                                    .into_iter()
                                    .map(|s| s.to_string())
                                    .collect();
                        }
                    }

                    if settings.schema_version < 4 {
                        // Migração de schema v3 para v4 (Update Center):
                        // Preserva todas as preferências do usuário e atualiza a versão do schema.
                        settings.schema_version = CURRENT_SCHEMA_VERSION;
                    }

                    // Normaliza e deduplica extensões ignoradas
                    let mut seen = std::collections::HashSet::new();
                    settings.ignored_default_extensions.retain(|ext| {
                        let norm = crate::system::assoc::normalize_extension(ext);
                        if norm.is_empty() || seen.contains(&norm) {
                            false
                        } else {
                            seen.insert(norm);
                            true
                        }
                    });
                    for ext in &mut settings.ignored_default_extensions {
                        *ext = crate::system::assoc::normalize_extension(ext);
                    }

                    (settings, None)
                }
            }
            Err(err) => {
                // Arquivo corrompido: cria backup de diagnóstico e não encerra o aplicativo
                let corrupted_path = path.with_extension("corrupted");
                let _ = fs::copy(path, &corrupted_path);

                let msg = format!(
                    "Arquivo de configurações corrompido ({}). Backup salvo em {}. Padrões restaurados.",
                    err,
                    corrupted_path.display()
                );
                (Self::default(), Some(msg))
            }
        }
    }

    /// Salva as configurações de forma atômica no caminho padrão (`%LOCALAPPDATA%\Visor\settings.json`).
    pub fn save(&self) -> Result<(), String> {
        Self::save_to_path(self, &Self::settings_path())
    }

    /// Salva as configurações de forma atômica no caminho especificado.
    pub fn save_to_path(&self, target_path: &Path) -> Result<(), String> {
        if self.is_future_version {
            return Err(format!(
                "Gravação bloqueada: arquivo possui versão futura de schema (v{}) e está protegido contra sobrescrita automática.",
                self.schema_version
            ));
        }

        static SAVE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = SAVE_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let parent = target_path
            .parent()
            .ok_or_else(|| "Caminho inválido: sem diretório pai".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|e| format!("Falha ao criar diretório {}: {e}", parent.display()))?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Falha na serialização JSON: {e}"))?;

        let temp_path = target_path.with_extension("tmp");

        // 1. Escreve no arquivo temporário e força flush para disco (sync_all)
        {
            let mut file = File::create(&temp_path).map_err(|e| {
                format!(
                    "Falha ao criar arquivo temporário {}: {e}",
                    temp_path.display()
                )
            })?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("Falha ao gravar arquivo temporário: {e}"))?;
            file.sync_all()
                .map_err(|e| format!("Falha ao sincronizar dados em disco: {e}"))?;
        }

        // 2. Substituição atômica no Windows
        let result = atomic_replace(&temp_path, target_path);

        // Se falhar, garante a limpeza do arquivo temporário
        if result.is_err() && temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        result
    }
}

use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};

enum WriterMsg {
    Save(AppSettings, Option<PathBuf>),
    Shutdown,
}

/// Gerenciador de gravações assíncronas serializadas de configurações.
///
/// Garante:
/// 1. Gravações 100% serializadas em um worker thread dedicado.
/// 2. Ausência total de concorrência sobre `settings.json.tmp`.
/// 3. Coalescência: drena atualizações intermediárias de rajadas para gravar sempre o estado mais recente.
/// 4. Nenhuma gravação antiga sobrescreve uma configuração mais recente.
/// 5. Encerramento gracioso aguardando a finalização da gravação final.
pub struct SettingsWriter {
    tx: SyncSender<WriterMsg>,
    handle: Option<JoinHandle<()>>,
}

impl SettingsWriter {
    pub fn new() -> Self {
        Self::new_internal(None)
    }

    #[allow(dead_code)]
    pub fn new_with_path(target_path: PathBuf) -> Self {
        Self::new_internal(Some(target_path))
    }

    fn new_internal(custom_target: Option<PathBuf>) -> Self {
        let (tx, rx): (SyncSender<WriterMsg>, Receiver<WriterMsg>) = mpsc::sync_channel(64);
        let default_target = custom_target.unwrap_or_else(AppSettings::settings_path);
        let handle = thread::Builder::new()
            .name("visor-settings-writer".to_string())
            .spawn(move || {
                while let Ok(msg) = rx.recv() {
                    match msg {
                        WriterMsg::Save(settings, override_path) => {
                            let mut latest_settings = settings;
                            let mut latest_path = override_path;
                            let mut should_exit = false;

                            // Coalescência: drena mensagens acumuladas para processar apenas o snapshot mais recente
                            while let Ok(next) = rx.try_recv() {
                                match next {
                                    WriterMsg::Save(newer_settings, newer_path) => {
                                        latest_settings = newer_settings;
                                        if newer_path.is_some() {
                                            latest_path = newer_path;
                                        }
                                    }
                                    WriterMsg::Shutdown => {
                                        should_exit = true;
                                        break;
                                    }
                                }
                            }

                            // Grava o estado mais recente
                            let target = latest_path.as_deref().unwrap_or(&default_target);
                            let _ = latest_settings.save_to_path(target);

                            if should_exit {
                                break;
                            }
                        }
                        WriterMsg::Shutdown => break,
                    }
                }
            })
            .ok();

        Self { tx, handle }
    }

    /// Agenda a gravação assíncrona da configuração.
    pub fn schedule_save(&self, settings: AppSettings) {
        let _ = self.tx.send(WriterMsg::Save(settings, None));
    }

    /// Agenda a gravação em um caminho específico (útil para testes isolados).
    #[allow(dead_code)]
    pub fn schedule_save_to_path(&self, settings: AppSettings, path: PathBuf) {
        let _ = self.tx.send(WriterMsg::Save(settings, Some(path)));
    }

    /// Encerra o worker aguardando o flush completo da gravação final para disco.
    pub fn shutdown_and_wait(&mut self) {
        let _ = self.tx.send(WriterMsg::Shutdown);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for SettingsWriter {
    fn drop(&mut self) {
        self.shutdown_and_wait();
    }
}

/// Realiza substituição atômica de arquivos.
///
/// No Windows, utiliza `ReplaceFileW` para arquivos existentes e `MoveFileExW`
/// com `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`. Em caso de fallback
/// ou outras plataformas, utiliza `std::fs::rename`.
fn atomic_replace(temp: &Path, target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        if target.exists() {
            // Tenta ReplaceFileW para substituição atômica segura
            if win32::replace_file(target, temp) {
                return Ok(());
            }
        }

        // Se target não existia ou ReplaceFileW falhou, usa MoveFileExW
        if win32::move_file_atomic(temp, target) {
            return Ok(());
        }
    }

    // Fallback padrão via std::fs
    fs::rename(temp, target).map_err(|e| {
        format!(
            "Falha ao substituir arquivo de configurações de {} para {}: {e}",
            temp.display(),
            target.display()
        )
    })
}

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x00000008;

    unsafe extern "system" {
        fn MoveFileExW(
            lpExistingFileName: *const u16,
            lpNewFileName: *const u16,
            dwFlags: u32,
        ) -> i32;

        fn ReplaceFileW(
            lpReplacedFileName: *const u16,
            lpReplacementFileName: *const u16,
            lpBackupFileName: *const u16,
            dwReplaceFlags: u32,
            lpExclude: *mut c_void,
            lpReserved: *mut c_void,
        ) -> i32;
    }

    fn to_wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn replace_file(target: &Path, replacement: &Path) -> bool {
        let w_target = to_wide(target);
        let w_replacement = to_wide(replacement);
        unsafe {
            ReplaceFileW(
                w_target.as_ptr(),
                w_replacement.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) != 0
        }
    }

    pub fn move_file_atomic(source: &Path, destination: &Path) -> bool {
        let w_source = to_wide(source);
        let w_destination = to_wide(destination);
        unsafe {
            MoveFileExW(
                w_source.as_ptr(),
                w_destination.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            ) != 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_values() {
        let settings = AppSettings::default();
        assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(settings.mouse_wheel_action, MouseWheelAction::Zoom);
        assert_eq!(settings.default_zoom, DefaultZoomMode::Actual100);
        assert_eq!(settings.interpolation, InterpolationMode::Auto);
        assert_eq!(settings.background, CanvasBackground::DarkChecker);
        assert!(settings.show_filmstrip);
        assert!(!settings.show_info);
        assert!(!settings.association_prompt_dismissed);
        assert_eq!(settings.association_prompt_version, 1);
    }

    #[test]
    fn test_mouse_wheel_action_labels() {
        assert_eq!(MouseWheelAction::Zoom.label(), "Zoom na imagem (padrão)");
        assert_eq!(MouseWheelAction::Navigate.label(), "Navegar entre imagens");
        assert!(
            MouseWheelAction::Navigate
                .description()
                .contains("Ctrl+Roda sempre aplica zoom")
        );
    }

    #[test]
    fn test_serialize_and_deserialize_json() {
        let original = AppSettings {
            schema_version: 2,
            mouse_wheel_action: MouseWheelAction::Navigate,
            default_zoom: DefaultZoomMode::Actual100,
            interpolation: InterpolationMode::Nearest,
            background: CanvasBackground::White,
            custom_color: default_custom_color(),
            show_filmstrip: false,
            show_info: true,
            association_prompt_dismissed: false,
            association_prompt_version: 1,
            ignored_default_extensions: Vec::new(),
            updates: UpdateConfig::default(),
            language: crate::system::i18n::AppLanguage::System,
            is_future_version: false,
        };

        let json = serde_json::to_string(&original).expect("serialização falhou");
        assert!(json.contains("\"mouse_wheel_action\":\"navigate\""));
        assert!(json.contains("\"default_zoom\":\"actual100\""));
        assert!(json.contains("\"interpolation\":\"nearest\""));
        assert!(json.contains("\"background\":\"white\""));

        let restored: AppSettings = serde_json::from_str(&json).expect("desserialização falhou");
        assert_eq!(restored, original);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("visor_cfg_test_{}", std::process::id()));
        let settings_file = temp_dir.join("settings.json");

        let to_save = AppSettings {
            schema_version: CURRENT_SCHEMA_VERSION,
            mouse_wheel_action: MouseWheelAction::Navigate,
            default_zoom: DefaultZoomMode::Fit,
            interpolation: InterpolationMode::Linear,
            background: CanvasBackground::Black,
            custom_color: default_custom_color(),
            show_filmstrip: true,
            show_info: true,
            association_prompt_dismissed: false,
            association_prompt_version: 1,
            ignored_default_extensions: Vec::new(),
            updates: UpdateConfig::default(),
            language: crate::system::i18n::AppLanguage::System,
            is_future_version: false,
        };

        to_save
            .save_to_path(&settings_file)
            .expect("salvamento falhou");
        assert!(settings_file.exists());

        let (loaded, notice) = AppSettings::load_from_path(&settings_file);
        assert!(notice.is_none(), "não deveria haver aviso: {notice:?}");
        assert_eq!(loaded, to_save);

        // Teste de sobrescrita atômica subsequente
        let mut modified = to_save.clone();
        modified.mouse_wheel_action = MouseWheelAction::Zoom;
        modified
            .save_to_path(&settings_file)
            .expect("segundo salvamento falhou");

        let (loaded_modified, notice2) = AppSettings::load_from_path(&settings_file);
        assert!(notice2.is_none());
        assert_eq!(loaded_modified.mouse_wheel_action, MouseWheelAction::Zoom);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_non_existent_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_nonexist_{}", std::process::id()));
        let non_existent = temp_dir.join("does_not_exist.json");

        let (settings, notice) = AppSettings::load_from_path(&non_existent);
        assert_eq!(settings, AppSettings::default());
        assert!(notice.is_none());
    }

    #[test]
    fn test_load_corrupted_file_recovery() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_corrupt_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let corrupt_file = temp_dir.join("settings.json");

        // Escreve conteúdo JSON corrompido / truncado
        fs::write(&corrupt_file, b"{ \"mouse_wheel_action\": \"inval").expect("gravação falhou");

        let (settings, notice) = AppSettings::load_from_path(&corrupt_file);

        // Deve retornar configurações padrão
        assert_eq!(settings, AppSettings::default());

        // Deve emitir aviso amigável
        let msg = notice.expect("deveria retornar aviso sobre arquivo corrompido");
        assert!(msg.contains("Backup salvo em"));
        assert!(msg.contains("Padrões restaurados"));

        // O arquivo de backup .corrupted deve existir com o conteúdo original intacto
        let backup_path = corrupt_file.with_extension("corrupted");
        assert!(backup_path.exists());
        let backup_content = fs::read(&backup_path).expect("leitura do backup falhou");
        assert_eq!(backup_content, b"{ \"mouse_wheel_action\": \"inval");

        // O arquivo original NÃO deve ter sido deletado
        assert!(corrupt_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_forward_compatibility_unknown_and_missing_fields() {
        let json_with_future_fields = r#"{
            "schema_version": 2,
            "mouse_wheel_action": "navigate",
            "future_experimental_feature": true,
            "another_unknown_field": [1, 2, 3]
        }"#;

        let parsed: AppSettings = serde_json::from_str(json_with_future_fields)
            .expect("deveria ignorar campos desconhecidos");

        assert_eq!(parsed.schema_version, 2);
        assert_eq!(parsed.mouse_wheel_action, MouseWheelAction::Navigate);
        // Campos ausentes recebem defaults
        assert_eq!(parsed.interpolation, InterpolationMode::Auto);
        assert_eq!(parsed.background, CanvasBackground::DarkChecker);
        assert!(parsed.show_filmstrip);
        assert!(!parsed.show_info);
    }

    #[test]
    fn test_future_schema_version_downgrade_and_preservation() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_future_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let future_file = temp_dir.join("settings.json");

        let raw_future_json = r#"{
            "schema_version": 99,
            "mouse_wheel_action": "navigate",
            "interpolation": "nearest",
            "future_ai_upscaling": true,
            "future_cloud_sync_url": "https://visor.example.com/sync"
        }"#;

        fs::write(&future_file, raw_future_json.as_bytes()).expect("gravação falhou");

        // 1. Carregamento detecta versão futura
        let (mut loaded, notice) = AppSettings::load_from_path(&future_file);
        assert!(
            loaded.is_future_version,
            "deve sinalizar versão futura de schema"
        );
        assert_eq!(loaded.schema_version, 99);
        let msg = notice.expect("deve retornar aviso sobre versão futura");
        assert!(msg.contains("Versão futura de schema detectada"));

        // 2. Modificações em memória
        loaded.mouse_wheel_action = MouseWheelAction::Zoom;

        // 3. Tentativa de salvamento deve ser bloqueada para não corromper o arquivo futuro
        let save_result = loaded.save_to_path(&future_file);
        assert!(
            save_result.is_err(),
            "deve impedir sobrescrita de schema futuro"
        );

        // 4. Arquivo em disco deve permanecer 100% intacto com todos os campos desconhecidos
        let content_on_disk = fs::read_to_string(&future_file).expect("leitura falhou");
        assert_eq!(content_on_disk, raw_future_json);

        // 5. Ação explícita de redefinição / padrões permite salvar
        let reset_defaults = AppSettings::default();
        assert!(!reset_defaults.is_future_version);
        assert!(reset_defaults.save_to_path(&future_file).is_ok());

        let (after_reset, reset_notice) = AppSettings::load_from_path(&future_file);
        assert!(reset_notice.is_none());
        assert_eq!(after_reset.schema_version, CURRENT_SCHEMA_VERSION);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_custom_color_mandated_values_rgba_integrity_and_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_mandated_colors_{}", std::process::id()));
        let settings_file = temp_dir.join("settings.json");

        let test_cases: &[[u8; 4]] = &[
            [24, 128, 240, 215],
            [255, 0, 0, 128],
            [0, 255, 0, 64],
            [0, 0, 0, 0],
            [255, 255, 255, 255],
        ];

        for &expected_rgba in test_cases {
            let [r, g, b, a] = expected_rgba;
            let color32 = eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a);
            assert_eq!(
                color32.to_srgba_unmultiplied(),
                expected_rgba,
                "to_srgba_unmultiplied() deve recuperar exatamente o valor RGBA convencional para {:?}",
                expected_rgba
            );

            let mut settings = AppSettings::default();
            settings.custom_color = expected_rgba;
            settings.background = CanvasBackground::Custom(color32);

            // Salva no disco
            settings
                .save_to_path(&settings_file)
                .expect("salvamento falhou");

            // Recarrega do disco
            let (loaded, _) = AppSettings::load_from_path(&settings_file);
            assert_eq!(
                loaded.custom_color, expected_rgba,
                "custom_color persistido deve coincidir exatamente com {:?}",
                expected_rgba
            );
            assert_eq!(
                loaded.background,
                CanvasBackground::Custom(color32),
                "background persistido deve ser CanvasBackground::Custom({:?})",
                color32
            );

            // Verifica JSON cru para garantir convenção desmultiplicada
            let json_str = fs::read_to_string(&settings_file).expect("leitura do JSON falhou");
            let v: serde_json::Value =
                serde_json::from_str(&json_str).expect("parse do JSON falhou");
            let color_in_json = v.get("custom_color").expect("campo custom_color ausente");
            let arr = color_in_json
                .as_array()
                .expect("custom_color deve ser array");
            let json_rgba: Vec<u8> = arr
                .iter()
                .map(|item| item.as_u64().unwrap() as u8)
                .collect();
            assert_eq!(
                json_rgba.as_slice(),
                expected_rgba,
                "custom_color em settings.json deve ser armazenado desmultiplicado {:?}",
                expected_rgba
            );
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_background_cycling_preserves_custom_color() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_bg_preserve_{}", std::process::id()));
        let settings_file = temp_dir.join("settings.json");

        let original_custom = [24, 128, 240, 215];
        let mut settings = AppSettings::default();
        settings.custom_color = original_custom;
        settings.background =
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(
                original_custom[0],
                original_custom[1],
                original_custom[2],
                original_custom[3],
            ));
        settings
            .save_to_path(&settings_file)
            .expect("salvamento falhou");

        // Alterna entre outros modos de fundo e salva sucessivamente
        let other_backgrounds = [
            CanvasBackground::DarkChecker,
            CanvasBackground::LightChecker,
            CanvasBackground::Black,
            CanvasBackground::White,
        ];

        for other_bg in other_backgrounds {
            let (mut current, _) = AppSettings::load_from_path(&settings_file);
            current.background = other_bg;
            // custom_color NÃO deve ser modificado
            current
                .save_to_path(&settings_file)
                .expect("salvamento com outro fundo falhou");

            let (reloaded, _) = AppSettings::load_from_path(&settings_file);
            assert_eq!(reloaded.background, other_bg);
            assert_eq!(
                reloaded.custom_color, original_custom,
                "custom_color deve ser estritamente preservado ao alternar para {:?}",
                other_bg
            );
        }

        // Reativa Cor Personalizada restaurando de custom_color
        let (mut restored, _) = AppSettings::load_from_path(&settings_file);
        let [r, g, b, a] = restored.custom_color;
        restored.background =
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a));
        restored
            .save_to_path(&settings_file)
            .expect("salvamento restauração falhou");

        let (final_loaded, _) = AppSettings::load_from_path(&settings_file);
        assert_eq!(final_loaded.custom_color, original_custom);
        assert_eq!(
            final_loaded.background,
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(
                original_custom[0],
                original_custom[1],
                original_custom[2],
                original_custom[3],
            ))
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_custom_color_rgba_persistence_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("visor_cfg_color_{}", std::process::id()));
        let settings_file = temp_dir.join("settings.json");

        let custom_rgba = [15, 85, 175, 220];
        let mut settings = AppSettings::default();
        settings.custom_color = custom_rgba;
        settings.background =
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(
                custom_rgba[0],
                custom_rgba[1],
                custom_rgba[2],
                custom_rgba[3],
            ));

        // 1. Salva
        settings
            .save_to_path(&settings_file)
            .expect("salvamento falhou");

        // 2. Recarrega e valida integridade RGBA
        let (loaded, _) = AppSettings::load_from_path(&settings_file);
        assert_eq!(loaded.custom_color, custom_rgba);
        assert_eq!(
            loaded.background,
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(
                custom_rgba[0],
                custom_rgba[1],
                custom_rgba[2],
                custom_rgba[3]
            ))
        );

        // 3. Modifica outra preferência e salva novamente
        let mut modified = loaded;
        modified.mouse_wheel_action = MouseWheelAction::Navigate;
        modified.interpolation = InterpolationMode::Nearest;
        modified
            .save_to_path(&settings_file)
            .expect("segundo salvamento falhou");

        // 4. Confirma que a cor RGBA continua preservada
        let (loaded_again, _) = AppSettings::load_from_path(&settings_file);
        assert_eq!(loaded_again.mouse_wheel_action, MouseWheelAction::Navigate);
        assert_eq!(loaded_again.interpolation, InterpolationMode::Nearest);
        assert_eq!(loaded_again.custom_color, custom_rgba);
        assert_eq!(
            loaded_again.background,
            CanvasBackground::Custom(eframe::egui::Color32::from_rgba_unmultiplied(
                custom_rgba[0],
                custom_rgba[1],
                custom_rgba[2],
                custom_rgba[3]
            ))
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_rapid_modifications_and_sequential_saving() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_writer_{}", std::process::id()));
        let settings_file = temp_dir.join("settings.json");

        let mut writer = SettingsWriter::new_with_path(settings_file.clone());

        // Dispara rajada de atualizações rápidas concorrentes
        for i in 0..10 {
            let s = AppSettings {
                schema_version: CURRENT_SCHEMA_VERSION,
                mouse_wheel_action: if i % 2 == 0 {
                    MouseWheelAction::Zoom
                } else {
                    MouseWheelAction::Navigate
                },
                default_zoom: DefaultZoomMode::Actual100,
                interpolation: InterpolationMode::Auto,
                background: CanvasBackground::DarkChecker,
                custom_color: [i as u8, 0, 0, 255],
                show_filmstrip: true,
                show_info: false,
                association_prompt_dismissed: false,
                association_prompt_version: 1,
                ignored_default_extensions: Vec::new(),
                updates: UpdateConfig::default(),
                language: crate::system::i18n::AppLanguage::System,
                is_future_version: false,
            };
            writer.schedule_save(s);
        }

        // Aguarda término da gravação
        writer.shutdown_and_wait();

        // Valida que o arquivo final existe e é um JSON válido
        assert!(settings_file.exists());
        let (loaded, notice) = AppSettings::load_from_path(&settings_file);
        assert!(notice.is_none());
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);

        // O arquivo .tmp deve ter sido limpo
        assert!(!settings_file.with_extension("tmp").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_schema_v1_to_v2_migration_and_association_prompt_persistence() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_cfg_migration_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let settings_file = temp_dir.join("settings.json");

        // 1. Cria um arquivo representativo de versão 1 (sem os novos campos de associação)
        let v1_json = r#"{
            "schema_version": 1,
            "mouse_wheel_action": "navigate",
            "interpolation": "nearest",
            "background": "dark_checker",
            "custom_color": [32, 32, 36, 255],
            "show_filmstrip": true,
            "show_info": false
        }"#;
        fs::write(&settings_file, v1_json.as_bytes()).expect("gravação de v1 falhou");

        // 2. Carrega: deve migrar transparentemente para CURRENT_SCHEMA_VERSION (3) com valores padrão seguros
        let (loaded, notice) = AppSettings::load_from_path(&settings_file);
        assert!(notice.is_none());
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(loaded.mouse_wheel_action, MouseWheelAction::Navigate);
        assert_eq!(loaded.interpolation, InterpolationMode::Nearest);
        assert!(
            !loaded.association_prompt_dismissed,
            "deve iniciar como não dispensado"
        );
        assert_eq!(loaded.association_prompt_version, 1);

        // 3. Usuário escolhe 'Não perguntar novamente' no banner de associação
        let mut modified = loaded;
        modified.association_prompt_dismissed = true;
        modified
            .save_to_path(&settings_file)
            .expect("salvamento v3 falhou");

        // 4. Recarrega do disco e confirma persistência da preferência
        let (reloaded, _) = AppSettings::load_from_path(&settings_file);
        assert_eq!(reloaded.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(
            reloaded.association_prompt_dismissed,
            "preferência de dispensar banner deve ser persistida"
        );
        assert_eq!(reloaded.association_prompt_version, 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
