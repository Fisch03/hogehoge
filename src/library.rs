use rayon::prelude::*;
use tracing::*;

use crate::plugin::PluginSystem;

#[derive(Debug, Clone)]
pub struct Library {}

impl Library {
    pub fn new() -> Self {
        Library {}
    }

    #[instrument(skip_all)]
    pub fn scan(&self, plugin_system: &PluginSystem) {
        let prepared_scans = plugin_system
            .plugins
            .par_iter()
            .filter_map(|(uuid, pool)| {
                let Ok(mut plugin) = pool.get_free_plugin() else {
                    warn!("Failed to get free plugin for UUID: {}", uuid);
                    return None;
                };

                match plugin.prepare_scan() {
                    Ok(prepared_scan) => {
                        debug!("Prepared scan for plugin UUID: {}", uuid);
                        Some((uuid.clone(), prepared_scan))
                    }
                    Err(e) => {
                        warn!("Failed to prepare scan for plugin UUID {}: {}", uuid, e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        info!("Prepared scans: {:#?}", prepared_scans);
    }
}
