use std::path::PathBuf;
use std::fs;
use std::io;
use crate::file::File;
use crate::directory::{Directory, DirectoryBuilder};

pub struct Crawler {
    max_depth: Option<usize>,
    include_hidden: bool,
}

impl Crawler {
    pub fn new() -> Self {
        Crawler {
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
    
    pub fn crawl(&self, path: PathBuf) -> io::Result<Directory> {
        self.crawl_recursive(path, 0)
    }
    
    fn crawl_recursive(&self, path: PathBuf, current_depth: usize) -> io::Result<Directory> {
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
        if let Some(max_depth) = self.max_depth {
            if current_depth >= max_depth {
                return Ok(builder.build());
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
            if !self.include_hidden && name.starts_with('.') {
                continue;
            }
            
            if metadata.is_file() {
                let file = File::new(entry_path)?;
                builder = builder.add_file(file);
            } else if metadata.is_dir() {
                let subdirectory = self.crawl_recursive(entry_path, current_depth + 1)?;
                builder = builder.add_subdirectory(subdirectory);
            }
        }
        
        Ok(builder.build())
    }
}

impl Default for Crawler {
    fn default() -> Self {
        Self::new()
    }
}
