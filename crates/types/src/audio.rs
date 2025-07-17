use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct PlaybackId(Uuid);

impl PlaybackId {
    #[cfg(feature = "internal")]
    pub fn new() -> Self {
        PlaybackId(Uuid::new_v4())
    }
}

#[cfg(feature = "internal")]
impl Default for PlaybackId {
    fn default() -> Self {
        PlaybackId::new()
    }
}

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct AudioFile {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub format_hint: Option<String>,
}

pub type Sample = f32;
pub type SampleRate = u32;
pub type ChannelCount = u16;

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct AudioBlock {
    pub samples: Vec<Sample>,
    pub sample_rate: u32,
    pub channel_count: u16,
}
