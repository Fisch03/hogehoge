use crate::ui::*;
use futures::stream::{StreamExt};
use std::{collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};
use tokio::{time, sync::watch};
use tokio_stream::wrappers::WatchStream;

#[derive(Debug, Clone)]
pub struct NotificationManager {
    pub notifications: Signal<HashMap<usize, NotificationState>>,
    pub add_notification: Callback<Notification>,
}
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: Cow<'static, str>,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone)]
pub enum NotificationKind {
    Fixed {
        message: Cow<'static, str>,
        silent: bool,
    },
    Progress(watch::Receiver<ProgressNotificationData>)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgressNotificationData {
    pub message: Cow<'static, str>,
    pub progress: f32,
    pub is_done: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressNotificationHandle(watch::Sender<ProgressNotificationData>);

#[derive(Debug, Clone, PartialEq)]
struct NotificationState {
    id: usize,
    show_toast: bool,
    title: Cow<'static, str>,
    kind: NotificationStateKind,
}

#[derive(Debug, Clone, PartialEq)]
enum NotificationStateKind {
    Fixed {
        message: Cow<'static, str>,
    },
    Progress(ProgressNotificationData),
}

impl NotificationManager {
    pub fn add(&self, notification: Notification) {
        (self.add_notification)(notification);
    }
}

impl Notification {
    pub fn new<T, M>(title: T, message: M) -> Self
    where
        T: Into<Cow<'static, str>>,
        M: Into<Cow<'static, str>>,
    {
        let title = title.into();
        let message = message.into();

        Self {
            title,
            kind: NotificationKind::Fixed { message, silent: false },
        }
    }

    pub fn new_silent<T, M>(title: T, message: M) -> Self
    where
        T: Into<Cow<'static, str>>,
        M: Into<Cow<'static, str>>,
        
    {
        let title = title.into();
        let message = message.into();
        Self {
            title,
            kind: NotificationKind::Fixed { message, silent: true },
        }
    }

    pub fn new_progress<T>(
        title: T,
    ) -> (Self, ProgressNotificationHandle) where T: Into<Cow<'static, str>> {
        let title = title.into();

        let (tx, rx) = watch::channel(ProgressNotificationData::default());
        (
            Self {
                title,
                kind: NotificationKind::Progress(rx),
            },
            ProgressNotificationHandle(tx),
        )
    }
}

impl ProgressNotificationHandle {
    pub fn modify_state<F>(&self, f: F)
    where
        F: FnOnce(&mut ProgressNotificationData),
    {
        self.0.send_modify(f);
    }

    pub fn complete(&self) {
        self.0.send_modify(|s| {
            s.progress = 100.0;
            s.is_done = true;
        });
    }
}

pub fn use_notification_provider() -> NotificationManager {
    let mut notifications = use_signal(|| HashMap::new());
    
    let add_notification =
        use_callback(move |notification: Notification| {
            static NOTIFICATION_ID: AtomicUsize = AtomicUsize::new(0);
            let id = NOTIFICATION_ID.fetch_add(1, Ordering::SeqCst);

            let (show_toast, state_kind) = match notification.kind {
                NotificationKind::Fixed { message, silent } => { 
                    spawn(async move {
                        time::sleep(time::Duration::from_secs(5)).await;
                        notifications.write().get_mut(&id).map(|state: &mut NotificationState| {
                            state.show_toast = false;
                        });
                    });

                    (!silent, NotificationStateKind::Fixed {message})
                }
                NotificationKind::Progress(handle) => {
                    let initial_data = handle.borrow().clone();

                    let mut stream = WatchStream::new(handle);
                    spawn(async move {
                        while let Some(data) = stream.next().await {
                            notifications.write().get_mut(&id).map(|state: &mut NotificationState| {
                                if data.is_done && state.show_toast {
                                    state.show_toast = false;   
                                }

                                state.kind = NotificationStateKind::Progress(data);
                            });

                        }
                    });

                    (true, NotificationStateKind::Progress(initial_data))
                }
            };

            let state = NotificationState {
                id,
                title: notification.title,
                show_toast,
                kind: state_kind,
            };

            notifications.write().insert(id, state);


        });

    use_context_provider(move || {
        NotificationManager {
            notifications: notifications,
            add_notification: add_notification,
        }
    })
}

#[component]
pub fn ToastNotificationTarget() -> Element {
    let manager = use_context::<NotificationManager>();

    let notifications = manager.notifications.read();

    let mut toast_notifications = notifications.values()
        .filter(|state| state.show_toast)
        .cloned()
        .collect::<Vec<_>>();
    toast_notifications.sort_by_key(|state| state.id);
    

    rsx!(rect {
        position: "global",
        position_top: "16",
        position_right: "16",
        width: "300",
        spacing: "8",
        for notification in toast_notifications {
            ToastNotification { notification }
        }
    })
}

#[component]
fn ToastNotification(notification: NotificationState) -> Element {
    let theme = use_context::<Theme>();

    let animation = use_animation(|conf| {
        conf.auto_start(true);
        AnimNum::new(400.0, 0.0).time(300)
            .ease(Ease::Out)
            .function(Function::Cubic)
    });
    let offset_x = animation.get().read().read();

    let message = match &notification.kind {
        NotificationStateKind::Fixed { message } => message,
        NotificationStateKind::Progress(data) => &data.message,
    };

    rsx!(rect {
        width: "fill",
        offset_x: "{offset_x}",
        rect {
            width: "fill",
            padding: "8",
            corner_radius: "6",
            background: theme.colors.container,
            shadow: "0 0 4 2 rgb(0, 0, 0, 30)",
            label {
                font_weight: "bold",
                "{notification.title}",
            },
            if let NotificationStateKind::Progress(data) = &notification.kind {
                ProgressBar {
                    progress: data.progress,
                }
            },
            label {
                "{message}",
            },
        }
    })
}