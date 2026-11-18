//! Integração moderna com associações de arquivo e configurações do Windows (10 e 11).
//!
//! Este módulo gerencia o registro seguro e idempotente do VISOR como candidato
//! a aplicativo de imagens em `HKCU`, sem exigir privilégios de administrador (sem UAC),
//! sem manipular `UserChoice` e em estrita conformidade com as diretrizes do Windows.

use std::fs;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use winreg::RegKey;
use winreg::enums::*;

use super::paths::local_dir;

pub const PROG_ID: &str = "Visor.Image";
pub const APP_NAME: &str = "VISOR";
pub const APP_DESCRIPTION: &str = "Visualizador rápido e moderno de imagens para Windows";
pub const SETTINGS_DIRECTED_URI: &str = "ms-settings:defaultapps?registeredAppUser=VISOR";
pub const SETTINGS_FALLBACK_URI: &str = "ms-settings:defaultapps";

use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const CLSID_APPLICATION_ASSOCIATION_REGISTRATION: GUID = GUID {
    data1: 0x591209C7,
    data2: 0x767B,
    data3: 0x42B2,
    data4: [0x9F, 0xBA, 0x44, 0xEE, 0x46, 0x15, 0xF2, 0xC7],
};

pub const IID_IAPPLICATION_ASSOCIATION_REGISTRATION: GUID = GUID {
    data1: 0x4E530B0A,
    data2: 0xE611,
    data3: 0x4C77,
    data4: [0xA3, 0xAC, 0x90, 0x31, 0xD0, 0x22, 0x28, 0x1B],
};

pub const CLSCTX_INPROC_SERVER: u32 = 0x1;
pub const COINIT_APARTMENTTHREADED: u32 = 0x2;
pub const COINIT_DISABLE_OLE1DDE: u32 = 0x4;

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoInitializeEx(pv_reserved: *mut c_void, dw_co_init: u32) -> i32;
    fn CoUninitialize();
    fn CoCreateInstance(
        rclsid: *const GUID,
        p_unk_outer: *mut c_void,
        dw_cls_context: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> i32;
    fn CoTaskMemFree(pv: *mut c_void);
}

#[allow(non_snake_case, dead_code)]
#[repr(C)]
pub struct IApplicationAssociationRegistrationVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        this: *mut c_void,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> i32,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub QueryCurrentDefault: unsafe extern "system" fn(
        this: *mut c_void,
        pszQuery: *const u16,
        atQueryType: i32,
        alQueryLevel: i32,
        ppszAssociation: *mut *mut u16,
    ) -> i32,
    pub QueryAppIsDefault: unsafe extern "system" fn(
        this: *mut c_void,
        pszQuery: *const u16,
        atQueryType: i32,
        alQueryLevel: i32,
        pszAppRegistryName: *const u16,
        pfDefault: *mut i32,
    ) -> i32,
    pub QueryAppIsDefaultAll: unsafe extern "system" fn(
        this: *mut c_void,
        alQueryLevel: i32,
        pszAppRegistryName: *const u16,
        pfDefault: *mut i32,
    ) -> i32,
    pub SetAppAsDefault: unsafe extern "system" fn(
        this: *mut c_void,
        pszAppRegistryName: *const u16,
        pszSet: *const u16,
        atSetType: i32,
    ) -> i32,
    pub SetAppAsDefaultAll:
        unsafe extern "system" fn(this: *mut c_void, pszAppRegistryName: *const u16) -> i32,
    pub ClearUserAssociations: unsafe extern "system" fn(this: *mut c_void) -> i32,
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        lpoperation: *const u16,
        lpfile: *const u16,
        lpparameters: *const u16,
        lpdirectory: *const u16,
        nshowcmd: i32,
    ) -> isize;
}

#[link(name = "shlwapi")]
unsafe extern "system" {
    fn AssocQueryStringW(
        flags: u32,
        str: u32,
        psz_assoc: *const u16,
        psz_extra: *const u16,
        psz_out: *mut u16,
        pcch_out: *mut u32,
    ) -> i32;
}

pub const ASSOCF_NONE: u32 = 0x00000000;
#[allow(dead_code)]
pub const ASSOCSTR_COMMAND: u32 = 1;
pub const ASSOCSTR_EXECUTABLE: u32 = 2;
#[allow(dead_code)]
pub const ASSOCSTR_FRIENDLYAPPNAME: u32 = 4;
pub const ASSOCSTR_PROGID: u32 = 20;

/// Retorna a lista completa de extensões suportadas nativamente pelo VISOR para associação.
/// Inclui formatos com suporte completo (Full) e leitura verificada (ReadOnly e Partial),
/// excluindo estritamente formatos externos (ImageMagick) e não suportados.
pub fn target_extensions() -> Vec<&'static str> {
    let mut exts = visor::verified_extensions();
    for ext in visor::partial_extensions() {
        if !exts.contains(&ext) {
            exts.push(ext);
        }
    }
    exts
}

/// Normaliza uma extensão removendo espaços, pontos iniciais e convertendo para minúsculas.
pub fn normalize_extension(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_ascii_lowercase()
}

/// Verifica se um executável corresponde ao OpenWith.exe (seletor do sistema quando não há app padrão).
pub fn is_open_with_exe(exec: &Path) -> bool {
    let exec_str = exec.to_string_lossy();
    let clean_str = exec_str.trim().trim_matches('"').trim_matches('\'');
    let path = Path::new(clean_str);
    if let Some(file_name) = path.file_name() {
        if file_name
            .to_string_lossy()
            .eq_ignore_ascii_case("openwith.exe")
        {
            return true;
        }
    }
    false
}

/// Verifica se uma string de ProgID é nula, vazia ou o identificador especial 'Unknown' do Windows.
pub fn is_unknown_or_empty_progid(prog_id: &str) -> bool {
    let clean = prog_id.trim();
    clean.is_empty() || clean.eq_ignore_ascii_case("unknown")
}

/// Verifica se um executável corresponde ao executável oficial do VISOR.
/// Considera caminhos absolutos, aspas, maiúsculas/minúsculas, espaços e Unicode.
pub fn is_same_or_matching_visor_exe(exec: &Path, current_exe: Option<&Path>) -> bool {
    let exec_str = exec.to_string_lossy();
    let clean_str = exec_str.trim().trim_matches('"').trim_matches('\'');
    let clean_path = Path::new(clean_str);

    if let Some(curr) = current_exe {
        if let (Ok(can_exec), Ok(can_curr)) = (clean_path.canonicalize(), curr.canonicalize()) {
            if can_exec == can_curr {
                return true;
            }
        }
        let norm_exec = clean_str.replace('/', "\\").to_ascii_lowercase();
        let norm_curr = curr
            .to_string_lossy()
            .replace('/', "\\")
            .to_ascii_lowercase();
        if norm_exec == norm_curr {
            return true;
        }
    }

    if let Some(file_name) = clean_path.file_name() {
        if file_name
            .to_string_lossy()
            .eq_ignore_ascii_case("visor.exe")
        {
            return true;
        }
    }

    false
}

