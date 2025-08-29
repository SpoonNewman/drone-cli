use async_trait::async_trait;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug};
// use crate::{Result, DroneError};
use crate::dtos::responses::telemetry_snapshot::TelemetrySnapshot;
use crate::dtos::requests::drone_requests::DroneLandRequestDTO;
use super::api::DroneApi;

pub enum SupportedHttpMethods {
    Get,
    Post
}

pub struct HttpDroneApi {
    client: Client,
    base: String, // e.g. "http://192.168.1.10:8080"
}

impl HttpDroneApi {
    pub fn new(base: impl Into<String>) -> Self {
        Self { client: Client::new(), base: base.into() }
    }
    fn url(&self, p: &str) -> String { format!("{}/{}", self.base.trim_end_matches('/'), p.trim_start_matches('/')) }
}

#[async_trait]
impl DroneApi for HttpDroneApi {
    async fn connect(&mut self) -> Result<()> {
        // optional health check
        let resp = self.client.get(self.url("health")).send().await
            .map_err(|e| DroneError::Connect(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(DroneError::Connect(format!("health: {}", resp.status())));
        }
        Ok(())
    }

    async fn arm(&mut self) -> Result<()> {
        self.client.post(self.url("commands/arm")).send().await
            .map_err(|e| DroneError::Io(e.to_string()))?
            .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn takeoff(&mut self, altitude_m: f32) -> Result<()> {
        self.client.post(self.url("commands/takeoff"))
            .json(&serde_json::json!({ "altitude_m": altitude_m }))
            .send().await
            .map_err(|e| DroneError::Io(e.to_string()))?
            .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn land(&mut self) -> Result<()> {
        self.client.post(self.url("commands/land")).send().await
            .map_err(|e| DroneError::Io(e.to_string()))?
            .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn status(&mut self) -> Result<TelemetrySnapshot> {
        let resp = self.client.get(self.url("status")).send().await
            .map_err(|e| DroneError::Io(e.to_string()))?
            .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        let snap = resp.json::<TelemetrySnapshot>().await
            .map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(snap)
    }
}
