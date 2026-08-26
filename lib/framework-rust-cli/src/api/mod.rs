#![allow(dead_code)]
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{}", friendly_http_error(.0))]
    Http(#[from] reqwest::Error),
    #[error("{message}")]
    Api { status: u16, message: String },
}

fn friendly_http_error(e: &reqwest::Error) -> String {
    if e.is_connect() {
        "Could not connect to the server. Check the URL and try again.".to_string()
    } else if e.is_timeout() {
        "Request timed out. The server may be busy.".to_string()
    } else {
        "Network error. Check your connection.".to_string()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    pub base_url: String,
    token: Option<String>,
    actor_urdd_id: Option<String>,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self {
            inner: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            base_url,
            token: None,
            actor_urdd_id: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn set_actor_context(&mut self, urdd_id: Option<String>) {
        self.actor_urdd_id = urdd_id;
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut map = reqwest::header::HeaderMap::new();
        if let Some(token) = &self.token {
            map.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {token}").parse().unwrap(),
            );
        }
        if let Some(urdd_id) = &self.actor_urdd_id {
            map.insert("x-framework-rust-urdd-id", urdd_id.parse().unwrap());
        }
        map
    }

    fn tenant_headers(&self, tenant_id: &str) -> reqwest::header::HeaderMap {
        let mut map = self.auth_headers();
        map.insert("x-framework-rust-tenant-id", tenant_id.parse().unwrap());
        map
    }

    pub async fn list_access_contexts(&self, tenant_id: &str) -> ApiResult<Vec<AccessContext>> {
        let resp = self
            .inner
            .get(format!("{}/users/me/access-contexts", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    async fn check<T: for<'de> Deserialize<'de>>(&self, resp: reqwest::Response) -> ApiResult<T> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<T>().await?)
        } else {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();

            let message = if let Some(fields) = body.get("fields").and_then(|f| f.as_object()) {
                fields
                    .iter()
                    .map(|(k, v)| {
                        let field_name = k
                            .chars()
                            .next()
                            .map(|c| c.to_uppercase().collect::<String>() + &k[1..])
                            .unwrap_or_default();
                        format!("{}: {}", field_name, v.as_str().unwrap_or("Invalid"))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                match body.get("error").and_then(|e| e.as_str()).unwrap_or("") {
                    "unauthorized" => match status.as_u16() {
                        401 => "Incorrect email or password".to_string(),
                        _ => "You don't have permission to perform this action".to_string(),
                    },
                    "forbidden" => "You don't have permission to perform this action".to_string(),
                    "not_found" => "Not found".to_string(),
                    "already_exists" => "This already exists".to_string(),
                    "too_many_requests" => {
                        "Too many attempts. Please wait before trying again".to_string()
                    }
                    "server_error" => "Something went wrong. Please try again".to_string(),
                    "invalid_input" => body
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Check your input")
                        .to_string(),
                    _ => match status.as_u16() {
                        0 => "Could not connect to the server".to_string(),
                        408 | 504 => "Request timed out".to_string(),
                        _ => body
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("An unexpected error occurred")
                            .to_string(),
                    },
                }
            };

            Err(ApiError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    async fn check_empty(&self, resp: reqwest::Response) -> ApiResult<()> {
        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        self.check::<serde_json::Value>(resp).await.map(|_| ())
    }
}

mod access;
mod auth;
mod identity;
mod models;
mod rbac;
mod settings;
mod tenancy;

pub use models::*;

#[cfg(test)]
mod tests {
    use super::Client;

    #[test]
    fn tenant_requests_include_selected_performer_urdd() {
        let mut client = Client::new("http://localhost:3000".into());
        client.set_token("token".into());
        client.set_actor_context(Some("69bdc119-57fb-4c8a-9927-655200bb6db5".into()));
        let headers = client.tenant_headers("fdcaa640-e2b6-4d0e-a389-e2e62cb61e30");
        assert_eq!(
            headers["x-framework-rust-urdd-id"],
            "69bdc119-57fb-4c8a-9927-655200bb6db5"
        );
        assert_eq!(
            headers["x-framework-rust-tenant-id"],
            "fdcaa640-e2b6-4d0e-a389-e2e62cb61e30"
        );
    }
}
