use extism_pdk::{FnResult, plugin_fn};
use hogehoge_types::PluginMetadata;

// #[host_fn]
// extern "ExtismHost" {
//     fn add_track()
// }

#[plugin_fn]
pub fn get_metadata() -> FnResult<PluginMetadata> {
    Ok(PluginMetadata {
        name: "Filesystem".to_string(),
        uuid: "c2940863-8121-447e-ae25-499a809c361e".to_string(),
        description: Some("Load, import and manage tracks from the local filesystem".to_string()),
        author: None,
    })
}
