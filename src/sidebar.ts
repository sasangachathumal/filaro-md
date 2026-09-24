import { displayLocation } from "./path-display";

/** The file currently shown in the preview pane. */
export interface CurrentFile {
  name: string;
  path: string;
}

/** One "Recent" list entry. `exists` reflects a point-in-time disk check. */
export interface RecentFileEntry {
  name: string;
  path: string;
  exists: boolean;
}

export interface SidebarProps {
  currentFile: CurrentFile | null;
  recentFiles: RecentFileEntry[];
  /** Used to shorten locations to e.g. "Downloads"; `null` shows full folder paths. */
  homeDir: string | null;
  onSelectRecent: (path: string) => void;
}

/**
 * Renders the "Now previewing" / "Recent" sidebar into a container element.
 * Takes its data as props and a selection callback — it has no knowledge
 * of file paths beyond what it's given, and calls no Tauri commands.
 */
export class Sidebar {
  private container: HTMLElement;
  private props: SidebarProps;

  constructor(container: HTMLElement, props: SidebarProps) {
    this.container = container;
    this.props = props;
    this.render();
  }

  setProps(props: SidebarProps): void {
    this.props = props;
    this.render();
  }

  private render(): void {
    this.container.innerHTML = "";
    this.container.appendChild(this.renderNowPreviewing());
    this.container.appendChild(this.renderRecent());
  }

  private renderNowPreviewing(): HTMLElement {
    const section = document.createElement("section");
    section.className = "sidebar-section";
    section.appendChild(this.renderHeading("Now previewing"));

    const { currentFile } = this.props;
    if (!currentFile) {
      section.appendChild(this.renderEmptyState("No file open"));
      return section;
    }

    const item = document.createElement("div");
    item.className = "sidebar-item sidebar-item--current";
    item.title = currentFile.path;

    const name = document.createElement("span");
    name.className = "sidebar-item-name";
    name.textContent = currentFile.name;
    name.title = currentFile.name;
    item.appendChild(name);

    section.appendChild(item);
    return section;
  }

  private renderRecent(): HTMLElement {
    const section = document.createElement("section");
    section.className = "sidebar-section";
    section.appendChild(this.renderHeading("Recent"));

    const { recentFiles } = this.props;
    if (recentFiles.length === 0) {
      section.appendChild(this.renderEmptyState("No recent files"));
      return section;
    }

    const list = document.createElement("ul");
    list.className = "sidebar-recent-list";
    for (const entry of recentFiles) {
      list.appendChild(this.renderRecentItem(entry));
    }

    section.appendChild(list);
    return section;
  }

  private renderRecentItem(entry: RecentFileEntry): HTMLElement {
    const item = document.createElement("li");
    const isCurrent = this.props.currentFile?.path === entry.path;

    const content = document.createElement(entry.exists ? "button" : "div");
    content.className = "sidebar-item sidebar-item--recent";
    if (!entry.exists) {
      content.classList.add("sidebar-item--missing");
      content.setAttribute("aria-disabled", "true");
    }
    if (isCurrent) {
      content.classList.add("sidebar-item--current");
    }
    if (content instanceof HTMLButtonElement) {
      content.type = "button";
      content.addEventListener("click", () => this.props.onSelectRecent(entry.path));
    }

    // Name and location lines are each truncated with an ellipsis; hovering
    // the name shows it in full, hovering the location shows the full path.
    const name = document.createElement("span");
    name.className = "sidebar-item-name";
    name.textContent = entry.name;
    name.title = entry.name;
    content.appendChild(name);

    const path = document.createElement("span");
    path.className = "sidebar-item-path";
    path.textContent = displayLocation(entry.path, this.props.homeDir);
    path.title = entry.path;
    content.appendChild(path);

    if (!entry.exists) {
      const badge = document.createElement("span");
      badge.className = "sidebar-item-badge";
      badge.textContent = "missing";
      content.appendChild(badge);
    }

    item.appendChild(content);
    return item;
  }

  private renderHeading(text: string): HTMLElement {
    const heading = document.createElement("h2");
    heading.className = "sidebar-heading";
    heading.textContent = text;
    return heading;
  }

  private renderEmptyState(text: string): HTMLElement {
    const empty = document.createElement("p");
    empty.className = "sidebar-empty";
    empty.textContent = text;
    return empty;
  }
}
