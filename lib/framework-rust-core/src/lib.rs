#[path = "runtime/config.rs"]
pub mod config;
#[path = "runtime/db.rs"]
pub mod db;
pub mod entity;
#[path = "runtime/error.rs"]
pub mod error;
#[path = "runtime/extractors.rs"]
pub mod extractors;
#[cfg(test)]
#[path = "quality/folder_layout.rs"]
mod folder_layout;
pub mod handlers;
pub mod middleware;
#[path = "platform/openapi.rs"]
pub mod openapi;
#[path = "platform/router.rs"]
pub mod router;
pub mod routes;
pub mod services;
#[path = "runtime/state.rs"]
pub mod state;
