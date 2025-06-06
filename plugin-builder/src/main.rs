use ansi_term::Color;
use anyhow::Result;
use clap::Parser;
use std::{path::PathBuf, process::Command};
use wasm_opt::OptimizationOptions;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short)]
    in_dir: PathBuf,

    #[arg(long, short)]
    build_dir: PathBuf,

    #[arg(long, short)]
    out_dir: PathBuf,

    #[arg(long)]
    release: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", Color::Blue.bold().paint("Building plugins..."));

    for entry in std::fs::read_dir(&args.in_dir).unwrap() {
        let entry = entry?;

        println!(
            "{}",
            Color::Green.bold().paint(format!(
                "Building plugin {:?}...",
                entry.path().file_name().unwrap()
            ))
        );

        let mut command = Command::new("cargo");
        command.arg("build");
        command.arg("--target=wasm32-unknown-unknown");
        command.current_dir(entry.path());
        command.arg("--target-dir").arg(&args.build_dir);
        command.stdout(std::process::Stdio::null());

        if args.release {
            command.arg("--release");
        }

        let status = command.status()?;
        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to build plugin in {:?}",
                entry.path()
            ));
        }
    }

    if args.out_dir.exists() {
        std::fs::remove_dir_all(&args.out_dir)?;
    }
    std::fs::create_dir_all(&args.out_dir)?;

    println!("{}", Color::Blue.bold().paint("Optimizing plugins..."));

    let glob_pattern = format!(
        "{}/wasm32-unknown-unknown/{}/*.wasm",
        args.build_dir.to_string_lossy(),
        if args.release { "release" } else { "debug" }
    );

    for path in glob::glob(&glob_pattern).unwrap() {
        let path = path?;
        let size_before = path.metadata()?.len();
        let name = path.file_name().unwrap();

        let out_path = args.out_dir.join(name);

        let options = if args.release {
            OptimizationOptions::new_opt_level_3()
        } else {
            OptimizationOptions::new_opt_level_1()
        };

        options
            .run(&path, &out_path)
            .map_err(|e| anyhow::anyhow!("Failed to optimize {:?}: {}", path, e))?;

        let size_after = out_path.metadata()?.len();
        println!(
            "{}",
            Color::Green.bold().paint(format!(
                "Optimized {:?} from {:.1} KiB to {:.1} KiB",
                name,
                size_before as f64 / 1024.0,
                size_after as f64 / 1024.0,
            ))
        );
    }

    Ok(())
}
