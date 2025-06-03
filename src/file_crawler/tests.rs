#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::file_crawler::{File, DirectoryBuilder, DefaultFileCrawler, FileCrawler, TreeNode};

    #[test]
    fn test_file_node_creation() {
        let file_path = PathBuf::from("Cargo.toml");
        let file = File::new(file_path).expect("Failed to create File");
        let node = TreeNode::from(file);
        
        if let TreeNode::File(f) = node {
            assert_eq!(f.name, "Cargo.toml");
            assert!(!f.is_hidden);
            assert!(f.size > 0);
        } else {
            panic!("Expected File variant");
        }
    }

    #[test]
    fn test_directory_node_creation() {
        let dir_path = PathBuf::from("src");
        let builder = DirectoryBuilder::new(dir_path);
        let directory = builder.build();
        let node = TreeNode::from(directory);
        
        if let TreeNode::Directory(d) = node {
            assert_eq!(d.name, "src");
            assert!(!d.is_hidden);
            assert_eq!(d.children.len(), 0);
        } else {
            panic!("Expected Directory variant");
        }
    }

    #[test]
    fn test_crawler_returns_directory_node() {
        let crawler = DefaultFileCrawler::new();
        let src_path = PathBuf::from("src");
        let result = crawler.crawl(&src_path, None, true).expect("Failed to crawl directory");
        
        if let TreeNode::Directory(directory) = result {
            assert_eq!(directory.name, "src");
            assert!(directory.children.len() > 0);
            let file_count = directory.children.iter()
                .filter(|n| matches!(n, TreeNode::File(_)))
                .count();
            assert!(file_count >= 4);
        } else {
            panic!("Expected Directory variant");
        }
    }

    #[test]
    fn test_crawler_with_max_depth() {
        let crawler = DefaultFileCrawler::new().with_max_depth(1);
        let src_path = PathBuf::from("src");
        let result = crawler.crawl(&src_path, None, true).expect("Failed to crawl directory");
        
        if let TreeNode::Directory(directory) = result {
            // Verify we have children but no nested directories due to depth limit
            assert!(directory.children.len() > 0);
            let dir_count = directory.children.iter()
                .filter(|n| matches!(n, TreeNode::Directory(_)))
                .count();
            assert_eq!(dir_count, 0);
        } else {
            panic!("Expected Directory variant");
        }
    }

    #[test]
    fn test_crawler_error_cases() {
        let crawler = DefaultFileCrawler::new();
        
        // Non-existent path
        let bad_path = PathBuf::from("nonexistent");
        assert!(crawler.crawl(&bad_path, None, true).is_err());
        
        // File path instead of directory
        let file_path = PathBuf::from("Cargo.toml");
        assert!(crawler.crawl(&file_path, None, true).is_err());
    }

    #[test]
    fn test_hidden_file_handling() {
        let crawler = DefaultFileCrawler::new().include_hidden(false);
        let src_path = PathBuf::from("src");
        let result = crawler.crawl(&src_path, None, false).expect("Failed to crawl directory");
        
        if let TreeNode::Directory(directory) = result {
            // Verify no hidden files are included
            let hidden_count = directory.children.iter()
                .filter_map(|n| match n {
                    TreeNode::File(f) => Some(f.is_hidden),
                    _ => None
                })
                .filter(|&h| h)
                .count();
            assert_eq!(hidden_count, 0);
        } else {
            panic!("Expected Directory variant");
        }
    }

    #[test]
    fn test_directory_totals() {
        let crawler = DefaultFileCrawler::new();
        let src_path = PathBuf::from("src");
        let result = crawler.crawl(&src_path, None, true).expect("Failed to crawl directory");
        
        if let TreeNode::Directory(directory) = result {
            let total_size = directory.total_size();
            assert!(total_size > 0);
            
            let direct_files_size: u64 = directory.children.iter()
                .filter_map(|n| match n {
                    TreeNode::File(f) => Some(f.size),
                    _ => None
                })
                .sum();
            assert!(total_size >= direct_files_size);
        } else {
            panic!("Expected Directory variant");
        }
    }
}
