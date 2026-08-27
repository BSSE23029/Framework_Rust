use framework_rust_core::{
    config::Config,
    db,
    entity::{tenant_domains, tenants},
    error::AppError,
    router::build_router,
    services::jwk_service::JwkService,
    services::tenant_hierarchy_service,
    state::AppState,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;
use webauthn_rs::prelude::{Url, WebauthnBuilder};

async fn create_tenant(database: &sea_orm::DatabaseConnection, label: &str) -> tenants::Model {
    let suffix = Uuid::new_v4();
    tenants::ActiveModel {
        name: Set(format!("{label} {suffix}")),
        slug: Set(format!("{}-{suffix}", label.to_ascii_lowercase())),
        encryption_key: Set("tenant-hierarchy-test-key-32-chars".into()),
        ..Default::default()
    }
    .insert(database)
    .await
    .expect("tenant")
}

#[tokio::test]
async fn tenant_hierarchy_and_domains_complete_cycle() {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let database = db::connect(&config.database_url, 5, 1).await.expect("db");
    Migrator::up(&database, None).await.expect("migrations");

    let organization = create_tenant(&database, "Organization").await;
    let hotel = create_tenant(&database, "Hotel").await;
    let branch = create_tenant(&database, "Branch").await;
    let outsider = create_tenant(&database, "Outsider").await;

    let txn = db::begin_tenant_txn(&database, organization.id)
        .await
        .expect("organization txn");
    tenant_hierarchy_service::set_hierarchy(&txn, organization.id, "organization", None)
        .await
        .expect("organization hierarchy");
    txn.commit().await.expect("organization commit");

    let txn = db::begin_tenant_txn(&database, hotel.id)
        .await
        .expect("hotel txn");
    tenant_hierarchy_service::set_hierarchy(&txn, hotel.id, "hotel", Some(organization.id))
        .await
        .expect("hotel hierarchy");
    txn.commit().await.expect("hotel commit");

    let txn = db::begin_tenant_txn(&database, branch.id)
        .await
        .expect("branch txn");
    tenant_hierarchy_service::set_hierarchy(&txn, branch.id, "branch", Some(hotel.id))
        .await
        .expect("branch hierarchy");
    assert!(matches!(
        tenant_hierarchy_service::set_hierarchy(&txn, branch.id, "branch", Some(organization.id))
            .await,
        Err(AppError::InvalidInput(_))
    ));
    txn.commit().await.expect("branch commit");

    let txn = db::begin_tenant_txn(&database, organization.id)
        .await
        .expect("children txn");
    assert_eq!(
        tenant_hierarchy_service::list_children(&txn, organization.id)
            .await
            .expect("organization children")
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![hotel.id]
    );
    txn.commit().await.expect("children commit");

    let domain_suffix = Uuid::new_v4();
    let first_name = format!("admin-{domain_suffix}.example.test");
    let second_name = format!("portal-{domain_suffix}.example.test");
    let txn = db::begin_tenant_txn(&database, hotel.id)
        .await
        .expect("domain txn");
    let first = tenant_hierarchy_service::create_domain(
        &txn,
        hotel.id,
        &first_name.to_ascii_uppercase(),
        "custom",
        true,
    )
    .await
    .expect("first domain");
    assert_eq!(first.domain_name, first_name);
    tenant_hierarchy_service::verify_domain(&txn, hotel.id, first.id)
        .await
        .expect("verify domain");
    let second =
        tenant_hierarchy_service::create_domain(&txn, hotel.id, &second_name, "custom", true)
            .await
            .expect("second domain");
    let refreshed_first = tenant_domains::Entity::find_by_id(first.id)
        .one(&txn)
        .await
        .expect("first query")
        .expect("first domain exists");
    assert!(!refreshed_first.is_primary);
    assert!(
        tenant_domains::Entity::find_by_id(second.id)
            .one(&txn)
            .await
            .expect("second query")
            .expect("second domain exists")
            .is_primary
    );

    tenant_hierarchy_service::set_domain_active(&txn, hotel.id, first.id, false)
        .await
        .expect("deactivate domain");
    assert_eq!(
        tenant_hierarchy_service::list_domains(&txn, hotel.id)
            .await
            .expect("active domains")
            .len(),
        1
    );
    tenant_hierarchy_service::set_domain_active(&txn, hotel.id, first.id, true)
        .await
        .expect("restore domain");
    assert_eq!(
        tenant_hierarchy_service::list_domains(&txn, hotel.id)
            .await
            .expect("restored domains")
            .len(),
        2
    );
    txn.commit().await.expect("domain commit");

    let txn = db::begin_tenant_txn(&database, outsider.id)
        .await
        .expect("outsider txn");
    assert!(matches!(
        tenant_hierarchy_service::set_primary(&txn, outsider.id, first.id).await,
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        tenant_hierarchy_service::create_domain(&txn, outsider.id, &first_name, "custom", false)
            .await,
        Err(AppError::Conflict)
    ));
    txn.rollback().await.expect("outsider rollback");

    let rp_origin = Url::parse(&config.framework_rust_issuer).expect("issuer");
    let webauthn = Arc::new(
        WebauthnBuilder::new(rp_origin.host_str().unwrap_or("localhost"), &rp_origin)
            .expect("webauthn builder")
            .rp_name("Hierarchy test")
            .build()
            .expect("webauthn"),
    );
    let app = build_router(AppState::new(
        database.clone(),
        config.clone(),
        JwkService::generate(),
        None,
        webauthn,
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("serve hierarchy API");
    });

    let api_organization = create_tenant(&database, "ApiOrganization").await;
    let api_hotel = create_tenant(&database, "ApiHotel").await;
    let admin_key = config.admin_key.as_deref().expect("development admin key");
    let client = reqwest::Client::new();
    for (tenant, tenant_type, parent) in [
        (&api_organization, "organization", None),
        (&api_hotel, "hotel", Some(api_organization.id)),
    ] {
        let response = client
            .put(format!(
                "http://{address}/admin/tenants/{}/hierarchy",
                tenant.id
            ))
            .header("x-framework-rust-admin-key", admin_key)
            .json(&serde_json::json!({
                "tenant_type": tenant_type,
                "parent_tenant_id": parent,
            }))
            .send()
            .await
            .expect("set hierarchy request");
        assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    }
    let children = client
        .get(format!(
            "http://{address}/admin/tenants/{}/children",
            api_organization.id
        ))
        .header("x-framework-rust-admin-key", admin_key)
        .send()
        .await
        .expect("list children request")
        .error_for_status()
        .expect("children response")
        .json::<Vec<serde_json::Value>>()
        .await
        .expect("children JSON");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0]["id"], api_hotel.id.to_string());

    let department = client
        .post(format!(
            "http://{address}/admin/tenants/{}/access/departments",
            api_hotel.id
        ))
        .header("x-framework-rust-admin-key", admin_key)
        .json(&serde_json::json!({
            "name": "API Operations",
            "code": format!("API-{}", Uuid::new_v4()),
            "description": "route-level access test",
        }))
        .send()
        .await
        .expect("create department request")
        .error_for_status()
        .expect("department response")
        .json::<serde_json::Value>()
        .await
        .expect("department JSON");
    let departments = client
        .get(format!(
            "http://{address}/admin/tenants/{}/access/departments",
            api_hotel.id
        ))
        .header("x-framework-rust-admin-key", admin_key)
        .send()
        .await
        .expect("list departments request")
        .error_for_status()
        .expect("departments response")
        .json::<Vec<serde_json::Value>>()
        .await
        .expect("departments JSON");
    assert!(departments.iter().any(|row| row["id"] == department["id"]));
}
