//! Módulo de integração com a Lixeira do Windows (Recycle Bin).
//!
//! # Diretrizes de Segurança
//! - **REGRA ABSOLUTA:** É estritamente proibido utilizar `DeleteFile`, `std::fs::remove_file`
//!   ou qualquer mecanismo de exclusão permanente como caminho padrão desta funcionalidade.
//! - **API Primária:** `IFileOperation` com o sinalizador `FOFX_RECYCLEONDELETE` (`0x00080000`).
//!   Preserva o histórico de Desfazer (`Ctrl+Z`) no Windows Explorer e as caixas de diálogo padrão do Shell.
//! - **Fallback Seguro:** Caso a interface `IFileOperation` não possa ser instanciada antes do início
//!   da exclusão, realiza fallback controlado para `SHFileOperationW` com `FOF_ALLOWUNDO`.
//! - **Isolamento de Erro:** Se `PerformOperations` já foi iniciado ou falhou, NENHUM fallback automático
//!   é disparado para evitar estados parciais ou duplicações.

#![allow(dead_code, non_snake_case, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::UI::Shell::{
    FO_DELETE, FOF_ALLOWUNDO, SHCreateItemFromParsingName, SHFILEOPSTRUCTW, SHFileOperationW,
};
use windows_sys::core::GUID;

/// GUID da coclass `FileOperation`: `{3ad05575-8857-4850-9277-11b85bdb8e09}`.
pub const CLSID_FILE_OPERATION: GUID = GUID {
    data1: 0x3ad05575,
    data2: 0x8857,
    data3: 0x4850,
    data4: [0x92, 0x77, 0x11, 0xb8, 0x5b, 0xdb, 0x8e, 0x09],
};

/// GUID da interface `IFileOperation`: `{947aab5f-0a5c-4c13-b4d6-4bf7836fc9f8}`.
pub const IID_IFILE_OPERATION: GUID = GUID {
    data1: 0x947aab5f,
    data2: 0x0a5c,
    data3: 0x4c13,
    data4: [0xb4, 0xd6, 0x4b, 0xf7, 0x83, 0x6f, 0xc9, 0xf8],
};

/// GUID da interface `IShellItem`: `{43826d1e-e718-42ee-bc55-a1e261c37bfe}`.
pub const IID_ISHELL_ITEM: GUID = GUID {
    data1: 0x43826d1e,
    data2: 0xe718,
    data3: 0x42ee,
    data4: [0xbc, 0x55, 0xa1, 0xe2, 0x61, 0xc3, 0x7b, 0xfe],
};

pub const CLSCTX_INPROC_SERVER: u32 = 0x1;
pub const FOFX_RECYCLEONDELETE: u32 = 0x00080000;
pub const FOFX_ADDUNDORECORD: u32 = 0x20000000;

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoCreateInstance(
        rclsid: *const GUID,
        p_unk_outer: *mut c_void,
        dw_cls_context: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetActiveWindow() -> *mut c_void;
    fn GetForegroundWindow() -> *mut c_void;
    fn GetWindowThreadProcessId(h_wnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
    pub fn IsWindow(h_wnd: *mut c_void) -> i32;
    fn EnumThreadWindows(
        dw_thread_id: u32,
        lpfn: Option<unsafe extern "system" fn(*mut c_void, isize) -> i32>,
        l_param: isize,
    ) -> i32;
}

/// Obtém o handle real da janela principal pertencente estritamente ao processo do VISOR.
///
/// Deve ser invocada na thread da interface gráfica (UI).
/// Valida se o HWND pertence ao PID do processo corrente (`GetCurrentProcessId()`),
/// garantindo que NUNCA seja capturado o HWND de outro aplicativo em segundo ou primeiro plano.
pub fn get_visor_main_window_handle() -> Option<*mut c_void> {
    let current_pid = unsafe { GetCurrentProcessId() };

    // 1. Tenta a janela ativa na thread da UI
    let active = unsafe { GetActiveWindow() };
    if !active.is_null() && unsafe { IsWindow(active) } != 0 {
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(active, &mut pid) };
        if pid == current_pid {
            return Some(active);
        }
    }

    // 2. Tenta a janela em primeiro plano caso pertença ao VISOR
    let fg = unsafe { GetForegroundWindow() };
    if !fg.is_null() && unsafe { IsWindow(fg) } != 0 {
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(fg, &mut pid) };
        if pid == current_pid {
            return Some(fg);
        }
    }

    // 3. Enumera as janelas da thread atual para encontrar uma janela do VISOR
    unsafe extern "system" fn enum_thread_wnd(hwnd: *mut c_void, lparam: isize) -> i32 {
        let out_ptr = lparam as *mut *mut c_void;
        if IsWindow(hwnd) != 0 {
            *out_ptr = hwnd;
            0 // Interrompe enumeração ao encontrar a primeira janela válida
        } else {
            1
        }
    }

    let thread_id = unsafe { GetCurrentThreadId() };
    let mut found: *mut c_void = ptr::null_mut();
    unsafe {
        EnumThreadWindows(
            thread_id,
            Some(enum_thread_wnd),
            &mut found as *mut _ as isize,
        );
    }
    if !found.is_null() && unsafe { IsWindow(found) } != 0 {
        return Some(found);
    }

    None
}