/// Dados de associação obtidos do Windows para uma determinada extensão.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtAssociationInfo {
    pub prog_id: Option<String>,
    pub executable: Option<PathBuf>,
    pub is_registered_app_default: bool,
    pub is_inconclusive: bool,
}

impl ExtAssociationInfo {
    /// Construtor simplificado para retrocompatibilidade em testes unitários.
    #[allow(dead_code)]
    pub fn simple(prog_id: Option<String>, executable: Option<PathBuf>) -> Self {
        Self {
            prog_id,
            executable,
            is_registered_app_default: false,
            is_inconclusive: false,
        }
    }

    /// Verifica se as informações comprovam que o VISOR é o aplicativo padrão desta extensão.
    pub fn matches_visor(&self, current_exe: Option<&Path>) -> bool {
        // 1. ProgID explícito como Visor.Image
        if let Some(prog_id) = &self.prog_id {
            if prog_id.eq_ignore_ascii_case(PROG_ID) {
                return true;
            }
        }
        // 2. Executável associado é o próprio visor.exe
        if let Some(exec) = &self.executable {
            if is_same_or_matching_visor_exe(exec, current_exe) {
                return true;
            }
        }
        // 3. Confirmado como aplicativo padrão pelo Windows através de IApplicationAssociationRegistration
        // sem nenhum aplicativo concorrente confirmado
        if self.is_registered_app_default && !self.has_confirmed_competing_app(current_exe) {
            return true;
        }
        false
    }

    /// Verifica se há outro aplicativo comprovadamente associado a esta extensão.
    pub fn has_confirmed_competing_app(&self, current_exe: Option<&Path>) -> bool {
        if let Some(exec) = &self.executable {
            if !is_same_or_matching_visor_exe(exec, current_exe) && !is_open_with_exe(exec) {
                return true;
            }
        }
        if let Some(prog_id) = &self.prog_id {
            if !prog_id.eq_ignore_ascii_case(PROG_ID) && !is_unknown_or_empty_progid(prog_id) {
                return true;
            }
        }
        false
    }

    /// Retorna o nome legível do aplicativo concorrente confirmado, se houver.
    pub fn competing_handler_name(&self, current_exe: Option<&Path>) -> Option<String> {
        if self.matches_visor(current_exe) {
            return None;
        }
        if let Some(exec) = &self.executable {
            if !is_same_or_matching_visor_exe(exec, current_exe) && !is_open_with_exe(exec) {
                if let Some(name) = exec.file_name() {
                    return Some(name.to_string_lossy().to_string());
                }
            }
        }
        if let Some(prog_id) = &self.prog_id {
            if !prog_id.eq_ignore_ascii_case(PROG_ID) && !is_unknown_or_empty_progid(prog_id) {
                return Some(prog_id.clone());
            }
        }
        None
    }
}

/// Estado lógico de associação de uma extensão com o VISOR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionStatus {
    /// O VISOR já é o aplicativo padrão daquela extensão.
    Configured,
    /// O VISOR ainda não é o aplicativo padrão e o usuário não a ignorou.
    Pending,
    /// O usuário decidiu que não deseja associar esta extensão ao VISOR.
    Ignored,
    /// Não foi possível determinar o resultado por falha na consulta.
    Unknown,
}

impl ExtensionStatus {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Configured => "Configurada",
            Self::Pending => "Pendente",
            Self::Ignored => "Ignorada",
            Self::Unknown => "Desconhecida",
        }
    }
}

/// Avaliação individual de uma extensão.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionAssessment {
    pub extension: String,
    pub status: ExtensionStatus,
    pub current_handler: Option<String>,
    pub error: Option<String>,
}

/// Estado agregado de todas as extensões do VISOR.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OverallAssociationState {
    pub assessments: Vec<ExtensionAssessment>,
    pub configured_count: usize,
    pub pending_count: usize,
    pub ignored_count: usize,
    pub unknown_count: usize,
    pub is_checking: bool,
    pub last_checked: Option<std::time::Instant>,
}

impl OverallAssociationState {
    pub fn pending_extensions(&self) -> Vec<String> {
        self.assessments
            .iter()
            .filter(|a| a.status == ExtensionStatus::Pending)
            .map(|a| a.extension.clone())
            .collect()
    }

    pub fn ignored_extensions(&self) -> Vec<String> {
        self.assessments
            .iter()
            .filter(|a| a.status == ExtensionStatus::Ignored)
            .map(|a| a.extension.clone())
            .collect()
    }

    #[allow(dead_code)]
    pub fn configured_extensions(&self) -> Vec<String> {
        self.assessments
            .iter()
            .filter(|a| a.status == ExtensionStatus::Configured)
            .map(|a| a.extension.clone())
            .collect()
    }

    pub fn has_pending(&self) -> bool {
        self.pending_count > 0
    }
}

/// Executa consulta a uma string de associação via API oficial `AssocQueryStringW`.
pub fn query_assoc_string(assoc_str: u32, wide_ext: &[u16]) -> Result<String, i32> {
    let mut buffer = vec![0u16; 1024];
    let mut len = buffer.len() as u32;

    let hr = unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            assoc_str,
            wide_ext.as_ptr(),
            std::ptr::null(),
            buffer.as_mut_ptr(),
            &mut len,
        )
    };

    if hr == 0 {
        let actual_len = buffer.iter().position(|&c| c == 0).unwrap_or(len as usize);
        let s = String::from_utf16_lossy(&buffer[..actual_len])
            .trim()
            .to_string();
        if s.is_empty() {
            Err(-2147023741)
        } else {
            Ok(s)
        }
    } else {
        Err(hr)
    }
}

/// Verifica se o código HRESULT representa ausência legítima de associação no Windows.
#[allow(dead_code)]
pub fn is_no_association_error(hr: i32) -> bool {
    // 0x80070483 = ERROR_NO_ASSOCIATION (-2147023741)
    // 0x80070002 = ERROR_FILE_NOT_FOUND (-2147024894)
    // 0x80004005 = E_FAIL (-2147467259)
    hr == -2147023741 || hr == -2147024894 || hr == -2147467259
}

/// Estrutura interna para resultados da consulta COM oficial.
#[derive(Clone, Debug, Default)]
struct ComAssociationQuery {
    pub user_choice_progid: Option<String>,
    pub effective_progid: Option<String>,
    pub is_app_default: bool,
}

