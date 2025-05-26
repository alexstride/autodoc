pub mod crawler;
pub mod tree_renderer;
pub mod common;

use clap::{Parser, ValueHint};
use std::path::PathBuf;
use crawler::{Crawler, Directory};
use tree_renderer::render_directory_tree;

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

fn print_directory(dir: &Directory, _indent: usize, cli: &Cli) {
    let show_sizes = !(cli.dirs_only || cli.files_only); // Show sizes unless filtering
    let tree = render_directory_tree(dir, show_sizes);
    println!("{}", tree);
}
