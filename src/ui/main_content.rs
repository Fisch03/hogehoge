use crate::ui::*;
use crate::{Library, PluginSystem};

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

    let library = use_context_resource::<Library>()?;
    let plugin_system = use_context_resource::<PluginSystem>()?;
    let notifications = use_context::<NotificationManager>();
    use_hook(|| notifications.add(library.read().scan(plugin_system.read().clone())));

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