/// Consulta o subsistema de aplicativos padrão do Windows via interface COM oficial IApplicationAssociationRegistration.
fn query_com_default_association(dot_ext: &str) -> Option<ComAssociationQuery> {
    unsafe {
        let hr_init = CoInitializeEx(
            null_mut(),
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        );
        let should_uninit = hr_init == 0; // S_OK

        let mut p_reg: *mut c_void = null_mut();
        let hr = CoCreateInstance(
            &CLSID_APPLICATION_ASSOCIATION_REGISTRATION,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IAPPLICATION_ASSOCIATION_REGISTRATION,
            &mut p_reg,
        );

        if hr != 0 || p_reg.is_null() {
            if should_uninit {
                CoUninitialize();
            }
            return None;
        }

        let vtbl = *(p_reg as *mut *mut IApplicationAssociationRegistrationVtbl);
        let wide_ext: Vec<u16> = dot_ext.encode_utf16().chain(std::iter::once(0)).collect();
        let wide_app: Vec<u16> = APP_NAME.encode_utf16().chain(std::iter::once(0)).collect();

        // 1. QueryCurrentDefault com AL_USER (1)
        let mut p_user: *mut u16 = null_mut();
        let hr_user = ((*vtbl).QueryCurrentDefault)(p_reg, wide_ext.as_ptr(), 0, 1, &mut p_user);
        let user_choice_progid = if hr_user == 0 && !p_user.is_null() {
            let s = wide_ptr_to_string(p_user);
            CoTaskMemFree(p_user as *mut c_void);
            if is_unknown_or_empty_progid(&s) {
                None
            } else {
                Some(s)
            }
        } else {
            None
        };

        // 2. QueryCurrentDefault com AL_EFFECTIVE (0)
        let mut p_eff: *mut u16 = null_mut();
        let hr_eff = ((*vtbl).QueryCurrentDefault)(p_reg, wide_ext.as_ptr(), 0, 0, &mut p_eff);
        let effective_progid = if hr_eff == 0 && !p_eff.is_null() {
            let s = wide_ptr_to_string(p_eff);
            CoTaskMemFree(p_eff as *mut c_void);
            if is_unknown_or_empty_progid(&s) {
                None
            } else {
                Some(s)
            }
        } else {
            None
        };

        // 3. QueryAppIsDefault com AL_EFFECTIVE (0) para "VISOR"
        let mut is_def: i32 = 0;
        let hr_app = ((*vtbl).QueryAppIsDefault)(
            p_reg,
            wide_ext.as_ptr(),
            0,
            0,
            wide_app.as_ptr(),
            &mut is_def,
        );
        let is_app_default = hr_app == 0 && is_def != 0;

        // Liberar instância COM
        ((*vtbl).Release)(p_reg);

        if should_uninit {
            CoUninitialize();
        }

        Some(ComAssociationQuery {
            user_choice_progid,
            effective_progid,
            is_app_default,
        })
    }
}

unsafe fn wide_ptr_to_string(p: *mut u16) -> String {
    let mut len = 0;
    unsafe {
        while *p.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(p, len);
        String::from_utf16_lossy(slice).trim().to_string()
    }
}

/// Consulta a associação real do Windows para uma extensão através de `IApplicationAssociationRegistration`
/// com fallback estruturado para `AssocQueryStringW`.
pub fn query_windows_association(ext: &str) -> Result<Option<ExtAssociationInfo>, String> {
    let clean = normalize_extension(ext);
    let dot_ext = format!(".{clean}");
    let wide_ext: Vec<u16> = dot_ext.encode_utf16().chain(std::iter::once(0)).collect();

    // 1. Consulta COM oficial de IApplicationAssociationRegistration
    let com_info = query_com_default_association(&dot_ext);

    // 2. Consulta via AssocQueryStringW (Win32 Shell)
    let prog_id_res = query_assoc_string(ASSOCSTR_PROGID, &wide_ext);
    let exec_res = query_assoc_string(ASSOCSTR_EXECUTABLE, &wide_ext);

    let raw_prog_id = com_info
        .as_ref()
        .and_then(|c| c.user_choice_progid.clone())
        .or_else(|| prog_id_res.ok())
        .or_else(|| com_info.as_ref().and_then(|c| c.effective_progid.clone()));

    let raw_exec = exec_res
        .ok()
        .map(|e| PathBuf::from(e.trim().trim_matches('"')));

    let is_app_default = com_info.as_ref().map_or(false, |c| c.is_app_default);

    // Filtrar OpenWith.exe e ProgID "Unknown"
    let is_open_with = raw_exec.as_ref().map_or(false, |e| is_open_with_exe(e));
    let is_unknown_prog = raw_prog_id
        .as_ref()
        .map_or(false, |p| is_unknown_or_empty_progid(p));

    let clean_prog_id = if is_unknown_prog { None } else { raw_prog_id };
    let clean_exec = if is_open_with { None } else { raw_exec };

    // Se o Windows reporta o VISOR como aplicativo padrão de Capabilities (is_app_default)
    // e não existe aplicativo concorrente confirmado:
    if is_app_default && clean_exec.is_none() && clean_prog_id.is_none() {
        return Ok(Some(ExtAssociationInfo {
            prog_id: Some(PROG_ID.to_string()),
            executable: None,
            is_registered_app_default: true,
            is_inconclusive: false,
        }));
    }

    let is_inconclusive =
        (is_open_with || is_unknown_prog) && clean_prog_id.is_none() && clean_exec.is_none();

    Ok(Some(ExtAssociationInfo {
        prog_id: clean_prog_id,
        executable: clean_exec,
        is_registered_app_default: is_app_default,
        is_inconclusive,
    }))
}

