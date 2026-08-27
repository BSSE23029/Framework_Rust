use super::*;

impl Client {
    pub async fn list_users(&self, tenant_id: &str) -> ApiResult<Vec<User>> {
        let resp = self
            .inner
            .get(format!("{}/users", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_user(
        &self,
        tenant_id: &str,
        email: &str,
        password: &str,
    ) -> ApiResult<User> {
        let resp = self
            .inner
            .post(format!("{}/users", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn deactivate_user(&self, tenant_id: &str, id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/users/{}", self.base_url, id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "deactivate failed".into(),
            })
        }
    }

    // ── Roles ─────────────────────────────────────────────────────────────────

    pub async fn list_roles(&self, tenant_id: &str) -> ApiResult<Vec<Role>> {
        let resp = self
            .inner
            .get(format!("{}/roles", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_role(
        &self,
        tenant_id: &str,
        name: &str,
        description: &str,
    ) -> ApiResult<Role> {
        let resp = self
            .inner
            .post(format!("{}/roles", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn update_role(
        &self,
        tenant_id: &str,
        role_id: &str,
        name: &str,
        description: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .put(format!("{}/roles/{}", self.base_url, role_id))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "update failed".into(),
            })
        }
    }

    pub async fn delete_role(&self, tenant_id: &str, role_id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/roles/{}", self.base_url, role_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "delete failed".into(),
            })
        }
    }

    pub async fn list_role_permissions(
        &self,
        tenant_id: &str,
        role_id: &str,
    ) -> ApiResult<Vec<Permission>> {
        let resp = self
            .inner
            .get(format!("{}/roles/{}/permissions", self.base_url, role_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn assign_role_permission(
        &self,
        tenant_id: &str,
        role_id: &str,
        permission_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .post(format!("{}/roles/{}/permissions", self.base_url, role_id))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "permission_id": permission_id }))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "assign failed".into(),
            })
        }
    }

    pub async fn revoke_role_permission(
        &self,
        tenant_id: &str,
        role_id: &str,
        perm_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!(
                "{}/roles/{}/permissions/{}",
                self.base_url, role_id, perm_id
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "revoke failed".into(),
            })
        }
    }

    pub async fn list_user_roles(&self, tenant_id: &str, user_id: &str) -> ApiResult<Vec<Role>> {
        let resp = self
            .inner
            .get(format!("{}/users/{}/roles", self.base_url, user_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn assign_user_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .post(format!("{}/users/{}/roles", self.base_url, user_id))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "role_id": role_id }))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "assign failed".into(),
            })
        }
    }

    pub async fn revoke_user_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!(
                "{}/users/{}/roles/{}",
                self.base_url, user_id, role_id
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "revoke failed".into(),
            })
        }
    }

    // ── Sessions ──────────────────────────────────────────────────────────────

    pub async fn list_sessions(&self, tenant_id: &str) -> ApiResult<Vec<Session>> {
        let resp = self
            .inner
            .get(format!("{}/sessions", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn delete_session(&self, tenant_id: &str, id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/sessions/{}", self.base_url, id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "delete failed".into(),
            })
        }
    }

    // ── Permissions ───────────────────────────────────────────────────────────

    pub async fn list_permissions(&self, tenant_id: &str) -> ApiResult<Vec<Permission>> {
        let resp = self
            .inner
            .get(format!("{}/permissions", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_permission(
        &self,
        tenant_id: &str,
        name: &str,
        description: &str,
    ) -> ApiResult<Permission> {
        let resp = self
            .inner
            .post(format!("{}/permissions", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn update_permission(
        &self,
        tenant_id: &str,
        perm_id: &str,
        name: &str,
        description: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .put(format!("{}/permissions/{}", self.base_url, perm_id))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "name": name, "description": description }))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "update failed".into(),
            })
        }
    }

    pub async fn delete_permission(&self, tenant_id: &str, perm_id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/permissions/{}", self.base_url, perm_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "delete failed".into(),
            })
        }
    }

    pub async fn update_user_email(
        &self,
        tenant_id: &str,
        user_id: &str,
        email: &str,
        password: Option<&str>,
        is_active: bool,
    ) -> ApiResult<()> {
        let mut body = serde_json::json!({ "email": email, "is_active": is_active });
        if let Some(pw) = password {
            body["password"] = serde_json::Value::String(pw.to_string());
        }
        let resp = self
            .inner
            .put(format!("{}/users/{}", self.base_url, user_id))
            .headers(self.tenant_headers(tenant_id))
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "update failed".into(),
            })
        }
    }
}
