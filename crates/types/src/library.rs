use crate::plugin::UniqueTrackIdentifier;
use extism_convert::{FromBytes, Msgpack, ToBytes};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct ArtistId(pub i64);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct Artist {
    id: ArtistId,
    mbid: Option<String>,

    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct AlbumId(pub i64);

#[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
#[encoding(Msgpack)]
pub struct Album {
    title: String,

    mbid: Option<String>,
    artist_id: Option<ArtistId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct TrackId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ToBytes, FromBytes, Serialize, Deserialize)]
#[cfg_attr(feature = "internal", derive(sqlx::Type))]
#[cfg_attr(feature = "internal", sqlx(transparent))]
#[encoding(Msgpack)]
pub struct TrackGroupId(pub i64);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "internal", derive(sqlx::FromRow))]
pub struct Track {
    pub track_group_id: TrackGroupId,

    pub artist_id: Option<ArtistId>,
    pub album_artist_id: Option<ArtistId>,
    pub album_id: Option<AlbumId>,

    #[cfg_attr(feature = "internal", sqlx(flatten))]
    pub identifier: UniqueTrackIdentifier,
    #[cfg_attr(feature = "internal", sqlx(flatten))]
    pub tags: Tags,
}

impl std::ops::Deref for Track {
    type Target = Tags;
    fn deref(&self) -> &Self::Target {
        &self.tags
    }
}
impl std::ops::DerefMut for Track {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tags
    }
}

#[cfg(feature = "internal")]
macro_rules! insert_track {
    ($($field:ident => $field_camel:ident => $type:ty,)*) => {
        impl Track {
            pub async fn upsert_into(self, pool: &sqlx::sqlite::SqlitePool) -> sqlx::Result<TrackId> {
                use sqlx::Arguments;
                use std::sync::LazyLock;

                const FIELDS: &[&str] = &[
                    "track_group_id",
                    "artist_id",
                    "album_artist_id",
                    "album_id",
                    "plugin_id",
                    "plugin_data",
                    $(stringify!($field),)*
                ];

                // this kinda sucks but i really dont wanna write the full query out in case i
                // change the fields later
                static QUERY: LazyLock<String> = LazyLock::new(|| {
                    format!(
                        "INSERT INTO tracks ({}) VALUES ({}) ON CONFLICT (plugin_id, plugin_data) DO UPDATE SET {}",
                        FIELDS.join(", "),
                        vec!["?"; FIELDS.len()].join(", "),
                        FIELDS.iter().map(|f| format!("{} = EXCLUDED.{}", f, f)).collect::<Vec<_>>().join(", ")
                    )
                });

                // tracing::info!("{}", QUERY.as_str());

                let mut arguments = sqlx::sqlite::SqliteArguments::default();
                arguments.add(self.track_group_id).unwrap();
                arguments.add(self.artist_id).unwrap();
                arguments.add(self.album_artist_id).unwrap();
                arguments.add(self.album_id).unwrap();

                arguments.add(self.identifier.plugin_id).unwrap();
                arguments.add(self.identifier.plugin_data.clone()).unwrap();
                $(
                    arguments.add(self.$field.clone()).unwrap();
                )*

                let result = sqlx::query_with(&*QUERY, arguments)
                    .execute(pool)
                    .await?;

                Ok(TrackId(
                    result.last_insert_rowid()
                ))
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagValue<'a>(pub Option<TagValueKind<'a>>);

#[derive(Debug, Clone, PartialEq)]
pub enum TagValueKind<'a> {
    String(&'a str),
    Uuid(Uuid),
    Float(f32),
}

impl std::fmt::Display for TagValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(TagValueKind::String(s)) => write!(f, "{}", s),
            Some(TagValueKind::Uuid(u)) => write!(f, "{}", u),
            Some(TagValueKind::Float(fl)) => write!(f, "{}", fl),
            None => Ok(()),
        }
    }
}

impl<'a> From<&'a String> for TagValue<'a> {
    fn from(value: &'a String) -> Self {
        Self(Some(TagValueKind::String(value)))
    }
}

