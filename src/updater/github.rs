//! Cliente oficial de consulta à API do GitHub Releases para o VISOR.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::updater::version::is_newer_than_current;

/// URL oficial da API do GitHub para a release estável mais recente.
pub const OFFICIAL_API_LATEST_RELEASE: &str =
    "https://api.github.com/repos/Tsujiu/visor/releases/latest";

/// Prefixo oficial do repositório no GitHub para validação estrita de segurança.
pub const OFFICIAL_REPO_URL_PREFIX: &str = "https://github.com/Tsujiu/visor/releases/";

/// Timeout padrão para requisições de consulta de versão (10 segundos).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Representação de um asset associado a uma release do GitHub.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    #[serde(default)]
    pub content_type: Option<String>,
}

impl GitHubAsset {
    /// Valida se a URL de download pertence à infraestrutura oficial do GitHub (HTTPS obrigatório).
    pub fn is_trusted_url(&self) -> bool {
        let url = &self.browser_download_url;
        if !url.starts_with("https://") {
            return false;
        }
        if url.contains("..") || url.contains('@') {
            return false;
        }
        url.starts_with("https://github.com/Tsujiu/visor/releases/download/")
            || url.starts_with("https://objects.githubusercontent.com/")
    }

    /// Valida se o nome do arquivo do pacote é seguro contra ataques de travessia de caminho (Path Traversal).
    pub fn is_safe_filename(&self) -> bool {
        let name = &self.name;
        if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
            return false;
        }
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    }
}

/// Sanitiza texto recebido de APIs remotas, removendo caracteres não imprimíveis de controle e limitando o tamanho.
pub fn sanitize_text(input: &str, max_len: usize) -> String {
    let mut out = String::with_capacity(input.len().min(max_len));
    for c in input.chars() {
        if out.len() >= max_len {
            break;
        }
        // Permite quebras de linha e tabulação, rejeitando outros caracteres de controle ANSI/ASCII
        if c == '\n' || c == '\r' || c == '\t' || !c.is_control() {
            out.push(c);
        }
    }
    out
}

/// Representação de uma Release retornada pela API do GitHub.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GitHubAsset>,
}

impl GitHubRelease {
    /// Sanitiza campos textuais da release para prevenir injeções de controle ou formatação maliciosa.
    pub fn sanitize(&mut self) {
        if let Some(ref mut name) = self.name {
            *name = sanitize_text(name, 256);
        }
        if let Some(ref mut body) = self.body {
            *body = sanitize_text(body, 65536);
        }
        self.tag_name = sanitize_text(&self.tag_name, 64);
    }

    /// Valida se a release é confiável e pertence ao repositório oficial.
    pub fn is_valid_official_release(&self) -> bool {
        if self.draft {
            return false;
        }
        if !self.html_url.starts_with(OFFICIAL_REPO_URL_PREFIX) {
            return false;
        }
        true
    }

    /// Localiza o asset correspondente ao instalador executável convencional (`Visor-Setup-*.exe`).
    pub fn find_installer_asset(&self) -> Option<&GitHubAsset> {
        self.assets.iter().find(|a| {
            let name = a.name.to_ascii_lowercase();
            name.starts_with("visor-setup-")
                && name.ends_with(".exe")
                && a.is_trusted_url()
                && a.is_safe_filename()
        })
    }

    /// Localiza o asset correspondente ao pacote portátil compactado (`Visor-Portable-*.zip`).
    pub fn find_portable_asset(&self) -> Option<&GitHubAsset> {
        self.assets.iter().find(|a| {
            let name = a.name.to_ascii_lowercase();
            name.starts_with("visor-portable-")
                && name.ends_with(".zip")
                && a.is_trusted_url()
                && a.is_safe_filename()
        })
    }