/// VTable base `IUnknown`.
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        this: *mut c_void,
        riid: *const GUID,
        ppv_object: *mut *mut c_void,
    ) -> i32,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

/// VTable completa da interface COM `IFileOperation` (Windows Shell).
#[repr(C)]
pub struct IFileOperationVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        this: *mut c_void,
        riid: *const GUID,
        ppv_object: *mut *mut c_void,
    ) -> i32,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Advise: unsafe extern "system" fn(
        this: *mut c_void,
        pfops: *mut c_void,
        pdw_cookie: *mut u32,
    ) -> i32,
    pub Unadvise: unsafe extern "system" fn(this: *mut c_void, dw_cookie: u32) -> i32,
    pub SetOperationFlags:
        unsafe extern "system" fn(this: *mut c_void, dw_operation_flags: u32) -> i32,
    pub SetProgressMessage:
        unsafe extern "system" fn(this: *mut c_void, psz_message: *const u16) -> i32,
    pub SetProgressDialog: unsafe extern "system" fn(this: *mut c_void, popd: *mut c_void) -> i32,
    pub SetProperties: unsafe extern "system" fn(this: *mut c_void, pproparray: *mut c_void) -> i32,
    pub SetOwnerWindow:
        unsafe extern "system" fn(this: *mut c_void, hwnd_owner: *mut c_void) -> i32,
    pub ApplyPropertiesToItem:
        unsafe extern "system" fn(this: *mut c_void, psi_item: *mut c_void) -> i32,
    pub ApplyPropertiesToItems:
        unsafe extern "system" fn(this: *mut c_void, punk_items: *mut c_void) -> i32,
    pub RenameItem: unsafe extern "system" fn(
        this: *mut c_void,
        psi_item: *mut c_void,
        psz_new_name: *const u16,
        pfops_item: *mut c_void,
    ) -> i32,
    pub RenameItems: unsafe extern "system" fn(
        this: *mut c_void,
        p_unk_items: *mut c_void,
        psz_new_name: *const u16,
    ) -> i32,
    pub MoveItem: unsafe extern "system" fn(
        this: *mut c_void,
        psi_item: *mut c_void,
        psi_destination_folder: *mut c_void,
        psz_new_name: *const u16,
        pfops_item: *mut c_void,
    ) -> i32,
    pub MoveItems: unsafe extern "system" fn(
        this: *mut c_void,
        punk_items: *mut c_void,
        psi_destination_folder: *mut c_void,
    ) -> i32,
    pub CopyItem: unsafe extern "system" fn(
        this: *mut c_void,
        psi_item: *mut c_void,
        psi_destination_folder: *mut c_void,
        psz_copy_name: *const u16,
        pfops_item: *mut c_void,
    ) -> i32,
    pub CopyItems: unsafe extern "system" fn(
        this: *mut c_void,
        punk_items: *mut c_void,
        psi_destination_folder: *mut c_void,
    ) -> i32,
    pub DeleteItem: unsafe extern "system" fn(
        this: *mut c_void,
        psi_item: *mut c_void,
        pfops_item: *mut c_void,
    ) -> i32,
    pub DeleteItems: unsafe extern "system" fn(this: *mut c_void, punk_items: *mut c_void) -> i32,
    pub NewItem: unsafe extern "system" fn(
        this: *mut c_void,
        psi_folder: *mut c_void,
        dw_file_attributes: u32,
        psz_name: *const u16,
        psz_template_name: *const u16,
        pfops_item: *mut c_void,
    ) -> i32,
    pub PerformOperations: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub GetAnyOperationsAborted:
        unsafe extern "system" fn(this: *mut c_void, pf_any_operations_aborted: *mut i32) -> i32,
}

