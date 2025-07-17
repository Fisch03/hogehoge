use crate::plugin::{PluginError, PluginHandle, PluginSystem};
use hogehoge_types::{
    AudioFile, ChannelCount, PlaybackId, PluginId, Sample, SampleRate, UniqueTrackIdentifier,
};
use rodio::{OutputStream, OutputStreamBuilder, Source, source::Zero};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;
use tokio::{runtime, sync::oneshot};
use tracing::*;

const SILENCE_LENGTH: usize = 512;
fn make_silence() -> Box<dyn Source + Send> {
    Box::new(Zero::new_samples(1, 44100, SILENCE_LENGTH))
}

pub struct AudioPlayer {
    _output_stream: OutputStream,
    queue: Arc<Mutex<Queue>>,
}

struct QueueSource {
    current_playing: Box<dyn Source + Send>,
    queue: Arc<Mutex<Queue>>,
}

#[derive(Debug)]
struct Queue {
    items: Vec<UniqueTrackIdentifier>,
    cache: HashMap<
        UniqueTrackIdentifier,
        oneshot::Receiver<Result<PluginAudioSource, PluginAudioSourceError>>,
    >,
    rt: runtime::Handle,
    plugin_system: PluginSystem,

    current_index: usize,
}

impl AudioPlayer {
    pub fn new(plugins: PluginSystem) -> Arc<AudioPlayer> {
        let _output_stream = OutputStreamBuilder::open_default_stream()
            .expect("Failed to open default audio output stream");

        let (queue, src) = Queue::new(plugins);

        _output_stream.mixer().add(src);

        Arc::new(AudioPlayer {
            _output_stream,
            queue,
        })
    }

    pub fn queue_track(&self, track: UniqueTrackIdentifier) {
        let mut queue = self.queue.lock().unwrap();

        info!("Queued track: {:?}", track);
        queue.items.push(track.clone());
        queue.ensure_upcoming_cache();
    }
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("queue", &self.queue)
            .finish()
    }
}

impl Queue {
    pub fn new(plugins: PluginSystem) -> (Arc<Mutex<Queue>>, QueueSource) {
        let rt = runtime::Handle::current();

        let queue = Arc::new(Mutex::new(Queue {
            items: Vec::new(),
            cache: HashMap::new(),
            current_index: 0,
            rt,
            plugin_system: plugins,
        }));

        let src = QueueSource {
            queue: queue.clone(),
            current_playing: make_silence(),
        };

        (queue, src)
    }

    fn ensure_upcoming_cache(&mut self) {
        if let Some(next_track) = self.items.get(self.current_index) {
            if !self.cache.contains_key(next_track) {
                self.start_cache(next_track.clone());
            }
        }
    }

    fn start_cache(
        &mut self,
        track: UniqueTrackIdentifier,
    ) -> oneshot::Receiver<Result<PluginAudioSource, PluginAudioSourceError>> {
        let (tx, rx) = oneshot::channel();

        info!("Starting cache for track: {:?}", track);

        let plugin_system = self.plugin_system.clone();
        self.rt.spawn_blocking(move || {
            let audio_source =
                PluginAudioSource::from_track_identifier(&plugin_system, track.clone());

            if tx.send(audio_source).is_err() {
                warn!("Failed to send cached track for {:?}", track);
            }
        });

        rx
    }

    pub fn cache_track(&mut self, track: UniqueTrackIdentifier) {
        let rx = self.start_cache(track.clone());

        self.cache.insert(track, rx);
    }

    pub fn get_loaded_track(
        &mut self,
        track_id: &UniqueTrackIdentifier,
    ) -> Result<PluginAudioSource, PluginAudioSourceError> {
        let rx = self.cache.remove(track_id).unwrap_or_else(|| {
            warn!("Track {:?} wasnt precached, starting cache now", track_id);
            self.start_cache(track_id.clone())
        });

        match rx.blocking_recv() {
            Ok(result) => result,
            Err(_) => {
                error!("Failed to receive cached track for {:?}", track_id);
                Err(PluginAudioSourceError::NoAudioData)
            }
        }
    }
}

impl Source for QueueSource {
    fn current_span_len(&self) -> Option<usize> {
        if let Some(span_len) = self.current_playing.current_span_len() {
            if span_len != 0 {
                return Some(span_len);
            }
        }

        let (lower_bound, _) = self.current_playing.size_hint();
        if lower_bound > 0 {
            return Some(lower_bound);
        }

        Some(SILENCE_LENGTH)
    }

    fn sample_rate(&self) -> SampleRate {
        self.current_playing.sample_rate()
    }

