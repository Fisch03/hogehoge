use hogehoge_types::{
    AlbumId, ArtistId, TrackGroupId, TrackId,
    library::{TagKey, Tags, Track},
    plugin::{PluginId, Uuid},
};
use sea_query::{IntoIden, OnConflict, Query, SimpleExpr, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::{path::Path, str::FromStr};
use tracing::*;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct AlbumInfo<'a> {
    title: Option<&'a str>,
    mbid: Option<&'a str>,
    album_artist: ArtistInfo<'a>,
    artist: ArtistInfo<'a>,
}

impl<'a> AlbumInfo<'a> {
    fn from_tags(tags: &'a Tags) -> Self {
        let title = tags.get(&TagKey::AlbumTitle).map(|s| s.as_str());
        let mbid = tags
            .get(&TagKey::MusicBrainzReleaseGroupId)
            .map(|s| s.as_str());

        let album_artist = ArtistInfo::album_artist_from_tags(tags);

        let artist = ArtistInfo::from_tags(tags);

        Self {
            title,
            mbid,
            album_artist,
            artist,
        }
    }

    fn is_complete(&self) -> bool {
        self.mbid.is_some()
            || (self.title.is_some()
                && (self.album_artist.is_complete() || self.artist.is_complete()))
    }
}

#[derive(Debug, Clone)]
pub struct ArtistInfo<'a> {
    name: Option<&'a str>,
    mbid: Option<&'a str>,
}

impl<'a> ArtistInfo<'a> {
    fn from_tags(tags: &'a Tags) -> Self {
        let name = tags.get(&TagKey::TrackArtist).map(|s| s.as_str());
        let mbid = tags.get(&TagKey::MusicBrainzArtistId).map(|s| s.as_str());
        Self { name, mbid }
    }

    fn album_artist_from_tags(tags: &'a Tags) -> Self {
        let name = tags.get(&TagKey::AlbumArtist).map(|s| s.as_str());
        let mbid = tags
            .get(&TagKey::MusicBrainzReleaseArtistId)
            .map(|s| s.as_str());
        Self { name, mbid }
    }

