use crate::ui::*;

#[component]
pub fn MainContent() -> Element {
    rsx!(rect {
        height: "fill",
        width: "fill",
        ContentTab {}
    })
}

#[component]
pub fn ContentTab() -> Element {
    let theme = use_context::<Theme>();
    let notifications = use_context::<NotificationManager>();

    rsx!(rect {
        width: "fill",
        height: "fill",
        background: theme.colors.container,
        corner_radius: "8",
        Button {
            onclick: move |_| {
                notifications.add(Notification::new("2hoge", "this is a test notification"));
            },
            label {
                "notification test",
            }
        },
    })
}
