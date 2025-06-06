use extism::{Manifest, Plugin, PluginBuilder};
use hogehoge_types::PluginMetadata;
use std::path::PathBuf;
use thiserror::Error;

pub struct PluginSystem {
    plugins: Vec<Plugin>,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Specified plugin directory does not exist: {0}")]
    InvalidDirectory(PathBuf),
}

impl PluginSystem {
    pub fn initialize(plugin_dir: PathBuf) -> Result<Self, PluginError> {
        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(&plugin_dir)
            .map_err(|_| PluginError::InvalidDirectory(plugin_dir.clone()))?
        {
            let Ok(entry) = entry else {
                eprintln!("Failed to read entry in plugin directory: {:?}", plugin_dir);
                continue;
            };

            let manifest = Manifest::new([entry.path()]);
            let mut plugin = match PluginBuilder::new(manifest).with_wasi(true).build() {
                Ok(plugin) => plugin,
                Err(e) => {
                    eprintln!("Failed to build plugin {}: {:?}", entry.path().display(), e);
                    continue;
                }
            };

            if !plugin.function_exists("get_metadata") {
                eprintln!(
                    "Plugin {} does not implement required function 'get_metadata'",
                    entry.path().display()
                );
                continue;
            }

            let metadata = match plugin.call::<(), PluginMetadata>("get_metadata", ()) {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!(
                        "Failed to call 'get_metadata' on plugin {}: {:?}",
                        entry.path().display(),
                        e
                    );
                    continue;
                }
            };

            dbg!(metadata);

            plugins.push(plugin);
        }

        Ok(PluginSystem { plugins })
    }
}