/// Classifica uma lista de extensões conforme a regra oficial de prioridades:
/// 1. Se o VISOR for o aplicativo padrão -> Configurada.
/// 2. Caso contrário, se estiver na lista de ignoradas -> Ignorada.
/// 3. Caso contrário, se houver aplicativo concorrente confirmado -> Pendente.
/// 4. Caso contrário, se a verificação for inconclusiva -> Desconhecida.
/// 5. Caso contrário (sem escolha padrão) -> Pendente.
pub fn classify_extensions<F>(
    extensions: &[&str],
    ignored_list: &[String],
    current_exe: Option<&Path>,
    query_fn: F,
) -> OverallAssociationState
where
    F: Fn(&str) -> Result<Option<ExtAssociationInfo>, String>,
{
    let mut assessments = Vec::with_capacity(extensions.len());
    let mut configured_count = 0;
    let mut pending_count = 0;
    let mut ignored_count = 0;
    let mut unknown_count = 0;

    let normalized_ignored: std::collections::HashSet<String> = ignored_list
        .iter()
        .map(|e| normalize_extension(e))
        .collect();

    for ext_raw in extensions {
        let ext = normalize_extension(ext_raw);
        let query_res = query_fn(&ext);

        let (status, current_handler, error) = match query_res {
            Ok(Some(info)) => {
                let competing = info.competing_handler_name(current_exe);
                if info.matches_visor(current_exe) {
                    (ExtensionStatus::Configured, Some(PROG_ID.to_string()), None)
                } else if normalized_ignored.contains(&ext) {
                    (ExtensionStatus::Ignored, competing, None)
                } else if info.has_confirmed_competing_app(current_exe) {
                    (ExtensionStatus::Pending, competing, None)
                } else if info.is_inconclusive {
                    (
                        ExtensionStatus::Unknown,
                        None,
                        Some("O Windows não permitiu confirmar o aplicativo padrão para esta extensão".to_string()),
                    )
                } else {
                    (ExtensionStatus::Pending, None, None)
                }
            }
            Ok(None) => {
                if normalized_ignored.contains(&ext) {
                    (ExtensionStatus::Ignored, None, None)
                } else {
                    (ExtensionStatus::Pending, None, None)
                }
            }
            Err(err) => (ExtensionStatus::Unknown, None, Some(err)),
        };

        match status {
            ExtensionStatus::Configured => configured_count += 1,
            ExtensionStatus::Pending => pending_count += 1,
            ExtensionStatus::Ignored => ignored_count += 1,
            ExtensionStatus::Unknown => unknown_count += 1,
        }

        assessments.push(ExtensionAssessment {
            extension: ext,
            status,
            current_handler,
            error,
        });
    }

    OverallAssociationState {
        assessments,
        configured_count,
        pending_count,
        ignored_count,
        unknown_count,
        is_checking: false,
        last_checked: Some(std::time::Instant::now()),
    }
}

/// Consulta e classifica todas as 49 extensões nativas suportadas pelo VISOR.
pub fn check_all_associations(ignored_list: &[String]) -> OverallAssociationState {
    let extensions = target_extensions();
    let current_exe = std::env::current_exe().ok();
    classify_extensions(
        &extensions,
        ignored_list,
        current_exe.as_deref(),
        query_windows_association,
    )
}

/// Relatório estruturado do processo de registro de associações.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssociationReport {
    pub prog_id_registered: bool,
    pub capabilities_registered: bool,
    pub registered_app_added: bool,
    pub open_with_extensions_count: usize,
    pub exe_path: PathBuf,
}

impl AssociationReport {
    pub fn summary(&self) -> String {
        format!(
            "VISOR registrado com sucesso em HKCU como candidato para {} extensões de imagem.\nExecutável: {}",
            self.open_with_extensions_count,
            self.exe_path.display()
        )
    }
}

/// Abre a página de aplicativos padrão nas Configurações do Windows.
///
/// Tenta primeiramente a URI direcionada ao VISOR (`ms-settings:defaultapps?registeredAppUser=VISOR`)
/// suportada pelo Windows 11. Em caso de insucesso (ex: Windows 10 legado), tenta o fallback
/// geral (`ms-settings:defaultapps`).
pub fn open_default_apps_settings() -> Result<(), String> {
    // 1. Tenta URI direcionada do Windows 11
    if launch_uri(SETTINGS_DIRECTED_URI) {
        return Ok(());
    }

    // 2. Fallback para Windows 10
    if launch_uri(SETTINGS_FALLBACK_URI) {
        return Ok(());
    }

    Err("Não foi possível abrir as Configurações do Windows através das URIs oficiais.".to_owned())
}

/// Lança uma URI de sistema via ShellExecuteW sem abrir janela do prompt de comando.
fn launch_uri(uri: &str) -> bool {
    let wide_uri: Vec<u16> = uri.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_op: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();

    // SW_SHOWNORMAL = 1
    let res = unsafe {
        ShellExecuteW(
            null_mut(),
            wide_op.as_ptr(),
            wide_uri.as_ptr(),
            null_mut(),
            null_mut(),
            1,
        )
    };

    // Valores maiores que 32 indicam sucesso em ShellExecute
    res > 32
}

/// Registra o executável atual do VISOR no registro do Windows do usuário (`HKCU`).
///
/// Esta operação é estritamente idempotente e portátil:
/// - Se o executável for movido de pasta, atualiza os caminhos sem duplicar chaves.
/// - Nunca modifica `UserChoice` ou sobrescreve preferências existentes.
pub fn register_current_exe() -> Result<AssociationReport, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Não foi possível determinar o caminho do executável: {e}"))?;

    // NUNCA registrar no registro real do sistema binários em diretórios de build/teste
    let exe_str = exe.to_string_lossy().to_ascii_lowercase();
    if exe_str.contains("target\\")
        || exe_str.contains("target/")
        || exe_str.contains("deps\\")
        || exe_str.contains("deps/")
        || (!exe_str.ends_with("visor.exe") && exe_str.contains('-'))
    {
        return Ok(AssociationReport {
            prog_id_registered: false,
            capabilities_registered: false,
            registered_app_added: false,
            open_with_extensions_count: 0,
            exe_path: exe,
        });
    }

    register_exe_at_root(&RegKey::predef(HKEY_CURRENT_USER), "", &exe)
}

/// Assegura o registro do VISOR em segundo plano se necessário (ex: executável movido ou primeira execução).
/// Não bloqueia a thread de UI e nunca abre as Configurações compulsoriamente.
#[allow(dead_code)]
pub fn ensure_registered_in_background() {
    if cfg!(test) {
        return;
    }
    std::thread::spawn(|| {
        let _ = register_current_exe();
    });
}

