use std::path::PathBuf;

// use crate::file_crawler::Directory;

#[derive(Debug)]
pub enum ADTreeNode {
    File(ADFile),
    Dir(ADDir),
}

#[derive(Debug)]
pub struct ADFile {
    pub path: PathBuf,
    pub summary: String,
    pub size: u64,
    // pub content_hash: Hash,
    // pub last_prompt_hash: Hash,
}

#[derive(Debug)]
pub struct ADDir {
    pub path: PathBuf,
    pub children: Vec<ADTreeNode>,
    pub summary: String,
    pub size: u64,
    pub content_hash: Hash,
    pub last_prompt_hash: Hash,
}

pub fn create_tree_from_fs_tree(dir: &Directory) -> ADTreeNode {
    ADTreeNode::Dir(ADDir {
        path: PathBuf::new(),
        children: Vec::new(),
        summary: String::new(),
        size: 0,
        // content_hash: blake3::Hash::from_hex("a".repeat(64)).expect("Invalid hash"),
        // last_prompt_hash: blake3::Hash::from_hex("b".repeat(64)).expect("Invalid hash"),
    })
}
