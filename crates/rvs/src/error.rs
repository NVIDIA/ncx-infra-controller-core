use thiserror::Error;

/// Top-level RVS error type.
#[derive(Debug, Error)]
pub enum RvsError {
    /// gRPC call to BMMC failed.
    #[error("BMMC RPC error: {0}")]
    Rpc(#[from] tonic::Status),

    /// Tray ID string couldn't be parsed as MachineId.
    #[error("invalid machine ID: {0}")]
    InvalidMachineId(String),
}