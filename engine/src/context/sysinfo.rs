use macros::Get;

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
            cpu_model: utils::get_cpu_model().unwrap_or(String::from("Unknown model")),
            cpu_cores: utils::get_cpu_cores(),
            mem_total: utils::get_total_mem().unwrap_or(0),
            gpu_model: gpu_info.name,
            gpu_type: gpu_info.device_type,
            gpu_backend: gpu_info.backend.to_string(),
            gpu_driver: gpu_info.driver_info,
        }
    }
}
