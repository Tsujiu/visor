//! Fila de tarefas com prioridades, preservação de ordem FIFO, prevenção
//! de starvation e descarte automático de jobs cancelados antes da execução.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use super::cancellation::CancellationToken;

/// Níveis de prioridade para tarefas agendadas no pool de workers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Menor prioridade: pré-carregamento de imagens adjacentes na pasta.
    Prefetch = 0,
    /// Prioridade intermediária: geração de miniaturas para o carrossel visível.
    Thumbnail = 1,
    /// Maior prioridade: decodificação da imagem selecionada para exibição no viewport.
    Viewport = 2,
}

/// Item de trabalho armazenado na fila de prioridades.
pub struct TaskItem {
    pub priority: Priority,
    pub sequence: u64,
    pub token: Option<CancellationToken>,
    work: Box<dyn FnOnce() + Send + 'static>,
}

impl TaskItem {
    /// Executa o trabalho associado se o token não tiver sido cancelado.
    /// Retorna `true` se o trabalho foi executado, ou `false` se foi descartado por cancelamento.
    pub fn execute(self) -> bool {
        if let Some(token) = &self.token {
            if token.is_cancelled() {
                return false;
            }
        }
        (self.work)();
        true
    }
}

/// Limite de tarefas consecutivas de alta prioridade antes de forçar o atendimento
/// de uma tarefa de menor prioridade (prevenção de starvation).
pub const DEFAULT_STARVATION_LIMIT: usize = 5;

struct QueueInner {
    viewport: VecDeque<TaskItem>,
    thumbnail: VecDeque<TaskItem>,
    prefetch: VecDeque<TaskItem>,
    consecutive_high_served: usize,
    shutdown: bool,
}

impl QueueInner {
    fn new() -> Self {
        Self {
            viewport: VecDeque::new(),
            thumbnail: VecDeque::new(),
            prefetch: VecDeque::new(),
            consecutive_high_served: 0,
            shutdown: false,
        }
    }

    fn total_len(&self) -> usize {
        self.viewport.len() + self.thumbnail.len() + self.prefetch.len()
    }

    fn is_empty(&self) -> bool {
        self.total_len() == 0
    }
}

/// Fila sincronizada multithread com suporte a 3 níveis de prioridade e cancelamento.
pub struct TaskQueue {
    inner: Mutex<QueueInner>,
    condvar: Condvar,
    sequence: AtomicU64,
    dropped_cancelled: AtomicUsize,
    starvation_limit: usize,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new(DEFAULT_STARVATION_LIMIT)
    }
}

impl TaskQueue {
    /// Cria uma nova fila de tarefas com limite de starvation configurável.
    pub fn new(starvation_limit: usize) -> Self {
        Self {
            inner: Mutex::new(QueueInner::new()),
            condvar: Condvar::new(),
            sequence: AtomicU64::new(0),
            dropped_cancelled: AtomicUsize::new(0),
            starvation_limit: starvation_limit.max(1),
        }
    }

    /// Enfileira uma nova tarefa com prioridade e token de cancelamento opcional.
    /// Retorna `false` se a fila já estiver encerrada (shutdown).
    pub fn push<F>(&self, priority: Priority, token: Option<CancellationToken>, work: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        // Se a tarefa já chega cancelada antes de entrar na fila, descartamos de imediato
        if let Some(t) = &token {
            if t.is_cancelled() {
                self.dropped_cancelled.fetch_add(1, Ordering::Relaxed);
                return false;
            }
        }

        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        let item = TaskItem {
            priority,
            sequence: seq,
            token,
            work: Box::new(work),
        };

        {
            let mut inner = self.inner.lock().unwrap();
            if inner.shutdown {
                return false;
            }

            match priority {
                Priority::Viewport => inner.viewport.push_back(item),
                Priority::Thumbnail => inner.thumbnail.push_back(item),
                Priority::Prefetch => inner.prefetch.push_back(item),
            }
        }

        self.condvar.notify_one();
        true
    }

