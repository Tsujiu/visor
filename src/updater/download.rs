//! Gerenciador de downloads com progresso, velocidade, estimativa e validação de integridade.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::system::paths::local_dir;
use crate::updater::github::GitHubAsset;
use crate::updater::integrity::{parse_sha256sums, verify_file_integrity};

/// Limite máximo de segurança para download de pacotes oficiais (150 MB).
pub const MAX_ALLOWED_DOWNLOAD_BYTES: u64 = 150 * 1024 * 1024;

/// Estado atual do ciclo de vida de um download de atualização.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Ocioso, nenhum download em andamento.
    Idle,
    /// Conectando ao servidor e alocando recursos temporários.
    Preparing,
    /// Baixando dados com progresso contínuo.
    Downloading,
    /// Download concluído; calculando e verificando soma criptográfica SHA-256.
    Verifying,
    /// Download finalizado e validado com sucesso.
    Completed(PathBuf),
    /// Falha durante o download ou na validação de integridade.
    Failed(String),
    /// Operação cancelada pelo usuário.
    Cancelled,
}

/// Instantâneo do progresso do download para renderização na interface.
#[derive(Clone, Debug, PartialEq)]
pub struct DownloadProgress {
    pub version: String,
    pub filename: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f32,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<u64>,
    pub status: DownloadStatus,
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self {
            version: String::new(),
            filename: String::new(),
            downloaded_bytes: 0,
            total_bytes: None,
            percentage: 0.0,
            speed_bytes_per_sec: 0.0,
            eta_seconds: None,
            status: DownloadStatus::Idle,
        }
    }
}

impl DownloadProgress {
    /// Formata a velocidade em string legível (ex: "1.5 MB/s" ou "420 KB/s").
    pub fn format_speed(&self) -> String {
        let bytes = self.speed_bytes_per_sec;
        if bytes >= 1024.0 * 1024.0 {
            format!("{:.1} MB/s", bytes / (1024.0 * 1024.0))
        } else if bytes >= 1024.0 {
            format!("{:.0} KB/s", bytes / 1024.0)
        } else {
            format!("{:.0} B/s", bytes)
        }
    }

    /// Formata a quantidade baixada / total (ex: "4.2 MB / 6.7 MB").
    pub fn format_progress_size(&self) -> String {
        let mb_downloaded = self.downloaded_bytes as f64 / (1024.0 * 1024.0);
        if let Some(total) = self.total_bytes {
            let mb_total = total as f64 / (1024.0 * 1024.0);
            format!("{:.1} MB / {:.1} MB", mb_downloaded, mb_total)
        } else {
            format!("{:.1} MB", mb_downloaded)
        }
    }

    /// Formata o tempo restante estimado (ex: "15s restantes" ou "2m 10s").
    pub fn format_eta(&self) -> String {
        match self.eta_seconds {
            Some(secs) if secs < 60 => format!("{secs}s restantes"),
            Some(secs) => format!("{}m {}s restantes", secs / 60, secs % 60),
            None => "calculando…".to_string(),
        }
    }
}

/// Diretório seguro para download temporário de atualizações (`%LOCALAPPDATA%\Visor\updates`).
pub fn updates_dir() -> PathBuf {
    local_dir().join("updates")
}

