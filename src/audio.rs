use crate::plugin::{PluginError, PluginHandle, PluginSystem};
use crate::queue::{Queue, QueueUpdate, QueueUpdateRx};
use hogehoge_types::{
    AudioFile, ChannelCount, PlaybackId, PluginId, Sample, SampleRate, UniqueTrackIdentifier,
};
use rodio::source::TrackPosition;
use rodio::{OutputStream, OutputStreamBuilder, Source, source::Zero};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    runtime,
    sync::{oneshot, watch},
};
use tracing::*;

const SILENCE_LENGTH: usize = 512;

#[derive(Clone)]
pub struct AudioPlayer {
    pub queue: Arc<Queue>,
    playback_state: watch::Receiver<PlaybackState>,

    #[allow(dead_code)]
    inner: Arc<AudioPlayerInner>,
}

pub struct AudioPlayerInner {
    #[allow(dead_code)]
    output_stream: OutputStream,
}

#[derive(Default, Clone, Debug)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing {
        duration: Option<Duration>,
        position: Duration,
        paused: bool,
    },
}

pub struct QueueSource {
    current_playing: QueueCurrentSource,
    cache: Option<(
        UniqueTrackIdentifier,
        oneshot::Receiver<Result<PluginAudioSource, PluginAudioSourceError>>,
    )>,

    rt: runtime::Handle,
    queue: Arc<Queue>,
    update_rx: QueueUpdateRx,

    state_tx: watch::Sender<PlaybackState>,
    samples_to_state_update: usize,
}

#[allow(clippy::large_enum_variant)]
enum QueueCurrentSource {
    Source(TrackPosition<PluginAudioSource>),
    Nothing(Zero),
}

impl AudioPlayer {
    pub fn new(plugins: PluginSystem) -> AudioPlayer {
        let output_stream = OutputStreamBuilder::open_default_stream()
            .expect("Failed to open default audio output stream");

        let queue = Queue::new(plugins);

        let queue_src = QueueSource::new(queue.clone());

        let playback_state = queue_src.subscribe_state();

        output_stream.mixer().add(queue_src);

        let inner = AudioPlayerInner { output_stream };

        AudioPlayer {
            queue,
            playback_state,
            inner: Arc::new(inner),
        }
    }

    pub fn subscribe_state(&self) -> watch::Receiver<PlaybackState> {
        self.playback_state.clone()
    }
}

impl std::fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("queue", &self.queue)
            .field("playback_state", &self.playback_state)
            .finish()
    }
}

impl QueueSource {
    pub fn new(queue: Arc<Queue>) -> Self {
        let current_playing = QueueCurrentSource::new_silence();
        let update_rx = queue.subscribe_updates();

        let rt = runtime::Handle::current();

        let state_tx = watch::Sender::new(PlaybackState::default());

        QueueSource {
            current_playing,
            cache: None,
            queue,
            rt,
            update_rx,
            state_tx,
            samples_to_state_update: 1,
        }
    }

    pub fn subscribe_state(&self) -> watch::Receiver<PlaybackState> {
        self.state_tx.subscribe()
    }

    fn start_cache(
        &self,
        track: UniqueTrackIdentifier,
    ) -> oneshot::Receiver<Result<PluginAudioSource, PluginAudioSourceError>> {
        let (tx, rx) = oneshot::channel();

        info!("Starting cache for track: {:?}", track);

        let plugin_system = self.queue.plugin_system.clone();
        self.rt.spawn_blocking(move || {
            let audio_source =
                PluginAudioSource::from_track_identifier(&plugin_system, track.clone());

            info!("Finished caching track: {:?}", track);

            let _ = tx.send(audio_source);
        });

        rx
    }

    fn cache_track(&mut self, track: UniqueTrackIdentifier) {
        if matches!(&self.cache, Some((cached_id, _)) if *cached_id == track) {
            debug!("Track {:?} is already cached", track);
            return;
        }

        self.cache = Some((track.clone(), self.start_cache(track)));
    }

    fn get_cached_source(
        &mut self,
        track: &UniqueTrackIdentifier,
    ) -> Result<PluginAudioSource, PluginAudioSourceError> {
        let rx = self
            .cache
            .take()
            .filter(|(cached_id, _)| cached_id == track)
            .map(|(_, rx)| rx)
            .unwrap_or_else(|| self.start_cache(track.clone()));

        match rx.blocking_recv() {
            Ok(result) => result,

            Err(_) => {
                error!("Failed to receive cached track for {:?}", track);
                Err(PluginAudioSourceError::NoAudioData)
            }
        }
    }
}

