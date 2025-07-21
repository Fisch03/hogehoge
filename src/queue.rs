use crate::plugin::PluginSystem;
use hogehoge_types::UniqueTrackIdentifier;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Queue {
    items: Mutex<QueueItems>,
    update_tx: broadcast::Sender<QueueUpdate>,
    pub plugin_system: PluginSystem,
}

#[derive(Debug, Clone, Default)]
pub struct QueueItems {
    pub past: Vec<UniqueTrackIdentifier>,
    pub current: Option<UniqueTrackIdentifier>,
    pub future: VecDeque<UniqueTrackIdentifier>,
}

pub type QueueUpdateRx = broadcast::Receiver<QueueUpdate>;

#[derive(Debug, Clone)]
pub enum QueueUpdate {
    CurrentTrackChanged,
    #[allow(dead_code)]
    TrackAdded(UniqueTrackIdentifier),
}

impl QueueItems {
    pub fn get_at_offset(&self, offset: isize) -> Option<UniqueTrackIdentifier> {
        match offset {
            0 => self.current.clone(),
            ..0 => {
                let index = self.past.len() as isize + offset;
                self.past.get(index as usize).cloned()
            }
            1.. => {
                let index = offset as usize - 1;
                self.future.get(index).cloned()
            }
        }
    }

    pub fn push(&mut self, track: UniqueTrackIdentifier) {
        self.future.push_back(track);
    }

    pub fn forward(&mut self) -> Option<UniqueTrackIdentifier> {
        let next_track = self.future.pop_front();

        let prev_track = std::mem::replace(&mut self.current, next_track);

        if let Some(track) = prev_track {
            self.past.push(track);
        }

        self.current.clone()
    }
}

impl Queue {
    pub fn new(plugins: PluginSystem) -> Arc<Queue> {
        let update_tx = broadcast::Sender::new(16);

        Arc::new(Queue {
            items: Mutex::new(QueueItems::default()),

            update_tx,
            plugin_system: plugins,
        })
    }

    pub fn subscribe_updates(&self) -> QueueUpdateRx {
        self.update_tx.subscribe()
    }

    fn notify_update(&self, update: QueueUpdate) {
        let _ = self.update_tx.send(update);
    }

    pub fn get_next_track(&self) -> Option<UniqueTrackIdentifier> {
        self.items.lock().unwrap().get_at_offset(1)
    }

    pub fn forward(&self) -> Option<UniqueTrackIdentifier> {
        let new = self.items.lock().unwrap().forward();
        self.notify_update(QueueUpdate::CurrentTrackChanged);
        new
    }

    pub fn push(&self, track: UniqueTrackIdentifier) {
        self.items.lock().unwrap().push(track.clone());

        self.notify_update(QueueUpdate::TrackAdded(track));
    }
}
