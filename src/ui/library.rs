use std::{collections::HashMap, time::Duration};

use futures_util::TryStreamExt;
use hogehoge_db::Database;
use hogehoge_types::{TagKind, Track, TrackId};

use crate::Library;
use crate::ui::*;

#[component]
pub fn LibraryStats() -> Element {
    let library = use_context_resource::<Library>()?;

    let stats = library.read().stats();
    let stats = stats.read();
    rsx!(label {
        "Tracks: {stats.num_tracks}, Track Groups: {stats.num_track_groups}, Artists: {stats.num_artists}, Albums: {stats.num_albums}",
    })
}

#[component]
pub fn LibraryView() -> Element {
    const ITEM_SIZE: i32 = 32;

    let theme = use_context::<Theme>();

    let library = use_context_resource::<Library>()?;
    let db = use_context_resource::<Database>()?;
    let notifications = use_context::<NotificationManager>();
    // use_hook(|| notifications.add(library.read().scan()));

    let scroll_controller = use_scroll_controller(ScrollConfig::default);

    #[derive(Clone, Debug)]
    enum TrackCacheState {
        Loaded(Box<Track>),
        BeingLoaded,
        LoadFailed(String),
    }

    let mut track_list = use_signal(Vec::new);
    let db_clone = db.clone();

    let memoized_stats = use_memo(move || library.read().stats().cloned());

    let mut get_track_list = use_future(move || {
        let db = db_clone.clone();

        async move {
            let db = db.read();

            match db.get_track_listing().try_collect().await {
                Ok(tracks) => {
                    track_list.replace(tracks);
                }
                Err(e) => tracing::error!("Failed to fetch track listing: {e}"),
            };

            tracing::trace!("Track listing fetched, total tracks: {}", track_list.len());
        }
    });

    let _track_list_timeout = use_resource(move || {
        let num_tracks = memoized_stats.read().num_tracks;

        async move {
            // still fetch every few tracks imported
            if num_tracks % 10 != 0 && num_tracks > 100 {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }

            get_track_list.restart();
        }
    });

    let range = use_memo(move || {
        let start = (-(*scroll_controller.y().read() / ITEM_SIZE)) as usize;
        let count = 100; // TODO: figure out how to calculate visible ScrollView items
        let end = usize::min(track_list.read().len(), start + count);

        tracing::trace!("Setting target range to ({}, {})", start, end);

        (start, end)
    });

    let mut visible_track_ids = use_signal(Vec::new);
    let mut visible_tracks = use_signal(HashMap::new);

    use_memo(move || {
        let range = range.read();
        let track_list = track_list.read();

        let mut new_visible_tracks = Vec::new();
        {
            let mut visible_tracks = visible_tracks.write();
            let target_visible_tracks = track_list[range.0..range.1].to_owned();

            visible_tracks.retain(|id, _state| target_visible_tracks.contains(id));

            for track_id in &target_visible_tracks {
                if !visible_tracks.contains_key(track_id) {
                    new_visible_tracks.push(*track_id);
                    visible_tracks.insert(*track_id, TrackCacheState::BeingLoaded);
                }
            }

            visible_track_ids.set(target_visible_tracks.clone());
        }

        if new_visible_tracks.is_empty() {
            return;
        }

        tracing::trace!("Fetching {} new tracks", new_visible_tracks.len());

        let db = db.clone();
        spawn(async move {
            let db = db.read();

            let new_tracks = db.get_tracks_by_id(&new_visible_tracks).await;

            let mut visible_tracks = visible_tracks.write();
            match new_tracks {
                Ok(tracks) => {
                    for (track, id) in tracks.into_iter().zip(new_visible_tracks.into_iter()) {
                        visible_tracks.insert(id, TrackCacheState::Loaded(Box::new(track)));
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to fetch new tracks: {e}");
                    for track_id in &new_visible_tracks {
                        visible_tracks
                            .insert(*track_id, TrackCacheState::LoadFailed(e.to_string()));
                    }
                }
            }
        });
    });

    #[derive(Clone, Debug, PartialEq)]
    struct BuilderArgs {
        track_list: Signal<Vec<TrackId>>,
        visible_tracks: Signal<HashMap<TrackId, TrackCacheState>>,
    }

    let builder_args = BuilderArgs {
        track_list,
        visible_tracks,
    };

    rsx!(rect {
        width: "fill",
        height: "calc(100% - 32)", // 32px for the bottom bar
        background: theme.colors.container,
        corner_radius: "4",

        VirtualScrollView {
            scroll_controller,
            cache_elements: false,
            length: track_list.read().len(),
            item_size: ITEM_SIZE as f32,
            builder_args: Some(builder_args),
            builder: |i, args: &Option<BuilderArgs>| {
                let args = args.as_ref().unwrap();

                let track_list = args.track_list.read();
                let visible_tracks = args.visible_tracks.read();

                let track = track_list.get(i).and_then(|id| visible_tracks.get(id)).cloned().unwrap_or(TrackCacheState::LoadFailed("Track index out of range".to_string()));

                // tracing::info!("Rendering track at index {i}, target range: {:?}", range);

                match track {
                    TrackCacheState::Loaded(track) => {
                        rsx!(LibraryItem {
                            index: i,
                            track: *track,
                        })
                    }

                    TrackCacheState::BeingLoaded => {
                        rsx!(rect {
                            width: "fill",
                            height: "32",
                            // label {
                            //     "Loading...",
                            // }
                        })
                    }

                    TrackCacheState::LoadFailed(e) => {
                        rsx!(rect {
                            width: "fill",
                            height: "32",
                            label {
                                "Loading this track failed: {e}",
                            }
                        })
                    }
                }
            }

        }
    })
}

#[component]
pub fn LibraryItem(index: usize, track: ReadOnlySignal<Track>) -> Element {
    let theme = use_context::<Theme>();
    let library = use_context_resource::<Library>()?;

    const CELLS: &[TagKind] = &[
        TagKind::TrackTitle,
        TagKind::TrackArtist,
        TagKind::AlbumTitle,
        TagKind::AlbumArtist,
        TagKind::RecordingDate,
    ];

    let track_read = track.read();
    let cells = CELLS
        .iter()
        .map(|kind| track_read.tags.get(*kind))
        .collect::<Vec<_>>();

    enum State {
        Idle,
        Hovered,
        Pressed,
    }

    let mut state = use_signal(|| State::Idle);

    let use_alt_color = index % 2 == 0;

    rsx!(rect {
        width: "fill",
        height: "32",
        direction: "horizontal",

        onmouseenter: move |_| state.set(State::Hovered),
        onmouseleave: move |_| state.set(State::Idle),
        onmousedown: move |_| {
            library.read().play(
                track.read().identifier.clone()
            );

            state.set(State::Pressed);
        },
        onmouseup: move |_| state.set(State::Hovered),

        background: match *state.read() {
            State::Idle => if use_alt_color {
                Some(theme.colors.table_row_alt)
            } else {
                None
            },
            State::Hovered => Some(theme.colors.table_row_hover),
            State::Pressed => Some(theme.colors.table_row_press),
        },

        for cell in cells {
            rect {
                width: "calc(100% / {CELLS.len()})",
                height: "fill",
                padding: "0 8",
                main_align: "center",
                label {
                    width: "fill",
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "{cell}",
                }
            }
        }

    })
}
