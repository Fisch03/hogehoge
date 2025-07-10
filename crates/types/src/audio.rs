use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PlaybackId(Uuid);