    /// Retira a próxima tarefa a ser executada, respeitando prioridades, FIFO e política de starvation.
    /// Bloqueia a thread atual enquanto a fila estiver vazia.
    /// Retorna `None` se a fila for encerrada via `shutdown()`.
    pub fn pop(&self) -> Option<TaskItem> {
        let mut inner = self.inner.lock().unwrap();

        loop {
            // Tenta obter o próximo item elegível
            while let Some(candidate) = self.select_next_item(&mut inner) {
                // Se o item foi cancelado enquanto aguardava na fila, descartamos sem executar
                if let Some(token) = &candidate.token {
                    if token.is_cancelled() {
                        self.dropped_cancelled.fetch_add(1, Ordering::Relaxed);
                        continue; // Busca o próximo item válido
                    }
                }
                return Some(candidate);
            }

            if inner.shutdown && inner.is_empty() {
                return None;
            }

            // Aguarda sinalização de novo item ou de shutdown
            inner = self.condvar.wait(inner).unwrap();
        }
    }

    /// Seleciona o próximo item com base na política de prioridade e mitigação de starvation.
    fn select_next_item(&self, inner: &mut QueueInner) -> Option<TaskItem> {
        let has_viewport = !inner.viewport.is_empty();
        let has_thumb = !inner.thumbnail.is_empty();
        let has_prefetch = !inner.prefetch.is_empty();

        if !has_viewport && !has_thumb && !has_prefetch {
            return None;
        }

        // Prevenção de Starvation: se servimos muitas tarefas consecutivas de alta prioridade
        // e existem tarefas de menor prioridade aguardando, atendemos uma tarefa menor
        if inner.consecutive_high_served >= self.starvation_limit {
            if has_prefetch {
                inner.consecutive_high_served = 0;
                return inner.prefetch.pop_front();
            } else if has_thumb {
                inner.consecutive_high_served = 0;
                return inner.thumbnail.pop_front();
            }
        }

        // Ordem natural estrita: Viewport > Thumbnail > Prefetch
        if let Some(item) = inner.viewport.pop_front() {
            inner.consecutive_high_served += 1;
            Some(item)
        } else if let Some(item) = inner.thumbnail.pop_front() {
            inner.consecutive_high_served += 1;
            Some(item)
        } else {
            inner.consecutive_high_served = 0;
            inner.prefetch.pop_front()
        }
    }

