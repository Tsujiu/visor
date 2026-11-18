//! Executor STA (Single-Threaded Apartment) dedicado para operações Shell e COM do Windows.
//!
//! O `IFileOperation` e diversas outras interfaces do Shell do Windows exigem um
//! apartamento de thread única (STA) com bombeamento de mensagens Win32 ativo para
//! exibição correta de caixas de diálogo, sincronização de tarefas e liberação segura de recursos.
//!
//! Este módulo cria uma única thread de sistema operacional de longa duração com:
//! - Inicialização COM via `CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)`.
//! - RAII garantido com `CoUninitialize()` executado exatamente uma vez na finalização.
//! - Bombeamento contínuo de mensagens (`PeekMessageW`, `TranslateMessage`, `DispatchMessageW`).
//! - Despacho não-bloqueante de tarefas através de um canal MPSC thread-safe.

#![allow(dead_code)]

use std::ptr;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const COINIT_APARTMENTTHREADED: u32 = 0x2;
pub const COINIT_DISABLE_OLE1DDE: u32 = 0x4;
pub const S_OK: i32 = 0;
pub const S_FALSE: i32 = 1;
pub const RPC_E_CHANGED_MODE: i32 = -2147417850; // 0x80010106

pub const PM_REMOVE: u32 = 0x0001;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: *mut std::ffi::c_void,
    pub message: u32,
    pub w_param: usize,
    pub l_param: isize,
    pub time: u32,
    pub pt: POINT,
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoInitializeEx(pv_reserved: *mut std::ffi::c_void, dw_co_init: u32) -> i32;
    pub fn CoUninitialize();
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn PeekMessageW(
        lp_msg: *mut MSG,
        h_wnd: *mut std::ffi::c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> i32;
    pub fn TranslateMessage(lp_msg: *const MSG) -> i32;
    pub fn DispatchMessageW(lp_msg: *const MSG) -> isize;
}

type StaBoxFn = Box<dyn FnOnce() + Send + 'static>;

enum StaCommand {
    Execute(StaBoxFn),
    Shutdown,
}

/// Guard RAII para garantir que `CoUninitialize` seja chamado se COM foi inicializado.
struct ComScope {
    initialized: bool,
}

impl Drop for ComScope {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// Bombeia mensagens pendentes da fila do thread Win32 (necessário para componentes STA).
pub fn pump_sta_messages() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Executor gerenciado de tarefas executadas exclusivamente em uma thread STA.
pub struct ShellStaExecutor {
    tx: Sender<StaCommand>,
    handle: Option<JoinHandle<()>>,
}

impl ShellStaExecutor {
    /// Inicializa um novo executor STA com thread de longa duração.
    pub fn new() -> Result<Self, String> {
        let (tx, rx): (Sender<StaCommand>, Receiver<StaCommand>) = mpsc::channel();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), String>>();

        let handle = thread::Builder::new()
            .name("visor-shell-sta".to_string())
            .spawn(move || {
                let hr = unsafe {
                    CoInitializeEx(
                        ptr::null_mut(),
                        COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                    )
                };

                let co_initialized = if hr == S_OK || hr == S_FALSE {
                    let _ = init_tx.send(Ok(()));
                    true
                } else {
                    let err_msg = if hr == RPC_E_CHANGED_MODE {
                        format!(
                            "Falha ao inicializar COM STA: HRESULT 0x{hr:08X} (RPC_E_CHANGED_MODE: thread já inicializada em outro modo de apartamento)"
                        )
                    } else {
                        format!("Falha ao inicializar COM STA: HRESULT 0x{hr:08X}")
                    };
                    let _ = init_tx.send(Err(err_msg));
                    return;
                };

                let _guard = ComScope {
                    initialized: co_initialized,
                };

                loop {
                    pump_sta_messages();

                    match rx.recv_timeout(Duration::from_millis(50)) {
                        Ok(StaCommand::Execute(task)) => {
                            task();
                            pump_sta_messages();
                        }
                        Ok(StaCommand::Shutdown) => {
                            break;
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            // Loop e bombeamento de mensagens
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
            })
            .map_err(|e| format!("Não foi possível iniciar thread STA: {e}"))?;

        match init_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                tx,
                handle: Some(handle),
            }),
            Ok(Err(err)) => {
                let _ = handle.join();
                Err(err)
            }
            Err(e) => {
                let _ = handle.join();
                Err(format!("Falha na comunicação de inicialização STA: {e}"))
            }
        }
    }

