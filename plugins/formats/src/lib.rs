use extism_pdk::{FnResult, plugin_fn};
use hogehoge_types::{
    uuid, PlaybackId, PluginMetadata
};

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

struct DecoderState {
    playback_id: PlaybackId,
};

static ENDODING_STATE: Mutex<Option<DecoderState>> = Mutex::new(None);

#[plugin_fn]
pub fn init_playback(playback_id: PlaybackId) -> FnResult<()> {
    let mut state = ENDODING_STATE.lock().unwrap();
    
    *state = Some(DecoderState {
        playback_id,
    });
}