use std::sync::Arc;

use hogehoge_db::Database;
use hogehoge_types::{ScanResult, Track, UniqueTrackIdentifier};
use rayon::{ThreadPool, prelude::*};
use tokio::sync::mpsc;
use tracing::*;

use crate::plugin::PluginSystem;
use crate::ui::notifications::*;

#[derive(Debug, Clone)]
pub struct Library {
    scan_threads: Arc<ThreadPool>,
    import_queue: mpsc::Sender<(UniqueTrackIdentifier, ScanResult)>,
    db: Database,
}

// since bulk inserting cannot be done in parallel anyways on a sqlite database, use a separate
// worker in that case
#[derive(Debug)]
pub struct LibraryImportWorker {
    import_rx: mpsc::Receiver<(UniqueTrackIdentifier, ScanResult)>,
    db: Database,
}

impl Library {
    pub async fn new(db: Database) -> Self {
        // we cant use the global rayon thread pool because that one is also used by freya for
        // rendering, so hogging it during scans would cause the UI to freeze
        let scan_threads = rayon::ThreadPoolBuilder::new()
            .num_threads(
                std::thread::available_parallelism()
                    .map(|n| usize::max(n.get() - 1, 1))
                    .unwrap_or(1),
            )
            .build()
            .expect("Failed to build library thread pool");

        let (import_queue, import_rx) = mpsc::channel(128);

        let worker = LibraryImportWorker {
            import_rx,
            db: db.clone(),
        };

        tokio::spawn(async move {
            worker.run().await;
        });

        Library {
            db,
            import_queue,
            scan_threads: Arc::new(scan_threads),
        }
    }

    #[instrument(skip_all)]
    pub fn scan(&self, plugin_system: PluginSystem) -> Notification {
        let (notification, notification_handle) = Notification::new_progress("Music Scan");

        info!("Starting music scan...");
        let parent_span = Span::current();

        let import_queue = self.import_queue.clone();
        self.clone().scan_threads.spawn(move || {
            let _span = parent_span.enter();

            let plugin_count = plugin_system.plugins.len();

            let prepared_scans = plugin_system
                .plugins
                .par_iter()
                .filter_map(|(id, pool)| {
                    let _span = info_span!(parent: &parent_span, "prepare_scan").entered();
                    debug!("Preparing scan for plugin '{}'", pool.metadata.name);

                    let mut plugin = pool.get_free_plugin();

                    let result = match plugin.prepare_scan() {
                        Ok(prepared_scan) => {
                            debug!("Prepared scan for plugin '{}'", pool.metadata.name);
                            Some((*id, prepared_scan))
                        }

                        Err(e) => {
                            warn!(
                                "Failed to prepare scan for plugin '{}': {}",
                                pool.metadata.name, e
                            );
                            None
                        }
                    };

                    notification_handle.modify_state(|state| {
                        state.progress += 50.0 / plugin_count as f32;
                    });

                    result
                })
                .collect::<Vec<_>>();

            let tracks_count = prepared_scans
                .iter()
                .fold(0, |acc, (_, scan)| acc + scan.tracks.len());
            info!("Found {} tracks to scan", tracks_count);

            notification_handle.modify_state(|state| {
                state.progress = 50.0;
                state.message = "Scanning tracks...".into();
            });

            // TODO: prefer scanning tracks that are not already in the library
            prepared_scans
                .into_par_iter()
                .for_each(|(id, prepared_scan)| {
                    prepared_scan.tracks.into_par_iter().for_each(|track| {
                        let _span = parent_span.enter();

                        let mut plugin =
                            plugin_system.get_free_plugin(id).expect("Plugin not found");

                        match plugin.scan(&track) {
                            Ok(result) => {
                                let identifier = UniqueTrackIdentifier {
                                    plugin: id,
                                    plugin_data: track,
                                };

                                import_queue
                                    .blocking_send((identifier, result))
                                    .unwrap_or_else(|e| {
                                        error!("Failed to send scan result to import queue: {}", e);
                                    });
                            }
                            Err(e) => {
                                warn!("Failed to scan track '{:?}': {}", track, e);
                            }
                        }

                        notification_handle.modify_state(|state| {
                            state.progress += 50.0 / tracks_count as f32;
                        });
                    });
                });

            plugin_system.cleanup_pool();
            info!("Music scan completed.");

            notification_handle.complete();
        });

        notification
    }
}

impl LibraryImportWorker {
    #[tracing::instrument(skip(self))]
    pub async fn run(mut self) {
        while let Some((identifier, scan)) = self.import_rx.recv().await {
            self.process_scan(scan, identifier).await;
        }
    }

    #[instrument(skip(self))]
    async fn process_scan(&self, scan: ScanResult, identifier: UniqueTrackIdentifier) {
        let track = match Track::from_tags(identifier, scan.tags) {
            Ok(track) => track,
            Err(e) => {
                warn!("Failed to create track from scan result: {}", e);
                return;
            }
        };

        let title = track.title.clone();
        let db = self.db.clone();

        match db.find_or_create_track(track).await {
            Ok(_) => {
                trace!("Track '{:?}' added to the library", title);
            }
            Err(e) => {
                warn!("Failed to add track '{:?}' to the library: {}", title, e);
            }
        }
    }
}
