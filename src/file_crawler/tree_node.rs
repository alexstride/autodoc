use std::convert::From;
use crate::file_crawler::{File, Directory};


#[derive(Debug, Clone)]

pub enum TreeNode {
    File(File),
    Directory(Directory)
}

impl From<File> for TreeNode {
    fn from(f: File) -> Self {
        TreeNode::File(f)
    }
}

impl From<Directory> for TreeNode {
    fn from(d: Directory) -> Self {
        TreeNode::Directory(d)
    }
}