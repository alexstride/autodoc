use std::path::PathBuf;
use crate::crawler::file::File;
use crate::common::units::format_bytes;

#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub name: String,
    pub is_hidden: bool,
    pub files: Vec<File>,
    pub subdirectories: Vec<Directory>,
}

impl Directory {
    pub fn total_files(&self) -> usize {
        self.files.len() + self.subdirectories.iter().map(|d| d.total_files()).sum::<usize>()
    }
    
    pub fn total_size(&self) -> u64 {
        let files_size: u64 = self.files.iter().map(|f| f.size).sum();
        let subdirs_size: u64 = self.subdirectories.iter().map(|d| d.total_size()).sum();
        files_size + subdirs_size
    }
}

impl std::fmt::Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/ (total: {})", self.name, format_bytes(self.total_size()))
    }
}

pub struct DirectoryBuilder {
    path: PathBuf,
    name: String,
    is_hidden: bool,
    files: Vec<File>,
    subdirectories: Vec<Directory>,
}

impl DirectoryBuilder {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let is_hidden = name.starts_with('.');
        
        DirectoryBuilder {
            path,
            name,
            is_hidden,
            files: Vec::new(),
            subdirectories: Vec::new(),
        }
    }
    
    pub fn add_file(mut self, file: File) -> Self {
        self.files.push(file);
        self
    }
    
    pub fn add_subdirectory(mut self, directory: Directory) -> Self {
        self.subdirectories.push(directory);
        self
    }
    
    pub fn build(self) -> Directory {
        Directory {
            path: self.path,
            name: self.name,
            is_hidden: self.is_hidden,
            files: self.files,
            subdirectories: self.subdirectories,
        }
    }
}
