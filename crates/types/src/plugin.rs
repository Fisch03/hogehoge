use crate::{AudioFile, PlaybackId, Tags};
use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
use std::time::Duration;
pub use uuid::{Uuid, uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct PluginId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct PluginTrackIdentifier(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "internal", derive(sqlx::FromRow))]
pub struct UniqueTrackIdentifier {
    pub plugin_id: PluginId,
    pub plugin_data: PluginTrackIdentifier,
}

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PluginMetadata {
    pub name: String,
    pub uuid: Uuid,
    pub description: Option<String>,
    pub author: Option<String>,

    pub fs_mounts: Vec<FsMount>,
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

#[derive(Clone, Debug, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct InitDecodingArgs {
    pub playback_id: PlaybackId,
    pub file: AudioFile,
    pub gapless: bool,
}

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct InitDecodingResult {
    pub duration: Option<Duration>,
}
