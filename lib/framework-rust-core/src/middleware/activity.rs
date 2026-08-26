use crate::{
    middleware::{auth::AuthUser, tenant::TenantContext},
    services::request_activity_service,
    state::AppState,
};
use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::{net::SocketAddr, time::Instant};
use uuid::Uuid;

pub async fn activity_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let started = Instant::now();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .unwrap_or_else(Uuid::new_v4);
    let auth = request.extensions().get::<AuthUser>().cloned();
    let tenant = request.extensions().get::<TenantContext>().cloned();
    let requested_urdd_id = request
        .headers()
        .get("x-framework-rust-urdd-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok());
    let method = request.method().to_string();
    let path = request.uri().path().to_owned();
    let user_agent = header(&request, "user-agent");
    let platform = header(&request, "x-client-platform");
    let platform_version = header(&request, "x-client-platform-version");
    let mut response = next.run(request).await;
    let elapsed = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
    if let (Some(auth), Some(tenant)) = (auth, tenant) {
        let result = request_activity_service::record(
            &state.db,
            request_activity_service::RecordInput {
                tenant_id: tenant.tenant_id,
                user_id: auth.user_id,
                requested_urdd_id,
                request_id,
                http_method: method,
                request_path: path,
                response_status: response.status().as_u16().into(),
                response_time_ms: elapsed,
                ip_address: Some(addr.ip().to_string()),
                user_agent,
                platform,
                platform_version,
            },
        )
        .await;
        if let Err(error) = result {
            tracing::error!(%error, %request_id, "request activity write failed");
        }
    }
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id.to_string()).expect("UUID is a valid header"),
    );
    response
}

fn header(request: &Request<Body>, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}
