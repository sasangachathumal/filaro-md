//! Samples the app's own CPU and memory usage for the sidebar indicator.
//!
//! Only the app's core process is measured. The webview renders in separate
//! OS processes (WebKit's WebContent on macOS, msedgewebview2 on Windows)
//! that aren't counted here, so this understates the app's total footprint.

use std::fmt;
use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Long-lived sampler kept in Tauri state. CPU usage is computed from the
/// time elapsed between two refreshes, so the same `System` has to be
/// reused across calls rather than created per request.
pub struct ResourceMonitor {
    system: Mutex<System>,
    pid: Pid,
    cpu_count: f32,
    total_memory: u64,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        let pid = Pid::from_u32(std::process::id());
        let mut system = System::new();
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        // Prime the CPU baseline so the first real sample isn't always 0%.
        refresh_own_process(&mut system, pid);

        Self {
            total_memory: system.total_memory(),
            system: Mutex::new(system),
            pid,
            cpu_count: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1) as f32,
        }
    }
}

fn refresh_own_process(system: &mut System, pid: Pid) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    /// Share of the machine's total CPU capacity (0–100, across all cores).
    pub cpu_percent: f32,
    /// Resident memory as a share of total system RAM (0–100).
    pub memory_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug)]
pub enum ResourceUsageError {
    LockPoisoned,
    ProcessNotFound,
}

impl fmt::Display for ResourceUsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceUsageError::LockPoisoned => write!(f, "resource monitor is unavailable"),
            ResourceUsageError::ProcessNotFound => write!(f, "couldn't read the app's own process"),
        }
    }
}

impl std::error::Error for ResourceUsageError {}

impl serde::Serialize for ResourceUsageError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl ResourceMonitor {
    fn sample(&self) -> Result<ResourceUsage, ResourceUsageError> {
        let mut system = self
            .system
            .lock()
            .map_err(|_| ResourceUsageError::LockPoisoned)?;
        refresh_own_process(&mut system, self.pid);

        let process = system
            .process(self.pid)
            .ok_or(ResourceUsageError::ProcessNotFound)?;
        let memory_bytes = process.memory();
        let memory_percent = if self.total_memory == 0 {
            0.0
        } else {
            (memory_bytes as f64 / self.total_memory as f64 * 100.0) as f32
        };

        Ok(ResourceUsage {
            cpu_percent: process.cpu_usage() / self.cpu_count,
            memory_percent,
            memory_bytes,
        })
    }
}

#[tauri::command]
pub fn read_resource_usage(
    monitor: tauri::State<'_, ResourceMonitor>,
) -> Result<ResourceUsage, ResourceUsageError> {
    monitor.sample()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_own_process_within_bounds() {
        let monitor = ResourceMonitor::new();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        let usage = monitor.sample().unwrap();

        assert!(usage.memory_bytes > 0);
        assert!((0.0..=100.0).contains(&usage.memory_percent));
        assert!((0.0..=100.0).contains(&usage.cpu_percent));
    }
}
