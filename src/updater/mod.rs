//! Sistema completo de atualização (Update Center) do VISOR.
//!
//! Coordena a consulta a releases no GitHub, verificação SemVer, downloads com progresso,
//! validação estrita de integridade SHA-256 e exibição offline de novidades pós-atualização.

pub mod download;
pub mod github;
pub mod install;
pub mod integrity;
pub mod version;
pub mod whats_new;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::SystemTime;

pub use download::{DownloadProgress, DownloadStatus};
pub use github::{GitHubAsset, GitHubCheckResult, GitHubRelease, UpdateCheckError};
pub use whats_new::VersionNotes;

use crate::workers::ThreadPool;

/// Eventos assíncronos transmitidos das threads de trabalho para a interface egui.
#[derive(Clone, Debug)]
pub enum UpdateEvent {
    /// Consulta à API iniciada em segundo plano.
    CheckStarted,
    /// Consulta finalizada com sucesso ou erro.
    CheckFinished(Result<CheckOutcome, UpdateCheckError>),
    /// Atualização contínua do progresso do download do pacote.
    DownloadProgress(DownloadProgress),
    /// Download concluído e validado criptograficamente com sucesso.
    DownloadCompleted(PathBuf),
    /// Falha durante o download ou rejeição de integridade.
    DownloadFailed(String),
}

/// Resultado processado de uma verificação de release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckOutcome {
    /// O aplicativo já está na versão mais recente disponível no canal estável.
    UpToDate { checked_at: String },
    /// Nenhuma alteração no servidor (HTTP 304 Not Modified).
    NotModified { checked_at: String },
    /// Uma nova versão estável e oficial está disponível para instalação.
    UpdateAvailable {
        release: GitHubRelease,
        checked_at: String,
    },
}

/// Estado reativo do Update Center mantido na memória do aplicativo (`VisorApp`).
#[derive(Clone, Debug, Default)]
pub struct UpdaterState {
    /// Indica se há uma verificação de rede em andamento no momento.
    pub is_checking: bool,
    /// Último resultado de verificação processado.
    pub last_outcome: Option<CheckOutcome>,
    /// Mensagem do último erro de consulta ou download (se houver).
    pub last_error: Option<String>,
    /// Release disponível para atualização (se houver e não tiver sido ignorada).
    pub available_release: Option<GitHubRelease>,
    /// Tag da release temporariamente adiada na sessão atual ("Lembrar mais tarde").
    pub postponed_tag: Option<String>,
    /// Progresso atual do download ativo (se houver).
    pub download_progress: Option<DownloadProgress>,
    /// Sinalizador atômico para solicitar o cancelamento do download ativo.
    pub download_cancel_flag: Option<Arc<AtomicBool>>,
    /// Controla a visibilidade da janela modal de atualização / download.
    pub show_update_window: bool,
    /// Controla a visibilidade da janela modal de novidades da versão (What's New).
    pub show_whats_new_window: bool,
    /// Notas estruturadas atualmente carregadas para a janela de novidades.
    pub whats_new_notes: Option<VersionNotes>,
}

impl UpdaterState {
    /// Retorna se há um download em andamento no momento.
    #[allow(dead_code)]
    pub fn is_downloading(&self) -> bool {
        matches!(
            self.download_progress.as_ref().map(|p| &p.status),
            Some(
                DownloadStatus::Preparing | DownloadStatus::Downloading | DownloadStatus::Verifying
            )
        )
    }

