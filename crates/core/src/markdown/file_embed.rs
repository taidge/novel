use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

/// Information about a file embed directive
#[derive(Debug)]
pub struct FileEmbed {
    /// The file path to embed
    pub file_path: String,
    /// Optional line range (start, end) - 1-indexed, inclusive
    pub line_range: Option<(usize, usize)>,
}

/// Parse file embed meta from a code fence info string.
///
/// Supports:
/// - `file="./relative/path.rs"`
/// - `file="../parent/path.rs"`
/// - `file="<root>/src/file.rs"`
/// - `file="path.rs#L10-L20"`
pub fn parse_file_embed(info_string: &str) -> Option<FileEmbed> {
    let re = Regex::new(r#"file="([^"]+)""#).ok()?;
    let caps = re.captures(info_string)?;
    let raw_path = caps.get(1)?.as_str();

    // Check for line range suffix: #L10-L20 or #10-20
    let (file_path, line_range) = if let Some(hash_pos) = raw_path.rfind('#') {
        let path_part = &raw_path[..hash_pos];
        let range_part = &raw_path[hash_pos + 1..];
        let line_range = parse_line_range(range_part);
        (path_part.to_string(), line_range)
    } else {
        (raw_path.to_string(), None)
    };

    Some(FileEmbed {
        file_path,
        line_range,
    })
}

/// Parse line range from strings like "L10-L20", "10-20", "L5"
fn parse_line_range(s: &str) -> Option<(usize, usize)> {
    let s = s.trim_start_matches('L');
    if let Some((start_s, end_s)) = s.split_once('-') {
        let end_s = end_s.trim_start_matches('L');
        let start: usize = start_s.parse().ok()?;
        let end: usize = end_s.parse().ok()?;
        Some((start, end))
    } else {
        let line: usize = s.parse().ok()?;
        Some((line, line))
    }
}

/// Resolve the file path and read its content
pub fn read_embedded_file(
    embed: &FileEmbed,
    current_file_dir: &Path,
    project_root: &Path,
) -> Result<String> {
    let project_root = project_root
        .canonicalize()
        .with_context(|| format!("Failed to resolve project root: {}", project_root.display()))?;
    let resolved = if embed.file_path.starts_with("<root>/") {
        let rel = embed.file_path.trim_start_matches("<root>/");
        project_root.join(rel)
    } else {
        current_file_dir.join(&embed.file_path)
    };

    let resolved = resolved
        .canonicalize()
        .with_context(|| format!("Failed to resolve embedded file: {}", embed.file_path))?;

    if !resolved.starts_with(&project_root) {
        anyhow::bail!(
            "Embedded file is outside project root: {}",
            resolved.display()
        );
    }

    let content = std::fs::read_to_string(&resolved)
        .with_context(|| format!("Failed to read embedded file: {}", resolved.display()))?;

    if let Some((start, end)) = embed.line_range {
        let lines: Vec<&str> = content.lines().collect();
        let start = start.saturating_sub(1); // convert to 0-indexed
        let end = end.min(lines.len());
        if start >= end {
            return Ok(String::new());
        }
        Ok(lines[start..end].join("\n"))
    } else {
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_parse_file_embed_simple() {
        let result = parse_file_embed(r#"rust file="./main.rs""#);
        assert!(result.is_some());
        let embed = result.unwrap();
        assert_eq!(embed.file_path, "./main.rs");
        assert!(embed.line_range.is_none());
    }

    #[test]
    fn test_parse_file_embed_with_line_range() {
        let result = parse_file_embed(r#"rust file="./main.rs#L10-L20""#);
        assert!(result.is_some());
        let embed = result.unwrap();
        assert_eq!(embed.file_path, "./main.rs");
        assert_eq!(embed.line_range, Some((10, 20)));
    }

    #[test]
    fn test_parse_file_embed_root_path() {
        let result = parse_file_embed(r#"ts file="<root>/src/index.ts""#);
        assert!(result.is_some());
        let embed = result.unwrap();
        assert_eq!(embed.file_path, "<root>/src/index.ts");
    }

    #[test]
    fn test_no_file_embed() {
        let result = parse_file_embed("rust title=\"example\"");
        assert!(result.is_none());
    }

    #[test]
    fn read_embed_rejects_files_outside_project_root() {
        let root = temp_dir("novel-embed-root");
        let outside = temp_dir("novel-embed-outside");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), "secret").unwrap();

        let embed = FileEmbed {
            file_path: outside.join("secret.txt").to_string_lossy().to_string(),
            line_range: None,
        };
        let result = read_embedded_file(&embed, &root.join("docs"), &root);

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        assert!(result.is_err());
    }

    #[test]
    fn read_embed_handles_out_of_range_lines_without_panic() {
        let root = temp_dir("novel-embed-lines");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs").join("example.rs"), "one\ntwo\n").unwrap();

        let embed = FileEmbed {
            file_path: "./example.rs".to_string(),
            line_range: Some((10, 20)),
        };
        let result = read_embedded_file(&embed, &root.join("docs"), &root).unwrap();

        let _ = fs::remove_dir_all(&root);
        assert!(result.is_empty());
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), nanos))
    }
}
