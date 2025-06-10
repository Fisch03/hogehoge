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
        padding: "16",
        spacing: "8",
        TopBar {},
        PlayerBar {},
        MainContent {},
    })

    // rsx!(for state in task_states.read().values() {
    //     ProgressBar {
    //         width: "100%",
    //         progress: state.progress * 100.0,
    //         show_progress: true,
    //     }
    // })
}
