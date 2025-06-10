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
    rsx!(rect {
        width: "fill",
        height: "fill",
        background: theme.colors.container,
        corner_radius: "16",
    })
}
