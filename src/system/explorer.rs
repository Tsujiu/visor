//! Módulo de integração nativa para revelação e seleção de arquivos no Windows Explorer
//! e diálogo do sistema "Abrir com..." (`SHOpenWithDialog`).
//!
//! Utiliza preferencialmente as APIs nativas do Shell do Windows:
//! 1. `SHOpenFolderAndSelectItems`: reutiliza janelas abertas do Explorer e destaca o item desejado.
//! 2. `SHOpenWithDialog`: apresenta o diálogo nativo do Windows para escolher o aplicativo para abrir o arquivo.
//! Ambos são executados sob contexto COM STA devidamente inicializado e possuem fallbacks transparentes.

#![allow(dead_code)]

use std::ffi::c_void;
use std::path::Path;
use std::process::Command;
use std::ptr;

pub const OAIF_ALLOW_REGISTRATION: u32 = 0x00000001;
pub const OAIF_REGISTER_EXT: u32 = 0x00000002;
pub const OAIF_EXEC: u32 = 0x00000004;
pub const OAIF_FORCE_REGISTRATION: u32 = 0x00000008;
pub const OAIF_HIDE_REGISTRATION: u32 = 0x00000020;

#[repr(C)]
pub struct OPENASINFO {
    pub pcsz_file: *const u16,
    pub pcsz_class: *const u16,
    pub oaif_in_flags: u32,
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn SHOpenFolderAndSelectItems(
        pidl_folder: *const c_void,
        cidl: u32,
        apidl: *const *const c_void,
        dw_flags: u32,
    ) -> i32;

    pub fn SHOpenWithDialog(hwnd_parent: *mut c_void, poa_info: *const OPENASINFO) -> i32;

    pub fn ILCreateFromPathW(psz_path: *const u16) -> *mut c_void;
    pub fn ILFree(pidl: *mut c_void);
}

/// Guard RAII para inicialização e finalização COM STA garantida na thread executora.
pub struct ScopedComSta {
    need_uninit: bool,
}

impl ScopedComSta {
    pub fn enter() -> Self {
        let hr = unsafe {
            crate::system::shell_sta::CoInitializeEx(
                ptr::null_mut(),
                crate::system::shell_sta::COINIT_APARTMENTTHREADED
                    | crate::system::shell_sta::COINIT_DISABLE_OLE1DDE,
            )
        };
        // hr == S_OK (0): COM recém-inicializado neste thread.
        // hr == S_FALSE (1): COM já estava inicializado como STA; refcount incrementado.
        // Ambos exigem chamada simétrica a CoUninitialize().
        // hr == RPC_E_CHANGED_MODE (-2147417850): thread já é MTA. Não chamar CoUninitialize.
        Self {
            need_uninit: hr == 0 || hr == 1,
        }
    }
}

impl Drop for ScopedComSta {
    fn drop(&mut self) {
        if self.need_uninit {
            unsafe {
                crate::system::shell_sta::CoUninitialize();
            }
        }
    }
}