    /// Encerra a fila de tarefas e acorda todos os workers que estão aguardando.
    pub fn shutdown(&self) {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.shutdown = true;
        }
        self.condvar.notify_all();
    }

    /// Retorna a quantidade total de tarefas pendentes em todas as prioridades.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().total_len()
    }

    /// Retorna `true` se a fila estiver totalmente vazia.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }

    /// Contagem de tarefas descartadas antes do início da execução por cancelamento do token.
    pub fn dropped_cancelled_count(&self) -> usize {
        self.dropped_cancelled.load(Ordering::Relaxed)
    }

    /// Retorna as contagens individuais por prioridade: `(viewport, thumbnail, prefetch)`.
    pub fn counts_by_priority(&self) -> (usize, usize, usize) {
        let inner = self.inner.lock().unwrap();
        (
            inner.viewport.len(),
            inner.thumbnail.len(),
            inner.prefetch.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_priority_ordering() {
        let queue = TaskQueue::new(100); // Sem starvation para testar ordem pura
        let output = Arc::new(Mutex::new(Vec::new()));

        // Enfileira na ordem inversa da prioridade
        let out = Arc::clone(&output);
        queue.push(Priority::Prefetch, None, move || {
            out.lock().unwrap().push("prefetch");
        });

        let out = Arc::clone(&output);
        queue.push(Priority::Thumbnail, None, move || {
            out.lock().unwrap().push("thumb");
        });

        let out = Arc::clone(&output);
        queue.push(Priority::Viewport, None, move || {
            out.lock().unwrap().push("viewport");
        });

        // Executa os 3 itens
        for _ in 0..3 {
            let item = queue.pop().expect("item na fila");
            item.execute();
        }

        let executed = output.lock().unwrap().clone();
        assert_eq!(executed, vec!["viewport", "thumb", "prefetch"]);
    }

    #[test]
    fn test_fifo_within_same_priority() {
        let queue = TaskQueue::new(10);
        let output = Arc::new(Mutex::new(Vec::new()));

        for i in 1..=5 {
            let out = Arc::clone(&output);
            queue.push(Priority::Thumbnail, None, move || {
                out.lock().unwrap().push(i);
            });
        }

        for _ in 0..5 {
            let item = queue.pop().expect("item");
            item.execute();
        }

        let executed = output.lock().unwrap().clone();
        assert_eq!(
            executed,
            vec![1, 2, 3, 4, 5],
            "deve manter ordem FIFO estrita dentro da mesma prioridade"
        );
    }

    #[test]
    fn test_skip_cancelled_before_execution() {
        let queue = TaskQueue::new(10);
        let executed = Arc::new(AtomicU32::new(0));

        let token_cancelled = CancellationToken::cancelled();
        let e = Arc::clone(&executed);
        // Tarefa com token cancelado antes de entrar
        let accepted = queue.push(Priority::Viewport, Some(token_cancelled), move || {
            e.fetch_add(1, Ordering::SeqCst);
        });
        assert!(!accepted, "tarefa já cancelada deve ser recusada no push");

        // Tarefa que é cancelada enquanto está na fila
        let token_later = CancellationToken::new();
        let e2 = Arc::clone(&executed);
        queue.push(Priority::Viewport, Some(token_later.clone()), move || {
            e2.fetch_add(1, Ordering::SeqCst);
        });
        token_later.cancel(); // Cancela antes do worker fazer pop

        // Tarefa válida subsequente
        let e3 = Arc::clone(&executed);
        queue.push(Priority::Viewport, None, move || {
            e3.fetch_add(10, Ordering::SeqCst);
        });

        // O pop deve pular a tarefa cancelada e pegar a válida
        let item = queue.pop().expect("deve pegar a tarefa válida");
        item.execute();

        assert_eq!(executed.load(Ordering::SeqCst), 10);
        assert!(queue.dropped_cancelled_count() >= 1);
    }

    #[test]
    fn test_starvation_prevention() {
        // Starvation limit = 3: após 3 tarefas de Viewport, se houver Prefetch, serve 1 Prefetch
        let queue = TaskQueue::new(3);
        let output = Arc::new(Mutex::new(Vec::new()));

        // Insere 1 Prefetch
        let out = Arc::clone(&output);
        queue.push(Priority::Prefetch, None, move || {
            out.lock().unwrap().push("low");
        });

        // Insere 5 Viewport consecutivas
        for i in 1..=5 {
            let out = Arc::clone(&output);
            queue.push(Priority::Viewport, None, move || {
                out.lock().unwrap().push(format!("high_{i}").leak());
            });
        }

        // Descarrega todos
        for _ in 0..6 {
            let item = queue.pop().expect("item");
            item.execute();
        }

        let executed = output.lock().unwrap().clone();
        // Esperado: 3 high -> 1 low (pelo aging/starvation quota) -> 2 high restantes
        assert_eq!(
            executed,
            vec!["high_1", "high_2", "high_3", "low", "high_4", "high_5"],
            "tarefa de baixa prioridade deve ser atendida após a cota de starvation"
        );
    }

    #[test]
    fn test_queue_waiting_and_wakeup() {
        let queue = Arc::new(TaskQueue::new(10));
        let q_clone = Arc::clone(&queue);
        let completed = Arc::new(AtomicBool::new(false));
        let c_clone = Arc::clone(&completed);

        let handle = thread::spawn(move || {
            // Fila está inicialmente vazia, worker vai bloquear no pop()
            let item = q_clone.pop().expect("deve receber tarefa após acordar");
            item.execute();
            c_clone.store(true, Ordering::SeqCst);
        });

        thread::sleep(Duration::from_millis(30));
        assert!(
            !completed.load(Ordering::SeqCst),
            "worker deve estar bloqueado aguardando tarefa"
        );

        queue.push(Priority::Viewport, None, || {});

        handle.join().expect("join");
        assert!(
            completed.load(Ordering::SeqCst),
            "worker deve ter acordado e executado"
        );
    }

    #[test]
    fn test_queue_shutdown_no_deadlock() {
        let queue = Arc::new(TaskQueue::new(10));
        let q_clone = Arc::clone(&queue);

        let handle = thread::spawn(move || {
            // Bloqueia em fila vazia
            q_clone.pop()
        });

        thread::sleep(Duration::from_millis(20));
        queue.shutdown();

        let result = handle.join().expect("join");
        assert!(
            result.is_none(),
            "pop() em fila encerrada vazia deve retornar None"
        );
    }
}
