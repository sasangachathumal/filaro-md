use serde::Serialize;

/// A node in a folder tree that has been filtered down to markdown files.
/// Directories with no markdown files anywhere beneath them are pruned
/// before this tree is built, so every leaf is a `.md`/`.markdown` file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MarkdownEntry>>,
}
