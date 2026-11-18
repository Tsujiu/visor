//! Módulo de trabalhadores concorrentes, fila de prioridades e cancelamento.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod cancellation;
pub mod task_queue;
pub mod thread_pool;

pub use cancellation::CancellationToken;
pub use task_queue::{Priority, TaskItem, TaskQueue};
pub use thread_pool::ThreadPool;
