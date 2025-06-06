use super::TaskId;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{
    field::{Field, Visit},
    span, Event, Level, Metadata, Subscriber,
};
use tracing_subscriber::{
    layer::{Context, Layer},
    registry::LookupSpan,
};

pub type LogLine = (Level, String);

#[derive(Debug, Clone)]
pub struct BackgroundTaskLogs(Arc<Mutex<HashMap<TaskId, Vec<LogLine>>>>);

impl BackgroundTaskLogs {
    pub fn extract_logs(&self, id: TaskId) -> Vec<LogLine> {
        self.0.lock().unwrap().remove(&id).unwrap_or_default()
    }
}

pub struct BackgroundTaskLogSubscriber {
    logs: BackgroundTaskLogs,
}

impl BackgroundTaskLogSubscriber {
    pub fn new() -> Self {
        Self {
            logs: BackgroundTaskLogs(Arc::new(Mutex::new(HashMap::new()))),
        }
    }

    pub fn get_logs(&self) -> BackgroundTaskLogs {
        self.logs.clone()
    }
}

impl<S> Layer<S> for BackgroundTaskLogSubscriber
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut visitor = GetIdVisitor::new();
        attrs.record(&mut visitor);

        if let Some(task_id) = visitor.consume() {
            ctx.span(id).unwrap().extensions_mut().insert(task_id);
        }
    }

    fn on_event(&self, event: &Event, ctx: Context<S>) {
        if event.metadata().level() > &Level::INFO {
            return;
        }

        let task_id = ctx
            .lookup_current()
            .map(|span| {
                let extensions = span.extensions();
                extensions.get::<TaskId>().copied()
            })
            .flatten();

        if let Some(task_id) = task_id {
            let mut message_visitor = GetMsgVisitor::new();
            event.record(&mut message_visitor);

            if let Some(message) = message_visitor.consume() {
                self.logs
                    .0
                    .lock()
                    .unwrap()
                    .entry(task_id)
                    .or_default()
                    .push((*event.metadata().level(), message));
            }
        }
    }
}

#[derive(Debug)]
struct GetIdVisitor {
    id: Option<TaskId>,
}

impl GetIdVisitor {
    fn new() -> Self {
        Self { id: None }
    }

    fn consume(self) -> Option<TaskId> {
        self.id
    }
}

impl Visit for GetIdVisitor {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_u64(field, value as u64);
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "task_id" {
            self.id = Some(TaskId(value as usize));
        }
    }

    fn record_bool(&mut self, _field: &Field, _value: bool) {}
    fn record_str(&mut self, _field: &Field, _value: &str) {}
    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

#[derive(Debug)]
struct GetMsgVisitor {
    msg: Option<String>,
}

impl GetMsgVisitor {
    fn new() -> Self {
        Self { msg: None }
    }
    fn consume(self) -> Option<String> {
        self.msg
    }
}

impl Visit for GetMsgVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.msg = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.msg = Some(format!("{:?}", value));
        }
    }

    fn record_i64(&mut self, _field: &Field, _value: i64) {}
    fn record_u64(&mut self, _field: &Field, _value: u64) {}
    fn record_bool(&mut self, _field: &Field, _value: bool) {}
}
