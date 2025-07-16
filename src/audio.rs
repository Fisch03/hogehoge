use hogehoge_types::{AudioFile, ChannelCount, PlaybackId, Sample, SampleRate};
use crate::plugin::{PluginHandle, PluginError};
use thiserror::Error;
use rodio::Source;
use tracing::*;
use std::time::Duration;

#[derive(Debug)]
pub struct PluginAudioSource<'a> {
    playback_id: PlaybackId,
    plugin: PluginHandle<'a>,

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
}

impl<'a> PluginAudioSource<'a> {
    pub fn new(mut plugin: PluginHandle<'a>, file: AudioFile) -> Result<PluginAudioSource<'a>, PluginAudioSourceError> {
        if !plugin.capabilities().decode {
            return Err(PluginAudioSourceError::CannotDecode);
        }

        let playback_id = PlaybackId::new();

        let init_result = plugin.init_decoding(playback_id, file)?;
        let initial_block = plugin.decode_block(playback_id)?.ok_or(PluginAudioSourceError::NoAudioData)?;

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
}

impl Iterator for PluginAudioSource<'_> {
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

impl Source for PluginAudioSource<'_> {
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

impl Drop for PluginAudioSource<'_> {
    fn drop(&mut self) {
        self.plugin.finish_decoding(self.playback_id).unwrap_or_else(|e| {
            warn!("Failed to finish decoding for {:?}: {}", self.playback_id, e);
        });

        info!("Finished decoding for {:?}", self.playback_id);
    }
}