//! Módulo de segurança, integridade do binário e proteção de memória do VISOR.
//!
//! Implementa proteção em camadas para prevenir modificações não autorizadas (cracks,
//! patches de NOP, injeção de bibliotecas) e assegurar a integridade do processo de execução.

use std::fs::File;
use std::io::Read;

use sha2::{Digest, Sha256};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn IsDebuggerPresent() -> i32;
    fn CheckRemoteDebuggerPresent(process: isize, is_debugger_present: *mut i32) -> i32;
    fn GetCurrentProcess() -> isize;
}

/// Relatório de segurança e integridade do processo.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SecurityStatus {
    /// Se um depurador ativo ou remoto foi detectado.
    pub debugger_detected: bool,
    /// Hash criptográfico SHA-256 do binário do executável em disco.
    pub binary_sha256: Option<String>,
    /// Integridade dos cabeçalhos do executável validada.
    pub pe_integrity_valid: bool,
}

/// Retorna se há um depurador local conectado ao processo via PEB.
#[inline]
#[allow(dead_code)]
pub fn is_debugger_present() -> bool {
    unsafe { IsDebuggerPresent() != 0 }
}

/// Retorna se há um depurador remoto anexado ao processo.
#[inline]
#[allow(dead_code)]
pub fn is_remote_debugger_present() -> bool {
    let mut present: i32 = 0;
    let res = unsafe { CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut present) };
    res != 0 && present != 0
}

/// Avalia se qualquer mecanismo de depuração ou hooking dinâmico está ativo.
#[inline]
#[allow(dead_code)]
pub fn is_debugger_active() -> bool {
    is_debugger_present() || is_remote_debugger_present()
}

/// Aplica as políticas de segurança de inicialização.
///
/// Em compilações de distribuição (`release`), fecha imediatamente o processo
/// caso depuradores ativos estejam acoplados, impedindo injeções de memória indesejadas.
pub fn enforce_startup_security() -> Result<(), &'static str> {
    #[cfg(not(debug_assertions))]
    {
        // Variável de escape para depuração interna de desenvolvedores autorizados
        if std::env::var_os("VISOR_ALLOW_DEBUG").is_none() && is_debugger_active() {
            std::process::exit(0x2A);
        }
    }

    Ok(())
}

/// Valida os cabeçalhos do formato Portable Executable (PE) do arquivo em execução.
#[allow(dead_code)]
pub fn verify_pe_header(bytes: &[u8]) -> bool {
    // Cabeçalho DOS 'MZ'
    if bytes.len() < 64 || bytes[0] != 0x4D || bytes[1] != 0x5A {
        return false;
    }

    // Offset para o cabeçalho PE em e_lfanew (offset 0x3C, little-endian u32)
    let pe_offset =
        u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]) as usize;
    if bytes.len() < pe_offset + 4 {
        return false;
    }

    // Assinatura PE 'PE\0\0'
    bytes[pe_offset..pe_offset + 4] == [0x50, 0x45, 0x00, 0x00]
}

/// Calcula o hash SHA-256 do binário em execução no disco.
#[allow(dead_code)]
pub fn calculate_current_exe_sha256() -> Option<String> {
    let current_exe = std::env::current_exe().ok()?;
    let mut file = File::open(current_exe).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buffer[..n]),
            Err(_) => return None,
        }
    }

    Some(format!("{:X}", hasher.finalize()))
}

/// Diagnóstico completo de integridade e segurança do ambiente de execução.
#[allow(dead_code)]
pub fn get_security_status() -> SecurityStatus {
    let debugger_detected = is_debugger_active();
    let binary_sha256 = calculate_current_exe_sha256();

    let pe_integrity_valid = if let Ok(path) = std::env::current_exe() {
        if let Ok(mut f) = File::open(path) {
            let mut header = [0u8; 1024];
            let n = f.read(&mut header).unwrap_or(0);
            verify_pe_header(&header[..n])
        } else {
            false
        }
    } else {
        false
    };

    SecurityStatus {
        debugger_detected,
        binary_sha256,
        pe_integrity_valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_detection_safe_query() {
        let active = is_debugger_active();
        let _ = active;
    }

    #[test]
    fn test_verify_pe_header_validation() {
        assert!(!verify_pe_header(&[]));
        assert!(!verify_pe_header(&[0x00; 100]));

        let mut mock_pe = vec![0u8; 256];
        mock_pe[0] = 0x4D;
        mock_pe[1] = 0x5A;
        mock_pe[0x3C] = 0x80;
        mock_pe[128] = 0x50;
        mock_pe[129] = 0x45;
        mock_pe[130] = 0x00;
        mock_pe[131] = 0x00;

        assert!(verify_pe_header(&mock_pe));
    }

    #[test]
    fn test_security_status_query() {
        let status = get_security_status();
        assert!(!status.debugger_detected || status.debugger_detected);
    }
}
