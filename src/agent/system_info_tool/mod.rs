use adk_rust::prelude::*;
use adk_tool::tool;
use schemars::JsonSchema;
use serde_json::{json, Value};
use std::sync::Arc;
use sysinfo::{System, Disks};

// ─── Tools ────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, JsonSchema)]
struct NoArgs {}

/// Retrieves system information including CPU usage, memory stats, and disk space.
#[tool]
async fn get_system_stats(_args: NoArgs) -> std::result::Result<Value, AdkError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU information
    let cpu_count = sys.cpus().len();
    let global_cpu_usage = sys.global_cpu_usage();
    
    // Memory information
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    // Disk information
    let disks = Disks::new_with_refreshed_list();
    let mut disk_info = Vec::new();
    for disk in &disks {
        disk_info.push(json!({
            "name": disk.name().to_string_lossy(),
            "mount_point": disk.mount_point().to_string_lossy(),
            "total_space_gb": disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0,
            "available_space_gb": disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
            "is_removable": disk.is_removable(),
        }));
    }

    Ok(json!({
        "cpu": {
            "count": cpu_count,
            "global_usage_percent": global_cpu_usage,
        },
        "memory": {
            "total_gb": total_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": used_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_percent": (used_memory as f64 / total_memory as f64) * 100.0,
        },
        "swap": {
            "total_gb": total_swap as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": used_swap as f64 / 1024.0 / 1024.0 / 1024.0,
        },
        "disks": disk_info,
        "system_name": System::name(),
        "kernel_version": System::kernel_version(),
        "os_version": System::os_version(),
        "host_name": System::host_name(),
    }))
}

// ─── Registration ─────────────────────────────────────────────────────────────

pub fn system_info_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(GetSystemStats),
    ]
}
