use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct ArtistId(i32);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct Artist {
    id: ArtistId,
    mbid: Option<String>,

    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct AlbumId(i32);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct Album {
    id: AlbumId,
    mbid: Option<String>,

    title: String,

    artist_id: Option<ArtistId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct TrackId(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct TrackGroupId(i32);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct Tags(HashMap<TagKey, String>);

impl Tags {
    pub fn new() -> Self {
        Tags(HashMap::new())
    }
}

impl std::ops::Deref for Tags {
    type Target = HashMap<TagKey, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
#[non_exhaustive]
pub enum TagKey {
    // general
    TrackTitle,
    MusicBrainzWorkId,
    MusicBrainzTrackId,
    MusicBrainzRecordingId,
    TrackSubtitle,
    TrackTitleSortOrder,
    Comment,
    Description,
    Language,
    Script,
    Lyrics,

    // album
    AlbumTitle,
    SetSubtitle,
    MusicBrainzReleaseId,
    OriginalAlbumTitle,
    AlbumTitleSortOrder,
    AlbumArtist,
    MusicBrainzReleaseArtistId,
    ContentGroup,
    MusicBrainzReleaseGroupId,

    // artist
    TrackArtist,
    /// can occur multiple times
    TrackArtists,
    MusicBrainzArtistId,
    OriginalArtist,
    AlbumArtistSortOrder,
    TrackArtistSortOrder,

    // show
    ShowName,
    ShowNameSortOrder,

    // style
    Genre,
    InitialKey,
    Color,
    Mood,
    Bpm,

    // urls
    AudioFileUrl,
    AudioSourceUrl,
    CommercialInformationUrl,
    CopyrightUrl,
    TrackArtistUrl,
    RadioStationUrl,
    PaymentUrl,
    PublisherUrl,

    // numbering
    DiscNumber,
    DiscTotal,
    TrackNumber,
    TrackTotal,
    Movement,
    MovementNumber,
    MovementTotal,

    // dates
    Year,
    RecordingDate,
    ReleaseDate,
    OriginalReleaseDate,

    // file
    FileType,
    FileOwner,
    TaggingTime,
    Length,
    OriginalFileName,
    OriginalMediaType,

    // encoding
    EncodedBy,
    EncoderSoftware,
    EncoderSettings,
    EncodingTime,

    // replaygain
    ReplayGainAlbumGain,
    ReplayGainAlbumPeak,
    ReplayGainTrackGain,
    ReplayGainTrackPeak,

    // identification
    Isrc,
    Barcode,
    CatalogNumber,
    Work,

    // flags
    FlagCompilation,
    FlagPodcast,

    // legal
    CopyrightMessage,
    License,

    // misc
    Popularimeter,
    ParentalAdvisory,

    // people
    Arranger,
    Writer,
    Composer,
    ComposerSortOrder,
    Conductor,
    Director,
    Engineer,
    Lyricist,
    OriginalLyricist,
    MixDj,
    MixEngineer,
    MusicianCredits,
    Performer,
    Producer,
    Publisher,
    Label,
    InternetRadioStationName,
    InternetRadioStationOwner,
    Remixer,

    // podcast
    PodcastDescription,
    PodcastSeriesCategory,
    PodcastUrl,
    PodcastGlobalUniqueId,
    PodcastKeywords,

    Custom(String),
}

/*
#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct DBTrack {
    id: TrackId,

    track_group_id: TrackGroupId,

    plugin_id: PluginId,
    plugin_data: PluginTrackIdentifier,

    artist_id: Option<i32>,
    album_id: Option<i32>,

    // general
    title: Option<String>,
    mb_work_id: Option<String>,
    mb_track_id: Option<String>,
    mb_recording_id: Option<String>,
    subtitle: Option<String>,
    title_sort_order: Option<String>,
    comment: Option<String>,
    description: Option<String>,
    language: Option<String>,
    script: Option<String>,
    lyrics: Option<String>,

    // album
    album_title: Option<String>,
    set_subtitle: Option<String>,
    mb_release_id: Option<String>,
    original_album_title: Option<String>,
    album_title_sort_order: Option<String>,
    album_artist: Option<String>,
    mb_release_artist_id: Option<String>,
    album_artist_sort_order: Option<String>,
    content_group: Option<String>,
    mb_release_group_id: Option<String>,

    // artist
    artist: Option<String>,
    artists: Option<String>,
    mb_artist_id: Option<String>,
    original_artist: Option<String>,
    artist_sort_order: Option<String>,

    // show
    show_name: Option<String>,
    show_name_sort_order: Option<String>,

    // style
    genre: Option<String>,
    initial_key: Option<String>,
    color: Option<String>,
    mood: Option<String>,
    bpm: Option<f32>,

    // urls
    audio_file_url: Option<String>,
    audio_source_url: Option<String>,
    commercial_information_url: Option<String>,
    copyright_url: Option<String>,
    track_artist_url: Option<String>,
    radio_station_url: Option<String>,
    payment_url: Option<String>,
    publisher_url: Option<String>,

    // numbering
    disc_number: Option<String>,
    disc_total: Option<String>,
    track_number: Option<String>,
    track_total: Option<String>,
    movement: Option<String>,
    movement_number: Option<String>,
    movement_total: Option<String>,

    // dates
    year: Option<String>,
    recording_date: Option<String>,
    release_date: Option<String>,
    original_release_date: Option<String>,

    // file
    file_type: Option<String>,
    file_owner: Option<String>,
    tagging_time: Option<String>,
    length: Option<String>,
    original_file_name: Option<String>,
    original_media_type: Option<String>,

    // encoding
    encoded_by: Option<String>,
    encoder_software: Option<String>,
    encoding_settings: Option<String>,
    encoding_time: Option<String>,

    // replaygain
    replaygain_album_gain: Option<String>,
    replaygain_album_peak: Option<String>,
    replaygain_track_gain: Option<String>,
    replaygain_track_peak: Option<String>,

    // identification
    irsc: Option<String>,
    barcode: Option<String>,
    catalog_number: Option<String>,

    // flags
    flag_compilation: Option<String>,
    flag_podcast: Option<String>,

    // legal
    copyright_message: Option<String>,
    license: Option<String>,

    // misc
    popularimeter: Option<String>,
    parental_advisory: Option<String>,

    // people
    arranger: Option<String>,
    writer: Option<String>,
    composer: Option<String>,
    composer_sort_order: Option<String>,
    conductor: Option<String>,
    director: Option<String>,
    engineer: Option<String>,
    lyricist: Option<String>,
    original_lyricist: Option<String>,
    mix_dj: Option<String>,
    mix_engineer: Option<String>,
    musician_credits: Option<String>,
    performer: Option<String>,
    producer: Option<String>,
    publisher: Option<String>,
    label: Option<String>,
    internet_radio_station_name: Option<String>,
    internet_radio_station_owner: Option<String>,
    remixer: Option<String>,
}
*/
