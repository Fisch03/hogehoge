use hogehoge_types::{TagKey, Tags};

pub fn map_lofty_to_internal(input: &lofty::tag::Tag) -> Tags {
    let mut tags = Tags::new();

    for tag in input.items() {
        let Some(key) = map_lofty_key_to_internal(&tag.key()) else {
            continue;
        };

        let value = match tag.value() {
            lofty::tag::ItemValue::Text(text) => text.clone(),
            lofty::tag::ItemValue::Locator(locator) => locator.clone(),
            lofty::tag::ItemValue::Binary(_) => {
                continue;
            }
        };

        tags.insert(key, value);
    }

    tags
}

pub fn map_lofty_key_to_internal(key: &lofty::tag::ItemKey) -> Option<TagKey> {
    use lofty::tag::ItemKey;

    macro_rules! map_key {
        ($($keys:ident,)*) => {
            match key {
                $(ItemKey::$keys => Some(TagKey::$keys),)*
                _ => None,
            }
        }
    }

    return map_key!(
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
        AlbumTitle,
        SetSubtitle,
        MusicBrainzReleaseId,
        OriginalAlbumTitle,
        AlbumTitleSortOrder,
        AlbumArtist,
        MusicBrainzReleaseArtistId,
        ContentGroup,
        MusicBrainzReleaseGroupId,
        TrackArtist,
        TrackArtists,
        MusicBrainzArtistId,
        OriginalArtist,
        AlbumArtistSortOrder,
        TrackArtistSortOrder,
        ShowName,
        ShowNameSortOrder,
        Genre,
        InitialKey,
        Color,
        Mood,
        Bpm,
        AudioFileUrl,
        AudioSourceUrl,
        CommercialInformationUrl,
        CopyrightUrl,
        TrackArtistUrl,
        RadioStationUrl,
        PaymentUrl,
        PublisherUrl,
        DiscNumber,
        DiscTotal,
        TrackNumber,
        TrackTotal,
        Movement,
        MovementNumber,
        MovementTotal,
        Year,
        RecordingDate,
        ReleaseDate,
        OriginalReleaseDate,
        FileType,
        FileOwner,
        TaggingTime,
        Length,
        OriginalFileName,
        OriginalMediaType,
        EncodedBy,
        EncoderSoftware,
        EncoderSettings,
        EncodingTime,
        ReplayGainAlbumGain,
        ReplayGainAlbumPeak,
        ReplayGainTrackGain,
        ReplayGainTrackPeak,
        Isrc,
        Barcode,
        CatalogNumber,
        Work,
        FlagCompilation,
        FlagPodcast,
        CopyrightMessage,
        License,
        Popularimeter,
        ParentalAdvisory,
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
        PodcastDescription,
        PodcastSeriesCategory,
        PodcastUrl,
        PodcastGlobalUniqueId,
        PodcastKeywords,
    );
}
