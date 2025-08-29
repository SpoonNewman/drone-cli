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
        // // optional health check
        // let resp = self.client.get(self.url("health")).send().await
        //     .map_err(|e| DroneError::Connect(e.to_string()))?;
        // if !resp.status().is_success() {
        //     // return Err("health: {}", resp.status());
        //     eprintln!("health: {}", resp.status());
        // }
        Ok(())
    }

    async fn arm(&mut self) -> Result<()> {
        // self.client.post(self.url("commands/arm")).send().await
        //     .map_err(|e| DroneError::Io(e.to_string()))?
        //     .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn takeoff(&mut self, altitude_m: f32) -> Result<()> {
        // TODO: Start here! This needs built out similar to how `land` and `status` are
        // - Pay attention to post or get. Don't use methods with the `_` preceding it,
        // this are "under the hood" methods.
        // - Pay attention to the JSON in the commented out code, that will be your
        // post `body` but it needs to be represented as a DTO similar to `DroneLandRequestDTO`
        // instead of just raw json like below. So that means go build the DTO and use it.
        // - In the case of `DroneLandRequestDTO` there weren't any properties but we still needed
        // a body because it's a POST request and that's required.
        // - Additionally we need to assume that `connect()` is the only command that doesn't require
        // a token in the headers.

        // self.client.post(self.url("commands/takeoff"))
        //     .json(&serde_json::json!({ "altitude_m": altitude_m }))
        //     .send().await
        //     .map_err(|e| DroneError::Io(e.to_string()))?
        //     .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        Ok(())
    }

    async fn land(&mut self) -> Result<()> {
        // self.client.post(self.url("commands/land")).send().await
        //     .map_err(|e| DroneError::Io(e.to_string()))?
        //     .error_for_status().map_err(|e| DroneError::Protocol(e.to_string()))?;
        let endpoint = String::from("commands/land");
        let body = DroneLandRequestDTO::new()?;
        let token: String = self.get_my_token();
        let headers = self.retrieve_headers(&token)?;
        let result = self.post_with_headers(&endpoint, Some(&body), Some(headers)).await?;
        match result {
            Some(res) => {
                Ok(res)
            }

            None => {
                bail!("The server is unreachable!");
            }
        }
    }

    async fn status(&mut self) -> Result<TelemetrySnapshot> {
        // FIXME: The status endpoint requires a token in the headers but we're currently not sending that.
        // Fix this so it's sending headers with the token. Look above ^^^
        let endpoint = String::from("status");
        let result = self.get(&endpoint).await?;
        match result {
            Some(res) => {
                Ok(res)
            }

            None => {
                bail!("The server is unreachable!");
            }
        }
    }

    async fn post<T, B>(
        &mut self, 
        endpoint: &String, 
        body: Option<&B>
    ) -> anyhow::Result<T> 
    where 
        T: DeserializeOwned + Serialize + Debug + Clone, 
        B: Serialize + ?Sized + Sync 
    {
        self.post_with_headers(endpoint, body, None).await
    }

    async fn get<T>(&mut self, endpoint: &String) -> Result<Option<T>>
    where
        T: DeserializeOwned + Serialize + Debug
    {
        self.get_with_headers::<T>(endpoint, None).await
    }

    async fn get_with_headers<T>(
        &mut self,
        endpoint: &str,
        extra: Option<HeaderMap>
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned + Serialize + Debug
    {
        let result = self
            ._send_request::<T, ()>(endpoint, SupportedHttpMethods::Get, None, extra)
            .await;

        result
    }

    async fn post_with_headers<T, B>(
        &mut self,
        endpoint: &str,
        body: Option<&B>,
        extra: Option<HeaderMap>
    ) -> anyhow::Result<T>
    where
        T: DeserializeOwned + Serialize + Debug + Clone,
        B: Serialize + ?Sized + Sync,
    {
        let wrapped = self
            ._send_request::<T, B>(endpoint, SupportedHttpMethods::Post, body, extra)
            .await?
            .expect("POST endpoints must return a body");
    
        Ok(wrapped)
    }

    async fn _send_request<T, B>(
        &mut self,
        endpoint: &str,
        http_method: SupportedHttpMethods,
        body: Option<&B>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized + Sync,
    {
        match http_method {
            SupportedHttpMethods::Get => {
                self._get_request_by_http::<T>(endpoint, extra_headers).await
            }
    
            SupportedHttpMethods::Post => {
                let value = self
                    ._post_request_by_http::<T, B>(endpoint, body, extra_headers)
                    .await?;
                Ok(Some(value))         // ← no semicolon here
            }
        }
    }

    async fn _get_request_by_http<T>(
        &mut self,
        endpoint: &str,
        extra: Option<HeaderMap>,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/{}", self.base, endpoint);
        let mut req = self.client.get(url);
    
        if let Some(h) = extra {
            req = req.headers(h);
        }

        let resp = req.send().await?;

        /* ========== 204 No Content ========== */
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        /* ========== error branch ========== */
        if !resp.status().is_success() {
            let status = resp.status();
            let text   = resp.text().await?;

            eprintln!("HTTP {} – raw body: {}", status, text);
            bail!("HTTP {}: {}", status, text);
        }

        /* ========== success branch ========== */
        let value = resp.json::<T>().await?;
        Ok(Some(value))
    }

    async fn _post_request_by_http<T, B>(
        &mut self,
        endpoint: &str,
        body: Option<&B>,
        extra: Option<HeaderMap>,
    ) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized + Sync,
    {
        let url = format!("{}/{}", self.base, endpoint);
        let mut req = self.client.post(url);
    
        if let Some(h) = extra {
            req = req.headers(h);
        }

        if let Some(payload) = body {
            req = req.json(payload);
        }
    
        let resp = req.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text   = resp.text().await?;

            eprintln!("HTTP {} – raw body: {}", status, text);
            bail!("HTTP {}: {}", status, text);
        }

        /* ----------- success branch ----------- */
        let value = resp.json::<T>().await?;
        Ok(value)
    }

    fn retrieve_headers(&mut self, token: &String) -> anyhow::Result<HeaderMap> {
        let mut hdr: HeaderMap =  HeaderMap::new();
        hdr.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?
        );
        Ok((hdr))
    }

    // TODO: This needs to read from somewhere, probably a static JSON file unless there's an envvar
    // that has the token. There are some "Rust" ways of dealing with this.
    fn get_my_token(&mut self) -> String {
        return String::from("MY BADASS TOKEN")
    }
}
