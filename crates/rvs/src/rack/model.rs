use std::collections::HashMap;

use crate::partitions::IbTray;
use crate::partitions::NvlTray;

/// Resolved compute tray with partition-specific views.
#[derive(Debug)]
pub struct Tray {
    /// NVLink perspective of this tray.
    nvl: NvlTray,
    /// InfiniBand perspective of this tray.
    ib: IbTray,
}

impl Tray {
    /// Construct from partition-specific views.
    pub fn new(nvl: NvlTray, ib: IbTray) -> Self {
        Self { nvl, ib }
    }
}

/// Fully resolved rack: trays grouped by partition type.
#[derive(Debug)]
pub struct Rack {
    /// NVL domain UUID -> tray IDs in that domain.
    nvl: HashMap<String, Vec<String>>,
    /// IB fabric ID -> tray IDs on that fabric.
    ib: HashMap<String, Vec<String>>,
    /// Tray ID -> resolved tray data.
    all: HashMap<String, Tray>,
}

impl Rack {
    /// Construct from pre-built partition maps and tray lookup.
    pub fn new(
        nvl: HashMap<String, Vec<String>>,
        ib: HashMap<String, Vec<String>>,
        all: HashMap<String, Tray>,
    ) -> Self {
        Self { nvl, ib, all }
    }

    /// NVL domain partitions.
    pub fn nvl(&self) -> &HashMap<String, Vec<String>> {
        &self.nvl
    }

    /// IB fabric partitions.
    pub fn ib(&self) -> &HashMap<String, Vec<String>> {
        &self.ib
    }

    /// All trays keyed by tray ID.
    pub fn all(&self) -> &HashMap<String, Tray> {
        &self.all
    }
}

/// Collection of resolved racks.
#[derive(Debug)]
pub struct Racks {
    /// Rack ID -> resolved rack.
    inner: HashMap<String, Rack>,
}

impl Racks {
    /// Construct from pre-built rack map.
    pub fn new(inner: HashMap<String, Rack>) -> Self {
        Self { inner }
    }

    /// All racks keyed by rack ID.
    pub fn inner(&self) -> &HashMap<String, Rack> {
        &self.inner
    }
}