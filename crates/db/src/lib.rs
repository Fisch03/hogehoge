use freya::prelude::{Signal, SyncStorage, Writable};
use futures_util::stream::BoxStream;
use hogehoge_types::{
    AlbumId, ArtistId, TrackGroupId, TrackId, UniqueTrackIdentifier,
    library::{Tags, Track},
    plugin::{PluginId, Uuid},
};
use sqlx::{
    QueryBuilder, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use std::{path::Path, str::FromStr};
use tracing::*;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
    stats: Signal<DbStats, SyncStorage>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DbStats {
    pub num_tracks: usize,
    pub num_track_groups: usize,
    pub num_albums: usize,
    pub num_artists: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct AlbumInfo<'a> {
    title: Option<&'a str>,
    mbid: Option<Uuid>,
    album_artist: ArtistInfo<'a>,
    artist: ArtistInfo<'a>,
}

impl<'a> AlbumInfo<'a> {
    fn from_tags(tags: &'a Tags) -> Self {
        let title = tags.album_title.as_deref();
        let mbid = tags.musicbrainz_release_group_id;

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

#[derive(Debug, Clone, Copy)]
pub struct CreatedAlbum {
    pub id: AlbumId,
    pub album_artist_id: Option<ArtistId>,
}

#[derive(Debug, Clone, Copy)]
pub struct ArtistInfo<'a> {
    name: Option<&'a str>,
    mbid: Option<Uuid>,
}

impl<'a> ArtistInfo<'a> {
    fn from_tags(tags: &'a Tags) -> Self {
        let name = tags.track_artist.as_deref();
        let mbid = tags.musicbrainz_artist_id;

        Self { name, mbid }
    }

    fn album_artist_from_tags(tags: &'a Tags) -> Self {
        let name = tags.album_artist.as_deref();
        let mbid = tags.musicbrainz_release_artist_id;

        Self { name, mbid }
    }

    fn is_complete(&self) -> bool {
        self.mbid.is_some() || self.name.is_some()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackGroupInfo<'a> {
    title: &'a str,
    track_mbid: Option<Uuid>,
    album_id: Option<AlbumId>,
}

impl<'a> TrackGroupInfo<'a> {
    fn from_tags(tags: &'a Tags, album_id: Option<AlbumId>) -> Self {
        let title = &tags.track_title;
        let track_mbid = tags.musicbrainz_track_id;

        Self {
            title,
            track_mbid,
            album_id,
        }
    }
}

impl Database {
    pub async fn connect<P: AsRef<Path>>(
        db_path: P,
        stats: Signal<DbStats, SyncStorage>,
    ) -> sqlx::Result<Self> {
        let db_path = db_path
            .as_ref()
            .to_str()
            .ok_or_else(|| sqlx::Error::Configuration("Invalid database path".into()))?;

        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{db_path}?mode=rwc"))?
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePool::connect_with(opts).await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;

        let mut db = Self { pool, stats };
        db.update_stats().await?;

        Ok(db)
    }

    pub fn stats(&self) -> Signal<DbStats, SyncStorage> {
        self.stats
    }

    pub async fn update_stats(&mut self) -> sqlx::Result<()> {
        let result = sqlx::query!(
            "
            SELECT 
                (SELECT COUNT(*) FROM tracks) AS num_tracks,
                (SELECT COUNT(DISTINCT track_group_id) FROM tracks) AS num_track_groups,
                (SELECT COUNT(DISTINCT album_id) FROM tracks) AS num_albums,
                (SELECT COUNT(DISTINCT artist_id) FROM tracks) AS num_artists
            "
        )
        .fetch_one(&self.pool)
        .await?;

        let mut stats = self.stats.write();
        stats.num_tracks = result.num_tracks as usize;
        stats.num_track_groups = result.num_track_groups as usize;
        stats.num_albums = result.num_albums as usize;
        stats.num_artists = result.num_artists as usize;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn register_plugin(&self, uuid: Uuid) -> sqlx::Result<PluginId> {
        match self.get_plugin_id(uuid).await? {
            Some(plugin_id) => Ok(plugin_id),
            None => {
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
        Ok(
            sqlx::query_scalar!("SELECT plugin_id FROM plugins WHERE uuid = ?", uuid)
                .fetch_optional(&self.pool)
                .await?
                .map(PluginId),
        )
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_tracks_by_id(&self, track_ids: &[TrackId]) -> sqlx::Result<Vec<Track>> {
        let mut query = QueryBuilder::new("SELECT * FROM tracks WHERE track_id IN ");

        query.push_tuples(track_ids, |mut b, track_id| {
            b.push_bind(track_id.0);
        });

        query.push(" ORDER BY track_title");

        let query = query.build_query_as();

        // TODO: return a stream instead but that kinda sucks since the querybuilder data isnt
        // owned
        query.fetch_all(&self.pool).await
    }

    #[tracing::instrument(skip(self))]
    pub fn get_track_listing(&self) -> BoxStream<sqlx::Result<TrackId>> {
        sqlx::query_scalar(
            "SELECT track_id FROM tracks LEFT JOIN albums ON tracks.album_id = albums.album_id
            ORDER BY albums.title COLLATE NOCASE",
        )
        .fetch(&self.pool)
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_track(
        &mut self,
        identifier: UniqueTrackIdentifier,
        tags: Tags,
    ) -> sqlx::Result<TrackId> {
        let transaction = self.pool.begin().await?;

        let album = self
            .find_or_create_album(AlbumInfo::from_tags(&tags))
            .await?;

        let artist_id = self
            .find_or_create_artist(ArtistInfo::from_tags(&tags))
            .await?;

        let track_group_id = self
            .find_or_create_track_group(TrackGroupInfo::from_tags(&tags, album.map(|a| a.id)))
            .await?;

        let track = Track {
            track_group_id,
            artist_id,
            album_artist_id: album.and_then(|a| a.album_artist_id),
            album_id: album.map(|a| a.id),
            identifier,
            tags,
        };

        let track_id = track.upsert_into(&self.pool).await?;

        trace!("Created or found track with ID: {}", track_id.0);

        self.update_stats().await?;

        transaction.commit().await?;

        Ok(track_id)
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_or_create_track_group(
        &self,
        track_group_info: TrackGroupInfo<'_>,
    ) -> sqlx::Result<TrackGroupId> {
        if let Some(mbid) = track_group_info.track_mbid {
            if let Some(result) = sqlx::query!(
                "SELECT track_group_id FROM tracks WHERE musicbrainz_track_id = ? GROUP BY track_group_id ORDER BY COUNT(*) DESC LIMIT 1",
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

        if let Some(result) = sqlx::query!(
                "SELECT track_group_id FROM tracks WHERE track_title = ? AND album_id = ? GROUP BY track_group_id ORDER BY COUNT(*) DESC LIMIT 1",
                track_group_info.title,
                track_group_info.album_id,
            )
            .fetch_optional(&self.pool)
            .await? {
            trace!(
                "Found existing track group for title and album_id: {}",
                result.track_group_id
            );

            return Ok(TrackGroupId(result.track_group_id));
        }

        self.create_track_group().await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_track_group(&self) -> sqlx::Result<TrackGroupId> {
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
    ) -> sqlx::Result<Option<CreatedAlbum>> {
        if !album_info.is_complete() {
            return Ok(None);
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

                return Ok(Some(CreatedAlbum {
                    id: AlbumId(result.album_id),
                    album_artist_id: result.artist_id.map(ArtistId),
                }));
            }
        }

        let album_artist_id = match self.find_or_create_artist(album_info.album_artist).await? {
            Some(id) => Some(id),
            None => self.find_or_create_artist(album_info.artist).await?,
        };

        if let Some((title, album_artist_id)) = album_info.title.zip(album_artist_id) {
            if let Some(result) = sqlx::query!("SELECT album_id, artists.artist_id, albums.mbid AS album_mbid FROM albums, artists WHERE albums.artist_id = artists.artist_id AND albums.title = ? AND artists.name = ?", title, album_artist_id)
                .fetch_optional(&self.pool)
                .await?
            {
                trace!("Found existing album for title and artist: {}", result.album_id);


                // try filling in the MBID if its missing
                match (album_info.mbid, result.album_mbid.as_deref().map(Uuid::from_slice)) {
                    (Some(mbid), None) | (Some(mbid), Some(Err(_))) => {
                        sqlx::query!(
                            "UPDATE albums SET mbid = ? WHERE album_id = ?",
                            mbid,
                            result.album_id
                        )
                        .execute(&self.pool)
                        .await?;
                    }
                    (Some(mbid), Some(Ok(existing_mbid))) if mbid != existing_mbid => {
                        warn!(
                            "MBID mismatch for album '{}': found {}, expected {}",
                            title, existing_mbid, mbid
                        );
                    }

                    _ => {}
                }

                return Ok(Some(CreatedAlbum {
                    id: AlbumId(result.album_id),
                    album_artist_id: Some(ArtistId(result.artist_id)),
                }));
            }
        }

        let Some(title) = album_info.title else {
            return Ok(None);
        };

        let album_id = self
            .create_album(title, album_info.mbid, album_artist_id)
            .await?;

        Ok(Some(CreatedAlbum {
            id: album_id,
            album_artist_id,
        }))
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_album(
        &self,
        title: &str,
        mbid: Option<Uuid>,
        album_artist_id: Option<ArtistId>,
    ) -> sqlx::Result<AlbumId> {
        let album_id = sqlx::query!(
            "INSERT INTO albums (title, mbid, artist_id) VALUES (?, ?, ?) RETURNING album_id",
            title,
            mbid,
            album_artist_id
        )
        .fetch_one(&self.pool)
        .await?
        .album_id;

        trace!("Created new album with ID: {}", album_id);

        Ok(AlbumId(album_id))
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
                sqlx::query!("SELECT artist_id, mbid FROM artists WHERE name = ?", name,)
                    .fetch_optional(&self.pool)
                    .await?
            {
                trace!("Found existing artist for name: {}", result.artist_id);

                // try filling in the MBID if its missing
                match (
                    artist_info.mbid,
                    result.mbid.as_deref().map(Uuid::from_slice),
                ) {
                    (Some(mbid), None) | (Some(mbid), Some(Err(_))) => {
                        sqlx::query!(
                            "UPDATE artists SET mbid = ? WHERE artist_id = ?",
                            mbid,
                            result.artist_id
                        )
                        .execute(&self.pool)
                        .await?;
                    }
                    (Some(mbid), Some(Ok(existing_mbid))) if mbid != existing_mbid => {
                        warn!(
                            "MBID mismatch for artist '{}': found {}, expected {}",
                            name, existing_mbid, mbid
                        );
                    }
                    _ => {}
                }

                return Ok(Some(ArtistId(result.artist_id)));
            }
        }

        let Some(name) = artist_info.name else {
            return Ok(None);
        };

        let artist_id = self.create_artist(name, artist_info.mbid).await?;
        Ok(Some(artist_id))
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_artist(&self, name: &str, mbid: Option<Uuid>) -> sqlx::Result<ArtistId> {
        let artist_id = sqlx::query!(
            "INSERT INTO artists (name, mbid) VALUES (?, ?) RETURNING artist_id",
            name,
            mbid
        )
        .fetch_one(&self.pool)
        .await?
        .artist_id;
        trace!("Created new artist with ID: {}", artist_id);

        Ok(ArtistId(artist_id))
    }
}
