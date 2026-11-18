//! Tokens atômicos de cancelamento cooperativo.
//!
//! Permite que operações em background verifiquem de forma leve e segura se foram
//! canceladas por uma ação mais recente do usuário (ex: trocar de imagem).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Token compartilhável e seguro entre threads para sinalização de cancelamento.
#[derive(Clone, Debug)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// Cria um novo token no estado ativo (não cancelado).
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cria um token já no estado cancelado.
    pub fn cancelled() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Sinaliza o cancelamento de forma atômica e visível a todas as threads.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Verifica de forma atômica se o cancelamento foi solicitado.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Restaura o token para o estado ativo (útil em pools de tokens reutilizáveis).
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cancellation_initial_state() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_shared_across_threads() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = thread::spawn(move || {
            let mut checks = 0;
            while !token_clone.is_cancelled() && checks < 200 {
                thread::sleep(Duration::from_millis(5));
                checks += 1;
            }
            token_clone.is_cancelled()
        });

        thread::sleep(Duration::from_millis(20));
        token.cancel();

        let saw_cancellation = handle.join().expect("thread join");
        assert!(
            saw_cancellation,
            "thread deve observar o cancelamento emitido pela outra thread"
        );
    }

    #[test]
    fn test_cancellation_during_execution() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let stopped_early = Arc::new(AtomicBool::new(false));
        let stopped_early_clone = Arc::clone(&stopped_early);

        let handle = thread::spawn(move || {
            for i in 0..100 {
                if token_clone.is_cancelled() {
                    stopped_early_clone.store(true, Ordering::SeqCst);
                    return i;
                }
                thread::sleep(Duration::from_millis(5));
            }
            100
        });

        thread::sleep(Duration::from_millis(25));
        token.cancel();

        let iterations = handle.join().expect("thread join");
        assert!(
            iterations < 100,
            "deve interromper o loop cooperativamente antes de 100 iterações"
        );
        assert!(stopped_early.load(Ordering::SeqCst));
    }
}
