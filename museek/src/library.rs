use lofty::{error::ErrorKind, file::TaggedFileExt};
use sqlx::{pool::PoolConnection, Database, SqlitePool};
use std::path::Path;
use tokio::{runtime, sync::oneshot};
use tracing::{error, warn};

mod track;
pub use track::Track;

#[derive(Debug, Clone)]
pub struct Library {
    pool: SqlitePool,
    rt: runtime::Handle,
}

impl Library {
    pub async fn new() -> sqlx::Result<Self> {
        let pool = SqlitePool::connect("sqlite://museeklibrary.db?mode=rwc").await?;
        sqlx::migrate!().run(&pool).await?;

        Ok(Self {
            pool,
            rt: runtime::Handle::current(),
        })
    }

    pub async fn update(&self) {
        let (tx, rx) = oneshot::channel();

        let task_span = tracing::Span::current();
        let pool = self.pool.clone();
        let rt = self.rt.clone();
        jwalk::rayon::spawn(move || {
            for entry in jwalk::WalkDir::new("/home/sakanaa/nas/Audio/Music/") {
                if let Ok(entry) = entry {
                    let _enter = task_span.enter();

                    let path = entry.path();
                    if path.is_file() {
                        match parse_tags(rt.clone(), &path, pool.clone()) {
                            Ok(_) => {}
                            Err(e) if matches!(e.kind(), ErrorKind::UnknownFormat) => {}
                            Err(e) => {
                                error!("Failed to parse tags for '{:?}': {}", path, e);
                            }
                        }
                    }
                }
            }

            let _ = tx.send(());
        });

        rx.await.unwrap();
    }
}

fn parse_tags(rt: runtime::Handle, path: &Path, pool: SqlitePool) -> lofty::error::Result<()> {
    let tagged_file = lofty::read_from_path(path)?;
    let tag = match tagged_file.primary_tag() {
        Some(tag) => tag,
        None => {
            warn!("No tags found for '{:?}'", path);
            return Ok(());
        }
    };

    let track = Track::from_tags(path, &tag);
    rt.block_on(async move {
        match track.insert(&pool).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to insert track '{:?}': {}", track.path(), e);
            }
        }
    });

    // let r = track.insert(conn);

    // log::info!(
    //     "{:?} on {:?} by {:?}",
    //     tag.title(),
    //     tag.album_title(),
    //     tag.artist()
    // );

    Ok(())
}
