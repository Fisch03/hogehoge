use crate::audio::{AudioPlayer, PlaybackState};
use crate::ui::*;

#[component]
pub fn PlayerBar() -> Element {
    let player = use_context_resource::<AudioPlayer>()?;

    let mut playback_state = use_signal(PlaybackState::default);
    use_future(move || {
        let mut state_rx = player.read().subscribe_state();

        async move {
            loop {
                *playback_state.write() = state_rx.borrow_and_update().clone();

                // dbg!(playback_state.read());

                let _ = state_rx.changed().await;
            }
        }
    });

    rsx!(rect {
        width: "100%",
        height: "32",
        background: "white",

        rect {
            height: "calc(100% - 2px)",
        }


        ProgressBar {
            playback_state
        },
    })
}

#[component]
fn ProgressBar(playback_state: Signal<PlaybackState>) -> Element {
    let playback_state = playback_state.read();
    let theme = use_context::<Theme>();

    let progress = match &*playback_state {
        PlaybackState::Stopped => 0.0,
        PlaybackState::Playing {
            position, duration, ..
        } => duration.map_or(1.0, |d| {
            if d.as_secs() == 0 {
                1.0
            } else {
                position.as_secs_f64() / d.as_secs_f64()
            }
        }),
    };

    rsx!(rect {
        width: format!("{}%", progress * 100.0),
        height: "2",
        background: theme.colors.foreground,
    })
}
