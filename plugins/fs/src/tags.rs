use hogehoge_types::{Tags, Uuid};
use lofty::tag::{ItemKey, ItemValue};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TagsError {
    #[error("Missing title in tags")]
    MissingTitle,
}

pub fn map_lofty_to_internal(input: &lofty::tag::Tag) -> Result<Tags, TagsError> {
    let title = input
        .get_string(&ItemKey::TrackTitle)
        .ok_or(TagsError::MissingTitle)?; //TODO: try parsing title from filename (provide host fn?)

    let mut tags = Tags::new(title.to_string());

    for tag in input.items() {
        let value = match tag.value() {
            ItemValue::Text(text) => text.clone(),
            ItemValue::Locator(locator) => locator.clone(),
            ItemValue::Binary(_) => {
                continue;
            }
        };

        add_lofty_tag(tag.key(), value, &mut tags)
    }

    Ok(tags)
}

//TODO: log error
pub fn parse_uuid(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}

pub fn add_lofty_tag(key: &lofty::tag::ItemKey, value: String, tags: &mut Tags) {
    macro_rules! map_key {
        ($($tag:ident => $lofty_tag:ident,)*) => {
            match key {
                ItemKey::MusicBrainzWorkId => tags.musicbrainz_work_id = parse_uuid(&value),
                ItemKey::MusicBrainzTrackId => tags.musicbrainz_track_id = parse_uuid(&value),
                ItemKey::MusicBrainzRecordingId => tags.musicbrainz_recording_id = parse_uuid(&value),
                ItemKey::MusicBrainzReleaseId => tags.musicbrainz_release_id = parse_uuid(&value),
                ItemKey::MusicBrainzReleaseArtistId => tags.musicbrainz_release_artist_id = parse_uuid(&value),
                ItemKey::MusicBrainzReleaseGroupId => tags.musicbrainz_release_group_id = parse_uuid(&value),

                ItemKey::Bpm => tags.bpm = value.parse().ok(),

                $(ItemKey::$lofty_tag => tags.$tag = Some(value),)*
                _ => {
                    // For now just ignore it.
                }
            }
        }
    }

    map_key!(
        // track_title => TrackTitle,
        // musicbrainz_work_id => MusicBrainzWorkId,
        // musicbrainz_track_id => MusicBrainzTrackId,
        // musicbrainz_recording_id => MusicBrainzRecordingId,
        track_subtitle => TrackSubtitle,
        track_title_sort_order => TrackTitleSortOrder,
        comment => Comment,
        description => Description,
        language => Language,
        script => Script,
        lyrics => Lyrics,


        album_title => AlbumTitle,
        set_subtitle => SetSubtitle,
        // musicbrainz_release_id => MusicBrainzReleaseId,
        original_album_title => OriginalAlbumTitle,
        album_title_sort_order => AlbumTitleSortOrder,
        album_artist => AlbumArtist,
        // musicbrainz_release_artist_id => MusicBrainzReleaseArtistId,
        content_group => ContentGroup,
        // musicbrainz_release_group_id => MusicBrainzReleaseGroupId,

        track_artist => TrackArtist,
        track_artists => TrackArtists,
        // musicbrainz_artist_id => MusicBrainzArtistId,
        original_artist => OriginalArtist,
        album_artist_sort_order => AlbumArtistSortOrder,
        track_artist_sort_order => TrackArtistSortOrder,

        show_name => ShowName,
        show_name_sort_order => ShowNameSortOrder,

        genre => Genre,
        initial_key => InitialKey,
        color => Color,
        mood => Mood,
        // bpm => Bpm,

        audio_file_url => AudioFileUrl,
        audio_source_url => AudioSourceUrl,
        commercial_information_url => CommercialInformationUrl,
        copyright_url => CopyrightUrl,
        track_artist_url => TrackArtistUrl,
        radio_station_url => RadioStationUrl,
        payment_url => PaymentUrl,
        publisher_url => PublisherUrl,

        disc_number => DiscNumber,
        disc_total => DiscTotal,
        track_number => TrackNumber,
        track_total => TrackTotal,
        movement => Movement,
        movement_number => MovementNumber,
        movement_total => MovementTotal,

        year => Year,
        recording_date => RecordingDate,
        release_date => ReleaseDate,
        original_release_date => OriginalReleaseDate,

        file_type => FileType,
        file_owner => FileOwner,
        tagging_time => TaggingTime,
        length => Length,
        original_file_name => OriginalFileName,
        original_media_type => OriginalMediaType,

        encoded_by => EncodedBy,
        encoder_software => EncoderSoftware,
        encoder_settings => EncoderSettings,
        encoding_time => EncodingTime,

        replay_gain_album_gain => ReplayGainAlbumGain,
        replay_gain_album_peak => ReplayGainAlbumPeak,
        replay_gain_track_gain => ReplayGainTrackGain,
        replay_gain_track_peak => ReplayGainTrackPeak,

        isrc => Isrc,
        barcode => Barcode,
        catalog_number => CatalogNumber,
        work => Work,

        flag_compilation => FlagCompilation,
        flag_podcast => FlagPodcast,

        copyright_message => CopyrightMessage,
        license => License,

        popularimeter => Popularimeter,
        parental_advisory => ParentalAdvisory,

        arranger => Arranger,
        writer => Writer,
        composer => Composer,
        composer_sort_order => ComposerSortOrder,
        conductor => Conductor,
        director => Director,
        engineer => Engineer,
        lyricist => Lyricist,
        original_lyricist => OriginalLyricist,
        mix_dj => MixDj,
        mix_engineer => MixEngineer,
        musician_credits => MusicianCredits,
        performer => Performer,
        producer => Producer,
        publisher => Publisher,
        label => Label,
        internet_radio_station_name => InternetRadioStationName,
        internet_radio_station_owner => InternetRadioStationOwner,
        remixer => Remixer,

        podcast_description => PodcastDescription,
        podcast_series_category => PodcastSeriesCategory,
        podcast_url => PodcastUrl,
        podcast_global_unique_id => PodcastGlobalUniqueId,
        podcast_keywords => PodcastKeywords,
    )
}
