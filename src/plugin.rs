use extism::{Manifest, Plugin as LoadedPlugin, PluginBuilder};
use hogehoge_db::Database;
use hogehoge_types::{
    audio::{AudioBlock, AudioFile, PlaybackId},
    plugin::*,
};
use std::{
    collections::{HashMap, VecDeque},
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};
use thiserror::Error;
use tokio::time;
use tracing::*;

#[derive(Debug, Clone)]
pub struct PluginSystem {
    pub plugins: Arc<HashMap<PluginId, Arc<PluginPool>>>,
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

#[derive(Debug)]
pub struct PluginHandle {
    pool: Arc<PluginPool>,
    plugin: Option<Plugin>,
}

#[derive(Debug, Clone)]
pub struct PluginCapabilities {
    pub provide_tracks: bool,
    pub decode: bool,
}

impl PluginCapabilities {
    pub fn from_plugin(plugin: &Plugin) -> Self {
        PluginCapabilities {
            provide_tracks: plugin.has_fn("prepare_scan")
                && plugin.has_fn("scan")
                && plugin.has_fn("get_audio_file"),
            decode: plugin.has_fn("init_decoding")
                && plugin.has_fn("decode_block")
                && plugin.has_fn("finish_decoding"),
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
    pub fn has_fn(&self, function: &str) -> bool {
        self.0.function_exists(function)
    }

    pub fn get_metadata(&mut self) -> Result<PluginMetadata, PluginError> {
        self.call("get_metadata", ())
    }

    pub fn prepare_scan(&mut self) -> Result<PreparedScan, PluginError> {
        self.call("prepare_scan", ())
    }

    pub fn scan(&mut self, ident: &PluginTrackIdentifier) -> Result<ScanResult, PluginError> {
        self.call("scan", ident)
    }

    pub fn get_audio_file(
        &mut self,
        ident: &PluginTrackIdentifier,
    ) -> Result<AudioFile, PluginError> {
        self.call("get_audio_file", ident)
    }

    pub fn init_decoding(
        &mut self,
        playback_id: PlaybackId,
        file: AudioFile,
        gapless: bool,
    ) -> Result<InitDecodingResult, PluginError> {
        self.call(
            "init_decoding",
            InitDecodingArgs {
                playback_id,
                file,
                gapless,
            },
        )
    }

    pub fn decode_block(
        &mut self,
        playback_id: PlaybackId,
    ) -> Result<Option<AudioBlock>, PluginError> {
        self.call("decode_block", playback_id)
    }

    pub fn finish_decoding(&mut self, playback_id: PlaybackId) -> Result<(), PluginError> {
        self.call("finish_decoding", playback_id)
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
    pub fn try_new(path: &Path) -> Result<Arc<Self>, PluginError> {
        let mut plugin = Plugin::try_load(path)?;
        let metadata = plugin.get_metadata()?;

        let pool = Arc::new(PluginPool {
            metadata,
            capabilities: PluginCapabilities::from_plugin(&plugin),
            plugin_path: path.to_path_buf(),
            plugins: Mutex::new(VecDeque::new()),
            wait_condvar: Condvar::new(),
        });

        tokio::spawn({
            let pool = pool.clone();
            async move {
                let mut interval = time::interval(time::Duration::from_secs(30));

                loop {
                    interval.tick().await;
                    trace!(
                        "Performing plugin cleanup for plugin '{}'",
                        pool.metadata.name
                    );

                    let mut plugins = pool.plugins.lock().unwrap();
                    if !plugins.is_empty() {
                        info!(
                            "Cleaning up {} idle instances of plugin '{}'",
                            plugins.len(),
                            pool.metadata.name
                        );
                        plugins.clear();
                    }
                }
            }
        });

        Ok(pool)
    }

    pub fn get_plugin(self: &Arc<Self>) -> PluginHandle {
        let mut plugins = self.plugins.lock().unwrap();
        if let Some(plugin) = plugins.pop_front() {
            PluginHandle::new(self.clone(), plugin)
        } else {
            info!(
                "Creating new instance for plugin: {} ({})",
                self.metadata.name, self.metadata.uuid
            );

            let plugin = Plugin::try_load(&self.plugin_path).expect("Plugin loading to never fail");
            PluginHandle::new(self.clone(), plugin)
        }
    }
}

impl PluginHandle {
    pub fn new(pool: Arc<PluginPool>, plugin: Plugin) -> PluginHandle {
        PluginHandle {
            pool,
            plugin: Some(plugin),
        }
    }

    pub fn capabilities(&self) -> &PluginCapabilities {
        &self.pool.capabilities
    }
}

impl std::ops::Deref for PluginHandle {
    type Target = Plugin;
    fn deref(&self) -> &Self::Target {
        self.plugin
            .as_ref()
            .expect("PluginHandle to never be empty")
    }
}

impl std::ops::DerefMut for PluginHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.plugin
            .as_mut()
            .expect("PluginHandle to never be empty")
    }
}

impl Drop for PluginHandle {
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

    pub fn get_plugin(&self, id: PluginId) -> Option<PluginHandle> {
        self.plugins.get(&id).map(|pool| pool.get_plugin())
    }
}
