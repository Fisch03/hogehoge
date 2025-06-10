use crate::ui::*;
use futures::stream::{SelectAll, StreamExt};
use std::collections::HashMap;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

pub fn use_task_handler() -> TaskHandler {
    let mut task_states = use_signal(|| HashMap::new());
    let task_handler = use_coroutine(
        move |mut add_rx: UnboundedReceiver<BackgroundTaskHandle>| async move {
            let mut task_id = 0;
            let mut tasks = SelectAll::new();

            loop {
                tokio::select! {
                    Some(task) = add_rx.next() => {
                        let id = task_id;
                        task_id += 1;

                        tasks.push(task.get_stream().map(move |state| (id, state)));
                    }

                    Some((id, state)) = tasks.next() => {
                        let mut task_states = task_states.write();

                        task_states.insert(id, state.clone());

                    }
                    else => break,
                }
            }
        },
    );
    use_context_provider(move || TaskHandler {
        coroutine: task_handler,
        task_states,
    })
}

#[derive(Clone)]
pub struct TaskHandler {
    coroutine: Coroutine<BackgroundTaskHandle>,
    task_states: Signal<HashMap<usize, BackgroundTaskState>>,
}
impl TaskHandler {
    pub fn start(&self, task: BackgroundTaskHandle) {
        self.coroutine.send(task);
    }

    pub fn any_running(&self) -> bool {
        self.task_states.read().values().any(|state| !state.done)
    }

    pub fn get_states(&self) -> Signal<HashMap<usize, BackgroundTaskState>> {
        self.task_states.clone()
    }
}

#[derive(Debug, Clone)]
pub struct BackgroundTaskHandle {
    receiver: watch::Receiver<BackgroundTaskState>,
}

#[derive(Debug)]
pub struct BackgroundTask {
    progress: watch::Sender<BackgroundTaskState>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundTaskState {
    pub name: String,
    pub message: String,
    pub progress: f32,
    pub done: bool,
}

impl BackgroundTaskHandle {
    pub fn new(initial_state: BackgroundTaskState) -> (Self, BackgroundTask) {
        let (progress_tx, progress_rx) = watch::channel(initial_state);

        (
            Self {
                receiver: progress_rx,
            },
            BackgroundTask {
                progress: progress_tx,
            },
        )
    }

    pub fn get_stream(&self) -> WatchStream<BackgroundTaskState> {
        WatchStream::new(self.receiver.clone())
    }
}

impl BackgroundTask {
    pub fn modify_state<F>(&self, f: F)
    where
        F: FnOnce(&mut BackgroundTaskState),
    {
        self.progress.send_modify(f);
    }

    pub fn complete(&self) {
        self.progress.send_modify(|s| {
            s.progress = 1.0;
            s.done = true;
        });
    }
}