    /// Localiza o manifesto oficial de integridade (`SHA256SUMS.txt`).
    pub fn find_manifest_asset(&self) -> Option<&GitHubAsset> {
        self.assets.iter().find(|a| {
            a.name.eq_ignore_ascii_case("SHA256SUMS.txt")
                && a.is_trusted_url()
                && a.is_safe_filename()
        })
    }

    /// Verifica se esta release representa uma versão estritamente mais recente que a versão instalada.
    pub fn is_update_available(&self) -> bool {
        is_newer_than_current(&self.tag_name)
    }
}

/// Resultado retornado por uma consulta de verificação de atualização.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitHubCheckResult {
    /// Nenhuma alteração no servidor (HTTP 304 Not Modified).
    NotModified,
    /// Nova release oficial encontrada com sucesso.
    NewRelease {
        release: GitHubRelease,
        etag: Option<String>,
    },
}

/// Tipos de erros estruturados ocorridos durante a verificação de atualizações.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateCheckError {
    /// Falha de conexão ou rede (ex: sem internet, DNS inacessível).
    Network(String),
    /// Limite de requisições da API pública do GitHub atingido (HTTP 403/429).
    RateLimitExceeded,
    /// Resposta do servidor não pôde ser interpretada (JSON inválido ou corrompido).
    InvalidResponse(String),
    /// Release não atende aos requisitos de autenticidade (URL arbitrária ou fork).
    UntrustedRelease(String),
}

impl std::fmt::Display for UpdateCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Falha de rede: {msg}"),
            Self::RateLimitExceeded => write!(
                f,
                "Limite de consultas de atualização atingido. Tente novamente mais tarde."
            ),
            Self::InvalidResponse(msg) => {
                write!(f, "Resposta inválida do servidor de atualizações: {msg}")
            }
            Self::UntrustedRelease(msg) => write!(f, "Release não confiável rejeitada: {msg}"),
        }
    }
}

impl std::error::Error for UpdateCheckError {}

