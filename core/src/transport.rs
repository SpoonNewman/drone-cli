use crate::{DroneError, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn send(&mut self, bytes: &[u8]) -> Result<()>;
    async fn recv(&mut self) -> Result<Vec<u8>>;
}

/// A placeholder transport to help you test the CLI without hardware.
pub struct FakeTransport {
    connected: bool,
}

impl FakeTransport {
    pub fn new() -> Self { Self { connected: false } }
}

#[async_trait]
impl Transport for FakeTransport {
    async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn send(&mut self, bytes: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(DroneError::Connect("not connected".into()));
        }
        // In a real impl, write to UDP/serial here.
        if bytes.is_empty() {
            return Err(DroneError::Protocol("empty frame".into()));
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(DroneError::Connect("not connected".into()));
        }
        Ok(b"OK".to_vec())
    }
}
