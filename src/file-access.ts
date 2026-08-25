import { invoke } from "@tauri-apps/api/core";

/**
 * Thin wrapper around the Rust file-access commands. This is the only
 * module allowed to call `invoke` for file access — everything else
 * (rendering, UI) works with plain strings.
 */

/** One "Recent" list entry, as persisted and enriched by the Rust layer. */
export interface RecentEntry {
  name: string;
  path: string;
  exists: boolean;
}

/** Opens the native "open file" dialog filtered to markdown files. */
export function openMarkdownFile(): Promise<string | null> {
  return invoke<string | null>("open_markdown_file");
}

/** Reads a markdown file's contents as UTF-8 text. */
export function readMarkdownFile(path: string): Promise<string> {
  return invoke<string>("read_markdown_file", { path });
}

/** Reads the persisted recent-files list (missing/corrupt config -> empty list). */
export function readRecentFiles(): Promise<RecentEntry[]> {
  return invoke<RecentEntry[]>("read_recent_files");
}

/**
 * Overwrites the persisted recent-files list with `paths` (most-recent-first).
 * The Rust layer handles deduping, capping, and dropping directories.
 */
export function writeRecentFiles(paths: string[]): Promise<void> {
  return invoke<void>("write_recent_files", { paths });
}
