use anyhow::Result;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tracing::{info, Instrument};

mod log_layer;
use log_layer::LogLine;
pub use log_layer::{BackgroundTaskLogSubscriber, BackgroundTaskLogs};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(usize);

pub struct BackgroundTaskManager {
    rt: Runtime,
    running_tasks: HashMap<TaskId, RunningTask>,
    finished_tasks: Vec<FinishedTask>,
    task_logs: BackgroundTaskLogs,
    next_id: TaskId,
}

struct RunningTask {
    id: TaskId,
    name: String,
    display: bool,
    handle: tokio::task::JoinHandle<Result<()>>,
}

struct FinishedTask {
    name: String,
    logs: Vec<LogLine>,
    result: Result<()>,
}

impl BackgroundTaskManager {
    pub fn new(rt: Runtime, task_logs: BackgroundTaskLogs) -> Self {
        Self {
            rt,
            running_tasks: HashMap::new(),
            finished_tasks: Vec::new(),
            task_logs,
            next_id: TaskId(0),
        }
    }

    pub fn spawn<F>(&mut self, name: &str, display: bool, future: F)
    where
        F: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        info!("Starting Background task '{}'", name);
        let id = self.next_id;
        self.next_id.0 += 1;

        let span = tracing::info_span!("background task", task_id = id.0);
        let handle = self.rt.spawn(async { future.instrument(span).await });

        self.running_tasks.insert(
            id,
            RunningTask {
                id,
                name: name.to_string(),
                display,
                handle,
            },
        );
    }

    pub fn update(&mut self) {
        let mut finished = Vec::new();

        for (id, task) in &self.running_tasks {
            if task.handle.is_finished() {
                finished.push(*id);
            }
        }

        for id in finished {
            let task = self.running_tasks.remove(&id).unwrap();

            let result = self.rt.block_on(task.handle);
            let result = match result {
                Ok(r) => r,
                Err(e) => Err(anyhow::anyhow!(
                    "waiting for task to complete failed: {:?}",
                    e
                )),
            };

            match &result {
                Ok(()) => {
                    info!("Background task '{}' finished sucessfully", task.name);
                }
                Err(e) => {
                    info!("Background task '{}' failed with error: {:?}", task.name, e);
                }
            }

            if task.display {
                self.finished_tasks.push(FinishedTask {
                    result,
                    name: task.name,
                    logs: self.task_logs.extract_logs(task.id),
                });
            }
        }
    }
}