/// Executa o download de um pacote oficial do VISOR com validação SHA-256 e suporte a cancelamento.
///
/// Deve ser executado em uma worker thread dedicada fora da UI.
pub fn execute_package_download<F>(
    version: &str,
    package_asset: &GitHubAsset,
    manifest_asset: Option<&GitHubAsset>,
    cancel_flag: Arc<AtomicBool>,
    mut progress_callback: F,
) -> Result<PathBuf, String>
where
    F: FnMut(DownloadProgress),
{
    // Validação estrita de segurança da origem e do nome do arquivo
    if !package_asset.is_trusted_url() {
        return Err(
            "URL de download rejeitada: não pertence aos servidores oficiais do VISOR.".to_string(),
        );
    }
    if !package_asset.is_safe_filename() {
        return Err(
            "Nome do arquivo do pacote contém caracteres ou padrão de caminho não seguro."
                .to_string(),
        );
    }
    if package_asset.size > MAX_ALLOWED_DOWNLOAD_BYTES {
        return Err(format!(
            "Tamanho do arquivo informado ({} bytes) excede o limite máximo de segurança (150 MB).",
            package_asset.size
        ));
    }

    let target_dir = updates_dir();
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Não foi possível criar o diretório de downloads: {e}"))?;

    let filename = &package_asset.name;
    let final_path = target_dir.join(filename);
    let tmp_path = target_dir.join(format!("{filename}.tmp"));

    // Notifica início
    let mut progress = DownloadProgress {
        version: version.to_string(),
        filename: filename.clone(),
        downloaded_bytes: 0,
        total_bytes: Some(package_asset.size),
        percentage: 0.0,
        speed_bytes_per_sec: 0.0,
        eta_seconds: None,
        status: DownloadStatus::Preparing,
    };
    progress_callback(progress.clone());

    if cancel_flag.load(Ordering::Relaxed) {
        progress.status = DownloadStatus::Cancelled;
        progress_callback(progress);
        return Err("Download cancelado pelo usuário.".to_string());
    }

    // 1. Download do manifesto SHA256SUMS.txt
    let expected_hash = if let Some(manifest) = manifest_asset {
        if !manifest.is_trusted_url() || !manifest.is_safe_filename() {
            return Err("Manifesto de integridade SHA256SUMS.txt rejeitado por violação de segurança de origem.".to_string());
        }

        let manifest_resp = ureq::get(&manifest.browser_download_url)
            .timeout(Duration::from_secs(15))
            .call()
            .map_err(|e| format!("Falha ao baixar manifesto de integridade: {e}"))?;

        let manifest_text = manifest_resp
            .into_string()
            .map_err(|e| format!("Falha ao decodificar manifesto de integridade: {e}"))?;

        let map = parse_sha256sums(&manifest_text);
        map.get(&filename.to_ascii_lowercase()).cloned()
    } else {
        None
    };

    if cancel_flag.load(Ordering::Relaxed) {
        progress.status = DownloadStatus::Cancelled;
        progress_callback(progress);
        return Err("Download cancelado pelo usuário.".to_string());
    }

    // 2. Download do pacote principal
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();

    let response = agent
        .get(&package_asset.browser_download_url)
        .call()
        .map_err(|e| format!("Falha na conexão de download: {e}"))?;

    let total_len = response
        .header("Content-Length")
        .and_then(|h| h.parse::<u64>().ok())
        .or(Some(package_asset.size));

    if let Some(len) = total_len {
        if len > MAX_ALLOWED_DOWNLOAD_BYTES {
            return Err(format!(
                "Tamanho reportado pelo servidor ({len} bytes) excede o limite máximo permitido de 150 MB."
            ));
        }
    }

    progress.total_bytes = total_len;
    progress.status = DownloadStatus::Downloading;

    let mut reader = response.into_reader();
    let mut file = File::create(&tmp_path)
        .map_err(|e| format!("Não foi possível criar arquivo temporário: {e}"))?;

    let mut buffer = [0u8; 32 * 1024];
    let mut downloaded: u64 = 0;
    let start_time = Instant::now();
    let mut last_progress_emit = Instant::now();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            drop(file);
            let _ = fs::remove_file(&tmp_path);
            progress.status = DownloadStatus::Cancelled;
            progress_callback(progress);
            return Err("Download cancelado pelo usuário.".to_string());
        }

        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Erro de leitura durante download: {e}"))?;

        if bytes_read == 0 {
            break;
        }

        downloaded += bytes_read as u64;

        if downloaded > MAX_ALLOWED_DOWNLOAD_BYTES {
            drop(file);
            let _ = fs::remove_file(&tmp_path);
            let err_msg = "Download abortado: quantidade de dados recebidos excedeu o limite seguro (150 MB).".to_string();
            progress.status = DownloadStatus::Failed(err_msg.clone());
            progress_callback(progress);
            return Err(err_msg);
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Erro ao gravar dados no disco: {e}"))?;

        // Atualiza progresso a cada ~100ms para suavidade sem saturação de canais
        if last_progress_emit.elapsed() >= Duration::from_millis(100) {
            let elapsed_sec = start_time.elapsed().as_secs_f64().max(0.001);
            let speed = downloaded as f64 / elapsed_sec;

            progress.downloaded_bytes = downloaded;
            progress.speed_bytes_per_sec = speed;

            if let Some(total) = total_len {
                if total > 0 {
                    progress.percentage = (downloaded as f32 / total as f32).clamp(0.0, 1.0);
                    let remaining_bytes = total.saturating_sub(downloaded);
                    if speed > 0.0 {
                        progress.eta_seconds = Some((remaining_bytes as f64 / speed) as u64);
                    }
                }
            }

            progress_callback(progress.clone());
            last_progress_emit = Instant::now();
        }
    }

    file.flush()
        .map_err(|e| format!("Erro ao finalizar arquivo temporário: {e}"))?;
    drop(file);

    // 3. Verificação de integridade
    progress.status = DownloadStatus::Verifying;
    progress.percentage = 1.0;
    progress.downloaded_bytes = downloaded;
    progress_callback(progress.clone());

    if let Some(hash) = expected_hash {
        let is_valid = verify_file_integrity(&tmp_path, &hash)
            .map_err(|e| format!("Falha de I/O ao calcular hash do arquivo: {e}"))?;

        if !is_valid {
            let _ = fs::remove_file(&tmp_path);
            let err_msg = "A verificação de integridade SHA-256 falhou: o arquivo baixado diverge do manifesto oficial.".to_string();
            progress.status = DownloadStatus::Failed(err_msg.clone());
            progress_callback(progress);
            return Err(err_msg);
        }
    } else {
        // Se a release não tiver manifesto, informamos que o manifesto oficial estava ausente
        let _ = fs::remove_file(&tmp_path);
        let err_msg = "Manifesto oficial SHA256SUMS.txt não encontrado na release para validação de segurança.".to_string();
        progress.status = DownloadStatus::Failed(err_msg.clone());
        progress_callback(progress);
        return Err(err_msg);
    }

    // 4. Renomeia de .tmp para o arquivo final definitivo
    if final_path.exists() {
        let _ = fs::remove_file(&final_path);
    }
    fs::rename(&tmp_path, &final_path)
        .map_err(|e| format!("Não foi possível finalizar o arquivo de atualização: {e}"))?;

    progress.status = DownloadStatus::Completed(final_path.clone());
    progress_callback(progress);

    Ok(final_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress_formatting() {
        let mut p = DownloadProgress::default();
        p.speed_bytes_per_sec = 2.5 * 1024.0 * 1024.0;
        assert_eq!(p.format_speed(), "2.5 MB/s");

        p.speed_bytes_per_sec = 450.0 * 1024.0;
        assert_eq!(p.format_speed(), "450 KB/s");

        p.downloaded_bytes = 3 * 1024 * 1024;
        p.total_bytes = Some(6 * 1024 * 1024);
        assert_eq!(p.format_progress_size(), "3.0 MB / 6.0 MB");

        p.eta_seconds = Some(15);
        assert_eq!(p.format_eta(), "15s restantes");

        p.eta_seconds = Some(75);
        assert_eq!(p.format_eta(), "1m 15s restantes");
    }

    #[test]
    fn test_unsafe_package_rejected() {
        let bad_asset = GitHubAsset {
            name: "malicious.exe".to_string(),
            size: 500,
            browser_download_url: "http://insecure-http.com/malicious.exe".to_string(),
            content_type: None,
        };
        let flag = Arc::new(AtomicBool::new(false));
        let res = execute_package_download("1.0.0", &bad_asset, None, flag, |_| {});
        assert!(res.is_err());
    }
}