/// Função de registro centralizada e isolável (utilizada em produção e testes com prefixos isolados).
pub fn register_exe_at_root(
    root: &RegKey,
    prefix: &str,
    exe_path: &Path,
) -> Result<AssociationReport, String> {
    let base_prefix = if prefix.is_empty() {
        String::new()
    } else {
        format!("{prefix}\\")
    };

    let exe_display = exe_path.display().to_string();
    let command_value = format!("\"{exe_display}\" \"%1\"");
    let icon_value = format!("\"{exe_display}\",0");

    // 1. ProgID: HKCU\Software\Classes\Visor.Image
    let prog_key_path = format!("{base_prefix}Software\\Classes\\{PROG_ID}");
    let (prog_key, _) = root
        .create_subkey(&prog_key_path)
        .map_err(|e| format!("Falha ao criar chave {prog_key_path}: {e}"))?;
    prog_key
        .set_value("", &"VISOR Image")
        .map_err(|e| format!("Falha ao definir nome do ProgID: {e}"))?;

    let icon_key_path = format!("{base_prefix}Software\\Classes\\{PROG_ID}\\DefaultIcon");
    let (icon_key, _) = root
        .create_subkey(&icon_key_path)
        .map_err(|e| format!("Falha ao criar chave {icon_key_path}: {e}"))?;
    icon_key
        .set_value("", &icon_value)
        .map_err(|e| format!("Falha ao definir DefaultIcon: {e}"))?;

    let cmd_key_path = format!("{base_prefix}Software\\Classes\\{PROG_ID}\\shell\\open\\command");
    let (cmd_key, _) = root
        .create_subkey(&cmd_key_path)
        .map_err(|e| format!("Falha ao criar chave {cmd_key_path}: {e}"))?;
    cmd_key
        .set_value("", &command_value)
        .map_err(|e| format!("Falha ao definir shell open command: {e}"))?;

    // 2. Capabilities: HKCU\Software\Visor\Capabilities
    let caps_key_path = format!("{base_prefix}Software\\Visor\\Capabilities");
    let (caps_key, _) = root
        .create_subkey(&caps_key_path)
        .map_err(|e| format!("Falha ao criar chave {caps_key_path}: {e}"))?;
    caps_key
        .set_value("ApplicationName", &APP_NAME)
        .map_err(|e| format!("Falha ao definir ApplicationName: {e}"))?;
    caps_key
        .set_value("ApplicationDescription", &APP_DESCRIPTION)
        .map_err(|e| format!("Falha ao definir ApplicationDescription: {e}"))?;

    let file_assoc_path = format!("{caps_key_path}\\FileAssociations");
    let (file_assoc_key, _) = root
        .create_subkey(&file_assoc_path)
        .map_err(|e| format!("Falha ao criar chave {file_assoc_path}: {e}"))?;

    let extensions = target_extensions();
    for ext in &extensions {
        let ext_name = format!(".{ext}");
        file_assoc_key
            .set_value(&ext_name, &PROG_ID)
            .map_err(|e| format!("Falha ao associar {ext_name} em Capabilities: {e}"))?;
    }

    // 3. RegisteredApplications: HKCU\Software\RegisteredApplications
    let reg_apps_path = format!("{base_prefix}Software\\RegisteredApplications");
    let (reg_apps_key, _) = root
        .create_subkey(&reg_apps_path)
        .map_err(|e| format!("Falha ao criar chave {reg_apps_path}: {e}"))?;
    let relative_caps = if prefix.is_empty() {
        "Software\\Visor\\Capabilities".to_string()
    } else {
        format!("{prefix}\\Software\\Visor\\Capabilities")
    };
    reg_apps_key
        .set_value(APP_NAME, &relative_caps)
        .map_err(|e| format!("Falha ao definir RegisteredApplications {APP_NAME}: {e}"))?;

    // 4. OpenWithProgids: HKCU\Software\Classes\.ext\OpenWithProgids
    for ext in &extensions {
        let open_with_path = format!("{base_prefix}Software\\Classes\\.{ext}\\OpenWithProgids");
        if let Ok((open_with_key, _)) = root.create_subkey(&open_with_path) {
            // Valor com nome do ProgID e dado vazio
            let _ = open_with_key.set_value(PROG_ID, &"");
        }
    }

    Ok(AssociationReport {
        prog_id_registered: true,
        capabilities_registered: true,
        registered_app_added: true,
        open_with_extensions_count: extensions.len(),
        exe_path: exe_path.to_path_buf(),
    })
}

