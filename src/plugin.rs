use extism::{Manifest, Plugin as LoadedPlugin, PluginBuilder};
use hogehoge_db::Database;
use hogehoge_types::{PluginId, PluginMetadata, PluginTrackIdentifier, PreparedScan, ScanResult};
use std::{
    collections::{HashMap, VecDeque},
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};
use thiserror::Error;
use tracing::*;

#[derive(Debug, Clone)]
pub struct PluginSystem {
    pub plugins: Arc<HashMap<PluginId, PluginPool>>,
}

#[derive(Debug, Error)]
pub enum PluginSystemError {
    #[error("Specified plugin directory does not exist: {0}")]
    InvalidDirectory(PathBuf),
}

#[derive(Debug)]
pub struct PluginPool {
    pub metadata: PluginMetadata,
    pub capabilities: PluginCapabilities,

    plugin_path: PathBuf,
    plugins: Mutex<VecDeque<Plugin>>,
    wait_condvar: Condvar,
}

#[derive(Debug)]
pub struct Plugin(LoadedPlugin);

pub struct PluginHandle<'a> {
    pool: &'a PluginPool,
    plugin: Option<Plugin>,
}

#[derive(Debug, Clone)]
pub struct PluginCapabilities {
    pub scan_tracks: bool,
    pub playback: bool,
}

impl PluginCapabilities {
    pub fn from_plugin(plugin: &Plugin) -> Self {
        PluginCapabilities {
            scan_tracks: plugin.has_fn("prepare_scan") && plugin.has_fn("scan"),
            playback: plugin.has_fn("play") || plugin.has_fn("playback"),
        }
    }
}


#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to initialize plugin: {0}")]
    InitializationError(extism::Error),

    #[error("Plugin does not implement required function '{0}'")]
    MissingRequiredFunction(&'static str),
    #[error("Failed to call function '{0}': {1}")]
    FunctionCallError(String, extism::Error),
}

impl Plugin {
    pub fn get_metadata(&mut self) -> Result<PluginMetadata, PluginError> {
        self.call("get_metadata", ())
    }

    pub fn has_fn(&self, function: &str) -> bool {
        self.0.function_exists(function)
    }

    pub fn prepare_scan(&mut self) -> Result<PreparedScan, PluginError> {
        self.call("prepare_scan", ())
    }

    pub fn scan(&mut self, ident: &PluginTrackIdentifier) -> Result<ScanResult, PluginError> {
        self.call("scan", ident)
    }

    #[instrument]
    fn try_load(path: &Path) -> Result<Self, PluginError> {
        let manifest = Manifest::new([path.to_path_buf()])
            .with_allowed_path("/home/sakanaa/nas/Audio/Music/".to_string(), "music");

        let plugin = PluginBuilder::new(manifest)
            .with_wasi(true)
            .build()
            .map_err(PluginError::InitializationError)?;

        if !plugin.function_exists("get_metadata") {
            return Err(PluginError::MissingRequiredFunction("get_metadata"));
        }

        let mut plugin = Plugin(plugin);

        let metadata = plugin.get_metadata()?;
        plugin.0.id = metadata.uuid;

        Ok(plugin)
    }

    fn call<'a, 'b, T, R>(&'b mut self, function: &str, args: T) -> Result<R, PluginError>
    where
        T: extism::ToBytes<'a>,
        R: extism::FromBytes<'b>,
    {
        self.0
            .call(function, args)
            .map_err(move |e| PluginError::FunctionCallError(function.to_string(), e))
    }
}


impl PluginPool {
    pub fn try_new(path: &Path) -> Result<Self, PluginError> {
        let mut plugin = Plugin::try_load(path)?;
        let metadata = plugin.get_metadata()?;

        Ok(PluginPool {
            metadata,
            capabilities: PluginCapabilities::from_plugin(&plugin),
            plugin_path: path.to_path_buf(),
            plugins: Mutex::new(VecDeque::from([plugin])),
            wait_condvar: Condvar::new(),
        })
    }

