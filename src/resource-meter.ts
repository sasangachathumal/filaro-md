/** A usage sample, or `null` when one couldn't be read. */
export interface ResourceSample {
  cpuPercent: number;
  memoryPercent: number;
  memoryBytes: number;
}

export interface ResourceMeterProps {
  sample: ResourceSample | null;
}

type Level = "ok" | "warn" | "high" | "unknown";

const WARN_AT_PERCENT = 50;
const HIGH_AT_PERCENT = 80;

const SCOPE_NOTE =
  "Measures Filaro's main process only. The webview renders in separate system processes that aren't included.";

function levelFor(percent: number): Level {
  if (percent >= HIGH_AT_PERCENT) return "high";
  if (percent >= WARN_AT_PERCENT) return "warn";
  return "ok";
}

function formatMegabytes(bytes: number): string {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Renders compact CPU / RAM indicators, colored green, yellow, or red by
 * usage level. Takes a sample as props and calls no Tauri commands.
 */
export class ResourceMeter {
  private container: HTMLElement;

  constructor(container: HTMLElement, props: ResourceMeterProps) {
    this.container = container;
    this.setProps(props);
  }

  setProps({ sample }: ResourceMeterProps): void {
    const heading = document.createElement("h2");
    heading.className = "resource-heading";
    heading.textContent = "App resource usage";

    const meter = document.createElement("div");
    meter.className = "resource-meter";

    if (sample) {
      meter.appendChild(
        this.renderIndicator("CPU", sample.cpuPercent, `CPU: ${sample.cpuPercent.toFixed(1)}% of total capacity`),
      );
      meter.appendChild(
        this.renderIndicator(
          "RAM",
          sample.memoryPercent,
          `Memory: ${formatMegabytes(sample.memoryBytes)} (${sample.memoryPercent.toFixed(1)}% of system RAM)`,
        ),
      );
    } else {
      meter.appendChild(this.renderIndicator("CPU", null, "CPU usage unavailable"));
      meter.appendChild(this.renderIndicator("RAM", null, "Memory usage unavailable"));
    }

    this.container.replaceChildren(heading, meter);
  }

  private renderIndicator(label: string, percent: number | null, detail: string): HTMLElement {
    const indicator = document.createElement("div");
    const level = percent === null ? "unknown" : levelFor(percent);
    indicator.className = `resource-indicator resource-indicator--${level}`;
    indicator.title = `${detail}\n${SCOPE_NOTE}`;

    const dot = document.createElement("span");
    dot.className = "resource-dot";
    indicator.appendChild(dot);

    const name = document.createElement("span");
    name.className = "resource-label";
    name.textContent = label;
    indicator.appendChild(name);

    const value = document.createElement("span");
    value.className = "resource-value";
    value.textContent = percent === null ? "—" : `${percent.toFixed(1)}%`;
    indicator.appendChild(value);

    return indicator;
  }
}
