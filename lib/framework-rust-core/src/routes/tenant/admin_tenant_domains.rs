use axum::{routing::get, Router};

use crate::{handlers::admin_tenant_domains, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/admin/tenants/:tenant_id/domains",
            get(admin_tenant_domains::list_domains).post(admin_tenant_domains::create_domain),
        )
        .route(
            "/admin/tenants/:tenant_id/domains/:domain_id/:action",
            axum::routing::post(admin_tenant_domains::domain_action),
        )
}
