use clap::Parser;
use std::path::PathBuf;

use freya::prelude::*;

mod plugin;
use plugin::PluginSystem;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short, default_value = "./plugins")]
    plugin_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    let plugins =
        PluginSystem::initialize(args.plugin_dir).expect("Failed to initialize plugin system");

    launch_with_title(app, "2hoge")
}

fn app() -> Element {
    rsx!(
        rect {
            background: "red",
            width: "100%",
            onclick: |_| println!("Clicked!"),
            label {
                "Hello, World!"
            }
        }
    )
}
