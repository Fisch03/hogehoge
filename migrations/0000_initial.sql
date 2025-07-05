CREATE TABLE plugins(
    plugin_id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE
);

CREATE TABLE artists(
	artist_id INTEGER NOT NULL PRIMARY KEY,
	mbid TEXT,
	name TEXT NOT NULL
);

CREATE TABLE albums(
	album_id INTEGER NOT NULL PRIMARY KEY,
	mbid TEXT,
	title TEXT NOT NULL,
	artist_id INTEGER,
	FOREIGN KEY (artist_id) REFERENCES artists(artist_id)
);

CREATE TABLE track_groups(
    track_group_id INTEGER NOT NULL PRIMARY KEY
);

CREATE TABLE tracks(
    track_id INTEGER NOT NULL PRIMARY KEY,
    
    track_group_id INTEGER NOT NULL,

    plugin_id INTEGER NOT NULL,
    plugin_data TEXT,

    artist_id INTEGER,
    album_artist_id INTEGER,
    album_id INTEGER,

    -- general
    track_title TEXT,
    music_brainz_work_id TEXT,
    music_brainz_track_id TEXT,
    music_brainz_recording_id TEXT,
    subtitle TEXT,
    track_title_sort_order TEXT,

    comment TEXT,
    description TEXT,
    language TEXT,
    script TEXT,
    lyrics TEXT,

    -- album
    album_title TEXT,
    set_subtitle TEXT,
    music_brainz_release_group_id TEXT,
    music_brainz_release_id TEXT,
    original_album_title TEXT,
    album_title_sort_order TEXT,

    music_brainz_release_artist_id TEXT,
    album_artist_sort_order TEXT,

    content_group TEXT,

    -- artist
    -- track_artist TEXT,
    track_artists TEXT,
    music_brainz_artist_id TEXT,
    original_artist TEXT,
    track_artist_sort_order TEXT,

    -- show
    show_name TEXT,
    show_name_sort_order TEXT,

    -- style
    genre TEXT,
    initial_key TEXT,
    color TEXT,
    mood TEXT,
    bpm REAL,

    -- urls
    audio_file_url TEXT,
    audio_source_url TEXT,
    commercial_information_url TEXT,
    copyright_url TEXT,
    track_artist_url TEXT,
    radio_station_url TEXT,
    payment_url TEXT,
    publisher_url TEXT,

    -- numbering
    disc_number TEXT,
    disc_total TEXT,
    track_number TEXT,
    track_total TEXT,
    movement TEXT,
    movement_number TEXT,
    movement_total TEXT,

    -- dates
    year TEXT,
    recording_date TEXT,
    release_date TEXT,
    original_release_date TEXT,

    -- file
    file_type TEXT,
    file_owner TEXT,
    tagging_time TEXT,
    length TEXT,
    original_file_name TEXT,
    original_media_type TEXT,

    -- encoding
    encoded_by TEXT,
    encoder_software TEXT,
    encoder_settings TEXT,
    encoding_time TEXT,

    -- replay_gain
    replay_gain_album_gain TEXT,
    replay_gain_album_peak TEXT,
    replay_gain_track_gain TEXT,
    replay_gain_track_peak TEXT,

    -- identification
    isrc TEXT,
    barcode TEXT,
    catalog_number TEXT,
    work TEXT,

    -- flags
    flag_compilation TEXT,
    flag_podcast TEXT,

    -- legal
    copyright_message TEXT,
    license TEXT,

    -- misc
    popularimeter TEXT,
    parental_advisory TEXT,

    -- people
    arranger TEXT,

    writer TEXT,

    composer TEXT,
    composer_sort_order TEXT,

    conductor TEXT,

    director TEXT,

    engineer TEXT,

    lyricist TEXT,
    original_lyricist TEXT,

    mix_dj TEXT,

    mix_engineer TEXT,

    musician_credits TEXT,

    performer TEXT,

    producer TEXT,

    publisher TEXT,

    label TEXT,

    internet_radio_station_name TEXT,
    internet_radio_station_owner TEXT,

    remixer TEXT,

    FOREIGN KEY (track_group_id) REFERENCES track_groups(track_group_id),

    FOREIGN KEY (plugin_id) REFERENCES plugins(plugin_id),
    UNIQUE (plugin_id, plugin_data),

    FOREIGN KEY (artist_id) REFERENCES artists(artist_id),
    FOREIGN KEY (album_id) REFERENCES albums(album_id)
);