    fn channels(&self) -> ChannelCount {
        self.current_playing.channels()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for QueueSource {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current) = self.current_playing.next() {
                return Some(current);
            }

            let mut queue = self.queue.lock().unwrap();

            let source = queue
                .items
                .get(queue.current_index)
                .cloned()
                .and_then(|track_id| match queue.get_loaded_track(&track_id) {
                    Ok(audio_source) => {
                        queue.current_index += 1;
                        queue.ensure_upcoming_cache();

                        info!("Playing track: {:?}", track_id);
                        Some(Box::new(audio_source) as Box<dyn Source + Send>)
                    }
                    Err(e) => {
                        error!("Failed to load track {:?}: {}", track_id, e);
                        None
                    }
                })
                .unwrap_or_else(|| {
                    trace!("Queue is empty, returning silence");
                    make_silence()
                });

            self.current_playing = source;
        }
    }
}

impl std::fmt::Debug for QueueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSource")
            .field("queue", &self.queue)
            .finish()
    }
}

#[derive(Debug)]
pub struct PluginAudioSource {
    playback_id: PlaybackId,
    plugin: PluginHandle,

    duration: Option<Duration>,

    current_sample_rate: SampleRate,
    current_channel_count: ChannelCount,
    current_block: Vec<Sample>,
    block_index: usize,
}

#[derive(Error, Debug)]
pub enum PluginAudioSourceError {
    #[error("Cannot decode audio with this plugin")]
    CannotDecode,

    #[error("Decoding did not return any audio data")]
    NoAudioData,

    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),

    #[error("File provider plugin with ID {0:?} is missing")]
    MissingFileProvider(PluginId),

    #[error("No plugin found that can decode audio for track '{0:?}'")]
    NoPluginForTrack(UniqueTrackIdentifier),
}

impl PluginAudioSource {
    pub fn new(
        mut plugin: PluginHandle,
        file: AudioFile,
    ) -> Result<PluginAudioSource, PluginAudioSourceError> {
        if !plugin.capabilities().decode {
            return Err(PluginAudioSourceError::CannotDecode);
        }

        let playback_id = PlaybackId::new();

        let init_result = plugin.init_decoding(playback_id, file, true)?; //TODO: make gapless configurable
        let initial_block = plugin
            .decode_block(playback_id)?
            .ok_or(PluginAudioSourceError::NoAudioData)?;

        info!("Initialized decoding for {:?}", playback_id);

        Ok(PluginAudioSource {
            playback_id,
            plugin,

            duration: init_result.duration,

            current_sample_rate: initial_block.sample_rate,
            current_channel_count: initial_block.channel_count,
            current_block: initial_block.samples,
            block_index: 0,
        })
    }

    pub fn from_track_identifier(
        plugin_system: &PluginSystem,
        track: UniqueTrackIdentifier,
    ) -> Result<PluginAudioSource, PluginAudioSourceError> {
        let mut file_provider_plugin = plugin_system
            .get_free_plugin(track.plugin_id)
            .ok_or(PluginAudioSourceError::MissingFileProvider(track.plugin_id))?;

        let file = file_provider_plugin.get_audio_file(&track.plugin_data)?;

        let audio_source = plugin_system
            .plugins
            .values()
            .filter(|pool| pool.capabilities.decode)
            .find_map(|pool| {
                let decoder_plugin = pool.get_free_plugin();

                match PluginAudioSource::new(decoder_plugin, file.clone()) {
                    Ok(source) => Some(source),
                    Err(e) => {
                        debug!("Plugin '{}' cannot decode audio: {}", pool.metadata.name, e);
                        None
                    }
                }
            })
            .ok_or(PluginAudioSourceError::NoPluginForTrack(track.clone()))?;

        Ok(audio_source)
    }
}

impl Iterator for PluginAudioSource {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.block_index >= self.current_block.len() {
            let block = match self.plugin.decode_block(self.playback_id) {
                Ok(Some(decoded)) => decoded,
                Ok(None) => return None,

                Err(e) => {
                    warn!("Error decoding block: {}", e);
                    return None;
                }
            };

            self.current_block = block.samples;
            self.current_sample_rate = block.sample_rate;
            self.current_channel_count = block.channel_count;
            self.block_index = 0;
        }

        let sample = self.current_block.get(self.block_index).cloned()?;
        self.block_index += 1;

        Some(sample)
    }
}

impl Source for PluginAudioSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.current_block.len())
    }

    fn sample_rate(&self) -> SampleRate {
        self.current_sample_rate
    }

    fn channels(&self) -> ChannelCount {
        self.current_channel_count
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
}

impl Drop for PluginAudioSource {
    fn drop(&mut self) {
        self.plugin
            .finish_decoding(self.playback_id)
            .unwrap_or_else(|e| {
                warn!(
                    "Failed to finish decoding for {:?}: {}",
                    self.playback_id, e
                );
            });

        info!("Finished decoding for {:?}", self.playback_id);
    }
}
