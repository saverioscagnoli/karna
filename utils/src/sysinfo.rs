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
