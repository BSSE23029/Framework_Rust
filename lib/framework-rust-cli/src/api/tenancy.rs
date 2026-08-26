use super::*;

impl Client {
    pub async fn list_tenant_slugs(&self) -> ApiResult<Vec<(String, String)>> {
        #[derive(Deserialize)]
        struct Entry {
            slug: String,
            name: String,
        }
        let resp = self
            .inner
            .get(format!("{}/tenants/slugs", self.base_url))
            .send()
            .await?;
        let entries: Vec<Entry> = self.check(resp).await?;
        Ok(entries.into_iter().map(|e| (e.slug, e.name)).collect())
    }

    pub async fn list_tenants(&self) -> ApiResult<Vec<Tenant>> {
        let resp = self
            .inner
            .get(format!("{}/tenants", self.base_url))
            .headers(self.auth_headers())
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn list_domains(&self, tenant_id: &str) -> ApiResult<Vec<TenantDomain>> {
        let resp = self
            .inner
            .get(format!(
                "{}/admin/tenants/{tenant_id}/domains",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_domain(
        &self,
        tenant_id: &str,
        domain_name: &str,
        domain_type: &str,
        is_primary: bool,
    ) -> ApiResult<TenantDomain> {
        let resp = self
            .inner
            .post(format!(
                "{}/admin/tenants/{tenant_id}/domains",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "domain_name": domain_name,
                "domain_type": domain_type,
                "is_primary": is_primary,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn domain_action(
        &self,
        tenant_id: &str,
        domain_id: &str,
        action: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .post(format!(
                "{}/admin/tenants/{tenant_id}/domains/{domain_id}/{action}",
                self.base_url
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
                message: format!("domain {action} failed"),
            })
        }
    }

    pub async fn list_plans(&self) -> ApiResult<Vec<TenantPlan>> {
        let resp = self
            .inner
            .get(format!("{}/plans", self.base_url))
            .headers(self.auth_headers())
            .send()
            .await?;
        self.check(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_plan(
        &self,
        code: &str,
        name: &str,
        billing_interval: &str,
        price_cents: i64,
        currency: &str,
        trial_days: i32,
    ) -> ApiResult<TenantPlan> {
        let resp = self
            .inner
            .post(format!("{}/plans", self.base_url))
            .headers(self.auth_headers())
            .json(&serde_json::json!({
                "code": code,
                "name": name,
                "billing_interval": billing_interval,
                "price_cents": price_cents,
                "currency": currency,
                "trial_days": trial_days,
                "features": {},
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn list_plan_quotas(&self, plan_id: &str) -> ApiResult<Vec<PlanQuota>> {
        let resp = self
            .inner
            .get(format!("{}/plans/{plan_id}/quotas", self.base_url))
            .headers(self.auth_headers())
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn set_plan_quota(
        &self,
        plan_id: &str,
        resource: &str,
        limit: i64,
    ) -> ApiResult<PlanQuota> {
        let resp = self
            .inner
            .put(format!(
                "{}/plans/{plan_id}/quotas/{resource}",
                self.base_url
            ))
            .headers(self.auth_headers())
            .json(&serde_json::json!({ "limit": limit }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn subscribe_tenant(
        &self,
        tenant_id: &str,
        plan_id: &str,
        start_trial: bool,
    ) -> ApiResult<TenantSubscription> {
        let resp = self
            .inner
            .post(format!(
                "{}/tenants/{tenant_id}/subscription",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "plan_id": plan_id,
                "start_trial": start_trial,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn list_config_definitions(
        &self,
        tenant_id: &str,
    ) -> ApiResult<Vec<ConfigDefinition>> {
        let resp = self
            .inner
            .get(format!("{}/settings/config/definitions", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn resolve_config_value(
        &self,
        tenant_id: &str,
        definition_id: &str,
    ) -> ApiResult<Option<serde_json::Value>> {
        #[derive(Deserialize)]
        struct Response {
            value: Option<serde_json::Value>,
        }
        let resp = self
            .inner
            .get(format!("{}/settings/config/{definition_id}", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        Ok(self.check::<Response>(resp).await?.value)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn define_config(
        &self,
        tenant_id: &str,
        config_key: &str,
        display_name: &str,
        category: &str,
        value_type: &str,
        default_value: Option<serde_json::Value>,
    ) -> ApiResult<ConfigDefinition> {
        let resp = self
            .inner
            .post(format!("{}/settings/config/definitions", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "config_key": config_key,
                "display_name": display_name,
                "description": "Created from the admin TUI",
                "category": category,
                "scope_type": "tenant",
                "value_type": value_type,
                "is_required": default_value.is_some(),
                "is_multi_value": false,
                "default_value": default_value,
                "allowed_values": null,
                "constraints": {},
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn set_config_value(
        &self,
        tenant_id: &str,
        definition_id: &str,
        value: serde_json::Value,
    ) -> ApiResult<ConfigValue> {
        let resp = self
            .inner
            .put(format!("{}/settings/config/{definition_id}", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({ "scope_id": null, "value": value }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn list_trusted_devices(&self, tenant_id: &str) -> ApiResult<Vec<TrustedDevice>> {
        let resp = self
            .inner
            .get(format!("{}/users/me/trusted-devices", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn trust_device(
        &self,
        tenant_id: &str,
        name: &str,
        ttl_days: i64,
    ) -> ApiResult<TrustedDeviceIssued> {
        let resp = self
            .inner
            .post(format!("{}/users/me/trusted-devices", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "name": name,
                "fingerprint": null,
                "ttl_days": ttl_days,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn revoke_trusted_device(&self, tenant_id: &str, device_id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!(
                "{}/users/me/trusted-devices/{device_id}",
                self.base_url
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
                message: "trusted device revocation failed".into(),
            })
        }
    }

    pub async fn list_request_activity(&self, tenant_id: &str) -> ApiResult<Vec<RequestActivity>> {
        let resp = self
            .inner
            .get(format!("{}/users/me/activity?limit=100", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn create_tenant(&self, name: &str, slug: &str) -> ApiResult<Tenant> {
        let resp = self
            .inner
            .post(format!("{}/tenants", self.base_url))
            .headers(self.auth_headers())
            .json(&serde_json::json!({ "name": name, "slug": slug }))
            .send()
            .await?;
        self.check(resp).await
    }
}
