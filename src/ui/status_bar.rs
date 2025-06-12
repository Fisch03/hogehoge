use crate::ui::*;

#[component]
pub fn StatusBar() -> Element {
    rsx!(rect {
        width: "100%",
        height: "16",
        direction: "horizontal",
        cross_align: "center",

        LeftSection {},
        CenterSection {},
        RightSection {},
    })
}

#[component]
fn LeftSection() -> Element {
    rsx!(rect {
        width: "25%",
        main_align: "start",
        direction: "horizontal",
    })
}

#[component]
fn CenterSection() -> Element {
    rsx!(rect {
        width: "50%",
        main_align: "center",
        direction: "horizontal",
        // SearchBar {},
    })
}

#[component]
fn RightSection() -> Element {
    rsx!(rect {
        width: "25%",
        main_align: "end",
        direction: "horizontal",

        // NotificationButton {},
    })
}