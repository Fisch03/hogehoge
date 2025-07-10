use crate::ui::*;

#[component]
pub fn MainContent() -> Element {
    rsx!(rect {
        height: "fill",
        width: "fill",
        LibraryView {}
        BottomBar {}
    })
}



#[component]
pub fn BottomBar() -> Element {
    let theme = use_context::<Theme>();

    rsx!(rect {
        width: "fill",
        height: "32",
        corner_radius: "4",
        main_align: "center",
        background: theme.colors.background,
        LibraryStats {},
    })
}

#[component]
pub fn TestingArea() -> Element {
    let notifications = use_context::<NotificationManager>();

    rsx!(rect {
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