use std::path::PathBuf;
use std::fs;
use std::io;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_hidden: bool,
}

impl File {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = fs::metadata(&path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let is_hidden = name.starts_with('.');
        
        Ok(File {
            path,
            name,
            size: metadata.len(),
            is_hidden,
        })
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}B)", self.name, self.size)
    }
}
