use super::*;

impl Client {
    pub async fn list_clients(&self, tenant_id: &str) -> ApiResult<Vec<OAuthClient>> {
        let resp = self
            .inner
            .get(format!("{}/clients", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_client(
        &self,
        tenant_id: &str,
        name: &str,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
        is_confidential: bool,
        grant_types: Vec<String>,
        access_token_ttl_minutes: Option<i32>,
        refresh_token_ttl_days: Option<i32>,
    ) -> ApiResult<OAuthClient> {
        let resp = self
            .inner
            .post(format!("{}/clients", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "name": name,
                "redirect_uris": redirect_uris,
                "scopes": scopes,
                "is_confidential": is_confidential,
                "grant_types": grant_types,
                "access_token_ttl_minutes": access_token_ttl_minutes,
                "refresh_token_ttl_days": refresh_token_ttl_days,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_client(
        &self,
        tenant_id: &str,
        id: &str,
        name: &str,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
        access_token_ttl_minutes: Option<i32>,
        refresh_token_ttl_days: Option<i32>,
        is_confidential: bool,
        grant_types: Vec<String>,
    ) -> ApiResult<OAuthClient> {
        let resp = self
            .inner
            .put(format!("{}/clients/{}", self.base_url, id))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "name": name,
                "redirect_uris": redirect_uris,
                "scopes": scopes,
                "access_token_ttl_minutes": access_token_ttl_minutes,
                "refresh_token_ttl_days": refresh_token_ttl_days,
                "is_confidential": is_confidential,
                "grant_types": grant_types,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    // ── Client Roles ──────────────────────────────────────────────────────────

    pub async fn list_client_roles(
        &self,
        tenant_id: &str,
        client_id: &str,
    ) -> ApiResult<Vec<Role>> {
        let resp = self
            .inner
            .get(format!("{}/clients/{}/roles", self.base_url, client_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn assign_client_role(
        &self,
        tenant_id: &str,
        client_id: &str,
        role_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .post(format!("{}/clients/{}/roles", self.base_url, client_id))
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

    pub async fn revoke_client_role(
        &self,
        tenant_id: &str,
        client_id: &str,
        role_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!(
                "{}/clients/{}/roles/{}",
                self.base_url, client_id, role_id
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

    // ── Identity Providers ────────────────────────────────────────────────────

    pub async fn list_identity_providers(
        &self,
        tenant_id: &str,
    ) -> ApiResult<Vec<IdentityProvider>> {
        let resp = self
            .inner
            .get(format!("{}/identity-providers", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_identity_provider(
        &self,
        tenant_id: &str,
        provider: &str,
        client_id: &str,
        client_secret: &str,
        redirect_url: &str,
        scopes: Vec<String>,
    ) -> ApiResult<IdentityProvider> {
        let resp = self
            .inner
            .post(format!("{}/identity-providers", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "provider": provider,
                "client_id": client_id,
                "client_secret": client_secret,
                "redirect_url": redirect_url,
                "scopes": scopes,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_identity_provider(
        &self,
        tenant_id: &str,
        id: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_url: &str,
        scopes: Vec<String>,
        enabled: bool,
    ) -> ApiResult<IdentityProvider> {
        let mut body = serde_json::json!({
            "client_id": client_id,
            "redirect_url": redirect_url,
            "scopes": scopes,
            "enabled": enabled,
        });
        if let Some(secret) = client_secret {
            body["client_secret"] = serde_json::Value::String(secret.to_string());
        }
        let resp = self
            .inner
            .put(format!("{}/identity-providers/{}", self.base_url, id))
            .headers(self.tenant_headers(tenant_id))
            .json(&body)
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn delete_identity_provider(&self, tenant_id: &str, id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/identity-providers/{}", self.base_url, id))
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

    pub async fn deactivate_client(&self, tenant_id: &str, id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/clients/{}", self.base_url, id))
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
}
