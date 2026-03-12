/// Per-tray InfiniBand view.
#[derive(Debug)]
pub struct IbTray {
    /// Total IB interfaces on this tray.
    port_count: u32,
    /// IB interfaces with active LID (not 0 or 0xffff).
    active_port_count: u32,
}

impl IbTray {
    /// Construct from port counts.
    pub fn new(port_count: u32, active_port_count: u32) -> Self {
        Self {
            port_count,
            active_port_count,
        }
    }
}