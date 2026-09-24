import {
  getHomeDir,
  openMarkdownFile,
  readMarkdownFile,
  readRecentFiles,
  writeRecentFiles,
} from "./file-access";
import { MarkdownPreview } from "./markdown-preview";
import { ResourceMeter } from "./resource-meter";
import { readResourceUsage } from "./resource-usage";
import { Sidebar, type CurrentFile, type RecentFileEntry, type SidebarProps } from "./sidebar";

const RESOURCE_POLL_INTERVAL_MS = 2000;

function fileNameFromPath(path: string): string {
  const segments = path.split(/[\\/]/);
  return segments[segments.length - 1] || path;
}

window.addEventListener("DOMContentLoaded", () => {
  const openButton = document.querySelector<HTMLButtonElement>("#open-file-button")!;
  const statusEl = document.querySelector<HTMLElement>("#status")!;
  const previewRoot = document.querySelector<HTMLElement>("#preview-root")!;
  const sidebarRoot = document.querySelector<HTMLElement>("#sidebar-root")!;
  const resourceRoot = document.querySelector<HTMLElement>("#resource-root")!;

  const preview = new MarkdownPreview(previewRoot);
  const resourceMeter = new ResourceMeter(resourceRoot, { sample: null });

  let currentFile: CurrentFile | null = null;
  let recentFiles: RecentFileEntry[] = [];
  let homeDir: string | null = null;

  function sidebarProps(): SidebarProps {
    return {
      currentFile,
      recentFiles,
      homeDir,
      onSelectRecent: (path) => void openFile(path),
    };
  }

  const sidebar = new Sidebar(sidebarRoot, sidebarProps());

  function updateSidebar(): void {
    sidebar.setProps(sidebarProps());
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

  // Without the home folder, locations just fall back to full folder paths.
  getHomeDir()
    .then((dir) => {
      homeDir = dir;
      updateSidebar();
    })
    .catch(() => {});

  // A failed sample just shows as unavailable ("—") rather than filling
  // the status line with a repeating error every poll.
  async function refreshResourceUsage(): Promise<void> {
    try {
      resourceMeter.setProps({ sample: await readResourceUsage() });
    } catch {
      resourceMeter.setProps({ sample: null });
    }
  }

  void refreshResourceUsage();
  window.setInterval(() => void refreshResourceUsage(), RESOURCE_POLL_INTERVAL_MS);
});
