use super::{
    ApiResult, Client, Department, Designation, PermissionGroup, PermissionScope,
    RoleDesignationDepartment, UserRoleDesignationDepartment,
};

impl Client {
    pub async fn list_departments(&self, tenant_id: &str) -> ApiResult<Vec<Department>> {
        self.get_access(tenant_id, "departments").await
    }

    pub async fn create_department(
        &self,
        tenant_id: &str,
        name: &str,
        code: &str,
        description: &str,
    ) -> ApiResult<Department> {
        self.post_access(
            tenant_id,
            "departments",
            serde_json::json!({"name": name, "code": code, "description": description}),
        )
        .await
    }

    pub async fn list_designations(&self, tenant_id: &str) -> ApiResult<Vec<Designation>> {
        self.get_access(tenant_id, "designations").await
    }

    pub async fn create_designation(
        &self,
        tenant_id: &str,
        name: &str,
        code: &str,
        description: &str,
        senior_designation_id: Option<&str>,
    ) -> ApiResult<Designation> {
        self.post_access(
            tenant_id,
            "designations",
            serde_json::json!({
                "name": name,
                "code": code,
                "description": description,
                "senior_designation_id": senior_designation_id,
            }),
        )
        .await
    }

    pub async fn list_rdds(&self, tenant_id: &str) -> ApiResult<Vec<RoleDesignationDepartment>> {
        self.get_access(tenant_id, "rdds").await
    }

    pub async fn create_rdd(
        &self,
        tenant_id: &str,
        role_id: &str,
        designation_id: &str,
        department_id: &str,
    ) -> ApiResult<RoleDesignationDepartment> {
        self.post_access(
            tenant_id,
            "rdds",
            serde_json::json!({
                "role_id": role_id,
                "designation_id": designation_id,
                "department_id": department_id,
            }),
        )
        .await
    }

    pub async fn list_urdds(
        &self,
        tenant_id: &str,
    ) -> ApiResult<Vec<UserRoleDesignationDepartment>> {
        self.get_access(tenant_id, "urdds").await
    }

    pub async fn assign_user_context(
        &self,
        tenant_id: &str,
        user_id: &str,
        rdd_id: &str,
        attributes: serde_json::Value,
    ) -> ApiResult<UserRoleDesignationDepartment> {
        self.post_access(
            tenant_id,
            "urdds",
            serde_json::json!({"user_id": user_id, "rdd_id": rdd_id, "attributes": attributes}),
        )
        .await
    }

    pub async fn revoke_user_context(&self, tenant_id: &str, urdd_id: &str) -> ApiResult<()> {
        let response = self
            .inner
            .delete(format!(
                "{}/admin/tenants/{tenant_id}/access/urdds/{urdd_id}",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check_empty(response).await
    }

    pub async fn list_permission_groups(&self, tenant_id: &str) -> ApiResult<Vec<PermissionGroup>> {
        self.get_access(tenant_id, "permission-groups").await
    }

    pub async fn create_permission_group(
        &self,
        tenant_id: &str,
        name: &str,
        description: &str,
    ) -> ApiResult<PermissionGroup> {
        self.post_access(
            tenant_id,
            "permission-groups",
            serde_json::json!({"name": name, "description": description}),
        )
        .await
    }

    pub async fn add_group_permission(
        &self,
        tenant_id: &str,
        group_id: &str,
        permission_id: &str,
    ) -> ApiResult<()> {
        self.post_empty_access(
            tenant_id,
            &format!("permission-groups/{group_id}/permissions/{permission_id}"),
        )
        .await
    }

    pub async fn link_rdd_group(
        &self,
        tenant_id: &str,
        rdd_id: &str,
        group_id: &str,
    ) -> ApiResult<()> {
        self.post_empty_access(
            tenant_id,
            &format!("rdds/{rdd_id}/permission-groups/{group_id}"),
        )
        .await
    }

    pub async fn list_permission_scopes(&self, tenant_id: &str) -> ApiResult<Vec<PermissionScope>> {
        self.get_access(tenant_id, "permission-scopes").await
    }

    pub async fn create_permission_scope(
        &self,
        tenant_id: &str,
        body: serde_json::Value,
    ) -> ApiResult<PermissionScope> {
        self.post_access(tenant_id, "permission-scopes", body).await
    }

    pub async fn assign_permission_scope(
        &self,
        tenant_id: &str,
        urdd_id: &str,
        permission_id: &str,
        scope_id: &str,
    ) -> ApiResult<()> {
        self.post_empty_access(
            tenant_id,
            &format!("urdds/{urdd_id}/permissions/{permission_id}/scopes/{scope_id}"),
        )
        .await
    }

    async fn get_access<T: for<'de> serde::Deserialize<'de>>(
        &self,
        tenant_id: &str,
        path: &str,
    ) -> ApiResult<T> {
        let response = self
            .inner
            .get(format!(
                "{}/admin/tenants/{tenant_id}/access/{path}",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(response).await
    }

    async fn post_access<T: for<'de> serde::Deserialize<'de>>(
        &self,
        tenant_id: &str,
        path: &str,
        body: serde_json::Value,
    ) -> ApiResult<T> {
        let response = self
            .inner
            .post(format!(
                "{}/admin/tenants/{tenant_id}/access/{path}",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .json(&body)
            .send()
            .await?;
        self.check(response).await
    }

    async fn post_empty_access(&self, tenant_id: &str, path: &str) -> ApiResult<()> {
        let response = self
            .inner
            .post(format!(
                "{}/admin/tenants/{tenant_id}/access/{path}",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check_empty(response).await
    }
}
