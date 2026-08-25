import { openMarkdownFile, readMarkdownFile, readRecentFiles, writeRecentFiles } from "./file-access";
import { MarkdownPreview } from "./markdown-preview";
import { Sidebar, type CurrentFile, type RecentFileEntry } from "./sidebar";

function fileNameFromPath(path: string): string {
  const segments = path.split(/[\\/]/);
  return segments[segments.length - 1] || path;
}

window.addEventListener("DOMContentLoaded", () => {
  const openButton = document.querySelector<HTMLButtonElement>("#open-file-button")!;
  const statusEl = document.querySelector<HTMLElement>("#status")!;
  const previewRoot = document.querySelector<HTMLElement>("#preview-root")!;
  const sidebarRoot = document.querySelector<HTMLElement>("#sidebar-root")!;

  const preview = new MarkdownPreview(previewRoot);

  let currentFile: CurrentFile | null = null;
  let recentFiles: RecentFileEntry[] = [];

  const sidebar = new Sidebar(sidebarRoot, {
    currentFile,
    recentFiles,
    onSelectRecent: (path) => void openFile(path),
  });

  function updateSidebar(): void {
    sidebar.setProps({
      currentFile,
      recentFiles,
      onSelectRecent: (path) => void openFile(path),
    });
  }

  async function recordRecentFile(path: string): Promise<void> {
    const previousPaths = recentFiles.map((entry) => entry.path);
    await writeRecentFiles([path, ...previousPaths]);
    recentFiles = await readRecentFiles();
    updateSidebar();
  }

  async function openFile(path: string): Promise<void> {
    statusEl.textContent = "";

    try {
      const content = await readMarkdownFile(path);
      preview.setContent(content);
      currentFile = { name: fileNameFromPath(path), path };
      updateSidebar();
    } catch (error) {
      statusEl.textContent = String(error);
      return;
    }

    try {
      await recordRecentFile(path);
    } catch (error) {
      statusEl.textContent = String(error);
    }
  }

  openButton.addEventListener("click", async () => {
    statusEl.textContent = "";
    openButton.disabled = true;

    try {
      const path = await openMarkdownFile();
      if (path === null) {
        return;
      }
      await openFile(path);
    } catch (error) {
      statusEl.textContent = String(error);
    } finally {
      openButton.disabled = false;
    }
  });

  readRecentFiles()
    .then((entries) => {
      recentFiles = entries;
      updateSidebar();
    })
    .catch((error) => {
      statusEl.textContent = String(error);
    });
});
