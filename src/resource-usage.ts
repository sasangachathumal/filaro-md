import { invoke } from "@tauri-apps/api/core";

/** A point-in-time sample of the app process's own CPU and memory use. */
export interface ResourceUsage {
  cpuPercent: number;
  memoryPercent: number;
  memoryBytes: number;
}

export function readResourceUsage(): Promise<ResourceUsage> {
  return invoke<ResourceUsage>("read_resource_usage");
}
