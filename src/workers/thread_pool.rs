//! Infraestrutura de workers e pool de threads dimensionado integrado à fila de prioridades.
//!
//! Executa tarefas concorrentes em background com limite rigoroso de threads,
//! isolamento contra panics em tarefas e encerramento limpo (graceful shutdown) no Drop.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

use super::cancellation::CancellationToken;
use super::task_queue::{Priority, TaskQueue};

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

/// Pool de threads com dimensionamento controlado alimentado pela `TaskQueue`.
pub struct ThreadPool {
    workers: Vec<Worker>,
    queue: Arc<TaskQueue>,
    active_count: Arc<AtomicUsize>,
}

impl ThreadPool {
    /// Calcula a quantidade recomendada de workers a partir de um valor de paralelismo disponível.
    ///
    /// Se o valor for 0 ou 1 (ex: máquina com 1 núcleo lógico), retorna 1 worker.
    /// Em sistemas com múltiplos núcleos, limita inicialmente a 4 workers para evitar
    /// contenção com a thread gráfica do egui e o compositor do Windows.
    pub fn calculate_worker_count(available: usize) -> usize {
        available.clamp(1, 4)
    }

    /// Determina a contagem recomendada de workers com base na topologia do hardware detectada.
    pub fn default_worker_count() -> usize {
        let available = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self::calculate_worker_count(available)
    }

    /// Cria um novo `ThreadPool` com contagem automática de workers.
    pub fn with_default_workers() -> Self {
        Self::new(Self::default_worker_count())
    }

    /// Cria um novo `ThreadPool` com número explícito de workers (mínimo de 1)
    /// utilizando uma nova `TaskQueue` interna.
    pub fn new(num_workers: usize) -> Self {
        Self::with_queue(num_workers, Arc::new(TaskQueue::default()))
    }

    /// Cria um novo `ThreadPool` associado a uma `TaskQueue` compartilhada.
    pub fn with_queue(num_workers: usize, queue: Arc<TaskQueue>) -> Self {
        let count = num_workers.max(1);
        let active_count = Arc::new(AtomicUsize::new(0));

        let mut workers = Vec::with_capacity(count);
        for id in 0..count {
            let q_clone = Arc::clone(&queue);
            let active_clone = Arc::clone(&active_count);
            let thread = thread::Builder::new()
                .name(format!("visor-worker-{id}"))
                .spawn(move || worker_loop(id, q_clone, active_clone))
                .expect("falha ao criar worker thread no pool");

            workers.push(Worker {
                _id: id,
                thread: Some(thread),
            });
        }

        Self {
            workers,
            queue,
            active_count,
        }
    }

    /// Referência para a `TaskQueue` subjacente.
    pub fn queue(&self) -> &Arc<TaskQueue> {
        &self.queue
    }

    /// Quantidade de worker threads alocadas no pool.
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// Quantidade de tarefas atualmente aguardando na fila.
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Quantidade de tarefas atualmente em execução ativa por algum worker.
    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Retorna `true` se não houver tarefas na fila nem tarefas ativas.
    pub fn is_idle(&self) -> bool {
        self.queued_count() == 0 && self.active_count() == 0
    }

    /// Envia uma tarefa genérica com prioridade intermediária (Thumbnail) e sem token.
    /// Retorna `true` se a tarefa foi aceita na fila, ou `false` se foi rejeitada
    /// por o pool já estar em processo de desligamento (shutdown).
    pub fn spawn<F>(&self, f: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        self.spawn_with_priority(Priority::Thumbnail, None, f)
    }

