/// NVLink view of a tray.
#[derive(Debug)]
pub struct NvlTray {
    /// GPUs visible to this tray via NVLink
    gpu_count: u32,
}

impl NvlTray {
    /// Construct from GPU count.
    pub fn new(gpu_count: u32) -> Self {
        Self { gpu_count }
    }
}