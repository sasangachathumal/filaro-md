/**
 * Turns a file's absolute path into a short, readable location: the folder
 * it's in, relative to the user's home folder.
 *
 *   /Users/sam/Downloads/notes.md           -> "Downloads"
 *   /Users/sam/Documents/work/plan.md       -> "Documents/work"
 *   C:\Users\sam\Documents\plan.md          -> "Documents"
 *   /Users/sam/todo.md                      -> "Home"
 *   /Volumes/USB/docs/readme.md             -> "/Volumes/USB/docs" (outside home: unchanged)
 *
 * The file name itself is left out — the sidebar already shows it on the
 * line above. Pure string handling; no OS or Tauri access.
 */
export function displayLocation(filePath: string, homeDir: string | null): string {
  const isWindows = filePath.includes("\\");
  const separator = isWindows ? "\\" : "/";
  const folder = parentFolder(filePath, separator);

  if (!homeDir) {
    return folder;
  }

  const home = homeDir.replace(/[\\/]+$/, "");
  const same = (a: string, b: string) => (isWindows ? a.toLowerCase() === b.toLowerCase() : a === b);

  if (same(folder, home)) {
    return "Home";
  }

  const homePrefix = home + separator;
  if (folder.length > homePrefix.length && same(folder.slice(0, homePrefix.length), homePrefix)) {
    return folder.slice(homePrefix.length);
  }

  return folder;
}

function parentFolder(filePath: string, separator: string): string {
  const lastSeparator = filePath.lastIndexOf(separator);
  if (lastSeparator < 0) {
    return filePath;
  }

  const folder = filePath.slice(0, lastSeparator);
  // Keep a root folder recognisable: "/" on macOS, "C:\" on Windows.
  if (folder === "" || folder.endsWith(":")) {
    return folder + separator;
  }
  return folder;
}
