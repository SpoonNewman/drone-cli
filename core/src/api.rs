use async_trait::async_trait;
use crate::{Result};
use crate::dtos::responses::telemetry_snapshot::TelemetrySnapshot;

#[async_trait]
pub trait DroneApi: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn arm(&mut self) -> Result<()>;
    async fn takeoff(&mut self, altitude_m: f32) -> Result<()>;
    async fn land(&mut self) -> Result<()>;
    async fn status(&mut self) -> Result<TelemetrySnapshot>;
}
