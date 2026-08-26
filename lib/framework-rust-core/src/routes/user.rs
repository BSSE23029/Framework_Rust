use axum::{routing::get, Router};

use crate::{
    handlers::{
        me::me,
        request_activity,
        trusted_devices::{list_devices, revoke_device, trust_device},
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/me", get(me))
        .route("/users/me/activity", get(request_activity::list))
        .route(
            "/users/me/trusted-devices",
            get(list_devices).post(trust_device),
        )
        .route(
            "/users/me/trusted-devices/:device_id",
            axum::routing::delete(revoke_device),
        )
}
