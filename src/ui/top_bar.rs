use crate::ui::*;

#[component]
pub fn TopBar() -> Element {
    rsx!(rect {
        width: "100%",
        height: "32",
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
    let task_handler = use_context::<TaskHandler>();

    rsx!(rect {
        width: "25%",
        main_align: "end",
        direction: "horizontal",

        TaskWidget {}
    })
}

#[component]
fn TaskWidget() -> Element {
    let theme = use_context::<Theme>();
    let task_handler = use_context::<TaskHandler>();

    let icon_button_anim = use_animation(|conf| {
        conf.auto_start(true);
        conf.on_finish(OnFinish::Restart);
        AnimSequential::new([
            AnimNum::new(0.0, 180.0)
                .time(1000)
                .ease(Ease::InOut)
                .function(Function::Elastic),
            AnimNum::new(0.0, 0.0).time(5000),
        ])
    });

    let any_running = task_handler.any_running();

    let icon_button_rotation = if any_running {
        let anim = icon_button_anim.get();
        anim.read()[0].read() as f32
    } else {
        0.0
    };

    let mut task_list_open = use_signal(|| false);

    rsx!(IconButton {
        shadow: "none",
        icon: theme.icons.background_task_running,
        onclick: move |_| task_list_open.toggle(),

        inner_rotation: "{icon_button_rotation}deg",
        if *task_list_open.read() {
            TaskList {
                onclose: move |_| task_list_open.set(false),
            }
        },
    })
}

#[component]
fn TaskList(onclose: Callback<()>) -> Element {
    let theme = use_context::<Theme>();
    let task_handler = use_context::<TaskHandler>();
    let task_states = task_handler.get_states();

    rsx!(rect {
        position: "absolute",
        position_top: "0",
        position_right: "0",
        corner_radius: "8",
        width: "600",
        background: theme.colors.background,
        shadow: "0 0 10 rgb(0, 0, 0, 120)",
        padding: "8",
        spacing: "8",

        label {
            font_weight: "bold",
            "Background Tasks"
        },
        for task in task_states.read().values() {
            TaskListItem {
                task: task.clone(),
            }
        }
    })
}

#[component]
fn TaskListItem(task: BackgroundTaskState) -> Element {
    rsx!(rect {
        width: "100%",

        label {
            "{task.name}"
        }
        if !task.done {
            ProgressBar {
                progress: task.progress,
                width: "100%",
            },
            label {
                "{task.message}"
            }
        } else {
            "done"
        }
    })
}
