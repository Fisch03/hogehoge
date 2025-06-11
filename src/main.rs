use clap::Parser;
use std::path::PathBuf;

mod library;
use library::Library;

mod plugin;
use plugin::PluginSystem;

mod logging;

mod ui;
use ui::*;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(long, short, default_value = "./plugins")]
    plugin_dir: PathBuf,
    #[arg(long, short, default_value = "./themes")]
    theme_dir: PathBuf,
}

fn main() {
    logging::init();
    tracing::info!("Starting 2hoge!");

    let args = Args::parse();

    launch_cfg(
        app,
        LaunchConfig::new().with_title("2hoge").with_state(args),
    );
}

fn app() -> Element {
    let args = use_context::<Args>();

    let theme = use_context_provider(|| ui::DEFAULT_THEME.clone());

    let library = use_context_provider(|| Library::new());
    let plugin_system =
        use_context_provider(|| PluginSystem::initialize(args.plugin_dir.clone()).unwrap());

    let task_handler = use_task_handler();

    use_hook(|| task_handler.start(library.scan(plugin_system)));

    rsx!(rect {
        width: "100%",
        height: "100%",
        background: theme.colors.background,
        color: theme.colors.foreground,
        PlayerBar {},
        rect {
            height: "calc(100% - 16 - 32)", // 16 for status bar, 48 for player bar
            width: "100%",
            direction: "horizontal",
            SideBar {},
            MainContent {},
        }
        StatusBar {},
    })
}
