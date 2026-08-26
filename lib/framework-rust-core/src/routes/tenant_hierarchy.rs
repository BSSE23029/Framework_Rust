use axum::{routing::get, Router};

use crate::{handlers::tenant_hierarchy, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/admin/tenants/:tenant_id/hierarchy",
            axum::routing::put(tenant_hierarchy::set_hierarchy),
        )
        .route(
            "/admin/tenants/:tenant_id/children",
            get(tenant_hierarchy::list_children),
        )
}
