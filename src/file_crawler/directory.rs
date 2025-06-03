use std::path::PathBuf;
use crate::common::units::format_bytes;

use super::tree_node::TreeNode;

#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub name: String,
    pub is_hidden: bool,
    pub children: Vec<TreeNode>,
}

impl Directory {
    pub fn total_size(&self) -> u64 {
        // let files_size: u64 = self.files.iter().map(|f| f.size).sum();
        self.children.iter().map(|node| {
            match node {
                TreeNode::File(file) => file.size,
                TreeNode::Directory(dir) => dir.total_size()
            }
        }).sum()
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
    children: Vec<TreeNode>,
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
            children: Vec::new(),
        }
    }
    
    pub fn add_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }
    
    pub fn build(self) -> Directory {
        Directory {
            path: self.path,
            name: self.name,
            is_hidden: self.is_hidden,
            children: self.children,
        }
    }
}