    /// Enfileira uma tarefa para ser executada no thread STA dedicado.
    pub fn spawn<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx
            .send(StaCommand::Execute(Box::new(f)))
            .map_err(|e| format!("Falha ao enviar tarefa para thread STA: {e}"))
    }
}

impl Drop for ShellStaExecutor {
    fn drop(&mut self) {
        let _ = self.tx.send(StaCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_shell_sta_lifecycle() {
        let executor = ShellStaExecutor::new().expect("STA executor deve ser criado com sucesso");
        let counter = Arc::new(AtomicUsize::new(0));

        let c1 = Arc::clone(&counter);
        executor
            .spawn(move || {
                c1.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();

        let c2 = Arc::clone(&counter);
        executor
            .spawn(move || {
                c2.fetch_add(10, Ordering::SeqCst);
            })
            .unwrap();

        // Aguarda execução
        let mut waited = 0;
        while counter.load(Ordering::SeqCst) != 11 && waited < 40 {
            thread::sleep(Duration::from_millis(25));
            waited += 1;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 11);
        drop(executor); // Shutdown limpo
    }

    #[test]
    fn test_shell_sta_com_initialized() {
        let executor = ShellStaExecutor::new().unwrap();
        let (tx, rx) = mpsc::channel();

        executor
            .spawn(move || {
                // Tentar CoInitializeEx novamente no mesmo thread deve retornar S_FALSE
                let hr = unsafe {
                    CoInitializeEx(
                        ptr::null_mut(),
                        COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                    )
                };
                if hr == S_FALSE {
                    unsafe { CoUninitialize() }; // Equilibrar a chamada adicional
                }
                let _ = tx.send(hr);
            })
            .unwrap();

        let hr = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(hr, S_FALSE);
    }

    #[test]
    fn test_shell_sta_empty_queue_clean_shutdown() {
        let executor = ShellStaExecutor::new().expect("STA executor deve ser criado com sucesso");
        drop(executor); // Shutdown imediato com fila vazia
    }

    #[test]
    fn test_shell_sta_rapid_sequential_tasks() {
        let executor = ShellStaExecutor::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..50 {
            let c = Arc::clone(&counter);
            executor
                .spawn(move || {
                    c.fetch_add(1, Ordering::SeqCst);
                })
                .unwrap();
        }

        let mut waited = 0;
        while counter.load(Ordering::SeqCst) < 50 && waited < 50 {
            thread::sleep(Duration::from_millis(20));
            waited += 1;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 50);
    }

    #[test]
    fn test_shell_sta_rejects_rpc_e_changed_mode() {
        let (tx, rx) = mpsc::channel();

        // Cria uma thread deliberadamente inicializada em MTA (COINIT_MULTITHREADED = 0)
        let handle = thread::spawn(move || {
            let hr_mta = unsafe { CoInitializeEx(ptr::null_mut(), 0) };
            assert_eq!(hr_mta, S_OK);

            // Agora tenta CoInitializeEx com STA: o Windows deve retornar RPC_E_CHANGED_MODE
            let hr_sta = unsafe {
                CoInitializeEx(
                    ptr::null_mut(),
                    COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                )
            };

            // Nosso tratamento: se hr == RPC_E_CHANGED_MODE, deve rejeitar STA e não executar IFileOperation
            let is_rejected = hr_sta == RPC_E_CHANGED_MODE;

            // Limpa o MTA que foi inicializado
            unsafe { CoUninitialize() };

            tx.send(is_rejected).unwrap();
        });

        handle.join().unwrap();
        let is_rejected = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(
            is_rejected,
            "RPC_E_CHANGED_MODE deve ser detectado e rejeitado para proteger o thread"
        );
    }
}
