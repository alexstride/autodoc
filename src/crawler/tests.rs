#[cfg(test)]
use crate::crawler::{File, DirectoryBuilder, Crawler};
#[cfg(test)]
use std::path::PathBuf;

#[test]
fn test_file_creation() {
    let file_path = PathBuf::from("Cargo.toml");
    let file = File::new(file_path).expect("Failed to create File");
    
    assert_eq!(file.name, "Cargo.toml");
    assert!(!file.is_hidden);
    assert!(file.size > 0);
}

#[test]
fn test_directory_builder() {
    let dir_path = PathBuf::from("src");
    let builder = DirectoryBuilder::new(dir_path);
    let directory = builder.build();
    
    assert_eq!(directory.name, "src");
    assert!(!directory.is_hidden);
    assert_eq!(directory.files.len(), 0);
    assert_eq!(directory.subdirectories.len(), 0);
}

#[test]
fn test_crawler_basic() {
    let crawler = Crawler::new();
    let src_path = PathBuf::from("src");
    let directory = crawler.crawl(src_path).expect("Failed to crawl directory");
    
    assert_eq!(directory.name, "src");
    assert!(directory.files.len() > 0); // Should contain our source files
    assert!(directory.total_files() >= 4); // At least main.rs, file.rs, directory.rs, crawler.rs
}

#[test]
fn test_directory_totals() {
    let crawler = Crawler::new();
    let src_path = PathBuf::from("src");
    let directory = crawler.crawl(src_path).expect("Failed to crawl directory");
    
    let total_files = directory.total_files();
    let total_size = directory.total_size();
    
    assert!(total_files > 0);
    assert!(total_size > 0);
    
    // The total should be at least the sum of individual file sizes
    let direct_files_size: u64 = directory.files.iter().map(|f| f.size).sum();
    assert!(total_size >= direct_files_size);
}