    pub fn get_free_plugin(&self) -> PluginHandle<'_> {
        let mut plugins = self.plugins.lock().unwrap();
        if let Some(plugin) = plugins.pop_front() {
            PluginHandle::new(self, plugin)
        } else {
            if self.metadata.allow_concurrency {
                info!(
                    "Creating new instance for plugin: {} ({})",
                    self.metadata.name, self.metadata.uuid
                );
                match Plugin::try_load(&self.plugin_path) {
                    Ok(plugin) => {
                        return PluginHandle::new(self, plugin);
                    }

                    Err(e) => {
                        error!(
                            "Failed to create instance of plugin, falling back to waiting for a free one: {}",
                            e
                        );
                    }
                }
            }

            // wait for a free plugin
            loop {
                plugins = self.wait_condvar.wait(plugins).unwrap();
                if let Some(plugin) = plugins.pop_front() {
                    return PluginHandle::new(self, plugin);
                }
            }
        }
    }
}

impl<'a> PluginHandle<'a> {
    pub fn new(pool: &'a PluginPool, plugin: Plugin) -> PluginHandle<'a> {
        PluginHandle {
            pool,
            plugin: Some(plugin),
        }
    }
}

impl std::ops::Deref for PluginHandle<'_> {
    type Target = Plugin;
    fn deref(&self) -> &Self::Target {
        self.plugin
            .as_ref()
            .expect("PluginHandle to never be empty")
    }
}

impl std::ops::DerefMut for PluginHandle<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.plugin
            .as_mut()
            .expect("PluginHandle to never be empty")
    }
}

impl Drop for PluginHandle<'_> {
    fn drop(&mut self) {
        let mut plugins = self.pool.plugins.lock().unwrap();
        if let Some(plugin) = self.plugin.take() {
            plugins.push_back(plugin);
        }
        self.pool.wait_condvar.notify_one();
    }
}

impl PluginSystem {
    #[instrument]
    pub async fn initialize(plugin_dir: PathBuf, db: Database) -> Result<Self, PluginSystemError> {
        info!(
            "Initializing plugin system with directory: {:?}",
            plugin_dir
        );

        let mut plugins = HashMap::new();

        for entry in std::fs::read_dir(&plugin_dir)
            .map_err(|_| PluginSystemError::InvalidDirectory(plugin_dir.clone()))?
        {
            let Ok(entry) = entry else {
                error!("failed to read directory entry");
                continue;
            };

            if !entry.path().is_file() || entry.path().extension() != Some(OsStr::new("wasm")) {
                debug!("Skipping non-WASM file: {:?}", entry.file_name());
                continue;
            }

            let pool = match PluginPool::try_new(&entry.path()) {
                Ok(plugin) => plugin,
                Err(e) => {
                    warn!(error = %e, "Failed to load plugin {:?}: {}", entry.file_name(), e);
                    continue;
                }
            };

            let id = match db.register_plugin(pool.metadata.uuid).await {
                Ok(plugin_id) => {
                    info!(
                        "Loaded plugin '{}' with ID {}",
                        pool.metadata.name, plugin_id.0
                    );
                    plugin_id
                }
                Err(e) => {
                    warn!(error = %e, "Failed to register plugin {:?} in database: {}", entry.file_name(), e);
                    continue;
                }
            };
            plugins.insert(id, pool);
        }

        info!("Loaded {} plugins", plugins.len());

        Ok(PluginSystem {
            plugins: Arc::new(plugins),
        })
    }

    pub fn get_free_plugin(&self, id: PluginId) -> Option<PluginHandle<'_>> {
        self.plugins.get(&id).map(|pool| pool.get_free_plugin())
    }

    pub fn cleanup_pool(&self) {
        debug!("Cleaning up plugin pool...");
        for pool in self.plugins.values() {
            let mut plugins = pool.plugins.lock().unwrap();
            while plugins.len() > 1 {
                plugins.pop_back();
            }
        }
    }
}
