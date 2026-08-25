use framework_rust_core::{
    config::Config,
    db,
    entity::{tenant_domains, tenants},
    error::AppError,
    services::tenant_hierarchy_service,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

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
}
