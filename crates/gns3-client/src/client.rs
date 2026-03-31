//! HTTP client implementing [`Gns3Api`] for a real GNS3 server.

use std::time::Duration;

use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use gns3_mcp_core::{
    AddDrawingRequest, Compute, CreateNodeRequest, Drawing, ExportResult, Gns3Error, Link,
    LinkEndpoint, Node, Project, Snapshot, SwitchPort, Template, UpdateNodeRequest, Version,
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError},
};

use crate::config::Gns3ClientConfig;

/// Maximum number of retry attempts for 5xx server errors.
const MAX_RETRIES: u32 = 3;

/// Base delay in milliseconds for exponential backoff.
const BACKOFF_BASE_MS: u64 = 100;

/// HTTP client for the GNS3 REST API v2.
///
/// Created via [`Gns3Client::new`] with a [`Gns3ClientConfig`].
/// Implements [`gns3_mcp_core::Gns3Api`] so it can be injected
/// into the MCP server as `Arc<dyn Gns3Api>`.
///
/// All outbound API calls are protected by a [`CircuitBreaker`] to prevent
/// cascading failures when the GNS3 server becomes unavailable.
pub struct Gns3Client {
    http: Client,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    circuit_breaker: CircuitBreaker,
}

impl Gns3Client {
    /// Create a new client from the given configuration.
    ///
    /// The circuit breaker is initialized with default configuration:
    /// failure threshold of 5 consecutive failures and a 30-second recovery timeout.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::Config`] if the HTTP client cannot be built.
    pub fn new(config: Gns3ClientConfig) -> Result<Self, Gns3Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| Gns3Error::Config(format!("failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: config.base_url,
            username: config.username,
            password: config.password,
            circuit_breaker: CircuitBreaker::with_defaults(),
        })
    }

    /// Create a new client with custom circuit breaker configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::Config`] if the HTTP client cannot be built.
    pub fn with_circuit_breaker(
        config: Gns3ClientConfig,
        cb_config: CircuitBreakerConfig,
    ) -> Result<Self, Gns3Error> {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| Gns3Error::Config(format!("failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: config.base_url,
            username: config.username,
            password: config.password,
            circuit_breaker: CircuitBreaker::new(cb_config),
        })
    }

    /// Build a URL for the given API path.
    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Apply basic auth if credentials are configured.
    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => req.basic_auth(user, Some(pass)),
            (Some(user), None) => req.basic_auth(user, None::<&str>),
            _ => req,
        }
    }

    /// Send a request and deserialize the JSON response.
    ///
    /// Retries up to [`MAX_RETRIES`] times on 5xx server errors using
    /// exponential backoff (100ms, 200ms, 400ms). Network errors and 4xx
    /// client errors are returned immediately without retrying.
    ///
    /// The entire retry loop is wrapped by the circuit breaker. Only transient
    /// failures (5xx after retries exhausted) and network errors count toward
    /// circuit breaker failure thresholds. 4xx client errors do not.
    ///
    /// The `make_req` closure is called once per attempt so that the
    /// [`RequestBuilder`] (which is consumed on send) can be reconstructed.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::CircuitOpen`] if the circuit breaker is open.
    async fn send<T, F>(&self, make_req: F) -> Result<T, Gns3Error>
    where
        T: DeserializeOwned,
        F: Fn() -> RequestBuilder + 'static,
    {
        self.circuit_breaker
            .call(|| async {
                let mut attempt = 0u32;
                loop {
                    let req = self.auth(make_req());
                    let response = req
                        .send()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    let status = response.status();

                    if status.is_server_error() && attempt < MAX_RETRIES {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        let delay_ms = BACKOFF_BASE_MS * (1u64 << attempt);
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = MAX_RETRIES,
                            status = status.as_u16(),
                            delay_ms,
                            "GNS3 server error — retrying after {delay_ms}ms: {message}"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        return Err(Gns3Error::Http {
                            status: status.as_u16(),
                            message,
                        });
                    }

                    return response
                        .json::<T>()
                        .await
                        .map_err(|e| Gns3Error::Deserialize(e.to_string()));
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Inner(err) => err,
                CircuitBreakerError::Open => Gns3Error::CircuitOpen,
                _ => Gns3Error::Server("unexpected circuit breaker state".into()),
            })
    }

    /// Send a POST and deserialize the response.
    ///
    /// If the response body is empty (e.g. 204 No Content), fall back to a
    /// GET on `get_fallback_url` to retrieve the current resource state.
    ///
    /// Retries the POST up to [`MAX_RETRIES`] times on 5xx errors with
    /// exponential backoff. The entire retry loop is wrapped by the circuit breaker.
    /// The GET fallback delegates to [`Gns3Client::send`] which applies circuit
    /// breaker protection independently.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::CircuitOpen`] if the circuit breaker is open.
    async fn post_with_get_fallback<T: DeserializeOwned>(
        &self,
        post_url: &str,
        get_fallback_url: &str,
    ) -> Result<T, Gns3Error> {
        self.circuit_breaker
            .call(|| async {
                let mut attempt = 0u32;
                loop {
                    let req = self.auth(self.http.post(post_url));
                    let response = req
                        .send()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    let status = response.status();

                    if status.is_server_error() && attempt < MAX_RETRIES {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        let delay_ms = BACKOFF_BASE_MS * (1u64 << attempt);
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = MAX_RETRIES,
                            status = status.as_u16(),
                            delay_ms,
                            "GNS3 server error — retrying after {delay_ms}ms: {message}"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        return Err(Gns3Error::Http {
                            status: status.as_u16(),
                            message,
                        });
                    }

                    let body = response
                        .text()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    if body.is_empty() {
                        let http = self.http.clone();
                        let fallback_url = get_fallback_url.to_string();
                        return self.send(move || http.get(&fallback_url)).await;
                    }

                    return serde_json::from_str::<T>(&body)
                        .map_err(|e| Gns3Error::Deserialize(e.to_string()));
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Inner(err) => err,
                CircuitBreakerError::Open => Gns3Error::CircuitOpen,
                _ => Gns3Error::Server("unexpected circuit breaker state".into()),
            })
    }

    /// POST to `post_url` (no body expected), then GET `get_list_url` to
    /// retrieve the current state as a `Vec<T>`.
    ///
    /// Used for bulk operations like start/stop all nodes that return
    /// 204 No Content and require a follow-up GET to observe the result.
    ///
    /// The entire operation is wrapped by the circuit breaker. The GET fallback
    /// delegates to [`Gns3Client::send`] which applies circuit breaker protection
    /// independently.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::CircuitOpen`] if the circuit breaker is open.
    async fn post_then_get_list<T: DeserializeOwned>(
        &self,
        post_url: &str,
        get_list_url: &str,
    ) -> Result<T, Gns3Error> {
        self.circuit_breaker
            .call(|| async {
                let mut attempt = 0u32;
                loop {
                    let req = self.auth(self.http.post(post_url));
                    let response = req
                        .send()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    let status = response.status();

                    if status.is_server_error() && attempt < MAX_RETRIES {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        let delay_ms = BACKOFF_BASE_MS * (1u64 << attempt);
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = MAX_RETRIES,
                            status = status.as_u16(),
                            delay_ms,
                            "GNS3 server error — retrying after {delay_ms}ms: {message}"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        return Err(Gns3Error::Http {
                            status: status.as_u16(),
                            message,
                        });
                    }

                    // Discard the body (204 or empty 200) and fetch the list.
                    let http = self.http.clone();
                    let fallback_url = get_list_url.to_string();
                    return self.send(move || http.get(&fallback_url)).await;
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Inner(err) => err,
                CircuitBreakerError::Open => Gns3Error::CircuitOpen,
                _ => Gns3Error::Server("unexpected circuit breaker state".into()),
            })
    }

    /// Send an authenticated DELETE request and verify success.
    ///
    /// Retries up to [`MAX_RETRIES`] times on 5xx server errors with
    /// exponential backoff. The entire retry loop is wrapped by the circuit breaker.
    /// 4xx errors do not count toward circuit breaker failure thresholds.
    ///
    /// # Errors
    ///
    /// Returns [`Gns3Error::CircuitOpen`] if the circuit breaker is open.
    async fn delete(&self, url: &str) -> Result<(), Gns3Error> {
        let url_owned = url.to_string();
        self.circuit_breaker
            .call(|| async {
                let mut attempt = 0u32;
                loop {
                    let req = self.auth(self.http.delete(&url_owned));
                    let response = req
                        .send()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    let status = response.status();

                    if status.is_server_error() && attempt < MAX_RETRIES {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        let delay_ms = BACKOFF_BASE_MS * (1u64 << attempt);
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = MAX_RETRIES,
                            status = status.as_u16(),
                            delay_ms,
                            "GNS3 server error — retrying after {delay_ms}ms: {message}"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        return Err(Gns3Error::Http {
                            status: status.as_u16(),
                            message,
                        });
                    }
                    return Ok(());
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Inner(err) => err,
                CircuitBreakerError::Open => Gns3Error::CircuitOpen,
                _ => Gns3Error::Server("unexpected circuit breaker state".into()),
            })
    }
}

#[async_trait::async_trait]
impl gns3_mcp_core::Gns3Api for Gns3Client {
    async fn get_version(&self) -> Result<Version, Gns3Error> {
        let http = self.http.clone();
        let url = self.url("/v2/version");
        self.send(move || http.get(&url)).await
    }

    async fn list_projects(&self) -> Result<Vec<Project>, Gns3Error> {
        let http = self.http.clone();
        let url = self.url("/v2/projects");
        self.send(move || http.get(&url)).await
    }

    async fn create_project(&self, name: &str) -> Result<Project, Gns3Error> {
        let http = self.http.clone();
        let url = self.url("/v2/projects");
        let body = serde_json::json!({ "name": name });
        self.send(move || http.post(&url).json(&body)).await
    }

    async fn open_project(&self, project_id: Uuid) -> Result<Project, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/open"));
        self.send(move || http.post(&url)).await
    }

    async fn list_templates(&self) -> Result<Vec<Template>, Gns3Error> {
        let http = self.http.clone();
        let url = self.url("/v2/templates");
        self.send(move || http.get(&url)).await
    }

    async fn create_node(
        &self,
        project_id: Uuid,
        template_id: Uuid,
        request: CreateNodeRequest,
    ) -> Result<Node, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!(
            "/v2/projects/{project_id}/templates/{template_id}"
        ));
        self.send(move || http.post(&url).json(&request)).await
    }

    async fn start_node(&self, project_id: Uuid, node_id: Uuid) -> Result<Node, Gns3Error> {
        // GNS3 start endpoint may return 200 with body or 204 with no body.
        self.post_with_get_fallback(
            &self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}/start")),
            &self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}")),
        )
        .await
    }

    async fn create_link(
        &self,
        project_id: Uuid,
        endpoints: Vec<LinkEndpoint>,
    ) -> Result<Link, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/links"));
        let body = serde_json::json!({ "nodes": endpoints });
        self.send(move || http.post(&url).json(&body)).await
    }

    async fn list_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/nodes"));
        self.send(move || http.get(&url)).await
    }

    async fn stop_node(&self, project_id: Uuid, node_id: Uuid) -> Result<Node, Gns3Error> {
        // GNS3 stop endpoint may return 200 with body or 204 with no body.
        self.post_with_get_fallback(
            &self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}/stop")),
            &self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}")),
        )
        .await
    }

    async fn delete_node(&self, project_id: Uuid, node_id: Uuid) -> Result<(), Gns3Error> {
        self.delete(&self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}")))
            .await
    }

    async fn list_links(&self, project_id: Uuid) -> Result<Vec<Link>, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/links"));
        self.send(move || http.get(&url)).await
    }

    async fn delete_link(&self, project_id: Uuid, link_id: Uuid) -> Result<(), Gns3Error> {
        self.delete(&self.url(&format!("/v2/projects/{project_id}/links/{link_id}")))
            .await
    }

    async fn close_project(&self, project_id: Uuid) -> Result<Project, Gns3Error> {
        // GNS3 returns 204 No Content on close — fall back to GET for project state.
        self.post_with_get_fallback(
            &self.url(&format!("/v2/projects/{project_id}/close")),
            &self.url(&format!("/v2/projects/{project_id}")),
        )
        .await
    }

    async fn delete_project(&self, project_id: Uuid) -> Result<(), Gns3Error> {
        self.delete(&self.url(&format!("/v2/projects/{project_id}")))
            .await
    }

    async fn list_computes(&self) -> Result<Vec<Compute>, Gns3Error> {
        let http = self.http.clone();
        let url = self.url("/v2/computes");
        self.send(move || http.get(&url)).await
    }

    async fn start_all_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
        self.post_then_get_list(
            &self.url(&format!("/v2/projects/{project_id}/nodes/start")),
            &self.url(&format!("/v2/projects/{project_id}/nodes")),
        )
        .await
    }

    async fn stop_all_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
        self.post_then_get_list(
            &self.url(&format!("/v2/projects/{project_id}/nodes/stop")),
            &self.url(&format!("/v2/projects/{project_id}/nodes")),
        )
        .await
    }

    async fn update_node(
        &self,
        project_id: Uuid,
        node_id: Uuid,
        update: UpdateNodeRequest,
    ) -> Result<Node, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}"));
        self.send(move || http.put(&url).json(&update)).await
    }

    async fn update_template(
        &self,
        template_id: Uuid,
        properties: serde_json::Value,
    ) -> Result<serde_json::Value, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/templates/{template_id}"));
        self.send(move || http.put(&url).json(&properties)).await
    }

    async fn add_drawing(
        &self,
        project_id: Uuid,
        request: AddDrawingRequest,
    ) -> Result<Drawing, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/drawings"));
        self.send(move || http.post(&url).json(&request)).await
    }

    async fn export_project(
        &self,
        project_id: Uuid,
        include_images: bool,
    ) -> Result<ExportResult, Gns3Error> {
        let http = self.http.clone();
        let query_param = if include_images { "yes" } else { "no" };
        let url = self.url(&format!(
            "/v2/projects/{project_id}/export?include_images={query_param}"
        ));
        self.circuit_breaker
            .call(|| async {
                let mut attempt = 0u32;
                loop {
                    let req = self.auth(http.get(&url));
                    let response = req
                        .send()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;

                    let status = response.status();

                    if status.is_server_error() && attempt < MAX_RETRIES {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        let delay_ms = BACKOFF_BASE_MS * (1u64 << attempt);
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = MAX_RETRIES,
                            status = status.as_u16(),
                            delay_ms,
                            "GNS3 server error — retrying after {delay_ms}ms: {message}"
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                        continue;
                    }

                    if !status.is_success() {
                        let message = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "no response body".to_string());
                        return Err(Gns3Error::Http {
                            status: status.as_u16(),
                            message,
                        });
                    }

                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|e| Gns3Error::Network(e.to_string()))?;
                    return Ok(ExportResult {
                        size_bytes: bytes.len(),
                    });
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Inner(err) => err,
                CircuitBreakerError::Open => Gns3Error::CircuitOpen,
                _ => Gns3Error::Server("unexpected circuit breaker state".into()),
            })
    }

    async fn configure_switch(
        &self,
        project_id: Uuid,
        node_id: Uuid,
        ports: Vec<SwitchPort>,
    ) -> Result<Node, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/nodes/{node_id}"));
        let body = serde_json::json!({
            "properties": {
                "ports_mapping": ports
            }
        });
        self.send(move || http.put(&url).json(&body)).await
    }

    async fn snapshot_project(
        &self,
        project_id: Uuid,
        name: &str,
    ) -> Result<Snapshot, Gns3Error> {
        let http = self.http.clone();
        let url = self.url(&format!("/v2/projects/{project_id}/snapshots"));
        let body = serde_json::json!({ "name": name });
        self.send(move || http.post(&url).json(&body)).await
    }
}
