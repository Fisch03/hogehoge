use anyhow::Result;
use clap::Parser;
use nu_ansi_term::Color;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
        command.arg("--target=wasm32-wasip1");
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

    let glob_pattern = format!(
        "{}/wasm32-wasip1/{}/*.wasm",
        args.build_dir.to_string_lossy(),
        if args.release { "release" } else { "debug" }
    );

    if args.release {
        println!("{}", Color::Blue.bold().paint("Optimizing plugins..."));
    } else {
        println!("{}", Color::Blue.bold().paint("Copying plugins..."));
    }

    let plugins: Vec<_> = glob::glob(&glob_pattern)
        .unwrap()
        .map(|entry| {
            entry.unwrap_or_else(|e| {
                panic!("Failed to read plugin path: {}", e);
            })
        })
        .collect();

    plugins.par_iter().for_each(|in_path| {
        let name = in_path.file_name().unwrap();
        let out_path = args.out_dir.join(name);

        if args.release {
            optimize(&in_path, &out_path).unwrap_or_else(|e| {
                panic!("Failed to optimize plugin {:?}: {}", in_path, e);
            });
        } else {
            std::fs::copy(&in_path, &out_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to copy plugin {:?} to {:?}: {}",
                    in_path, out_path, e
                );
            });
        }
    });

    Ok(())
}

fn optimize(in_path: &Path, out_path: &Path) -> Result<()> {
    use wasm_opt::OptimizationOptions;

    let name = in_path.file_name().unwrap();

    let size_before = in_path.metadata()?.len();

    OptimizationOptions::new_opt_level_3()
        .run(&in_path, &out_path)
        .map_err(|e| anyhow::anyhow!("Failed to optimize {:?}: {}", in_path, e))?;

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

    Ok(())
}