    fn is_complete(&self) -> bool {
        self.mbid.is_some() || self.name.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct TrackGroupInfo<'a> {
    title: &'a str,
    track_mbid: Option<&'a str>,
    album_id: Option<AlbumId>,
}

impl<'a> TrackGroupInfo<'a> {
    fn from_track(track: &'a Track, album_id: Option<AlbumId>) -> Self {
        let title = track.title.as_str();
        let track_mbid = track
            .tags
            .get(&TagKey::MusicBrainzTrackId)
            .map(|s| s.as_str());
        Self {
            title,
            track_mbid,
            album_id,
        }
    }
}

impl Database {
    pub async fn connect<P: AsRef<Path>>(db_path: P) -> sqlx::Result<Self> {
        let db_path = db_path
            .as_ref()
            .to_str()
            .ok_or_else(|| sqlx::Error::Configuration("Invalid database path".into()))?;

        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{db_path}?mode=rwc"))?
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(opts).await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    #[tracing::instrument(skip(self))]
    pub async fn register_plugin(&self, uuid: Uuid) -> sqlx::Result<PluginId> {
        match self.get_plugin_id(uuid).await? {
            Some(plugin_id) => Ok(plugin_id),
            None => {
                let uuid = uuid.to_string();
                let plugin_id = sqlx::query!(
                    "INSERT INTO plugins (uuid) VALUES (?) RETURNING plugin_id",
                    uuid
                )
                .fetch_one(&self.pool)
                .await?
                .plugin_id;

                Ok(PluginId(plugin_id))
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_plugin_id(&self, uuid: Uuid) -> sqlx::Result<Option<PluginId>> {
        let uuid = uuid.to_string();
        let result = sqlx::query!("SELECT plugin_id FROM plugins WHERE uuid = ?", uuid)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.map(|r| PluginId(r.plugin_id)))
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_track(&self, mut track: Track) -> sqlx::Result<TrackId> {
        let title = track.title.clone();

        let (album_id, album_artist_id) = self
            .find_or_create_album(AlbumInfo::from_tags(&track.tags))
            .await?;
        track.tags.remove(&TagKey::AlbumTitle);
        track.tags.remove(&TagKey::AlbumArtist);

        let artist_id = self
            .find_or_create_artist(ArtistInfo::from_tags(&track.tags))
            .await?;
        track.tags.remove(&TagKey::TrackArtist);

        let track_group_id = self
            .find_or_create_track_group(TrackGroupInfo::from_track(&track, album_id))
            .await?;

        let columns = [
            "track_title",
            "track_group_id",
            "plugin_id",
            "plugin_data",
            "artist_id",
            "album_id",
            "album_artist_id",
        ]
        .iter()
        .map(|i| i.into_iden())
        .chain(track.tags.keys().map(|tag| tag.clone().into_iden()));

        let values = [
            SimpleExpr::Value(title.into()),
            track_group_id.0.into(),
            track.identifier.plugin.0.into(),
            track.identifier.plugin_data.0.into(),
            artist_id.map(|id| id.0).into(),
            album_id.map(|id| id.0).into(),
            album_artist_id.map(|id| id.0).into(),
        ]
        .into_iter()
        .chain(track.tags.values().map(|tag| tag.into()));

        let (sql, values) = Query::insert()
            .replace()
            .into_table("tracks")
            .columns(columns)
            .values_panic(values)
            .build_sqlx(SqliteQueryBuilder);

        let result = sqlx::query_with(&sql, values).execute(&self.pool).await?;
        let track_id = result.last_insert_rowid();
        trace!("Created or found track with ID: {}", track_id);

        Ok(TrackId(track_id))
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_track_group(
        &self,
        track_group_info: TrackGroupInfo<'_>,
    ) -> sqlx::Result<TrackGroupId> {
        if let Some(mbid) = track_group_info.track_mbid {
            if let Some(result) = sqlx::query!(
                "SELECT track_group_id FROM tracks WHERE music_brainz_track_id = ? GROUP BY track_group_id ORDER BY COUNT(*) DESC LIMIT 1",
                mbid
            )
                .fetch_optional(&self.pool)
            .await? {
                trace!(
                    "Found existing track group for MBID: {}",
                    result.track_group_id
                );

                return Ok(TrackGroupId(result.track_group_id));
            }
        }

        let album_id = track_group_info.album_id.map(|id| id.0);
        if let Some(result) = sqlx::query!(
                "SELECT track_group_id FROM tracks WHERE track_title = ? AND album_id = ? GROUP BY track_group_id ORDER BY COUNT(*) DESC LIMIT 1",
                track_group_info.title,
                album_id
            )
            .fetch_optional(&self.pool)
            .await? {
            trace!(
                "Found existing track group for title and album_id: {}",
                result.track_group_id
            );

            return Ok(TrackGroupId(result.track_group_id));
        }

        let track_group_id =
            sqlx::query!("INSERT INTO track_groups DEFAULT VALUES RETURNING track_group_id")
                .fetch_one(&self.pool)
                .await?
                .track_group_id;
        trace!("Created new track group with ID: {}", track_group_id);

        Ok(TrackGroupId(track_group_id))
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_album(
        &self,
        album_info: AlbumInfo<'_>,
    ) -> sqlx::Result<(Option<AlbumId>, Option<ArtistId>)> {
        if !album_info.is_complete() {
            return Ok((None, None));
        }

        if let Some(mbid) = album_info.mbid {
            if let Some(result) = sqlx::query!(
                "SELECT album_id, artist_id FROM albums WHERE mbid = ?",
                mbid
            )
            .fetch_optional(&self.pool)
            .await?
            {
                trace!("Found existing album for MBID: {}", result.album_id);

                return Ok((
                    Some(AlbumId(result.album_id)),
                    result.artist_id.map(ArtistId),
                ));
            }
        }

        let album_artist_id = match self.find_or_create_artist(album_info.album_artist).await? {
            Some(id) => Some(id),
            None => self.find_or_create_artist(album_info.artist).await?,
        };

        if let Some((title, album_artist_id)) = album_info.title.zip(album_artist_id) {
            if let Some(result) = sqlx::query!("SELECT album_id, artists.artist_id FROM albums, artists WHERE albums.artist_id = artists.artist_id AND albums.title = ? AND artists.name = ?", title, album_artist_id.0)
                .fetch_optional(&self.pool)
                .await?
            {
                trace!("Found existing album for title and artist: {}", result.album_id);

                return Ok((Some(AlbumId(result.album_id)), Some(ArtistId(result.artist_id))));
            }
        }

        let album_artist_id_i64 = album_artist_id.map(|id| id.0);
        let album_id = sqlx::query!(
            "INSERT INTO albums (title, mbid, artist_id) VALUES (?, ?, ?) RETURNING album_id",
            album_info.title,
            album_info.mbid,
            album_artist_id_i64
        )
        .fetch_one(&self.pool)
        .await?
        .album_id;
        trace!("Created new album with ID: {}", album_id);

        Ok((Some(AlbumId(album_id)), album_artist_id))
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_artist(
        &self,
        artist_info: ArtistInfo<'_>,
    ) -> sqlx::Result<Option<ArtistId>> {
        if !artist_info.is_complete() {
            return Ok(None);
        }

        if let Some(mbid) = artist_info.mbid {
            if let Some(result) = sqlx::query!("SELECT artist_id FROM artists WHERE mbid = ?", mbid)
                .fetch_optional(&self.pool)
                .await?
            {
                trace!("Found existing artist for MBID: {}", result.artist_id);
                return Ok(Some(ArtistId(result.artist_id)));
            }
        }

        if let Some(name) = artist_info.name {
            if let Some(result) =
                sqlx::query!("SELECT artist_id FROM artists WHERE name = ?", name,)
                    .fetch_optional(&self.pool)
                    .await?
            {
                trace!("Found existing artist for name: {}", result.artist_id);
                return Ok(Some(ArtistId(result.artist_id)));
            }
        }

        let artist_id = sqlx::query!(
            "INSERT INTO artists (name, mbid) VALUES (?, ?) RETURNING artist_id",
            artist_info.name,
            artist_info.mbid
        )
        .fetch_one(&self.pool)
        .await?
        .artist_id;
        trace!("Created new artist with ID: {}", artist_id);

        Ok(Some(ArtistId(artist_id)))
    }
}