/// Executa a consulta à API do GitHub de forma síncrona/bloqueante (deve rodar em worker thread).
pub fn query_github_latest(etag: Option<&str>) -> Result<GitHubCheckResult, UpdateCheckError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(&format!(
            "Visor/{} (+https://github.com/Tsujiu/visor)",
            env!("CARGO_PKG_VERSION")
        ))
        .build();

    let mut req = agent
        .get(OFFICIAL_API_LATEST_RELEASE)
        .set("Accept", "application/vnd.github.v3+json");

    if let Some(tag) = etag
        && !tag.is_empty() {
            req = req.set("If-None-Match", tag);
        }

    let response = match req.call() {
        Ok(resp) => resp,
        Err(ureq::Error::Status(304, _)) => return Ok(GitHubCheckResult::NotModified),
        Err(ureq::Error::Status(403, _)) | Err(ureq::Error::Status(429, _)) => {
            return Err(UpdateCheckError::RateLimitExceeded);
        }
        Err(ureq::Error::Status(code, resp)) => {
            let msg = resp
                .into_string()
                .unwrap_or_else(|_| format!("HTTP {code}"));
            return Err(UpdateCheckError::Network(format!("HTTP {code}: {msg}")));
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(UpdateCheckError::Network(err.to_string()));
        }
    };

    let new_etag = response.header("ETag").map(|s| s.to_string());

    let mut release: GitHubRelease = response
        .into_json()
        .map_err(|e| UpdateCheckError::InvalidResponse(e.to_string()))?;

    release.sanitize();

    if !release.is_valid_official_release() {
        return Err(UpdateCheckError::UntrustedRelease(format!(
            "URL da release '{}' não pertence ao repositório oficial Tsujiu/visor",
            release.html_url
        )));
    }

    Ok(GitHubCheckResult::NewRelease {
        release,
        etag: new_etag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_release_deserialization_and_asset_discovery() {
        let json_sample = r#"{
            "tag_name": "v99.0.0",
            "name": "VISOR v99.0.0 — Update Center",
            "body": "Novidades incríveis nesta versão!",
            "html_url": "https://github.com/Tsujiu/visor/releases/tag/v99.0.0",
            "published_at": "2026-11-09T00:00:00Z",
            "draft": false,
            "prerelease": false,
            "assets": [
                {
                    "name": "Visor-Setup-v99.0.0-x64.exe",
                    "size": 7000000,
                    "browser_download_url": "https://github.com/Tsujiu/visor/releases/download/v99.0.0/Visor-Setup-v99.0.0-x64.exe",
                    "content_type": "application/x-msdownload"
                },
                {
                    "name": "Visor-Portable-v99.0.0-x64.zip",
                    "size": 7500000,
                    "browser_download_url": "https://github.com/Tsujiu/visor/releases/download/v99.0.0/Visor-Portable-v99.0.0-x64.zip",
                    "content_type": "application/zip"
                },
                {
                    "name": "SHA256SUMS.txt",
                    "size": 194,
                    "browser_download_url": "https://github.com/Tsujiu/visor/releases/download/v99.0.0/SHA256SUMS.txt",
                    "content_type": "text/plain"
                }
            ]
        }"#;

        let mut release: GitHubRelease = serde_json::from_str(json_sample).unwrap();
        release.sanitize();
        assert_eq!(release.tag_name, "v99.0.0");
        assert!(release.is_valid_official_release());
        assert!(release.is_update_available());

        let installer = release.find_installer_asset().unwrap();
        assert_eq!(installer.name, "Visor-Setup-v99.0.0-x64.exe");

        let portable = release.find_portable_asset().unwrap();
        assert_eq!(portable.name, "Visor-Portable-v99.0.0-x64.zip");

        let manifest = release.find_manifest_asset().unwrap();
        assert_eq!(manifest.name, "SHA256SUMS.txt");
    }

    #[test]
    fn test_untrusted_fork_rejection() {
        let fake_json = r#"{
            "tag_name": "v99.0.0",
            "name": "Malicious Fork",
            "html_url": "https://github.com/attacker/visor/releases/tag/v99.0.0",
            "draft": false,
            "prerelease": false,
            "assets": []
        }"#;

        let release: GitHubRelease = serde_json::from_str(fake_json).unwrap();
        assert!(!release.is_valid_official_release());
    }

    #[test]
    fn test_draft_release_ignored() {
        let draft_json = r#"{
            "tag_name": "v1.0.0",
            "html_url": "https://github.com/Tsujiu/visor/releases/tag/v1.0.0",
            "draft": true,
            "prerelease": false,
            "assets": []
        }"#;

        let release: GitHubRelease = serde_json::from_str(draft_json).unwrap();
        assert!(!release.is_valid_official_release());
    }

    #[test]
    fn test_path_traversal_and_malicious_asset_name_rejected() {
        let bad_asset = GitHubAsset {
            name: "../../../Windows/System32/evil.exe".to_string(),
            size: 1024,
            browser_download_url:
                "https://github.com/Tsujiu/visor/releases/download/v1.0.0/evil.exe"
                    .to_string(),
            content_type: None,
        };
        assert!(!bad_asset.is_safe_filename());

        let good_asset = GitHubAsset {
            name: "Visor-Setup-v0.4.1-x64.exe".to_string(),
            size: 1024,
            browser_download_url: "https://github.com/Tsujiu/visor/releases/download/v0.4.1/Visor-Setup-v0.4.1-x64.exe".to_string(),
            content_type: None,
        };
        assert!(good_asset.is_safe_filename());
        assert!(good_asset.is_trusted_url());
    }

    #[test]
    fn test_sanitize_text_strips_control_characters() {
        let raw = "Hello\x1b[31m World\x00\x07!\nNew line";
        let cleaned = sanitize_text(raw, 100);
        assert_eq!(cleaned, "Hello[31m World!\nNew line");
    }
}
