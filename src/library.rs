use std::sync::Arc;

use hogehoge_types::ScanResult;
use rayon::{ThreadPool, prelude::*};
use tracing::*;

use crate::plugin::PluginSystem;
use crate::ui::background_task::*;

#[derive(Debug, Clone)]
pub struct Library {
    scan_threads: Arc<ThreadPool>,
}

impl Library {
    pub fn new() -> Self {
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

        Library {
            scan_threads: Arc::new(scan_threads),
        }
    }

    #[instrument(skip_all)]
    pub fn scan(self, plugin_system: PluginSystem) -> BackgroundTaskHandle {
        let (task, task_handle) = BackgroundTaskHandle::new(BackgroundTaskState {
            name: "Music Scan".to_string(),
            message: "Preparing scan...".to_string(),
            ..Default::default()
        });

        info!("Starting music scan...");
        let parent_span = Span::current();

        self.clone().scan_threads.spawn(move || {
            let _span = parent_span.enter();

            let plugin_count = plugin_system.plugins.len();

            let prepared_scans = plugin_system
                .plugins
                .par_iter()
                .filter_map(|(uuid, pool)| {
                    let _span = info_span!(parent: &parent_span, "prepare_scan").entered();
                    debug!("Preparing scan for plugin UUID: {}", uuid);

                    let mut plugin = pool.get_free_plugin();

                    let result = match plugin.prepare_scan() {
                        Ok(prepared_scan) => {
                            debug!("Prepared scan for plugin UUID: {}", uuid);
                            Some((uuid.clone(), prepared_scan))
                        }

                        Err(e) => {
                            warn!("Failed to prepare scan for plugin UUID {}: {}", uuid, e);
                            None
                        }
                    };

                    task_handle.modify_state(|state| {
                        state.progress += 50.0 / plugin_count as f32;
                    });

                    result
                })
                .collect::<Vec<_>>();

            let tracks_count = prepared_scans
                .iter()
                .fold(0, |acc, (_, scan)| acc + scan.tracks.len());
            info!("Found {} tracks to scan", tracks_count);

            task_handle.modify_state(|state| {
                state.progress = 50.0;
                state.message = "Scanning tracks...".to_string();
            });

            // TODO: prefer scanning tracks that are not already in the library
            prepared_scans
                .into_par_iter()
                .for_each(|(uuid, prepared_scan)| {
                    prepared_scan.tracks.into_par_iter().for_each(|track| {
                        let _span = parent_span.enter();

                        let mut plugin = plugin_system
                            .get_free_plugin(uuid)
                            .expect("Plugin not found");

                        match plugin.scan(&track).map(|scan| self.process_scan(scan)) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("Failed to scan track '{:?}': {}", track, e);
                            }
                        }

                        task_handle.modify_state(|state| {
                            state.progress += 50.0 / tracks_count as f32;
                        });
                    });
                });

            plugin_system.cleanup_pool();
            info!("Music scan completed.");

            task_handle.complete();
        });

        task
    }

    #[instrument]
    fn process_scan(&self, scan: ScanResult) {
        // info!("got scan result!");
    }
}
