//! Gerenciador de inicialização do instalador e instruções para atualização portátil.

use std::path::Path;
use std::process::Command;

use crate::system::explorer::reveal_in_explorer;
use crate::system::paths::local_dir;
use crate::updater::download::updates_dir;

/// Determina se a instância atual do VISOR está rodando como instalação convencional (Inno Setup)
/// ou como versão portátil autônoma (executada de pasta avulsa ou pen drive).
#[allow(dead_code)]
pub fn is_running_installed_version() -> bool {
    if let Ok(current) = std::env::current_exe() {
        let expected_install_dir = local_dir().join("Programs").join("Visor");
        if let Ok(canon_current) = current.canonicalize()
            && let Ok(canon_dir) = expected_install_dir.canonicalize() {
                return canon_current.starts_with(canon_dir);
            }
        current.starts_with(&expected_install_dir)
    } else {
        false
    }
}

/// Inicia o instalador oficial baixado (`Visor-Setup-*.exe`) com validações rigorosas de segurança.
///
/// O instalador convencional é executado como processo independente.
/// O VISOR atual deve ser fechado logo em seguida para que o instalador
/// possa realizar a atualização in-place sem conflitos de arquivo aberto.
pub fn launch_official_installer(installer_path: &Path) -> Result<(), String> {
    if !installer_path.exists() {
        return Err(format!(
            "O arquivo do instalador não foi encontrado em: {}",
            installer_path.display()
        ));
    }

    // 1. Validação estrita do padrão de nome do executável
    let file_name = installer_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("");
    let lower_name = file_name.to_ascii_lowercase();
    if !lower_name.starts_with("visor-setup-") || !lower_name.ends_with(".exe") {
        return Err(
            "Execução bloqueada: o arquivo não é um instalador oficial do VISOR.".to_string(),
        );
    }

    // 2. Validação do diretório seguro (deve residir dentro de %LOCALAPPDATA%\Visor\updates)
    let safe_dir = updates_dir();
    if let (Ok(canon_path), Ok(canon_dir)) =
        (installer_path.canonicalize(), safe_dir.canonicalize())
        && !canon_path.starts_with(&canon_dir) {
            return Err("Execução bloqueada: tentativa de executar instalador fora do diretório seguro de atualizações.".to_string());
        }

    Command::new(installer_path)
        .spawn()
        .map_err(|e| format!("Não foi possível iniciar o instalador: {e}"))?;

    Ok(())
}

/// Revela o pacote ZIP portátil baixado no Windows Explorer para extração pelo usuário.
pub fn reveal_portable_package(zip_path: &Path) -> Result<(), String> {
    if !zip_path.exists() {
        return Err(format!(
            "O arquivo do pacote portátil não foi encontrado em: {}",
            zip_path.display()
        ));
    }

    let _ = reveal_in_explorer(zip_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_installer_path_validation() {
        let bad_path = PathBuf::from("C:\\caminho_inexistente_12345\\setup.exe");
        let res = launch_official_installer(&bad_path);
        assert!(res.is_err());
    }

    #[test]
    fn test_arbitrary_binary_rejected() {
        // Arquivo que não seja Visor-Setup-*.exe deve ser rejeitado
        let cmd_path = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        let res = launch_official_installer(&cmd_path);
        assert!(res.is_err());
    }
}
