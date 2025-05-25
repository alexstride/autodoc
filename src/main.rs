use clap::{Parser, ValueHint};
use std::path::PathBuf;

mod file;
mod directory;
mod crawler;

use crawler::Crawler;

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
    let path = cli.path.clone();

    // Fail fast if the path doesn't exist
    if !path.exists() {
        anyhow::bail!("Path '{}' does not exist", path.display());
    }

    // Use the virtual file tree functionality
    let crawler = Crawler::new();
    let directory = crawler.crawl(path)?;
    
    print_directory(&directory, 0, &cli);
    
    println!("\nSummary:");
    println!("Total files: {}", directory.total_files());
    println!("Total size: {} bytes", directory.total_size());

    Ok(())
}

fn print_directory(dir: &directory::Directory, indent: usize, cli: &Cli) {
    let indent_str = "  ".repeat(indent);
    
    if !cli.files_only {
        println!("{}{}", indent_str, dir);
    }
    
    if !cli.dirs_only {
        for file in &dir.files {
            println!("{}  {}", indent_str, file);
        }
    }
    
    for subdir in &dir.subdirectories {
        print_directory(subdir, indent + 1, cli);
    }
}
