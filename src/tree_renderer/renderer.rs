use crate::{common::units::format_bytes, crawler::Directory};

pub fn render_directory_tree(directory: &Directory, show_sizes: bool) -> String {
    let mut output = String::new();
    render_directory_recursive(directory, 0, &mut output, show_sizes);
    output
}

fn render_directory_recursive(
    directory: &Directory,
    depth: usize,
    output: &mut String,
    show_sizes: bool,
) {
    // Add directory name with proper indentation
    let indent = "    ".repeat(depth);
    output.push_str(&indent);
    output.push_str(&directory.name);
    output.push('/');

    if show_sizes {
        output.push_str(&format!(" ({})", format_bytes(directory.total_size())));
    }
    output.push('\n');

    // Render files with consistent indentation
    for file in &directory.files {
        output.push_str(&indent);
        output.push_str("    ├── ");
        output.push_str(&file.name);
        if show_sizes {
            output.push_str(&format!(" ({})", format_bytes(file.size)));
        }
        output.push('\n');
    }

    // Render subdirectories
    for subdir in &directory.subdirectories {
        render_directory_recursive(subdir, depth + 1, output, show_sizes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::{File, DirectoryBuilder};

    #[test]
    fn test_render_simple_tree() {
        let file1 = File {
            path: "file1.txt".into(),
            name: "file1.txt".to_string(),
            size: 100,
            is_hidden: false,
        };

        let dir = DirectoryBuilder::new("test".into())
            .add_file(file1)
            .build();

        let rendered = render_directory_tree(&dir, false);
        assert_eq!(rendered, "test/\n    ├── file1.txt\n");
    }

    #[test]
    fn test_render_with_sizes() {
        let file1 = File {
            path: "file1.txt".into(),
            name: "file1.txt".to_string(),
            size: 100,
            is_hidden: false,
        };

        let dir = DirectoryBuilder::new("test".into())
            .add_file(file1)
            .build();

        let rendered = render_directory_tree(&dir, true);
        assert_eq!(rendered, "test/ (100B)\n    ├── file1.txt (100B)\n");
    }
}