/// Mantido para compatibilidade temporária com comandos CLI existentes.
#[allow(dead_code)]
pub fn register_and_store(_open_settings_on_failure: bool) -> String {
    match register_current_exe() {
        Ok(report) => {
            let dir = local_dir();
            let _ = fs::create_dir_all(&dir);
            let summary = report.summary();
            let _ = fs::write(dir.join("assoc-report.txt"), &summary);
            summary
        }
        Err(err) => {
            let msg = format!("Falha no registro de associações do VISOR: {err}");
            let dir = local_dir();
            let _ = fs::create_dir_all(&dir);
            let _ = fs::write(dir.join("assoc-report.txt"), &msg);
            msg
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(1);

    fn unique_test_prefix() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        format!("Software\\Visor_UnitTest_{pid}_{id}")
    }

    struct RegistryTestGuard {
        root: RegKey,
        prefix: String,
    }

    impl RegistryTestGuard {
        fn new() -> Self {
            let root = RegKey::predef(HKEY_CURRENT_USER);
            let prefix = unique_test_prefix();
            Self { root, prefix }
        }
    }

    impl Drop for RegistryTestGuard {
        fn drop(&mut self) {
            // Limpa a árvore temporária de teste
            let _ = self.root.delete_subkey_all(&self.prefix);
        }
    }

    #[test]
    fn test_target_extensions_inventory_contains_key_formats_and_no_unsupported() {
        let exts = target_extensions();

        // Deve conter os formatos de leitura nativa
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"webp"));
        assert!(exts.contains(&"dds"));
        assert!(exts.contains(&"svg"));
        assert!(exts.contains(&"exr"));
        assert!(exts.contains(&"jxl"));
        assert!(exts.contains(&"qoi"));

        // Formatos novos suportados devem estar presentes
        assert!(exts.contains(&"avif"));
        assert!(exts.contains(&"cr3"));
        assert!(exts.contains(&"wmf"));
        assert!(exts.contains(&"cdr"));
        assert!(!exts.contains(&"json"));

        // Não deve conter formatos externos que exigem ImageMagick
        assert!(!exts.contains(&"pdf"));
        assert!(!exts.contains(&"ai"));
        assert!(!exts.contains(&"ps"));
    }

    #[test]
    fn test_idempotent_registration_in_isolated_test_registry() {
        let guard = RegistryTestGuard::new();
        let exe_path = PathBuf::from(r"C:\Program Files\Visor App\visor.exe");

        // 1. Registro inicial
        let report1 = register_exe_at_root(&guard.root, &guard.prefix, &exe_path)
            .expect("primeiro registro falhou");
        assert!(report1.prog_id_registered);
        assert!(report1.capabilities_registered);
        assert!(report1.registered_app_added);
        assert!(report1.open_with_extensions_count >= 40);

        // Verifica ProgID
        let cmd_key = guard
            .root
            .open_subkey(format!(
                "{}\\Software\\Classes\\Visor.Image\\shell\\open\\command",
                guard.prefix
            ))
            .expect("chave command do ProgID deve existir");
        let cmd_val: String = cmd_key.get_value("").expect("valor default deve existir");
        assert_eq!(
            cmd_val,
            "\"C:\\Program Files\\Visor App\\visor.exe\" \"%1\""
        );

        // Verifica DefaultIcon
        let icon_key = guard
            .root
            .open_subkey(format!(
                "{}\\Software\\Classes\\Visor.Image\\DefaultIcon",
                guard.prefix
            ))
            .expect("chave DefaultIcon deve existir");
        let icon_val: String = icon_key.get_value("").expect("valor de ícone deve existir");
        assert_eq!(icon_val, "\"C:\\Program Files\\Visor App\\visor.exe\",0");

        // Verifica Capabilities
        let caps_key = guard
            .root
            .open_subkey(format!("{}\\Software\\Visor\\Capabilities", guard.prefix))
            .expect("chave Capabilities deve existir");
        let app_name: String = caps_key
            .get_value("ApplicationName")
            .expect("ApplicationName deve existir");
        assert_eq!(app_name, "VISOR");

        let file_assoc_key = guard
            .root
            .open_subkey(format!(
                "{}\\Software\\Visor\\Capabilities\\FileAssociations",
                guard.prefix
            ))
            .expect("chave FileAssociations deve existir");
        let png_assoc: String = file_assoc_key
            .get_value(".png")
            .expect(".png deve estar associado");
        assert_eq!(png_assoc, "Visor.Image");

        // Verifica RegisteredApplications
        let reg_apps_key = guard
            .root
            .open_subkey(format!(
                "{}\\Software\\RegisteredApplications",
                guard.prefix
            ))
            .expect("chave RegisteredApplications deve existir");
        let reg_val: String = reg_apps_key
            .get_value("VISOR")
            .expect("VISOR em RegisteredApplications deve existir");
        assert!(reg_val.ends_with("Software\\Visor\\Capabilities"));

        // Verifica OpenWithProgids
        let open_with_key = guard
            .root
            .open_subkey(format!(
                "{}\\Software\\Classes\\.png\\OpenWithProgids",
                guard.prefix
            ))
            .expect("chave OpenWithProgids para .png deve existir");
        let _visor_open_with: String = open_with_key
            .get_value("Visor.Image")
            .expect("Visor.Image deve estar presente em OpenWithProgids");

        // 2. Segunda execução (idempotência): não pode falhar nem corromper
        let report2 = register_exe_at_root(&guard.root, &guard.prefix, &exe_path)
            .expect("segundo registro deve ser idempotente");
        assert_eq!(report1, report2);
    }

    #[test]
    fn test_preservation_of_other_applications_and_default_associations() {
        let guard = RegistryTestGuard::new();

        // Cria uma associação existente prévia simulada de outro programa
        let png_key_path = format!("{}\\Software\\Classes\\.png", guard.prefix);
        let (png_key, _) = guard.root.create_subkey(&png_key_path).unwrap();
        png_key.set_value("", &"OutroPrograma.Image").unwrap();

        let open_with_path = format!("{png_key_path}\\OpenWithProgids");
        let (open_with_key, _) = guard.root.create_subkey(&open_with_path).unwrap();
        open_with_key.set_value("OutroPrograma.Image", &"").unwrap();

        // Executa o registro do VISOR
        let exe_path = PathBuf::from(r"C:\Visor\visor.exe");
        register_exe_at_root(&guard.root, &guard.prefix, &exe_path).unwrap();

        // 1. Confirma que o valor padrão de .png NÃO foi substituído (permanece OutroPrograma.Image)
        let png_key_reloaded = guard.root.open_subkey(&png_key_path).unwrap();
        let default_val: String = png_key_reloaded.get_value("").unwrap();
        assert_eq!(
            default_val, "OutroPrograma.Image",
            "O valor padrão de .png NUNCA deve ser sobrescrito pelo registro do VISOR"
        );

        // 2. Confirma que o outro programa permanece em OpenWithProgids
        let open_with_reloaded = guard.root.open_subkey(&open_with_path).unwrap();
        let other_prog: Result<String, _> = open_with_reloaded.get_value("OutroPrograma.Image");
        assert!(
            other_prog.is_ok(),
            "Outro programa deve continuar registrado em OpenWithProgids"
        );

        // 3. Confirma que o VISOR também foi adicionado
        let visor_prog: Result<String, _> = open_with_reloaded.get_value("Visor.Image");
        assert!(
            visor_prog.is_ok(),
            "Visor.Image deve ter sido adicionado como opção adicional"
        );
    }

    #[test]
    fn test_executable_moved_portable_path_update() {
        let guard = RegistryTestGuard::new();

        // 1. Registra com caminho inicial A
        let exe_path_a = PathBuf::from(r"D:\Apps\AntigaPasta\visor.exe");
        register_exe_at_root(&guard.root, &guard.prefix, &exe_path_a).unwrap();

        let cmd_key_path = format!(
            "{}\\Software\\Classes\\Visor.Image\\shell\\open\\command",
            guard.prefix
        );
        let cmd_key = guard.root.open_subkey(&cmd_key_path).unwrap();
        let val_a: String = cmd_key.get_value("").unwrap();
        assert_eq!(val_a, "\"D:\\Apps\\AntigaPasta\\visor.exe\" \"%1\"");

        // 2. Aplicativo é movido para pasta B
        let exe_path_b = PathBuf::from(r"C:\Programas\NovaPasta\visor.exe");
        register_exe_at_root(&guard.root, &guard.prefix, &exe_path_b).unwrap();

        let cmd_key_updated = guard.root.open_subkey(&cmd_key_path).unwrap();
        let val_b: String = cmd_key_updated.get_value("").unwrap();
        assert_eq!(val_b, "\"C:\\Programas\\NovaPasta\\visor.exe\" \"%1\"");
    }

    #[test]
    fn test_uri_constants_and_formatting() {
        assert_eq!(
            SETTINGS_DIRECTED_URI,
            "ms-settings:defaultapps?registeredAppUser=VISOR"
        );
        assert_eq!(SETTINGS_FALLBACK_URI, "ms-settings:defaultapps");
    }

    // =========================================================================
    // TESTES OBRIGATÓRIOS DO SISTEMA INTELIGENTE DE ASSOCIAÇÕES (SEÇÃO 11)
    // =========================================================================

    #[test]
    fn test_1_all_extensions_configured() {
        let exts = vec!["png", "jpg", "jpeg", "webp"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        // Mock: todas as extensões retornam Visor.Image
        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo::simple(
                Some("Visor.Image".to_string()),
                Some(PathBuf::from(r"C:\Visor\visor.exe")),
            )))
        });

        assert_eq!(state.pending_count, 0);
        assert_eq!(state.configured_count, 4);
        assert!(
            !state.has_pending(),
            "Banner deve ficar oculto se todas estiverem configuradas"
        );
        assert!(state.pending_extensions().is_empty());
    }

    #[test]
    fn test_2_some_pending_extensions() {
        let exts = vec!["png", "jpg", "webp", "tiff"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |ext| {
            match ext {
                "png" | "jpg" => Ok(Some(ExtAssociationInfo::simple(
                    Some("Visor.Image".to_string()),
                    Some(PathBuf::from(r"C:\Visor\visor.exe")),
                ))),
                "webp" => Ok(Some(ExtAssociationInfo::simple(
                    Some("ChromeHTML".to_string()),
                    Some(PathBuf::from(r"C:\Chrome\chrome.exe")),
                ))),
                "tiff" => Ok(None), // Sem associação
                _ => Ok(None),
            }
        });

        assert_eq!(state.configured_count, 2);
        assert_eq!(state.pending_count, 2);
        let pending = state.pending_extensions();
        assert_eq!(
            pending,
            vec!["webp".to_string(), "tiff".to_string()],
            "Somente webp e tiff devem aparecer como pendentes"
        );
    }

    #[test]
    fn test_3_ignore_extension_removes_from_pending_and_persists() {
        let exts = vec!["webp", "tiff"];
        let mut ignored = Vec::new();

        // 1. Inicialmente webp e tiff estão pendentes
        let state1 = classify_extensions(&exts, &ignored, None, |_ext| Ok(None));
        assert_eq!(state1.pending_count, 2);

        // 2. Usuário ignora webp
        let mut settings = crate::system::config::AppSettings::default();
        settings.ignore_extension("webp");
        ignored = settings.ignored_default_extensions.clone();

        // 3. Reclassifica com webp na lista de ignoradas
        let state2 = classify_extensions(&exts, &ignored, None, |_ext| Ok(None));
        assert_eq!(state2.pending_count, 1);
        assert_eq!(state2.ignored_count, 1);
        assert_eq!(state2.pending_extensions(), vec!["tiff".to_string()]);
        assert_eq!(state2.ignored_extensions(), vec!["webp".to_string()]);
    }

    #[test]
    fn test_4_persistence_after_restart() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_test_persist_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("settings.json");

        // 1. Ignora webp e salva
        let mut settings = crate::system::config::AppSettings::default();
        settings.ignore_extension("webp");
        settings.save_to_path(&file_path).unwrap();

        // 2. Destrói estado em memória e recarrega do disco
        let (reloaded, err) = crate::system::config::AppSettings::load_from_path(&file_path);
        assert!(err.is_none());
        assert!(reloaded.is_extension_ignored("webp"));

        // 3. Valida que webp não aparece como pendente após restart
        let exts = vec!["webp", "png"];
        let state =
            classify_extensions(&exts, &reloaded.ignored_default_extensions, None, |_ext| {
                Ok(None)
            });
        assert!(!state.pending_extensions().contains(&"webp".to_string()));
        assert_eq!(state.ignored_extensions(), vec!["webp".to_string()]);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_5_all_ignored_hides_banner() {
        let exts = vec!["webp", "tiff", "dds"];
        let ignored = vec!["webp".to_string(), "tiff".to_string(), "dds".to_string()];

        let state = classify_extensions(&exts, &ignored, None, |_ext| Ok(None));
        assert_eq!(state.pending_count, 0);
        assert_eq!(state.ignored_count, 3);
        assert!(
            !state.has_pending(),
            "Banner deve ficar completamente oculto quando todas forem ignoradas"
        );
    }

    #[test]
    fn test_6_unignore_extension_restores_pending_status() {
        let mut settings = crate::system::config::AppSettings::default();
        settings.ignore_extension("svg");
        assert!(settings.is_extension_ignored("svg"));

        // Desfaz o ignoramento
        settings.unignore_extension("svg");
        assert!(!settings.is_extension_ignored("svg"));

        // Reavalia com SVG ainda não sendo padrão no Windows
        let exts = vec!["svg"];
        let state =
            classify_extensions(&exts, &settings.ignored_default_extensions, None, |_ext| {
                Ok(None)
            });
        assert_eq!(state.pending_count, 1);
        assert_eq!(state.pending_extensions(), vec!["svg".to_string()]);
    }

    #[test]
    fn test_7_external_association_change_detected() {
        let exts = vec!["webp"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        // 1. Inicialmente WEBP associado ao navegador
        let state1 = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo::simple(
                Some("ChromeHTML".to_string()),
                Some(PathBuf::from(r"C:\Chrome\chrome.exe")),
            )))
        });
        assert_eq!(state1.pending_count, 1);

        // 2. Usuário altera nas Configurações do Windows para o VISOR
        let state2 = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo::simple(
                Some("Visor.Image".to_string()),
                Some(PathBuf::from(r"C:\Visor\visor.exe")),
            )))
        });
        assert_eq!(state2.pending_count, 0);
        assert_eq!(state2.configured_count, 1);
        assert!(!state2.has_pending());
    }

    #[test]
    fn test_8_query_error_classified_as_unknown_not_falsely_pending() {
        let exts = vec!["webp", "png"];
        let ignored = Vec::new();

        let state = classify_extensions(&exts, &ignored, None, |ext| match ext {
            "webp" => Err("Erro simulado de acesso à API de registro".to_string()),
            "png" => Ok(None),
            _ => Ok(None),
        });

        // webp deve ser Unknown (com mensagem de diagnóstico) e NÃO pendente
        assert_eq!(state.unknown_count, 1);
        assert_eq!(state.pending_count, 1);
        assert_eq!(state.pending_extensions(), vec!["png".to_string()]);

        let webp_assessment = state
            .assessments
            .iter()
            .find(|a| a.extension == "webp")
            .unwrap();
        assert_eq!(webp_assessment.status, ExtensionStatus::Unknown);
        assert!(webp_assessment.error.is_some());
    }

    #[test]
    fn test_9_legacy_config_migration() {
        let temp_dir = std::env::temp_dir().join(format!("visor_test_migr_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("legacy_settings.json");

        // JSON de versão antiga sem ignored_default_extensions e com schema v2
        let legacy_json = r#"{
            "schema_version": 2,
            "mouse_wheel_action": "zoom",
            "show_filmstrip": true,
            "association_prompt_dismissed": true
        }"#;
        std::fs::write(&file_path, legacy_json).unwrap();

        let (loaded, notice) = crate::system::config::AppSettings::load_from_path(&file_path);
        assert!(notice.is_none());
        assert_eq!(
            loaded.schema_version,
            crate::system::config::CURRENT_SCHEMA_VERSION
        );
        // Como association_prompt_dismissed era true, deve migrar para a lista de ignoradas
        assert!(!loaded.ignored_default_extensions.is_empty());
        assert!(loaded.is_extension_ignored("png"));
        assert!(loaded.is_extension_ignored("jpg"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_10_future_schema_compatibility() {
        let temp_dir =
            std::env::temp_dir().join(format!("visor_test_future_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("future_settings.json");

        let future_json = r#"{
            "schema_version": 99,
            "future_field_abc": 12345
        }"#;
        std::fs::write(&file_path, future_json).unwrap();

        let (loaded, notice) = crate::system::config::AppSettings::load_from_path(&file_path);
        assert!(notice.is_some());
        assert!(loaded.is_future_version);
        assert_eq!(loaded.schema_version, 99);

        // Tentativa de gravação deve ser bloqueada para não corromper o arquivo futuro
        let save_res = loaded.save_to_path(&file_path);
        assert!(save_res.is_err());

        // Confirma que o arquivo original não foi sobrescrito
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("future_field_abc"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_11_normalization_equivalence() {
        let variants = vec!["webp", ".webp", "WEBP", ".WEBP", "  .webp  "];
        for v in variants {
            assert_eq!(normalize_extension(v), "webp");
        }

        let mut settings = crate::system::config::AppSettings::default();
        settings.ignore_extension("WEBP");
        settings.ignore_extension(".webp");
        settings.ignore_extension("webp");
        assert_eq!(settings.ignored_default_extensions.len(), 1);
        assert_eq!(settings.ignored_default_extensions[0], "webp");
    }

    #[test]
    fn test_12_previously_ignored_now_configured() {
        let exts = vec!["webp"];
        // Usuário havia ignorado webp anteriormente
        let ignored = vec!["webp".to_string()];
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        // Mas o Windows agora tem o VISOR como padrão para webp
        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo::simple(
                Some("Visor.Image".to_string()),
                Some(PathBuf::from(r"C:\Visor\visor.exe")),
            )))
        });

        // Prioridade 1: Configurado sobrepõe o Ignorado!
        assert_eq!(state.configured_count, 1);
        assert_eq!(state.pending_count, 0);
        assert_eq!(state.ignored_count, 0);
        assert!(!state.has_pending(), "Não deve haver pendências nem banner");
    }

    // =========================================================================
    // NOVOS TESTES OBRIGATÓRIOS DA AUDITORIA DE CORREÇÃO DO VERIFICADOR
    // =========================================================================

    #[test]
    fn test_13_mrw_classified_as_configured_when_registered_app_default() {
        let exts = vec!["mrw"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Users\bruno\AppData\Local\Programs\Visor\visor.exe");

        // Simula exatamente o caso real do Windows 11 com UserChoice órfão:
        // AssocQueryString retornou OpenWith.exe com ProgID Unknown,
        // MAS QueryAppIsDefault do Windows confirmou is_registered_app_default = true!
        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo {
                prog_id: None,
                executable: None,
                is_registered_app_default: true,
                is_inconclusive: false,
            }))
        });

        assert_eq!(state.configured_count, 1);
        assert_eq!(state.pending_count, 0);
        assert_eq!(state.unknown_count, 0);
        assert!(
            !state.has_pending(),
            "MRW configurado como default nas Capabilities não pode gerar falso alerta"
        );
        assert_eq!(state.configured_extensions(), vec!["mrw".to_string()]);
    }

    #[test]
    fn test_14_extension_registered_but_divergent_other_app_is_pending() {
        let exts = vec!["mrw", "png"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        // Simula o caso em que o usuário REALMENTE escolheu o Photoshop para abrir MRW
        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |ext| match ext {
            "mrw" => Ok(Some(ExtAssociationInfo {
                prog_id: Some("Photoshop.Image.24".to_string()),
                executable: Some(PathBuf::from(r"C:\Adobe\photoshop.exe")),
                is_registered_app_default: false,
                is_inconclusive: false,
            })),
            "png" => Ok(Some(ExtAssociationInfo::simple(
                Some("Visor.Image".to_string()),
                Some(PathBuf::from(r"C:\Visor\visor.exe")),
            ))),
            _ => Ok(None),
        });

        assert_eq!(state.configured_count, 1);
        assert_eq!(state.pending_count, 1);
        assert_eq!(state.pending_extensions(), vec!["mrw".to_string()]);
        let mrw_item = state
            .assessments
            .iter()
            .find(|a| a.extension == "mrw")
            .unwrap();
        assert_eq!(mrw_item.status, ExtensionStatus::Pending);
        assert_eq!(mrw_item.current_handler.as_deref(), Some("photoshop.exe"));
    }

    #[test]
    fn test_15_inconclusive_openwith_without_app_default_is_unknown_not_pending() {
        let exts = vec!["raw"];
        let ignored = Vec::new();
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        // Simula quando não há app default e o Windows retorna OpenWith.exe / Unknown
        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |_ext| {
            Ok(Some(ExtAssociationInfo {
                prog_id: None,
                executable: None,
                is_registered_app_default: false,
                is_inconclusive: true,
            }))
        });

        assert_eq!(state.configured_count, 0);
        assert_eq!(state.pending_count, 0);
        assert_eq!(state.unknown_count, 1);
        assert!(
            !state.has_pending(),
            "Inconclusivo não deve gerar alerta de pendência contra o usuário"
        );
    }

    #[test]
    fn test_16_counter_accuracy_and_total_elegible_integrity() {
        let exts = vec!["png", "jpg", "gif", "mrw", "raw", "webp"];
        let ignored = vec!["raw".to_string()];
        let curr_exe = PathBuf::from(r"C:\Visor\visor.exe");

        let state = classify_extensions(&exts, &ignored, Some(&curr_exe), |ext| match ext {
            "png" | "jpg" | "gif" => Ok(Some(ExtAssociationInfo::simple(
                Some("Visor.Image".to_string()),
                Some(PathBuf::from(r"C:\Visor\visor.exe")),
            ))),
            "mrw" => Ok(Some(ExtAssociationInfo {
                prog_id: None,
                executable: None,
                is_registered_app_default: true,
                is_inconclusive: false,
            })),
            "raw" => Ok(None),
            "webp" => Ok(Some(ExtAssociationInfo::simple(
                Some("ChromeHTML".to_string()),
                Some(PathBuf::from(r"C:\Chrome\chrome.exe")),
            ))),
            _ => Ok(None),
        });

        assert_eq!(state.configured_count, 4); // png, jpg, gif, mrw
        assert_eq!(state.ignored_count, 1); // raw
        assert_eq!(state.pending_count, 1); // webp (Chrome)
        assert_eq!(state.unknown_count, 0);
        assert_eq!(
            state.configured_count
                + state.ignored_count
                + state.pending_count
                + state.unknown_count,
            exts.len(),
            "Soma das categorias deve ser estritamente igual ao total de extensões elegíveis"
        );
    }

    #[test]
    fn test_17_real_windows_query_for_mrw_on_host() {
        // Executa a consulta real no Windows da máquina host para a extensão .mrw
        let res = query_windows_association("mrw");
        assert!(
            res.is_ok(),
            "query_windows_association('mrw') deve executar sem erro"
        );
        let info = res
            .unwrap()
            .expect("deve retornar informações de associação para .mrw");

        // No Windows host, o VISOR está registrado em Capabilities e o Windows confirma QueryAppIsDefault = true
        let curr_exe = std::env::current_exe().ok();
        assert!(
            info.matches_visor(curr_exe.as_deref()),
            "No ambiente real do operador, .mrw deve comprovar correspondência ao VISOR sem falso positivo"
        );
    }
}
