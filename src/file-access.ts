import { invoke } from "@tauri-apps/api/core";

/**
 * Thin wrapper around the Rust file-access commands. This is the only
 * module allowed to call `invoke` for file access — everything else
 * (rendering, UI) works with plain strings.
 */

/** Opens the native "open file" dialog filtered to markdown files. */
export function openMarkdownFile(): Promise<string | null> {
  return invoke<string | null>("open_markdown_file");
}

/** Reads a markdown file's contents as UTF-8 text. */
export function readMarkdownFile(path: string): Promise<string> {
  return invoke<string>("read_markdown_file", { path });
}
