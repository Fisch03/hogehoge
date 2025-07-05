use anyhow::{Context, Result};
use clap::Parser;
use hogehoge_types::{PartialTheme, PartialThemeManifest};
use nu_ansi_term::Color;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short)]
    in_dir: PathBuf,

    #[arg(long, short)]
    out_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.out_dir.exists() {
        std::fs::remove_dir_all(&args.out_dir)?;
    }
    std::fs::create_dir_all(&args.out_dir)?;

    println!("{}", Color::Blue.bold().paint("Building themes..."));

    for entry in std::fs::read_dir(&args.in_dir).unwrap() {
        let entry = entry?;

        println!(
            "{}",
            Color::Green.bold().paint(format!(
                "Building theme {:?}...",
                entry.path().file_name().unwrap()
            ))
        );

        let manifest = entry.path().join("theme.toml");
        let manifest: PartialThemeManifest = toml::from_str(
            &std::fs::read_to_string(manifest).expect("Failed to read theme manifest"),
        )?;

        let out_path = args.out_dir.join(format!("{}.2ht", manifest.name));

        let mut archive =
            tar::Builder::new(File::create(&out_path).expect("Failed to create output file"));

        for file in entry
            .path()
            .read_dir()
            .expect("Failed to read theme directory")
        {
            let file = file?;
            let path = file.path();
            if path.is_file() {
                let relative_path = path.strip_prefix(&entry.path()).unwrap();
                archive.append_path_with_name(&path, relative_path)?;
            }
        }

        archive.finish()?;

        PartialTheme::load(File::open(&out_path)?)
            .with_context(|| format!("Test loading theme {:?} failed", manifest.name))?;
    }

    Ok(())
}
