use extism::{Manifest, Plugin as LoadedPlugin, PluginBuilder};
use hogehoge_types::{PluginMetadata, PreparedScan, Uuid};
use std::{
    collections::{HashMap, VecDeque},
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Condvar, Mutex},
};
use thiserror::Error;
use tracing::*;

pub struct PluginSystem {
    pub plugins: HashMap<Uuid, PluginPool>,
}

#[derive(Debug, Error)]
pub enum PluginSystemError {
    #[error("Specified plugin directory does not exist: {0}")]
    InvalidDirectory(PathBuf),
}

pub struct PluginPool {
    metadata: PluginMetadata,
    plugin_path: PathBuf,
    plugins: Mutex<VecDeque<Plugin>>,
    wait_condvar: Condvar,
}

pub struct Plugin(LoadedPlugin);

pub struct PluginHandle<'a> {
    pool: &'a PluginPool,
    plugin: Option<Plugin>,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to initialize plugin: {0}")]
    InitializationError(extism::Error),

    #[error("Plugin does not implement required function '{0}'")]
    MissingRequiredFunction(&'static str),
    #[error("Failed to call function '{0}': {1}")]
    FunctionCallError(String, extism::Error),

    #[error("Invalid plugin uuid")]
    InvalidUuid,
}

impl Plugin {
    pub fn get_metadata(&mut self) -> Result<PluginMetadata, PluginError> {
        self.call("get_metadata", ())
    }

    pub fn prepare_scan(&mut self) -> Result<PreparedScan, PluginError> {
        self.call("prepare_scan", ())
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
        plugin.0.id = metadata.uuid.clone();

        Ok(plugin)
    }

    fn call<T, R>(&mut self, function: &str, args: T) -> Result<R, PluginError>
    where
        T: for<'a> extism::ToBytes<'a>,
        R: for<'a> extism::FromBytes<'a>,
    {
        self.0
            .call(function, args)
            .map_err(|e| PluginError::FunctionCallError(function.to_string(), e))
    }
}

impl PluginPool {
    pub fn try_new(path: &Path) -> Result<Self, PluginError> {
        let mut plugin = Plugin::try_load(path)?;
        let metadata = plugin.get_metadata()?;

        Ok(PluginPool {
            metadata,
            plugin_path: path.to_path_buf(),
            plugins: Mutex::new(VecDeque::from([plugin])),
            wait_condvar: Condvar::new(),
        })
    }

    pub fn get_free_plugin(&self) -> Result<PluginHandle<'_>, PluginError> {
        let mut plugins = self.plugins.lock().unwrap();
        if let Some(plugin) = plugins.pop_front() {
            Ok(PluginHandle::new(self, plugin))
        } else {
            if !self.metadata.allow_concurrency {
                loop {
                    plugins = self.wait_condvar.wait(plugins).unwrap();
                    if let Some(plugin) = plugins.pop_front() {
                        return Ok(PluginHandle::new(self, plugin));
                    }
                }
            } else {
                let new_plugin = Plugin::try_load(&self.plugin_path)?;
                Ok(PluginHandle::new(self, new_plugin))
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
    pub fn initialize(plugin_dir: PathBuf) -> Result<Self, PluginSystemError> {
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

            plugins.insert(pool.metadata.uuid.clone(), pool);
        }

        Ok(PluginSystem { plugins })
    }

    pub fn get_free_plugin(&mut self, uuid: &Uuid) -> Result<PluginHandle<'_>, PluginError> {
        if let Some(pool) = self.plugins.get_mut(uuid) {
            pool.get_free_plugin()
        } else {
            Err(PluginError::InvalidUuid)
        }
    }
}
