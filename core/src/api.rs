use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug};
use crate::http_api::SupportedHttpMethods;
use anyhow::{Result};
use crate::dtos::responses::telemetry_snapshot::TelemetrySnapshot;

#[async_trait]
pub trait DroneApi: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn arm(&mut self) -> Result<()>;
    async fn takeoff(&mut self, altitude_m: f32) -> Result<()>;
    async fn land(&mut self) -> Result<()>;
    async fn status(&mut self) -> Result<TelemetrySnapshot>;
    async fn get<T>(&mut self, endpoint: &String) -> Result<Option<T>> where T: DeserializeOwned + Serialize + Debug;
    async fn post<T, B>(&mut self, endpoint: &String, body: Option<&B>) -> anyhow::Result<T> where T: DeserializeOwned + Serialize + Debug + Clone, B: Serialize + ?Sized + Sync;
    async fn get_with_headers<T>(&mut self, endpoint: &str, extra: Option<HeaderMap>) -> Result<Option<T>> where T: DeserializeOwned + Serialize + Debug;
    async fn post_with_headers<T, B>(
        &mut self,
        endpoint: &str,
        body: Option<&B>,
        extra: Option<HeaderMap>
    ) -> anyhow::Result<T>
    where
        T: DeserializeOwned + Serialize + Debug + Clone,
        B: Serialize + ?Sized + Sync;
    
    async fn _send_request<T, B>(
        &mut self,
        endpoint: &str,
        http_method: SupportedHttpMethods,
        body: Option<&B>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized + Sync;
    
    async fn _get_request_by_http<T>(
        &mut self,
        endpoint: &str,
        extra: Option<HeaderMap>,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned;

    async fn _post_request_by_http<T, B>(
        &mut self,
        endpoint: &str,
        body: Option<&B>,
        extra: Option<HeaderMap>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized + Sync;

    fn retrieve_headers(&mut self, token: &String) -> anyhow::Result<HeaderMap>;
    fn get_my_token(&mut self) -> String;
}
