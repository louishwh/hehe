use crate::error::{Result, ToolError};
use crate::traits::{Tool, ToolOutput};
use async_trait::async_trait;
use hehe_core::{Context, ToolDefinition, ToolParameter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tokio::fs;

pub struct ReadFileTool {
    def: ToolDefinition,
}

impl ReadFileTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("read_file", "Read the contents of a file")
            .with_required_param(
                "path",
                ToolParameter::string().with_description("Path to the file to read"),
            )
            .with_param(
                "encoding",
                ToolParameter::string()
                    .with_description("File encoding (default: utf-8)")
                    .with_default(Value::String("utf-8".into())),
            );
        Self { def }
    }
}

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ReadFileInput {
    path: String,
    #[serde(default = "default_encoding")]
    encoding: String,
}

fn default_encoding() -> String {
    "utf-8".to_string()
}

#[async_trait]
impl Tool for ReadFileTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: ReadFileInput = serde_json::from_value(input)?;
        
        let path = Path::new(&input.path);
        if !path.exists() {
            return Ok(ToolOutput::error(format!("File not found: {}", input.path)));
        }

        match fs::read_to_string(path).await {
            Ok(content) => {
                let size = content.len();
                Ok(ToolOutput::text(content)
                    .with_metadata("path", &input.path)
                    .with_metadata("size", size))
            }
            Err(e) => Ok(ToolOutput::error(format!("Failed to read file: {}", e))),
        }
    }
}

pub struct WriteFileTool {
    def: ToolDefinition,
}

impl WriteFileTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("write_file", "Write content to a file")
            .with_required_param(
                "path",
                ToolParameter::string().with_description("Path to the file to write"),
            )
            .with_required_param(
                "content",
                ToolParameter::string().with_description("Content to write"),
            )
            .with_param(
                "append",
                ToolParameter::boolean()
                    .with_description("Append to file instead of overwriting")
                    .with_default(Value::Bool(false)),
            )
            .dangerous();
        Self { def }
    }
}

impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct WriteFileInput {
    path: String,
    content: String,
    #[serde(default)]
    append: bool,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: WriteFileInput = serde_json::from_value(input)?;
        
        let path = Path::new(&input.path);
        
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    return Ok(ToolOutput::error(format!("Failed to create directory: {}", e)));
                }
            }
        }

        let result = if input.append {
            let existing = fs::read_to_string(path).await.unwrap_or_default();
            fs::write(path, format!("{}{}", existing, input.content)).await
        } else {
            fs::write(path, &input.content).await
        };

        match result {
            Ok(_) => Ok(ToolOutput::text(format!("Successfully wrote to {}", input.path))
                .with_metadata("path", &input.path)
                .with_metadata("bytes_written", input.content.len())),
            Err(e) => Ok(ToolOutput::error(format!("Failed to write file: {}", e))),
        }
    }
}

pub struct ListDirectoryTool {
    def: ToolDefinition,
}

impl ListDirectoryTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("list_directory", "List contents of a directory")
            .with_required_param(
                "path",
                ToolParameter::string().with_description("Path to the directory"),
            )
            .with_param(
                "recursive",
                ToolParameter::boolean()
                    .with_description("List recursively")
                    .with_default(Value::Bool(false)),
            );
        Self { def }
    }
}

impl Default for ListDirectoryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ListDirectoryInput {
    path: String,
    #[serde(default)]
    recursive: bool,
}

#[derive(Serialize, Deserialize)]
struct DirectoryEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
}

#[async_trait]
impl Tool for ListDirectoryTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: ListDirectoryInput = serde_json::from_value(input)?;
        
        let path = Path::new(&input.path);
        if !path.exists() {
            return Ok(ToolOutput::error(format!("Directory not found: {}", input.path)));
        }
        if !path.is_dir() {
            return Ok(ToolOutput::error(format!("Not a directory: {}", input.path)));
        }

        let mut entries = Vec::new();
        
        if input.recursive {
            collect_entries_recursive(path, &mut entries).await?;
        } else {
            let mut read_dir = fs::read_dir(path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let metadata = entry.metadata().await?;
                entries.push(DirectoryEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: metadata.is_dir(),
                    size: if metadata.is_file() { Some(metadata.len()) } else { None },
                });
            }
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        ToolOutput::json(&entries)
    }
}

async fn collect_entries_recursive(path: &Path, entries: &mut Vec<DirectoryEntry>) -> Result<()> {
    let mut read_dir = fs::read_dir(path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let entry_data = DirectoryEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
        };
        entries.push(entry_data);

        if metadata.is_dir() {
            Box::pin(collect_entries_recursive(&entry.path(), entries)).await?;
        }
    }
    Ok(())
}

pub struct SearchFilesTool {
    def: ToolDefinition,
}

impl SearchFilesTool {
    pub fn new() -> Self {
        let def = ToolDefinition::new("search_files", "Search for files matching a pattern")
            .with_required_param(
                "pattern",
                ToolParameter::string().with_description("Glob pattern to search for"),
            )
            .with_param(
                "path",
                ToolParameter::string()
                    .with_description("Base path to search from")
                    .with_default(Value::String(".".into())),
            );
        Self { def }
    }
}

impl Default for SearchFilesTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct SearchFilesInput {
    pattern: String,
    #[serde(default = "default_path")]
    path: String,
}

fn default_path() -> String {
    ".".to_string()
}

#[async_trait]
impl Tool for SearchFilesTool {
    fn definition(&self) -> &ToolDefinition {
        &self.def
    }

    async fn execute(&self, _ctx: &Context, input: Value) -> Result<ToolOutput> {
        let input: SearchFilesInput = serde_json::from_value(input)?;
        
        let full_pattern = format!("{}/{}", input.path, input.pattern);
        
        let matches: Vec<String> = glob::glob(&full_pattern)
            .map_err(|e| ToolError::invalid_input(format!("Invalid pattern: {}", e)))?
            .filter_map(|r| r.ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        ToolOutput::json(&matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let tool = ReadFileTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "path": file_path.to_string_lossy()
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);
        assert_eq!(output.content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let tool = ReadFileTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "path": "/nonexistent/file.txt"
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(output.is_error);
        assert!(output.content.contains("not found"));
    }

    #[tokio::test]
    async fn test_write_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("output.txt");

        let tool = WriteFileTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "content": "Test content"
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_write_file_append() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("append.txt");
        std::fs::write(&file_path, "First").unwrap();

        let tool = WriteFileTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "path": file_path.to_string_lossy(),
            "content": "Second",
            "append": true
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "FirstSecond");
    }

    #[tokio::test]
    async fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let tool = ListDirectoryTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "path": dir.path().to_string_lossy()
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);

        let entries: Vec<DirectoryEntry> = serde_json::from_str(&output.content).unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_search_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test1.txt"), "a").unwrap();
        std::fs::write(dir.path().join("test2.txt"), "b").unwrap();
        std::fs::write(dir.path().join("other.md"), "c").unwrap();

        let tool = SearchFilesTool::new();
        let ctx = Context::new();
        let input = serde_json::json!({
            "pattern": "*.txt",
            "path": dir.path().to_string_lossy()
        });

        let output = tool.execute(&ctx, input).await.unwrap();
        assert!(!output.is_error);

        let matches: Vec<String> = serde_json::from_str(&output.content).unwrap();
        assert_eq!(matches.len(), 2);
    }
}
