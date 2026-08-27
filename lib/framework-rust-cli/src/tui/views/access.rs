use crate::{
    api::ApiError,
    app::{App, Modal},
};

pub(super) fn section_len(app: &App) -> usize {
    match app.access_section {
        0 => app.departments.len(),
        1 => app.designations.len(),
        2 => app.rdds.len(),
        3 => app.urdds.len(),
        4 => app.permission_groups.len(),
        _ => app.permission_scopes.len(),
    }
}

pub(super) async fn load(app: &mut App, tenant_id: String) {
    app.access_loading = true;
    let results = tokio::join!(
        app.client.list_departments(&tenant_id),
        app.client.list_designations(&tenant_id),
        app.client.list_rdds(&tenant_id),
        app.client.list_urdds(&tenant_id),
        app.client.list_permission_groups(&tenant_id),
        app.client.list_permission_scopes(&tenant_id),
    );
    match results {
        (Ok(departments), Ok(designations), Ok(rdds), Ok(urdds), Ok(groups), Ok(scopes)) => {
            app.departments = departments;
            app.designations = designations;
            app.rdds = rdds;
            app.urdds = urdds;
            app.permission_groups = groups;
            app.permission_scopes = scopes;
            app.access_selected = app.access_selected.min(section_len(app).saturating_sub(1));
        }
        values => {
            let error = values
                .0
                .err()
                .or_else(|| values.1.err())
                .or_else(|| values.2.err())
                .or_else(|| values.3.err())
                .or_else(|| values.4.err())
                .or_else(|| values.5.err());
            if let Some(error) = error {
                super::on_load_err(app, error, "Access organization");
            }
        }
    }
    app.access_loading = false;
}

pub(super) async fn perform(app: &mut App, action: usize, fields: Vec<String>) {
    let Some(tenant_id) = app.active_tenant_id.clone() else {
        return;
    };
    let required = match action {
        0 | 1 | 3 | 6 | 7 => &[0, 1][..],
        2 | 5 | 8 => &[0, 1, 2][..],
        4 => &[0][..],
        _ => &[][..],
    };
    if required
        .iter()
        .any(|index| fields[*index].trim().is_empty())
    {
        app.modal = Modal::Error("Required access fields cannot be empty".into());
        return;
    }
    let result = match action {
        0 => app
            .client
            .create_department(&tenant_id, &fields[0], &fields[1], &fields[2])
            .await
            .map(|_| "Department created"),
        1 => app
            .client
            .create_designation(
                &tenant_id,
                &fields[0],
                &fields[1],
                &fields[2],
                (!fields[3].is_empty()).then_some(fields[3].as_str()),
            )
            .await
            .map(|_| "Designation created"),
        2 => app
            .client
            .create_rdd(&tenant_id, &fields[0], &fields[1], &fields[2])
            .await
            .map(|_| "Role/designation/department context created"),
        3 => match parse_json(&fields[2]) {
            Ok(attributes) => app
                .client
                .assign_user_context(&tenant_id, &fields[0], &fields[1], attributes)
                .await
                .map(|_| "User access context assigned"),
            Err(error) => Err(error),
        },
        4 => app
            .client
            .create_permission_group(&tenant_id, &fields[0], &fields[1])
            .await
            .map(|_| "Permission group created"),
        5 => match parse_json(&fields[4]) {
            Ok(conditions) => app
                .client
                .create_permission_scope(
                    &tenant_id,
                    serde_json::json!({
                        "permission_id": fields[0],
                        "name": fields[1],
                        "resource_type": fields[2],
                        "resource_id": (!fields[3].is_empty()).then_some(fields[3].as_str()),
                        "conditions": conditions,
                    }),
                )
                .await
                .map(|_| "Permission scope created"),
            Err(error) => Err(error),
        },
        6 => app
            .client
            .add_group_permission(&tenant_id, &fields[0], &fields[1])
            .await
            .map(|_| "Permission added to group"),
        7 => app
            .client
            .link_rdd_group(&tenant_id, &fields[0], &fields[1])
            .await
            .map(|_| "Permission group linked to RDD"),
        _ => app
            .client
            .assign_permission_scope(&tenant_id, &fields[0], &fields[1], &fields[2])
            .await
            .map(|_| "Permission scope assigned to URDD"),
    };
    match result {
        Ok(message) => {
            app.modal = Modal::None;
            app.set_status(message);
            load(app, tenant_id).await;
        }
        Err(error) => app.modal = Modal::Error(error.to_string()),
    }
}

fn parse_json(value: &str) -> Result<serde_json::Value, ApiError> {
    if value.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(value).map_err(|error| ApiError::Api {
        status: 422,
        message: format!("Invalid JSON: {error}"),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_json;

    #[test]
    fn json_defaults_to_an_empty_object() {
        assert_eq!(parse_json("").unwrap(), serde_json::json!({}));
    }

    #[test]
    fn malformed_attributes_are_rejected() {
        let error = parse_json("{broken").unwrap_err();
        assert!(error.to_string().contains("Invalid JSON"));
    }
}