/// Abre o Explorador de Arquivos do Windows na pasta do arquivo informado e o seleciona/destaca.
///
/// 1. Valida se o arquivo existe no disco.
/// 2. Converte para caminho absoluto canônico sem prefixos verbatim `\\?\`.
/// 3. Garante contexto COM STA ativo na thread.
/// 4. Cria PIDL absoluto via `ILCreateFromPathW`.
/// 5. Invoca a API Shell nativa `SHOpenFolderAndSelectItems` com cidl = 0 e apidl = NULL.
/// 6. Libera o PIDL via `ILFree`.
/// 7. Se a API Shell falhar, executa fallback transparente para `explorer.exe /select,"<caminho>"`.
pub fn reveal_in_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Arquivo não encontrado: {}", path.display()));
    }

    let abs_path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let path_str = abs_path.to_string_lossy();
    let clean_path = path_str.trim_start_matches(r"\\?\").to_string();

    let mut wide: Vec<u16> = clean_path.encode_utf16().collect();
    wide.push(0);

    let _com = ScopedComSta::enter();

    let pidl = unsafe { ILCreateFromPathW(wide.as_ptr()) };
    if !pidl.is_null() {
        // Conforme a especificação oficial da Microsoft:
        // "If cidl is 0, then pidlFolder must point to a fully qualified PIDL of a single item to select, and apidl is NULL."
        let hr = unsafe { SHOpenFolderAndSelectItems(pidl, 0, ptr::null(), 0) };
        unsafe { ILFree(pidl) };
        if hr == 0 {
            return Ok(());
        }
    }

    // Fallback controlado via explorer.exe
    let arg = format!("/select,{}", clean_path);
    Command::new("explorer.exe")
        .arg(&arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Não foi possível iniciar o Explorador de Arquivos: {e}"))
}

/// Apresenta o diálogo nativo do Windows "Abrir com..." (`SHOpenWithDialog`) para o arquivo indicado.
///
/// 1. Valida a existência do arquivo no disco.
/// 2. Normaliza para caminho absoluto sem prefixos verbatim `\\?\`.
/// 3. Garante contexto COM STA ativo.
/// 4. Configura `OPENASINFO` com flags `OAIF_EXEC | OAIF_ALLOW_REGISTRATION` (abre o arquivo sem forçar associação padrão).
/// 5. Trata cancelamento do usuário (`0x800704C7` / `ERROR_CANCELLED`) como operação limpa e bem-sucedida.
/// 6. Oferece fallback transparente via `rundll32.exe shell32.dll,OpenAs_RunDLL` se a API retornar erro inesperado.
pub fn open_with_dialog(path: &Path, hwnd: Option<*mut c_void>) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Arquivo não encontrado: {}", path.display()));
    }

    let abs_path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let path_str = abs_path.to_string_lossy();
    let clean_path = path_str.trim_start_matches(r"\\?\").to_string();

    let mut wide: Vec<u16> = clean_path.encode_utf16().collect();
    wide.push(0);

    let _com = ScopedComSta::enter();

    let info = OPENASINFO {
        pcsz_file: wide.as_ptr(),
        pcsz_class: ptr::null(),
        oaif_in_flags: OAIF_EXEC | OAIF_ALLOW_REGISTRATION,
    };

    let hwnd_ptr = hwnd.unwrap_or(ptr::null_mut());
    let hr = unsafe { SHOpenWithDialog(hwnd_ptr, &info) };

    const ERROR_CANCELLED_HRESULT: i32 = -2147023673; // 0x800704C7: HRESULT_FROM_WIN32(ERROR_CANCELLED)

    if hr == 0 || hr == ERROR_CANCELLED_HRESULT {
        return Ok(());
    }

    // Fallback controlado via rundll32 shell32.dll,OpenAs_RunDLL
    Command::new("rundll32.exe")
        .arg("shell32.dll,OpenAs_RunDLL")
        .arg(&clean_path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Não foi possível abrir o diálogo 'Abrir com': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reveal_in_explorer_nonexistent_fails() {
        let res = reveal_in_explorer(Path::new("C:\\nao_existe_visor_reveal_xyz.tmp"));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("não encontrado"));
    }

    #[test]
    fn test_reveal_in_explorer_valid_file_creates_pidl() {
        let cargo_toml = Path::new("Cargo.toml");
        assert!(cargo_toml.exists());

        let abs = std::path::absolute(cargo_toml).unwrap();
        let clean = abs
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_string();
        let mut wide: Vec<u16> = clean.encode_utf16().collect();
        wide.push(0);

        let _com = ScopedComSta::enter();
        let pidl = unsafe { ILCreateFromPathW(wide.as_ptr()) };
        assert!(
            !pidl.is_null(),
            "ILCreateFromPathW deve criar PIDL válido para arquivo existente"
        );
        unsafe { ILFree(pidl) };
    }

    #[test]
    fn test_open_with_dialog_nonexistent_fails() {
        let res = open_with_dialog(Path::new("C:\\nao_existe_visor_open_with_xyz.tmp"), None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("não encontrado"));
    }

    #[test]
    fn test_openasinfo_structure_layout_and_flags() {
        let path = "C:\\teste.png";
        let mut wide: Vec<u16> = path.encode_utf16().collect();
        wide.push(0);

        let info = OPENASINFO {
            pcsz_file: wide.as_ptr(),
            pcsz_class: ptr::null(),
            oaif_in_flags: OAIF_EXEC | OAIF_ALLOW_REGISTRATION,
        };

        assert_eq!(info.oaif_in_flags, 0x00000005);
        assert!(!info.pcsz_file.is_null());
        assert!(info.pcsz_class.is_null());
    }

    #[test]
    fn test_scoped_com_sta_raii_safety() {
        // Testa se múltiplas entradas de ScopedComSta são tolerantes e restauram o estado
        {
            let _guard1 = ScopedComSta::enter();
            {
                let _guard2 = ScopedComSta::enter();
            }
        }
    }
}
