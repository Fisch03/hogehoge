use std::path::Path;

use extism_pdk::{FnResult, plugin_fn};
use hogehoge_types::{
    FsMount, PluginMetadata, PluginTrackIdentifier, PreparedScan, ScanResult, uuid,
};
use std::fs;

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
        let path = path.to_string_lossy().to_string();
        let ident = PluginTrackIdentifier(path.clone());
        tracks.push(ident);
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            scan_recurse(tracks, entry.path())?;
        }
    }

    Ok(())
}

#[plugin_fn]
pub fn scan(ident: PluginTrackIdentifier) -> FnResult<ScanResult> {
    Ok(ScanResult::Path(ident.0))
}
