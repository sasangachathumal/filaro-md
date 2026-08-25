import { openMarkdownFile, readMarkdownFile } from "./file-access";
import { MarkdownPreview } from "./markdown-preview";

window.addEventListener("DOMContentLoaded", () => {
  const openButton = document.querySelector<HTMLButtonElement>("#open-file-button")!;
  const statusEl = document.querySelector<HTMLElement>("#status")!;
  const previewRoot = document.querySelector<HTMLElement>("#preview-root")!;

  const preview = new MarkdownPreview(previewRoot);

  openButton.addEventListener("click", async () => {
    statusEl.textContent = "";
    openButton.disabled = true;

    try {
      const path = await openMarkdownFile();
      if (path === null) {
        return;
      }
      const content = await readMarkdownFile(path);
      preview.setContent(content);
    } catch (error) {
      statusEl.textContent = String(error);
    } finally {
      openButton.disabled = false;
    }
  });
});