impl<'a> From<&'a Option<String>> for TagValue<'a> {
    fn from(value: &'a Option<String>) -> Self {
        Self(value.as_deref().map(TagValueKind::String))
    }
}

impl<'a> From<&'a Option<Uuid>> for TagValue<'a> {
    fn from(value: &'a Option<Uuid>) -> Self {
        Self(value.map(TagValueKind::Uuid))
    }
}

impl<'a> From<&'a Option<f32>> for TagValue<'a> {
    fn from(value: &'a Option<f32>) -> Self {
        Self(value.map(TagValueKind::Float))
    }
}

pub trait ExtractTag {
    type Type: 'static;

    fn extract(tags: &Tags) -> &Self::Type;

    fn extract_value<'a>(tags: &'a Tags) -> TagValue<'a>
    where
        TagValue<'a>: From<&'a Self::Type>,
    {
        TagValue::from(Self::extract(tags))
    }
}

macro_rules! make_tags {
    ($($field: ident => $field_camel: ident => $type:ty,)*) => {
        #[derive(Debug, Clone, ToBytes, FromBytes, Serialize, Deserialize)]
        #[cfg_attr(feature = "internal", derive(sqlx::FromRow))]
        #[encoding(Msgpack)]
        pub struct Tags {
            $(pub $field: $type,)*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum TagKind {
            $($field_camel,)*
        }

        pub mod tag {
            use super::ExtractTag;

            $(
            pub struct $field_camel;
            impl ExtractTag for $field_camel {
                type Type = $type;

                fn extract(tags: &super::Tags) -> &Self::Type {
                    &tags.$field
                }
            }
            )*
        }

        impl Tags {
            pub fn new(title: String) -> Self {
                let mut empty = Self {
                    $($field: Default::default(),)*
                };

                empty.track_title = title;

                empty
            }

            pub fn extract<T: ExtractTag>(&self) -> &T::Type {
                T::extract(self)
            }

            pub fn get<'a>(&'a self, kind: TagKind) -> TagValue<'a> {
                match kind {
                    $(
                        TagKind::$field_camel => TagValue::from(self.extract::<tag::$field_camel>()),
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! with_all_tags {
    ($macro_name:ident) => {
        $macro_name! {
            // general
            track_title => TrackTitle => String,
            musicbrainz_work_id => MusicbrainzWorkId => Option<uuid::Uuid>,
            musicbrainz_track_id => MusicbrainzTrackId => Option<uuid::Uuid>,
            musicbrainz_recording_id => MusicbrainzRecordingId => Option<uuid::Uuid>,
            track_subtitle => TrackSubtitle => Option<String>,
            track_title_sort_order => TrackTitleSortOrder => Option<String>,
            comment => Comment => Option<String>,
            description => Description => Option<String>,
            language => Language => Option<String>,
            script => Script => Option<String>,
            lyrics => Lyrics => Option<String>,

            // album
            album_title => AlbumTitle => Option<String>,
            set_subtitle => SetSubtitle => Option<String>,
            musicbrainz_release_id => MusicbrainzReleaseId => Option<uuid::Uuid>,
            original_album_title => OriginalAlbumTitle => Option<String>,
            album_title_sort_order => AlbumTitleSortOrder => Option<String>,
            album_artist => AlbumArtist => Option<String>,
            musicbrainz_release_artist_id => MusicbrainzReleaseArtistId => Option<uuid::Uuid>,
            content_group => ContentGroup => Option<String>,
            musicbrainz_release_group_id => MusicbrainzReleaseGroupId => Option<uuid::Uuid>,

            // artist
            track_artist => TrackArtist => Option<String>,
            track_artists => TrackArtists => Option<String>,
            musicbrainz_artist_id => MusicbrainzArtistId => Option<uuid::Uuid>,
            original_artist => OriginalArtist => Option<String>,
            album_artist_sort_order => AlbumArtistSortOrder => Option<String>,
            track_artist_sort_order => TrackArtistSortOrder => Option<String>,

            // show
            show_name => ShowName => Option<String>,
            show_name_sort_order => ShowNameSortOrder => Option<String>,

            // style
            genre => Genre => Option<String>,
            initial_key => InitialKey => Option<String>,
            color => Color => Option<String>,
            mood => Mood => Option<String>,
            bpm => Bpm => Option<f32>,

            // urls
            audio_file_url => AudioFileUrl => Option<String>,
            audio_source_url => AudioSourceUrl => Option<String>,
            commercial_information_url => CommercialInformationUrl => Option<String>,
            copyright_url => CopyrightUrl => Option<String>,
            track_artist_url => TrackArtistUrl => Option<String>,
            radio_station_url => RadioStationUrl => Option<String>,
            payment_url => PaymentUrl => Option<String>,
            publisher_url => PublisherUrl => Option<String>,

            // numbering
            disc_number => DiscNumber => Option<String>,
            disc_total => DiscTotal => Option<String>,
            track_number => TrackNumber => Option<String>,
            track_total => TrackTotal => Option<String>,
            movement => Movement => Option<String>,
            movement_number => MovementNumber => Option<String>,
            movement_total => MovementTotal => Option<String>,

            // dates
            year => Year => Option<String>,
            recording_date => RecordingDate => Option<String>,
            release_date => ReleaseDate => Option<String>,
            original_release_date => OriginalReleaseDate => Option<String>,

            // file
            file_type => FileType => Option<String>,
            file_owner => FileOwner => Option<String>,
            tagging_time => TaggingTime => Option<String>,
            length => Length => Option<String>,
            original_file_name => OriginalFileName => Option<String>,
            original_media_type => OriginalMediaType => Option<String>,

            // encoding
            encoded_by => EncodedBy => Option<String>,
            encoder_software => EncoderSoftware => Option<String>,
            encoder_settings => EncoderSettings => Option<String>,
            encoding_time => EncodingTime => Option<String>,

            // replaygain
            replay_gain_album_gain => ReplayGainAlbumGain => Option<String>,
            replay_gain_album_peak => ReplayGainAlbumPeak => Option<String>,
            replay_gain_track_gain => ReplayGainTrackGain => Option<String>,
            replay_gain_track_peak => ReplayGainTrackPeak => Option<String>,

            // identification
            isrc => Isrc => Option<String>,
            barcode => Barcode => Option<String>,
            catalog_number => CatalogNumber => Option<String>,
            work => Work => Option<String>,

            // flags
            flag_compilation => FlagCompilation => Option<String>,
            flag_podcast => FlagPodcast => Option<String>,

            // legal
            copyright_message => CopyrightMessage => Option<String>,
            license => License => Option<String>,

            // misc
            popularimeter => Popularimeter => Option<String>,
            parental_advisory => ParentalAdvisory => Option<String>,

            // people
            arranger => Arranger => Option<String>,
            writer => Writer => Option<String>,
            composer => Composer => Option<String>,
            composer_sort_order => ComposerSortOrder => Option<String>,
            conductor => Conductor => Option<String>,
            director => Director => Option<String>,
            engineer => Engineer => Option<String>,
            lyricist => Lyricist => Option<String>,
            original_lyricist => OriginalLyricist => Option<String>,
            mix_dj => MixDj => Option<String>,
            mix_engineer => MixEngineer => Option<String>,
            musician_credits => MusicianCredits => Option<String>,
            performer => Performer => Option<String>,
            producer => Producer => Option<String>,
            publisher => Publisher => Option<String>,
            label => Label => Option<String>,
            internet_radio_station_name => InternetRadioStationName => Option<String>,
            internet_radio_station_owner => InternetRadioStationOwner => Option<String>,
            remixer => Remixer => Option<String>,

            // podcast
            podcast_description => PodcastDescription => Option<String>,
            podcast_series_category => PodcastSeriesCategory => Option<String>,
            podcast_url => PodcastUrl => Option<String>,
            podcast_global_unique_id => PodcastGlobalUniqueId => Option<String>,
            podcast_keywords => PodcastKeywords => Option<String>,
        }
    };
}

with_all_tags!(make_tags);
#[cfg(feature = "internal")]
with_all_tags!(insert_track);
