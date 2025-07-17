use std::path::Path;
use thiserror::Error;

use extism_pdk::{FnResult, plugin_fn};
use hogehoge_types::{
    AudioFile, FsMount, PluginMetadata, PluginTrackIdentifier, PreparedScan, ScanResult, uuid,
};
use std::fs;

mod tags;

#[plugin_fn]
pub fn get_metadata() -> FnResult<PluginMetadata> {
    Ok(PluginMetadata {
        name: "Filesystem".to_string(),
        uuid: uuid!("c2940863-8121-447e-ae25-499a809c361e"),
        description: Some("Load, import and manage tracks from the local filesystem".to_string()),
        author: None,

        fs_mounts: vec![FsMount {
            internal_path: "/music".to_string(),
            description: "Music files".to_string(),
        }],

        allow_concurrency: true,
    })
}

#[plugin_fn]
pub fn prepare_scan() -> FnResult<PreparedScan> {
    let mut tracks = Vec::new();
    scan_recurse(&mut tracks, "/music")?;

    Ok(PreparedScan { tracks })
}
fn scan_recurse<P: AsRef<Path>>(tracks: &mut Vec<PluginTrackIdentifier>, path: P) -> FnResult<()> {
    let path = path.as_ref();

    if path.is_file() {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("aac") | Some("ape") | Some("aiff") | Some("flac") | Some("mp3") | Some("mp4")
            | Some("m4a") | Some("mpc") | Some("ogg") | Some("opus") | Some("wav")
            | Some("wma") | Some("wvc") | Some("wv") => {}
            _ => return Ok(()), // Unsupported file type, skip
        }

        let path = path
            .strip_prefix("/music")
            .unwrap()
            .to_string_lossy()
            .to_string();
        let ident = PluginTrackIdentifier(path);
        tracks.push(ident);
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            scan_recurse(tracks, entry.path())?;
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
enum ScanError {
    #[error("File did not contain any tags")]
    NoTags,
}

#[plugin_fn]
pub fn scan(ident: PluginTrackIdentifier) -> FnResult<ScanResult> {
    use lofty::file::TaggedFileExt;

    let path = Path::new("/music").join(ident.0);

    let tagged_file = lofty::read_from_path(path)?;
    let tag = tagged_file.primary_tag().ok_or(ScanError::NoTags)?;

    let tags = tags::map_lofty_to_internal(tag)?;

    Ok(ScanResult { tags })
}

#[derive(Debug, Error)]
enum GetAudioFileError {
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
}

#[plugin_fn]
pub fn get_audio_file(ident: PluginTrackIdentifier) -> FnResult<AudioFile> {
    let path = Path::new("/music").join(ident.0);

    let data = fs::read(&path).map_err(GetAudioFileError::ReadError)?;

    let extension = path.extension().and_then(|ext| ext.to_str());
    let format_hint = extension.map(|ext| ext.to_string());

    Ok(AudioFile { data, format_hint })
}
