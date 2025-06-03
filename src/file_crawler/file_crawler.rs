use std::path::PathBuf;
use std::fs;
use std::io;

use crate::file_crawler::file::File;
use crate::file_crawler::tree_node::TreeNode;
use crate::file_crawler::directory::{DirectoryBuilder};

pub trait FileCrawler {
    /// Walk up the directory tree (starting at `start_at`) until an
    /// `filename` file is found, or return `None` if not found.
    fn find_parent_with_file(&self, start_at: &PathBuf, filename: &str) -> Option<PathBuf>;

    /// Recursively walk the directory tree rooted at `root`, returning a
    /// `Directory` that represents the hierarchy.
    fn crawl(&self, root: &PathBuf, max_depth: Option<usize>, include_hidden: bool) -> io::Result<TreeNode>;
}

pub struct DefaultFileCrawler {
    max_depth: Option<usize>,
    include_hidden: bool,
}

impl DefaultFileCrawler {
    pub fn new() -> Self {
        DefaultFileCrawler {
            max_depth: None,
            include_hidden: false,
        }
    }
    
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }
}

impl FileCrawler for DefaultFileCrawler {
    fn crawl(&self, path: &PathBuf, max_depth: Option<usize>, include_hidden: bool) -> io::Result<TreeNode> {
        fn crawl_recursive(path: &PathBuf, current_depth: usize, max_depth: Option<usize>, include_hidden: bool) -> io::Result<TreeNode> {
            if !path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Path '{}' does not exist", path.display())
                ));
            }
            
            if !path.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Path '{}' is not a directory", path.display())
                ));
            }
            
            let mut builder = DirectoryBuilder::new(path.clone());
            
            // Check if we've reached max depth
            if let Some(max_depth) = max_depth {
                if current_depth >= max_depth {
                    return Ok(TreeNode::Directory(builder.build()));
                }
            }
            
            for dir_entry in fs::read_dir(&path)? {
                let dir_entry = dir_entry?;
                let entry_path = dir_entry.path();
                let metadata = dir_entry.metadata()?;
                
                let name = entry_path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                
                // Skip hidden files/directories if not included
                if !include_hidden && name.starts_with('.') {
                    continue;
                }
                
                if metadata.is_file() {
                    let file = File::new(entry_path)?;
                    builder = builder.add_child(TreeNode::from(file));
                } else if metadata.is_dir() {
                    let subdirectory = crawl_recursive(&entry_path, current_depth + 1, max_depth, include_hidden)?;
                    builder = builder.add_child(TreeNode::from(subdirectory));
                }
            }
            
            Ok(TreeNode::Directory(builder.build()))
        }
        crawl_recursive(path, 0, max_depth, include_hidden)
    }
    
    fn find_parent_with_file(&self, start_at: &PathBuf, filename: &str) -> Option<PathBuf> {
        todo!()
    }
    

}

impl Default for DefaultFileCrawler {
    fn default() -> Self {
        Self::new()
    }
}
