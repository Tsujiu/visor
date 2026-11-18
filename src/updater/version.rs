//! Comparação e validação de versões SemVer para o Update Center do VISOR.

use semver::Version;

/// Versão oficial do binário compilado.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Limpa uma string de versão removendo prefixos 'v'/'V' e espaços em branco.
pub fn clean_version_str(v: &str) -> &str {
    let trimmed = v.trim();
    if let Some(stripped) = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
    {
        stripped.trim()
    } else {
        trimmed
    }
}

/// Tenta interpretar uma string de versão como SemVer.
pub fn parse_version(v: &str) -> Result<Version, semver::Error> {
    Version::parse(clean_version_str(v))
}

/// Verifica se a string informada é uma versão SemVer válida.
#[allow(dead_code)]
pub fn is_valid_semver(v: &str) -> bool {
    parse_version(v).is_ok()
}

/// Compara duas versões e determina se a remota é estritamente mais recente que a local.
///
/// Retorna `true` se `remote` for maior que `local` e ambas forem válidas.
/// Em caso de erro de parsing ou se `remote <= local`, retorna `false`.
pub fn is_newer(remote: &str, local: &str) -> bool {
    match (parse_version(remote), parse_version(local)) {
        (Ok(rem), Ok(loc)) => rem > loc,
        _ => false,
    }
}

/// Determina se a versão remota é mais recente que a versão atual do executável.
pub fn is_newer_than_current(remote: &str) -> bool {
    is_newer(remote, CURRENT_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_version_str() {
        assert_eq!(clean_version_str("v0.3.0"), "0.3.0");
        assert_eq!(clean_version_str("V1.2.3"), "1.2.3");
        assert_eq!(clean_version_str("  v0.4.0-beta.1  "), "0.4.0-beta.1");
        assert_eq!(clean_version_str("0.2.2"), "0.2.2");
    }

    #[test]
    fn test_semver_comparisons() {
        // Remota maior que local -> Atualização disponível
        assert!(is_newer("0.3.0", "0.2.2"));
        assert!(is_newer("v0.4.0", "0.3.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.3.4", "0.3.3"));

        // Versões iguais -> Atualizado
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(!is_newer("v0.3.0", "0.3.0"));
        assert!(!is_newer("0.3.0", "v0.3.0"));

        // Remota menor que local -> Não oferecer downgrade
        assert!(!is_newer("0.2.2", "0.3.0"));
        assert!(!is_newer("v0.2.2", "0.3.0"));

        // Pré-lançamentos
        assert!(is_newer("0.4.0", "0.4.0-beta.1"));
        assert!(is_newer("0.4.0-beta.2", "0.4.0-beta.1"));
        assert!(!is_newer("0.4.0-beta.1", "0.4.0"));

        // Versões inválidas -> false seguro
        assert!(!is_newer("invalido", "0.3.0"));
        assert!(!is_newer("0.3.0", "invalido"));
        assert!(!is_newer("vX.Y.Z", "0.3.0"));
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("0.4.0"));
        assert!(is_valid_semver("v1.2.3"));
        assert!(!is_valid_semver("not_a_version"));
    }

    #[test]
    fn test_current_version_comparison() {
        assert!(!is_newer_than_current(CURRENT_VERSION));
        assert!(is_newer_than_current("99.0.0"));
        assert!(!is_newer_than_current("0.1.0"));
    }
}