impl Iterator for QueueSource {
    type Item = Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.samples_to_state_update == 0 {
                if let Ok(update) = self.update_rx.try_recv() {
                    match update {
                        QueueUpdate::CurrentTrackChanged | QueueUpdate::TrackAdded(_) => {
                            if let Some(upcoming) = self.queue.get_next_track() {
                                self.cache_track(upcoming);
                            }
                        }
                    }
                }

                match &mut self.current_playing {
                    QueueCurrentSource::Nothing(_) => {
                        self.state_tx.send_if_modified(|state| {
                            self.samples_to_state_update = SILENCE_LENGTH;

                            let new = !matches!(state, PlaybackState::Stopped);

                            if new {
                                *state = PlaybackState::Stopped;
                            }

                            new
                        });
                    }
                    QueueCurrentSource::Source(source) => {
                        let one_sec_in_samples =
                            source.sample_rate() as f32 * source.channels() as f32;

                        self.samples_to_state_update = (one_sec_in_samples / 16.0) as usize;

                        self.state_tx.send_replace(PlaybackState::Playing {
                            duration: source.total_duration(),
                            position: source.get_pos(),
                            paused: false,
                        });
                    }
                }
            }
            self.samples_to_state_update -= 1;

            if let Some(current) = self.current_playing.next() {
                return Some(current);
            }

            let upcoming = self.queue.forward();

            let source = upcoming
                .and_then(|upcoming| match self.get_cached_source(&upcoming) {
                    Ok(source) => {
                        info!("Playing next track: {:?}", upcoming);
                        Some(QueueCurrentSource::new_source(source))
                    }
                    Err(e) => {
                        warn!("Failed to get cached source for {:?}: {}", upcoming, e);
                        None
                    }
                })
                .unwrap_or_else(|| {
                    trace!("No more tracks in the queue, playing silence");
                    QueueCurrentSource::new_silence()
                });

            self.current_playing = source;
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

impl std::fmt::Debug for QueueSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueSource")
            .field("queue", &self.queue)
            .finish()
    }
}

impl QueueCurrentSource {
    pub fn new_source(source: PluginAudioSource) -> Self {
        QueueCurrentSource::Source(source.track_position())
    }

    pub fn new_silence() -> Self {
        QueueCurrentSource::Nothing(Zero::new_samples(1, 44100, SILENCE_LENGTH))
    }

    #[inline]
    pub fn as_source(&self) -> &dyn Source {
        match self {
            QueueCurrentSource::Source(source) => source,
            QueueCurrentSource::Nothing(silence) => silence,
        }
    }

    #[inline]
    pub fn as_mut_source(&mut self) -> &mut dyn Source {
        match self {
            QueueCurrentSource::Source(source) => source,
            QueueCurrentSource::Nothing(silence) => silence,
        }
    }
}

impl Iterator for QueueCurrentSource {
    type Item = Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.as_mut_source().next()
    }
}

impl Source for QueueCurrentSource {
    fn current_span_len(&self) -> Option<usize> {
        self.as_source().current_span_len()
    }
    fn sample_rate(&self) -> SampleRate {
        self.as_source().sample_rate()
    }
    fn channels(&self) -> ChannelCount {
        self.as_source().channels()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.as_source().total_duration()
    }
}

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

        debug!("Initialized decoding for {:?}", playback_id);

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
            .get_plugin(track.plugin_id)
            .ok_or(PluginAudioSourceError::MissingFileProvider(track.plugin_id))?;

        let file = file_provider_plugin.get_audio_file(&track.plugin_data)?;

        let audio_source = plugin_system
            .plugins
            .values()
            .filter(|pool| pool.capabilities.decode)
            .find_map(|pool| {
                let decoder_plugin = pool.get_plugin();

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

        debug!("Finished decoding for {:?}", self.playback_id);
    }
}

impl std::fmt::Debug for PluginAudioSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAudioSource")
            .field("playback_id", &self.playback_id)
            .field("duration", &self.duration)
            .field("current_sample_rate", &self.current_sample_rate)
            .field("current_channel_count", &self.current_channel_count)
            .field("current_block_length", &self.current_block.len())
            .finish()
    }
}
