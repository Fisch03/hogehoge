use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PluginMetadata {
    pub name: String,
    pub uuid: String,
    pub description: Option<String>,
    pub author: Option<String>,
}
