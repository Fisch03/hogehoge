use extism_pdk::{FnResult, plugin_fn};
use hogehoge_types::{AudioBlock, PlaybackId, InitDecodingArgs, PluginMetadata, uuid, Sample, InitDecodingResult};
use std::{collections::HashMap, io::Cursor, sync::{Mutex, LazyLock}};
use symphonia::core::{
    audio::{AudioBuffer,  AudioBufferRef, Signal}, codecs::{Decoder, DecoderOptions}, conv::IntoSample, errors::Error as SymphoniaError, formats::{FormatOptions, FormatReader}, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
    sample::Sample as SymphoniaSample
};
use thiserror::Error;


#[plugin_fn]
pub fn get_metadata() -> FnResult<PluginMetadata> {
    Ok(PluginMetadata {
        name: "Base Formats".to_string(),
        uuid: uuid!("6968fce9-2521-410c-b933-32c6e5800f93"),
        description: Some("Playback support for commonly used audio formats".to_string()),
        author: None,

        fs_mounts: vec![],

        allow_concurrency: true,
    })
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("File doesn't contain a valid audio track")]
    InvalidAudioTrack,

    #[error("No encoding initialized for given playback ID")]
    EncodingNotInitialized,
}

struct DecoderState {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
}

static DECODER_STATES: LazyLock<Mutex<HashMap<PlaybackId, DecoderState>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[plugin_fn]
pub fn init_decoding(InitDecodingArgs { playback_id, file }: InitDecodingArgs) -> FnResult<InitDecodingResult> {
    let mut state = DECODER_STATES.lock().unwrap();

    let mss = MediaSourceStream::new(Box::new(Cursor::new(file.data)), Default::default());

    let mut hint = Hint::new();
    if let Some(format) = file.format_hint {
        hint.with_extension(&format);
    }

    let meta_options: MetadataOptions = Default::default();
    let format_options = FormatOptions {
        enable_gapless: true, // TODO: make configurable
        ..Default::default()
    };
    let probed =
        symphonia::default::get_probe().format(&hint, mss, &format_options, &meta_options)?;

    let format = probed.format;

    let track = format.default_track().ok_or(DecodeError::InvalidAudioTrack)?;

    let decoder_options: DecoderOptions = Default::default();
    let decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_options)?;

    let duration = track.codec_params.time_base.zip(track.codec_params.n_frames)
        .map(|(time_base, n_frames)| time_base.calc_time(n_frames).into());

    let track_id = track.id;


    state.insert(
        playback_id,
        DecoderState {
            format,
            decoder,
            track_id,
        },
    );

    Ok(InitDecodingResult {
        duration
    })
}

#[plugin_fn]
pub fn decode_block(playback_id: PlaybackId) -> FnResult<Option<AudioBlock>> {
    let mut state = DECODER_STATES.lock().unwrap();
    let state = state
        .get_mut(&playback_id)
        .ok_or(DecodeError::EncodingNotInitialized)?;

    loop {
    let packet = state.format.next_packet()?;

    while !state.format.metadata().is_latest() {
        state.format.metadata().pop();
    }

    if packet.track_id() != state.track_id {
        continue;
    }

    match state.decoder.decode(&packet) {
        Ok(decoded) => {
            if decoded.frames() == 0 {
                continue;
            }

            let sample_rate = decoded.spec().rate;
            let channel_count = decoded.spec().channels.count() as u16;
            let samples = copy_decoded_samples(decoded);

            return Ok(Some(AudioBlock {
                samples,
                sample_rate,
                channel_count,
            }));
        }

        Err(SymphoniaError::IoError(_)) => continue,
        Err(SymphoniaError::DecodeError(_)) => continue,

        Err(_) => return Ok(None),
    }
}
}

#[plugin_fn]
pub fn finish_decoding(playback_id: PlaybackId) -> FnResult<()> {
    let mut state = DECODER_STATES.lock().unwrap();
    state.remove(&playback_id);
    Ok(())
}

fn copy_decoded_samples(src: AudioBufferRef) ->  Vec<Sample> {
    match src {
        AudioBufferRef::U8(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::U16(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::U24(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::U32(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::S8(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::S16(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::S24(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::S32(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::F32(buf) => copy_decoded_samples_inner(&buf),
        AudioBufferRef::F64(buf) => copy_decoded_samples_inner(&buf),
    }
}

fn copy_decoded_samples_inner<F>(src: &AudioBuffer<F>) -> Vec<Sample> where F: SymphoniaSample + IntoSample<Sample>{
    let n_channels = src.spec().channels.count();
    let n_samples = src.frames() * n_channels;

    let mut dest: Vec<Sample> = vec![0.0; n_samples];

    for ch in 0..n_channels {
        let src_channel = src.chan(ch);
        for (dst, src) in dest[ch..].iter_mut().step_by(n_channels).zip(src_channel) {
            *dst = (*src).into_sample();
        }
    }

    dest
}