#[repr(C)]
pub struct IFileOperation {
    pub lp_vtbl: *const IFileOperationVtbl,
}

/// Resultado detalhado e tipado da operação de envio para a Lixeira.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrashResult {
    /// Arquivo movido para a Lixeira do Windows com sucesso.
    Success,
    /// Usuário cancelou a operação no diálogo de confirmação do sistema.
    AbortedByUser,
    /// Erro durante a operação com descrição amigável.
    Failed(String),
}

/// Valida pré-requisitos de caminho para exclusão segura.
/// Retorna o caminho limpo (sem prefixos verbatim `\\?\`) para a API Shell.
pub fn validate_file_path(path: &Path) -> Result<String, TrashResult> {
    if path.as_os_str().is_empty() {
        return Err(TrashResult::Failed(
            "Caminho do arquivo está vazio.".to_owned(),
        ));
    }
    if !path.exists() {
        return Err(TrashResult::Failed(
            "Arquivo não encontrado no disco.".to_owned(),
        ));
    }
    if path.is_dir() {
        return Err(TrashResult::Failed(
            "Operação rejeitada: o caminho informado é uma pasta, não um arquivo individual."
                .to_owned(),
        ));
    }
    let abs = std::path::absolute(path)
        .map_err(|e| TrashResult::Failed(format!("Falha ao resolver caminho absoluto: {e}")))?;
    let path_str = abs.to_string_lossy();
    let clean = path_str.trim_start_matches(r"\\?\").to_owned();
    Ok(clean)
}

/// Implementação primária utilizando `IFileOperation`.
/// Deve ser executada em thread STA (fornecida pelo `ShellStaExecutor`).
unsafe fn delete_via_ifileoperation(
    clean_path: &str,
    hwnd: Option<*mut c_void>,
) -> Result<TrashResult, String> {
    let mut p_op: *mut IFileOperation = ptr::null_mut();
    let hr = CoCreateInstance(
        &CLSID_FILE_OPERATION,
        ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IFILE_OPERATION,
        &mut p_op as *mut _ as *mut *mut c_void,
    );
    if hr != 0 || p_op.is_null() {
        return Err(format!(
            "Falha ao instanciar IFileOperation: HRESULT 0x{hr:08X}"
        ));
    }

    struct OpGuard(*mut IFileOperation);
    impl Drop for OpGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let vtbl = &*(*self.0).lp_vtbl;
                    (vtbl.Release)(self.0 as *mut c_void);
                }
            }
        }
    }
    let _op_guard = OpGuard(p_op);
    let vtbl = &*(*p_op).lp_vtbl;

    let hr_flags = (vtbl.SetOperationFlags)(
        p_op as *mut c_void,
        FOFX_RECYCLEONDELETE | FOFX_ADDUNDORECORD,
    );
    if hr_flags != 0 {
        return Ok(TrashResult::Failed(format!(
            "SetOperationFlags falhou: HRESULT 0x{hr_flags:08X}"
        )));
    }

    if let Some(h) = hwnd {
        if !h.is_null() && unsafe { IsWindow(h) } != 0 {
            let _ = (vtbl.SetOwnerWindow)(p_op as *mut c_void, h);
        }
    }

    let mut wide_path: Vec<u16> = clean_path.encode_utf16().collect();
    wide_path.push(0);

    let mut p_item: *mut c_void = ptr::null_mut();
    let hr_item = SHCreateItemFromParsingName(
        wide_path.as_ptr(),
        ptr::null_mut(),
        &IID_ISHELL_ITEM,
        &mut p_item,
    );
    if hr_item != 0 || p_item.is_null() {
        return Ok(TrashResult::Failed(format!(
            "SHCreateItemFromParsingName falhou: HRESULT 0x{hr_item:08X}"
        )));
    }

    struct ItemGuard(*mut c_void);
    impl Drop for ItemGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let vtbl = *(self.0 as *mut *mut IUnknownVtbl);
                    ((*vtbl).Release)(self.0);
                }
            }
        }
    }
    let _item_guard = ItemGuard(p_item);

    let hr_del = (vtbl.DeleteItem)(p_op as *mut c_void, p_item, ptr::null_mut());
    if hr_del != 0 {
        return Ok(TrashResult::Failed(format!(
            "DeleteItem falhou: HRESULT 0x{hr_del:08X}"
        )));
    }

    let hr_perf = (vtbl.PerformOperations)(p_op as *mut c_void);

    let mut aborted: i32 = 0;
    let _ = (vtbl.GetAnyOperationsAborted)(p_op as *mut c_void, &mut aborted);
    if aborted != 0 {
        return Ok(TrashResult::AbortedByUser);
    }

    if hr_perf != 0 {
        return Ok(TrashResult::Failed(format!(
            "PerformOperations falhou: HRESULT 0x{hr_perf:08X}"
        )));
    }

    Ok(TrashResult::Success)
}

