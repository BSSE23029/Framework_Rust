use crate::{handlers::admin_plans, state::AppState};
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/plans", get(admin_plans::list).post(admin_plans::create))
        .route("/plans/:plan_id/quotas", get(admin_plans::list_quotas))
        .route(
            "/plans/:plan_id/quotas/:resource",
            put(admin_plans::set_quota),
        )
        .route(
            "/tenants/:tenant_id/subscription",
            post(admin_plans::subscribe),
        )
}
