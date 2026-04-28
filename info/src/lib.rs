use sysinfo::CpuRefreshKind;
use sysinfo::RefreshKind;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub cpu_cores: u64,
    pub cpu_model: String,
    pub cpu_freq: u64,
    pub mem_total: u64,
    pub gpu_model: String,
    pub graphics_backend: String,
    pub graphics_driver: String,
}

impl SystemInfo {
    pub fn fetch(gpu_info: wgpu::AdapterInfo) -> Self {
        let sys = System::new_all();
        let cpu = sys.cpus().first().expect("No cpu wtf");

        Self {
            cpu_cores: sys.cpus().len() as u64,
            cpu_model: cpu.brand().to_string(),
            cpu_freq: cpu.frequency(),
            mem_total: sys.total_memory(),
            gpu_model: gpu_info.name,
            graphics_backend: gpu_info.backend.to_string(),
            graphics_driver: gpu_info.driver_info,
        }
    }
}