/// Fallback controlado utilizando `SHFileOperationW`.
/// Acionado estritamente se `IFileOperation` não puder ser instanciado antes da operação.
pub unsafe fn delete_via_shfileop(clean_path: &str, hwnd: Option<*mut c_void>) -> TrashResult {
    let mut wide: Vec<u16> = clean_path.encode_utf16().collect();
    wide.push(0);
    wide.push(0); // Exigência do Win32: sequência duplamente terminada com NUL

    let safe_hwnd = match hwnd {
        Some(h) if !h.is_null() && unsafe { IsWindow(h) } != 0 => h,
        _ => ptr::null_mut(),
    };

    let mut op = SHFILEOPSTRUCTW {
        hwnd: safe_hwnd,
        wFunc: FO_DELETE,
        pFrom: wide.as_ptr(),
        pTo: ptr::null(),
        fFlags: FOF_ALLOWUNDO as u16,
        fAnyOperationsAborted: 0,
        hNameMappings: ptr::null_mut(),
        lpszProgressTitle: ptr::null(),
    };

    let ret = SHFileOperationW(&mut op);
    if ret != 0 {
        return TrashResult::Failed(format!("SHFileOperationW falhou com código de erro {ret}"));
    }
    if op.fAnyOperationsAborted != 0 {
        return TrashResult::AbortedByUser;
    }

    TrashResult::Success
}

