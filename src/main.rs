use clap::Parser;
use hogehoge_db::{Database, DbStats};
use std::path::PathBuf;
use tokio::task;

mod library;
use library::Library;

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

use std::sync::OnceLock;
static DB: OnceLock<Database> = OnceLock::new();

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
    use_resource_provider("Database", move || {
        let db_path = args.db_path.clone();
        async move {
            let db = Database::connect(db_path, db_stats)
                .await
                .expect("Failed to open database");

            match DB.set(db.clone()) {
                Ok(_) => tracing::info!("Database initialized successfully"),
                Err(_) => tracing::warn!("Database was initialized twice!"),
            }

            db
        }
    });

    use_resource_provider("Library", || async {
        let db = task::spawn_blocking(|| DB.wait()).await.unwrap();
        Library::new(db.clone()).await
    });

    use_resource_provider("Plugin System", move || {
        let plugin_dir = args.plugin_dir.clone();
        async move {
            let db = task::spawn_blocking(|| DB.wait()).await.unwrap();
            PluginSystem::initialize(plugin_dir, db.clone())
                .await
                .expect("Failed to initialize plugin system")
        }
    });

    use_notification_provider();

    children
}
