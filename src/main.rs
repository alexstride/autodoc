use clap::{Parser, ValueHint};
use std::{fs, path::PathBuf};

/// List child files and/or directories of the given path.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Target path (file or directory)
    #[arg(value_hint = ValueHint::AnyPath)]
    path: PathBuf,

    /// Show directories only
    #[arg(short = 'd', long, conflicts_with = "files_only")]
    dirs_only: bool,

    /// Show files only
    #[arg(short = 'f', long, conflicts_with = "dirs_only")]
    files_only: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Fail fast if the path doesn't exist
    if !cli.path.exists() {
        anyhow::bail!("Path '{}' does not exist", cli.path.display());
    }

    // If it's a single file we just echo it (unless filtered out)
    if cli.path.is_file() {
        if !cli.dirs_only {
            println!("{}", cli.path.display());
        }
        return Ok(());
    }

    // Non-recursive directory listing
    for entry in fs::read_dir(&cli.path)? {
        let entry = entry?;
        let md = entry.metadata()?;
        let p = entry.path();

        match (md.is_dir(), cli.dirs_only, cli.files_only) {
            (true, false, true) => continue,  // dir but files-only flag
            (false, true, false) => continue, // file but dirs-only flag
            _ => println!("{}", p.display()),
        }
    }

    Ok(())
}
