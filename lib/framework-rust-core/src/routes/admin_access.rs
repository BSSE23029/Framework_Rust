use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{
    handlers::admin_access::{organizations, permissions},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/admin/tenants/{tenant_id}/access/departments",
            get(organizations::list_departments).post(organizations::create_department),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/designations",
            get(organizations::list_designations).post(organizations::create_designation),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/rdds",
            get(organizations::list_rdds).post(organizations::create_rdd),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/urdds",
            get(organizations::list_urdds).post(organizations::assign_user_context),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/urdds/{urdd_id}",
            delete(organizations::revoke_user_context),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/permission-groups",
            get(permissions::list_groups).post(permissions::create_group),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/permission-groups/{group_id}/permissions/{permission_id}",
            post(permissions::add_group_permission),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/rdds/{rdd_id}/permission-groups/{group_id}",
            post(permissions::link_rdd_group),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/permission-scopes",
            get(permissions::list_scopes).post(permissions::create_scope),
        )
        .route(
            "/admin/tenants/{tenant_id}/access/urdds/{urdd_id}/permissions/{permission_id}/scopes/{scope_id}",
            post(permissions::assign_scope),
        )
}