    /// Envia uma tarefa com prioridade explícita e token de cancelamento opcional.
    pub fn spawn_with_priority<F>(
        &self,
        priority: Priority,
        token: Option<CancellationToken>,
        f: F,
    ) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.push(priority, token, f)
    }

    /// Encerra todos os workers de forma coordenada e bloqueia até o término das threads.
    ///
    /// Comportamento de Drenagem:
    /// - Tarefas que já foram aceitas na fila antes do `shutdown()` serão totalmente
    ///   processadas e concluídas pelos workers antes do término do join.
    /// - Quaisquer tarefas submetidas após a chamada de `shutdown()` são imediatamente
    ///   rejeitadas pelo método `spawn()` (retornam `false`) e não entram na fila.
    pub fn shutdown(&mut self) {
        self.queue.shutdown();

        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn worker_loop(_id: usize, queue: Arc<TaskQueue>, active_count: Arc<AtomicUsize>) {
    while let Some(task) = queue.pop() {
        active_count.fetch_add(1, Ordering::Relaxed);
        let _ = catch_unwind(AssertUnwindSafe(|| {
            task.execute();
        }));
        active_count.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::time::Duration;

    #[test]
    fn test_simulated_parallelism_scaling() {
        assert_eq!(
            ThreadPool::calculate_worker_count(0),
            1,
            "0 deve resultar no piso de 1"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(1),
            1,
            "1 deve resultar em 1 worker"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(2),
            2,
            "2 deve resultar em 2 workers"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(4),
            4,
            "4 deve resultar em 4 workers"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(8),
            4,
            "8 deve ser limitado a 4 inicialmente"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(12),
            4,
            "12 (ex: Ryzen 5 5500) deve ser limitado a 4 inicialmente"
        );
        assert_eq!(
            ThreadPool::calculate_worker_count(32),
            4,
            "32 deve ser limitado a 4"
        );
    }

    #[test]
    fn test_default_worker_count_bounds() {
        let count = ThreadPool::default_worker_count();
        assert!(count >= 1, "deve ter pelo menos 1 worker");
        assert!(
            count <= 4,
            "deve ser limitado a no máximo 4 workers inicialmente"
        );
    }

    #[test]
    fn test_spawn_rejected_after_shutdown() {
        let mut pool = ThreadPool::new(2);
        pool.shutdown();

        let executed = Arc::new(AtomicBool::new(false));
        let e = Arc::clone(&executed);
        let accepted = pool.spawn(move || {
            e.store(true, Ordering::SeqCst);
        });

        assert!(
            !accepted,
            "tarefas submetidas após o shutdown devem ser rejeitadas"
        );
        thread::sleep(Duration::from_millis(20));
        assert!(
            !executed.load(Ordering::SeqCst),
            "tarefa rejeitada não deve ser executada"
        );
    }

    #[test]
    fn test_pool_executes_tasks() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..10 {
            let c = Arc::clone(&counter);
            let accepted = pool.spawn(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
            assert!(accepted);
        }

        let mut tries = 0;
        while counter.load(Ordering::SeqCst) < 10 && tries < 200 {
            thread::sleep(Duration::from_millis(10));
            tries += 1;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_pool_single_worker_no_deadlock() {
        let pool = ThreadPool::new(1);
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..5 {
            let c = Arc::clone(&counter);
            pool.spawn(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        let mut tries = 0;
        while counter.load(Ordering::SeqCst) < 5 && tries < 200 {
            thread::sleep(Duration::from_millis(10));
            tries += 1;
        }

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_pool_clean_shutdown_on_drop() {
        let counter = Arc::new(AtomicU32::new(0));
        {
            let pool = ThreadPool::new(2);
            for _ in 0..5 {
                let c = Arc::clone(&counter);
                pool.spawn(move || {
                    thread::sleep(Duration::from_millis(10));
                    c.fetch_add(1, Ordering::SeqCst);
                });
            }
        } // pool sai de escopo aqui e executa Drop -> shutdown com join

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_worker_survives_panic() {
        let pool = ThreadPool::new(1);
        let success = Arc::new(AtomicBool::new(false));

        // Tarefa que entra em pânico
        pool.spawn(|| {
            panic!("pânico intencional de teste");
        });

        // Tarefa subsequente no mesmo worker
        let s = Arc::clone(&success);
        pool.spawn(move || {
            s.store(true, Ordering::SeqCst);
        });

        let mut tries = 0;
        while !success.load(Ordering::SeqCst) && tries < 200 {
            thread::sleep(Duration::from_millis(10));
            tries += 1;
        }

        assert!(
            success.load(Ordering::SeqCst),
            "worker deve sobreviver ao pânico e processar a próxima tarefa"
        );
    }
}
