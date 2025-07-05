use lofty::tag::{ItemKey, ItemValue, Tag, TagItem};
use museek_macros::InsertRow;
use sqlx::{Database, FromRow, SqlitePool};
use std::path::Path;
use tracing::warn;

pub struct TrackId(i64);

#[derive(Debug, Default, FromRow, InsertRow)]
#[table("tracks")]
pub struct Track {
    // id: i32,
    path: String,

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

impl Track {
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn from_tags(path: &Path, tags: &Tag) -> Self {
        let mut track = Track {
            path: path.to_string_lossy().to_string(),
            ..Default::default()
        };

        for tag in tags.items() {
            track.set_tag_field(tag);
        }

        track
    }

    // pub async fn insert(&self, pool: &SqlitePool) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
    //     sqlx::query!("INSERT INTO tracks (path) VALUES ($1)", self.path)
    //         .execute(pool)
    //         .await
    // }

    pub fn set_tag_field(&mut self, tag: &TagItem) {
        enum T<'a> {
            String(&'a mut Option<String>),
            Float(&'a mut Option<f32>),
        }

        let value = tag.value();

        let field = match tag.key() {
            // general
            ItemKey::TrackTitle => T::String(&mut self.title),
            ItemKey::MusicBrainzWorkId => T::String(&mut self.mb_work_id),
            ItemKey::MusicBrainzTrackId => T::String(&mut self.mb_track_id),
            ItemKey::MusicBrainzRecordingId => T::String(&mut self.mb_recording_id),
            ItemKey::TrackSubtitle => T::String(&mut self.subtitle),
            ItemKey::TrackTitleSortOrder => T::String(&mut self.title_sort_order),

            ItemKey::Comment => T::String(&mut self.comment),
            ItemKey::Description => T::String(&mut self.description),
            ItemKey::Language => T::String(&mut self.language),
            ItemKey::Script => T::String(&mut self.script),
            ItemKey::Lyrics => T::String(&mut self.lyrics),

            // album
            ItemKey::AlbumTitle => T::String(&mut self.album_title),
            ItemKey::SetSubtitle => T::String(&mut self.set_subtitle),
            ItemKey::MusicBrainzReleaseId => T::String(&mut self.mb_release_id),
            ItemKey::OriginalAlbumTitle => T::String(&mut self.original_album_title),
            ItemKey::AlbumTitleSortOrder => T::String(&mut self.album_title_sort_order),

            ItemKey::AlbumArtist => T::String(&mut self.album_artist),
            ItemKey::MusicBrainzReleaseArtistId => T::String(&mut self.mb_release_artist_id),
            ItemKey::AlbumArtistSortOrder => T::String(&mut self.album_artist_sort_order),

            ItemKey::ContentGroup => T::String(&mut self.content_group),
            ItemKey::MusicBrainzReleaseGroupId => T::String(&mut self.mb_release_group_id),

            // artist
            ItemKey::TrackArtist => T::String(&mut self.artist),
            ItemKey::TrackArtists => T::String(&mut self.artists),
            ItemKey::MusicBrainzArtistId => T::String(&mut self.mb_artist_id),
            ItemKey::OriginalArtist => T::String(&mut self.original_artist),
            ItemKey::TrackArtistSortOrder => T::String(&mut self.artist_sort_order),

            // show
            ItemKey::ShowName => T::String(&mut self.show_name),
            ItemKey::ShowNameSortOrder => T::String(&mut self.show_name_sort_order),

            // style
            ItemKey::Genre => T::String(&mut self.genre),
            ItemKey::InitialKey => T::String(&mut self.initial_key),
            ItemKey::Color => T::String(&mut self.color),
            ItemKey::Mood => T::String(&mut self.mood),
            ItemKey::Bpm | ItemKey::IntegerBpm => T::Float(&mut self.bpm),

            // urls
            ItemKey::AudioFileUrl => T::String(&mut self.audio_file_url),
            ItemKey::AudioSourceUrl => T::String(&mut self.audio_source_url),
            ItemKey::CommercialInformationUrl => T::String(&mut self.commercial_information_url),
            ItemKey::CopyrightUrl => T::String(&mut self.copyright_url),
            ItemKey::TrackArtistUrl => T::String(&mut self.track_artist_url),
            ItemKey::RadioStationUrl => T::String(&mut self.radio_station_url),
            ItemKey::PaymentUrl => T::String(&mut self.payment_url),
            ItemKey::PublisherUrl => T::String(&mut self.publisher_url),

            // numbering
            ItemKey::DiscNumber => T::String(&mut self.disc_number),
            ItemKey::DiscTotal => T::String(&mut self.disc_total),
            ItemKey::TrackNumber => T::String(&mut self.track_number),
            ItemKey::TrackTotal => T::String(&mut self.track_total),
            ItemKey::Movement => T::String(&mut self.movement),
            ItemKey::MovementNumber => T::String(&mut self.movement_number),
            ItemKey::MovementTotal => T::String(&mut self.movement_total),

            // dates
            ItemKey::Year => T::String(&mut self.year),
            ItemKey::RecordingDate => T::String(&mut self.recording_date),
            ItemKey::ReleaseDate => T::String(&mut self.release_date),
            ItemKey::OriginalReleaseDate => T::String(&mut self.original_release_date),

            // file
            ItemKey::FileType => T::String(&mut self.file_type),
            ItemKey::FileOwner => T::String(&mut self.file_owner),
            ItemKey::TaggingTime => T::String(&mut self.tagging_time),
            ItemKey::Length => T::String(&mut self.length),
            ItemKey::OriginalFileName => T::String(&mut self.original_file_name),
            ItemKey::OriginalMediaType => T::String(&mut self.original_media_type),

            // encoding
            ItemKey::EncodedBy => T::String(&mut self.encoded_by),
            ItemKey::EncoderSoftware => T::String(&mut self.encoder_software),
            ItemKey::EncoderSettings => T::String(&mut self.encoding_settings),
            ItemKey::EncodingTime => T::String(&mut self.encoding_time),

            // replaygain
            ItemKey::ReplayGainAlbumGain => T::String(&mut self.replaygain_album_gain),
            ItemKey::ReplayGainAlbumPeak => T::String(&mut self.replaygain_album_peak),
            ItemKey::ReplayGainTrackGain => T::String(&mut self.replaygain_track_gain),
            ItemKey::ReplayGainTrackPeak => T::String(&mut self.replaygain_track_peak),

            // identification
            ItemKey::Isrc => T::String(&mut self.irsc),
            ItemKey::Barcode => T::String(&mut self.barcode),
            ItemKey::CatalogNumber => T::String(&mut self.catalog_number),

            // flags
            ItemKey::FlagCompilation => T::String(&mut self.flag_compilation),
            ItemKey::FlagPodcast => T::String(&mut self.flag_podcast),

            // legal
            ItemKey::CopyrightMessage => T::String(&mut self.copyright_message),
            ItemKey::License => T::String(&mut self.license),

            // misc
            ItemKey::Popularimeter => T::String(&mut self.popularimeter),
            ItemKey::ParentalAdvisory => T::String(&mut self.parental_advisory),

            // people
            ItemKey::Arranger => T::String(&mut self.arranger),
            ItemKey::Writer => T::String(&mut self.writer),
            ItemKey::Composer => T::String(&mut self.composer),
            ItemKey::ComposerSortOrder => T::String(&mut self.composer_sort_order),
            ItemKey::Conductor => T::String(&mut self.conductor),
            ItemKey::Director => T::String(&mut self.director),
            ItemKey::Engineer => T::String(&mut self.engineer),
            ItemKey::Lyricist => T::String(&mut self.lyricist),
            ItemKey::OriginalLyricist => T::String(&mut self.original_lyricist),
            ItemKey::MixDj => T::String(&mut self.mix_dj),
            ItemKey::MixEngineer => T::String(&mut self.mix_engineer),
            ItemKey::MusicianCredits => T::String(&mut self.musician_credits),
            ItemKey::Performer => T::String(&mut self.performer),
            ItemKey::Producer => T::String(&mut self.producer),
            ItemKey::Publisher => T::String(&mut self.publisher),
            ItemKey::Label => T::String(&mut self.label),
            ItemKey::InternetRadioStationName => T::String(&mut self.internet_radio_station_name),
            ItemKey::InternetRadioStationOwner => T::String(&mut self.internet_radio_station_owner),
            ItemKey::Remixer => T::String(&mut self.remixer),

            ItemKey::Unknown(_) => return,

            _ => {
                warn!("unhandled tag: {:?}", tag.key());
                return;
            }
        };

        match (value, field) {
            (ItemValue::Text(value), T::String(field)) => *field = Some(value.to_string()),
            (ItemValue::Text(value), T::Float(field)) => match value.parse() {
                Ok(value) => *field = Some(value),
                Err(_) => {
                    warn!(
                        "tag {:?} in file {:?} contains invalid float: {}",
                        tag.key(),
                        self.path,
                        value
                    );
                }
            },
            (ItemValue::Locator(value), T::String(field)) => *field = Some(value.to_string()),
            (ItemValue::Binary(value), _) => {
                todo!("binary tag value: {:?} on field {:?}", value, tag.key())
            }
            _ => {
                warn!(
                    "tag type does not match field type: {:?} with value {:?}",
                    tag.key(),
                    tag.value()
                );
            }
        }
    }
}
