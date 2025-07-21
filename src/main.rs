use clap::Parser;
use hogehoge_db::{Database, DbStats};
use std::path::PathBuf;

mod library;
use library::Library;

mod audio;
mod queue;

mod plugin;
use plugin::PluginSystem;

mod logging;

mod ui;
use ui::*;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(long, short, default_value = "2hoge.db")]
    db_path: PathBuf,
    #[arg(long, short, default_value = "./plugins")]
    plugin_dir: PathBuf,
    #[arg(long, short, default_value = "./themes")]
    theme_dir: PathBuf,
}

fn main() {
    logging::init();
    tracing::info!("Starting 2hoge!");

    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    launch_cfg(
        app,
        LaunchConfig::new().with_title("2hoge").with_state(args),
    );
}

fn app() -> Element {
    rsx!(AppWrapper { ContextProvider {
        StatusBar {},
        PlayerBar {},
        rect {
            height: "calc(100% - 16 - 32)", // status + player bar height
            width: "100%",
            direction: "horizontal",
            SideBar {},
            MainContent {},
        },
        ToastNotificationTarget { },
    }})
}

#[component]
fn AppWrapper(children: Element) -> Element {
    let theme = use_context_provider(|| ui::DEFAULT_THEME.clone());

    rsx!(rect {
        width: "100%",
        height: "100%",
        background: theme.colors.background,
        color: theme.colors.foreground,
        SuspenseBoundary {
            fallback: |ctx: SuspenseContext| rsx!(
                rect {
                    width: "100%",
                    height: "100%",
                    cross_align: "center",
                    main_align: "center",
                    {
                        ctx.suspense_placeholder().unwrap_or_else(|| {
                            rsx!(label {
                                font_size: "16",
                                "Loading...",
                            })
                        })
                    }
                }
            ),
            children
        }
    })
}

#[component]
fn ContextProvider(children: Element) -> Element {
    let args = use_context::<Args>();

    let db_stats = use_signal_sync(DbStats::default);
    let db = use_resource_provider("Database", move || {
        let db_path = args.db_path.clone();
        async move {
            let db = Database::connect(db_path, db_stats)
                .await
                .expect("Failed to open database");

            tracing::info!("Database initialized successfully");

            db
        }
    })
    .suspend()?;

    let db_clone = db.clone();
    let plugin_system = use_resource_provider("Plugin System", move || {
        let plugin_dir = args.plugin_dir.clone();
        let db_clone = db_clone.peek().clone();
        async move {
            PluginSystem::initialize(plugin_dir, db_clone)
                .await
                .expect("Failed to initialize plugin system")
        }
    })
    .suspend()?;

    let db_clone = db.clone();
    let plugin_system_clone = plugin_system.clone();
    use_resource_provider("Library", move || {
        let db = db_clone.peek().clone();
        let plugin_system = plugin_system_clone.peek().clone();
        async move { Library::new(db, plugin_system).await }
    });

    use_notification_provider();

    children
}