    /// Cancela o download ativo caso esteja em execução.
    pub fn cancel_active_download(&mut self) {
        if let Some(flag) = &self.download_cancel_flag {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

/// Gera uma string com a data e hora atual em formato ISO-8601 UTC simples (ex: "2026-11-09T01:15:00Z").
pub fn current_iso8601_timestamp() -> String {
    let now = SystemTime::now();
    let epoch_secs = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Conversão direta de segundos POSIX para YYYY-MM-DDTHH:MM:SSZ
    let sec = (epoch_secs % 60) as u32;
    let min = ((epoch_secs / 60) % 60) as u32;
    let hour = ((epoch_secs / 3600) % 24) as u32;
    let days = (epoch_secs / 86400) as i64;

    // Algoritmo civil para dias desde 1970-01-01
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let final_year = if m <= 2 { y + 1 } else { y };

    format!("{final_year:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Converte um timestamp ISO-8601 em exibição amigável em português (ex: "09/11/2026 às 01:15").
pub fn format_timestamp_display(iso: &str) -> String {
    if iso.len() >= 16 && iso.contains('T') {
        let parts: Vec<&str> = iso.split('T').collect();
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        if date_parts.len() == 3 {
            let time_str = &parts[1][..5.min(parts[1].len())];
            return format!(
                "{}/{}/{} às {}",
                date_parts[2], date_parts[1], date_parts[0], time_str
            );
        }
    }
    iso.to_string()
}

/// Dispara a consulta de verificação de atualização no ThreadPool em segundo plano.
pub fn trigger_background_update_check(
    pool: &Arc<ThreadPool>,
    etag: Option<String>,
    tx: Sender<UpdateEvent>,
) {
    let _ = tx.send(UpdateEvent::CheckStarted);

    pool.spawn(move || {
        let outcome_result = match github::query_github_latest(etag.as_deref()) {
            Ok(GitHubCheckResult::NotModified) => Ok(CheckOutcome::NotModified {
                checked_at: current_iso8601_timestamp(),
            }),
            Ok(GitHubCheckResult::NewRelease { release, etag: _ }) => {
                let timestamp = current_iso8601_timestamp();
                if release.is_update_available() {
                    Ok(CheckOutcome::UpdateAvailable {
                        release,
                        checked_at: timestamp,
                    })
                } else {
                    Ok(CheckOutcome::UpToDate {
                        checked_at: timestamp,
                    })
                }
            }
            Err(err) => Err(err),
        };

        let _ = tx.send(UpdateEvent::CheckFinished(outcome_result));
    });
}

/// Dispara o download de um pacote no ThreadPool em segundo plano.
pub fn trigger_background_download(
    pool: &Arc<ThreadPool>,
    version: String,
    package_asset: GitHubAsset,
    manifest_asset: Option<GitHubAsset>,
    tx: Sender<UpdateEvent>,
    cancel_flag: Arc<AtomicBool>,
) {
    let tx_clone = tx.clone();
    pool.spawn(move || {
        let callback_tx = tx_clone.clone();
        let download_result = download::execute_package_download(
            &version,
            &package_asset,
            manifest_asset.as_ref(),
            cancel_flag,
            move |progress| {
                let _ = callback_tx.send(UpdateEvent::DownloadProgress(progress));
            },
        );

        match download_result {
            Ok(path) => {
                let _ = tx_clone.send(UpdateEvent::DownloadCompleted(path));
            }
            Err(err) => {
                let _ = tx_clone.send(UpdateEvent::DownloadFailed(err));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_iso8601_timestamp_format() {
        let ts = current_iso8601_timestamp();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert_eq!(ts.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
    }

    #[test]
    fn test_format_timestamp_display() {
        let iso = "2026-11-09T01:15:30Z";
        assert_eq!(format_timestamp_display(iso), "09/11/2026 às 01:15");
    }

    #[test]
    fn test_section17_version_matrix() {
        use crate::updater::version::is_newer;

        // Local menor que remota
        assert!(is_newer("0.4.0", "0.3.3"));
        assert!(is_newer("1.0.0", "0.3.3"));
        assert!(is_newer("0.3.4", "0.3.3"));

        // Versões iguais
        assert!(!is_newer("0.3.3", "0.3.3"));
        assert!(!is_newer("v0.3.3", "0.3.3"));

        // Local maior que remota (downgrade rejeitado)
        assert!(!is_newer("0.2.2", "0.3.3"));
        assert!(!is_newer("0.3.0", "0.3.3"));

        // Prefixo 'v' e 'V'
        assert!(is_newer("v0.4.0", "v0.3.3"));
        assert!(is_newer("V1.0.0", "0.3.3"));

        // Pré-lançamentos
        assert!(is_newer("0.4.0", "0.4.0-rc.1"));
        assert!(is_newer("0.4.0-rc.2", "0.4.0-rc.1"));
        assert!(!is_newer("0.4.0-rc.1", "0.4.0"));

        // Comprimentos numéricos variados
        assert!(is_newer("10.0.0", "9.9.9"));
        assert!(is_newer("0.10.0", "0.9.0"));

        // Strings inválidas tratadas com segurança (retorna false sem pânico)
        assert!(!is_newer("invalido", "0.3.3"));
        assert!(!is_newer("0.4.0", "invalido"));
        assert!(!is_newer("", ""));
    }

    #[test]
    fn test_section17_persistence_and_schema4_migration() {
        use crate::system::config::{AppSettings, CURRENT_SCHEMA_VERSION};

        let temp_dir = std::env::temp_dir().join(format!("visor_upd_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("settings.json");

        // Simula settings.json de schema v3 (sem o bloco updates)
        let v3_json = r#"{
            "schema_version": 3,
            "mouse_wheel_action": "zoom",
            "interpolation": "auto",
            "background": "dark_checker",
            "show_filmstrip": true,
            "ignored_default_extensions": ["png", "jpg"]
        }"#;
        std::fs::write(&path, v3_json).unwrap();

        let (loaded, notice) = AppSettings::load_from_path(&path);
        assert!(notice.is_none());
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(loaded.schema_version, 4);

        // Verifica que o UpdateConfig foi preenchido com padrões seguros
        assert!(loaded.updates.automatic_check);
        assert_eq!(loaded.updates.check_interval_hours, 24);
        assert!(loaded.updates.last_check.is_none());
        assert!(loaded.updates.ignored_version.is_none());

        // Salva e recarrega para testar roundtrip
        loaded.save_to_path(&path).unwrap();
        let (reloaded, _) = AppSettings::load_from_path(&path);
        assert_eq!(reloaded.schema_version, 4);
        assert_eq!(reloaded.updates.check_interval_hours, 24);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_section17_should_check_for_updates_timing() {
        use crate::system::config::AppSettings;

        let mut settings = AppSettings::default();
        settings.updates.automatic_check = true;
        settings.updates.check_interval_hours = 24;

        // Sem verificação prévia -> deve verificar
        settings.updates.last_check = None;
        assert!(settings.should_check_for_updates());

        // Atualização automática desativada -> não deve verificar
        settings.updates.automatic_check = false;
        assert!(!settings.should_check_for_updates());
        settings.updates.automatic_check = true;

        // Verificação recente (agora mesmo) -> não deve verificar
        settings.updates.last_check = Some(current_iso8601_timestamp());
        assert!(!settings.should_check_for_updates());

        // Verificação antiga (30 horas atrás) -> deve verificar
        let past_time = "2020-01-01T00:00:00Z";
        settings.updates.last_check = Some(past_time.to_string());
        assert!(settings.should_check_for_updates());
    }

    #[test]
    fn test_section17_ignored_version_behavior() {
        use crate::updater::github::GitHubRelease;
        use crate::updater::version::clean_version_str;

        let release = GitHubRelease {
            tag_name: "v0.4.0".to_string(),
            name: Some("VISOR v0.4.0".to_string()),
            body: Some("Notas".to_string()),
            html_url: "https://github.com/Tsujiu/visor/releases/tag/v0.4.0".to_string(),
            published_at: None,
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        };

        let ignored_version: Option<String> = Some("0.4.0".to_string());

        // Verifica se a versão v0.4.0 é ignorada (independente de prefixo 'v')
        let is_ignored = ignored_version
            .as_deref()
            .map(|ign| clean_version_str(ign) == clean_version_str(&release.tag_name))
            .unwrap_or(false);
        assert!(is_ignored);

        // Quando uma versão mais nova é publicada (v1.1.0), não deve ser ignorada
        let next_release_tag = "v1.1.0";
        let is_next_ignored = ignored_version
            .as_deref()
            .map(|ign| clean_version_str(ign) == clean_version_str(next_release_tag))
            .unwrap_or(false);
        assert!(!is_next_ignored);
    }
    #[test]
    fn test_section17_pending_release_persistence_and_not_modified_restoration() {
        use crate::app::VisorApp;

        use crate::updater::github::{GitHubAsset, GitHubRelease};
        use crate::updater::{CheckOutcome, UpdateEvent};
        use eframe::egui::Context;

        let ctx = Context::default();
        let mut app = VisorApp::new(&ctx, None);

        let test_release = GitHubRelease {
            tag_name: "v99.0.0".to_string(),
            name: Some("VISOR v99.0.0".to_string()),
            body: Some("Release de teste".to_string()),
            html_url: "https://github.com/Tsujiu/visor/releases/tag/v99.0.0".to_string(),
            published_at: Some("2026-11-09T12:00:00Z".to_string()),
            draft: false,
            prerelease: false,
            assets: vec![GitHubAsset {
                name: "Visor-Setup-v99.0.0-x64.exe".to_string(),
                size: 8_000_000,
                browser_download_url: "https://github.com/Tsujiu/visor/releases/download/v99.0.0/Visor-Setup-v99.0.0-x64.exe".to_string(),
                content_type: Some("application/x-msdownload".to_string()),
            }],
        };

        // 1. Processa UpdateAvailable e valida persistÃªncia em AppSettings
        app.handle_update_event(
            &ctx,
            UpdateEvent::CheckFinished(Ok(CheckOutcome::UpdateAvailable {
                release: test_release.clone(),
                checked_at: "2026-11-09T12:00:00Z".to_string(),
            })),
        );

        assert!(app.updater.available_release.is_some());
        assert!(app.updater.show_update_window);
        assert!(app.has_active_update_banner());
        assert_eq!(
            app.settings.updates.last_known_release_tag.as_deref(),
            Some("v99.0.0")
        );
        assert!(app.settings.updates.last_known_release_json.is_some());

        // 2. Simula encerramento do app e reinÃ­cio simulando 304 Not Modified
        app.updater.available_release = None;
        app.updater.show_update_window = false;

        app.handle_update_event(
            &ctx,
            UpdateEvent::CheckFinished(Ok(CheckOutcome::NotModified {
                checked_at: "2026-11-09T12:05:00Z".to_string(),
            })),
        );

        // Deve restaurar automaticamente a notificaÃ§Ã£o da release pendente
        assert!(app.updater.available_release.is_some());
        assert!(app.updater.show_update_window);
        assert!(app.has_active_update_banner());

        // 3. Simula que o usuÃ¡rio atualizou e agora o GitHub retorna UpToDate
        app.handle_update_event(
            &ctx,
            UpdateEvent::CheckFinished(Ok(CheckOutcome::UpToDate {
                checked_at: "2026-11-09T12:10:00Z".to_string(),
            })),
        );

        assert!(app.updater.available_release.is_none());
        assert!(!app.has_active_update_banner());
        assert!(app.settings.updates.last_known_release_tag.is_none());
        assert!(app.settings.updates.last_known_release_json.is_none());
    }
}
