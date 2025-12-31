use macros::Get;

pub fn get_cpu_model() -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string("/proc/cpuinfo")?;

    for line in content.lines() {
        if line.starts_with("model name") {
            if let Some(model) = line.split(':').nth(1) {
                return Ok(model.trim().to_string());
            }
        }
    }

    Ok("Unknown".to_string())
}

pub fn get_cpu_cores() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

pub fn get_total_mem() -> Result<u64, std::io::Error> {
    let content = std::fs::read_to_string("/proc/meminfo")?;

    for line in content.lines() {
        if line.starts_with("MemTotal") {
            if let Some(mem_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = mem_str.parse::<u64>() {
                    return Ok(kb * 1024);
                }
            }
        }
    }

    Ok(0)
}

/// Static System Information
///
/// This struct will never change, and is used to store information about the system.
/// To see dynamic information, such as cpu usage, memory usage, see [`globals::profiling`]
#[derive(Debug, Clone)]
#[derive(Get)]
pub struct SystemInfo {
    #[get(ty = &str)]
    cpu_model: String,

    #[get(copied)]
    cpu_cores: usize,

    #[get(copied)]
    mem_total: u64,

    #[get(ty = &str)]
    gpu_model: String,

    #[get(copied)]
    gpu_type: wgpu::DeviceType,

    #[get(ty = &str)]
    gpu_backend: String,

    #[get(ty = &str)]
    gpu_driver: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        let gpu_info = gpu::adapter().get_info();

        Self {
            cpu_model: get_cpu_model().unwrap_or(String::from("Unknown model")),
            cpu_cores: get_cpu_cores(),
            mem_total: get_total_mem().unwrap_or(0),
            gpu_model: gpu_info.name,
            gpu_type: gpu_info.device_type,
            gpu_backend: gpu_info.backend.to_string(),
            gpu_driver: gpu_info.driver_info,
        }
    }
}
