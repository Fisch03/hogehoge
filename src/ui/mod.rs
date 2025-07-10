pub use freya::prelude::*;
pub use hogehoge_types::theme::{PartialTheme, Theme};

pub mod notifications;
pub use notifications::{
    Notification, NotificationManager, ToastNotificationTarget, use_notification_provider,
};

mod status_bar;
pub use status_bar::StatusBar;
mod side_bar;
pub use side_bar::SideBar;
mod player;
pub use player::PlayerBar;
mod main_content;
pub use main_content::MainContent;
mod library;
pub use library::{LibraryView, LibraryStats};

use std::sync::LazyLock;
pub static DEFAULT_THEME: LazyLock<Theme> = LazyLock::new(|| {
    const DEFAULT_THEME: &str = include_str!("../../target/themes/ferra.2ht");
    let partial = PartialTheme::load(std::io::Cursor::new(DEFAULT_THEME))
        .expect("Expected default theme to always load");
    Theme::from_partial(partial).expect("Expected default theme to contain all theme fields")
});

#[derive(Debug, Clone)]
pub struct ContextResource<T: 'static> {
    name: &'static str,
    value: Signal<Option<T>>,
    task: Task,
    state: Signal<bool>,
}

impl<T> ContextResource<T> {
    pub fn suspend(&self) -> Result<MappedSignal<T>, RenderError> {
        if self.state.cloned() {
            Ok(self.value.map(|v| v.as_ref().unwrap()))
        } else {
            Err(RenderError::Suspended(SuspendedFuture::new(self.task)))
        }
    }
}

pub fn use_resource_provider<T, F>(
    name: &'static str,
    mut f: impl FnMut() -> F + 'static,
) -> ContextResource<T>
where
    T: Clone + 'static,
    F: Future<Output = T> + 'static,
{
    let mut value = use_signal(|| None);
    let mut state = use_signal(|| false);

    let task = spawn(async move {
        let res = f().await;

        state.set(true);
        value.set(Some(res));
    });

    use_context_provider(|| ContextResource {
        name,
        value,
        state,
        task,
    })
}

pub fn use_context_resource<T: Clone>() -> Result<MappedSignal<T>, RenderError> {
    let resource = use_context::<ContextResource<T>>();

    resource.suspend().with_loading_placeholder(|| {
        rsx!(label {
            font_size: "16",
            "Waiting for {resource.name} to initialize..."
        })
    })
}

#[component]
pub fn Icon(data: Vec<u8>, rotate: Option<String>) -> Element {
    let icon = dynamic_bytes(data);

    rsx!(svg {
        svg_data: icon,
        width: "24",
        height: "24",
        rotate,
    })
}

#[component]
pub fn Button(
    onclick: Option<Callback<()>>,
    #[props(default = "0 0 4 0 rgb(0, 0, 0, 100)".to_string())] shadow: String,
    children: Element,
) -> Element {
    let theme = use_context::<Theme>();

    enum State {
        Idle,
        Hovered,
        Pressed,
    }

    let mut state = use_signal(|| State::Idle);

    rsx!(
        rect {
            shadow,
            corner_radius: "6",
            padding: "6",
            cross_align: "center",

            onmouseenter: move |_| state.set(State::Hovered),
            onmouseleave: move |_| state.set(State::Idle),
            onmousedown: move |_| state.set(State::Pressed),
            onmouseup: move |_| state.set(State::Hovered),

            onclick: move |e| {
                if let Some(callback) = &onclick {
                    callback.call(());
                }
                e.stop_propagation();
            },

            background: match *state.read() {
                State::Idle => theme.colors.button_idle,
                State::Hovered => theme.colors.button_hover,
                State::Pressed => theme.colors.button_press,
            },

            {children}
        },

    )
}

#[component]
pub fn IconButton(
    icon: Vec<u8>,
    onclick: Option<Callback<()>>,
    inner_rotation: Option<String>,
    #[props(default = "0 0 4 0 rgb(0, 0, 0, 100)".to_string())] shadow: String,
    children: Element,
) -> Element {
    let theme = use_context::<Theme>();

    enum State {
        Idle,
        Hovered,
        Pressed,
    }

    let mut state = use_signal(|| State::Idle);

    rsx!(
        rect {
            shadow,
            corner_radius: "6",
            padding: "6",
            cross_align: "center",

            onmouseenter: move |_| state.set(State::Hovered),
            onmouseleave: move |_| state.set(State::Idle),
            onmousedown: move |_| state.set(State::Pressed),
            onmouseup: move |_| state.set(State::Hovered),

            onclick: move |e| {
                if let Some(callback) = &onclick {
                    callback.call(());
                }
                e.stop_propagation();
            },

            background: match *state.read() {
                State::Idle => theme.colors.button_idle,
                State::Hovered => theme.colors.button_hover,
                State::Pressed => theme.colors.button_press,
            },

            Icon {
                data: icon,
                rotate: inner_rotation
            },
            {children}
        },
    )
}
