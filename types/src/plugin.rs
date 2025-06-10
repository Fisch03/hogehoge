use crate::Tags;
use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
pub use uuid::{Uuid, uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PluginId(i32);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PluginTrackIdentifier(pub String);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PluginMetadata {
    pub name: String,
    pub uuid: Uuid,
    pub description: Option<String>,
    pub author: Option<String>,

    pub fs_mounts: Vec<FsMount>,
    pub allow_concurrency: bool,
}

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct FsMount {
    pub internal_path: String,
    pub description: String,
}

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PreparedScan {
    pub tracks: Vec<PluginTrackIdentifier>,
}

#[derive(Clone, Debug, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct ScanResult {
    pub tags: Tags,
}