/// Move o arquivo informado para a Lixeira do Windows de forma segura.
///
/// Executa no apartamento STA atual:
/// 1. Valida o caminho do arquivo (rejeita vazios, inexistentes e diretórios).
/// 2. Executa preferencialmente `IFileOperation` com `FOFX_RECYCLEONDELETE`.
/// 3. Em caso de falha de inicialização COM prévia, executa fallback seguro com `SHFileOperationW`.
/// 4. Nunca executa exclusão permanente.
pub fn send_to_trash(path: &Path, hwnd: Option<*mut c_void>) -> TrashResult {
    let clean_path = match validate_file_path(path) {
        Ok(p) => p,
        Err(res) => return res,
    };

    match unsafe { delete_via_ifileoperation(&clean_path, hwnd) } {
        Ok(res) => res,
        Err(_init_err) => {
            // Fallback somente quando a interface COM não pôde ser criada
            unsafe { delete_via_shfileop(&clean_path, hwnd) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::shell_sta::ShellStaExecutor;
    use std::fs;

    #[test]
    fn test_validate_path_empty() {
        let res = validate_file_path(Path::new(""));
        assert!(matches!(res, Err(TrashResult::Failed(_))));
    }

    #[test]
    fn test_validate_path_nonexistent() {
        let res = validate_file_path(Path::new("C:\\nao_existe_visor_test_12345.xyz"));
        assert!(matches!(res, Err(TrashResult::Failed(_))));
    }

    #[test]
    fn test_validate_path_directory_rejected() {
        let temp_dir = std::env::temp_dir();
        let res = validate_file_path(&temp_dir);
        assert!(matches!(res, Err(TrashResult::Failed(_))));
    }

    #[test]
    fn test_validate_path_valid_file() {
        let temp_file = std::env::temp_dir().join("visor_val_path_test.tmp");
        fs::write(&temp_file, b"test").unwrap();
        let res = validate_file_path(&temp_file);
        let _ = fs::remove_file(&temp_file);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_path_spaces_and_unicode() {
        let file_name = "visor teste com espaços e acentuação ção é.tmp";
        let temp_file = std::env::temp_dir().join(file_name);
        fs::write(&temp_file, b"test unicode").unwrap();
        let res = validate_file_path(&temp_file);
        let _ = fs::remove_file(&temp_file);
        assert!(res.is_ok());
        let clean = res.unwrap();
        assert!(clean.contains("espaços e acentuação"));
    }

    #[test]
    fn test_send_to_trash_nonexistent() {
        let res = send_to_trash(Path::new("C:\\caminho_inexistente.xyz"), None);
        assert!(matches!(res, TrashResult::Failed(_)));
    }

    #[test]
    fn test_send_to_trash_in_sta_executor() {
        let executor = ShellStaExecutor::new().expect("STA executor");
        let (tx, rx) = std::sync::mpsc::channel();

        let temp_file = std::env::temp_dir().join("visor_recycle_bin_unit_test.tmp");
        fs::write(&temp_file, b"dados para lixeira").unwrap();
        assert!(temp_file.exists());

        let target = temp_file.clone();
        executor
            .spawn(move || {
                let res = send_to_trash(&target, None);
                let _ = tx.send(res);
            })
            .unwrap();

        let result = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(result, TrashResult::Success);
        // O arquivo foi movido para a Lixeira, logo não deve mais existir no local original!
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_validate_path_special_characters() {
        let file_name = "visor_special_!#()[]{}+=_.tmp";
        let temp_file = std::env::temp_dir().join(file_name);
        fs::write(&temp_file, b"special chars").unwrap();
        let res = validate_file_path(&temp_file);
        let _ = fs::remove_file(&temp_file);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_path_long_path() {
        // Gera um caminho longo válido
        let long_sub = "subdir_level_long_test_string_for_testing_length";
        let temp_dir = std::env::temp_dir().join(long_sub);
        let _ = fs::create_dir_all(&temp_dir);
        let temp_file = temp_dir.join("long_path_file_name_test.tmp");
        fs::write(&temp_file, b"long path").unwrap();

        let res = validate_file_path(&temp_file);
        let _ = fs::remove_file(&temp_file);
        let _ = fs::remove_dir(&temp_dir);
        assert!(res.is_ok());
    }

    #[test]
    fn test_fallback_shfileop_in_sta_executor() {
        let executor = ShellStaExecutor::new().expect("STA executor");
        let (tx, rx) = std::sync::mpsc::channel();

        let temp_file = std::env::temp_dir().join("visor_fallback_shfileop_test.tmp");
        fs::write(&temp_file, b"fallback recycle bin").unwrap();
        assert!(temp_file.exists());

        let target = temp_file.clone();
        executor
            .spawn(move || {
                let clean = validate_file_path(&target).unwrap();
                let res = unsafe { delete_via_shfileop(&clean, None) };
                let _ = tx.send(res);
            })
            .unwrap();

        let result = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(result, TrashResult::Success);
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_get_visor_main_window_handle_pid_isolation() {
        let current_pid = unsafe { GetCurrentProcessId() };
        if let Some(hwnd) = get_visor_main_window_handle() {
            assert_ne!(hwnd, ptr::null_mut());
            assert_ne!(unsafe { IsWindow(hwnd) }, 0);
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
            assert_eq!(
                pid, current_pid,
                "HWND retornado deve pertencer exclusivamente ao PID do VISOR"
            );
        }
    }
}
