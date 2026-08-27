use super::*;

impl Client {
    pub async fn list_audit_log(&self, tenant_id: &str) -> ApiResult<Vec<AuditLogEntry>> {
        let resp = self
            .inner
            .get(format!("{}/audit-log", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn export_audit_log(
        &self,
        tenant_id: &str,
        limit: u64,
    ) -> ApiResult<Vec<AuditLogEntry>> {
        let resp = self
            .inner
            .get(format!("{}/audit-log?limit={limit}", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    // ── Health ────────────────────────────────────────────────────────────────

    pub async fn health(&self) -> ApiResult<serde_json::Value> {
        let resp = self
            .inner
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn get_password_policy(&self, tenant_id: &str) -> ApiResult<PasswordPolicyResponse> {
        let resp = self
            .inner
            .get(format!("{}/settings/password-policy", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn put_password_policy(
        &self,
        tenant_id: &str,
        min_length: i32,
        require_uppercase: bool,
        require_digit: bool,
        require_special: bool,
    ) -> ApiResult<serde_json::Value> {
        let resp = self
            .inner
            .put(format!("{}/settings/password-policy", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "min_length": min_length,
                "require_uppercase": require_uppercase,
                "require_digit": require_digit,
                "require_special": require_special,
                "history_size": 0,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn get_lockout_policy(&self, tenant_id: &str) -> ApiResult<LockoutPolicyResponse> {
        let resp = self
            .inner
            .get(format!("{}/settings/lockout", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn put_lockout_policy(
        &self,
        tenant_id: &str,
        max_attempts: i32,
        window_minutes: i32,
        duration_minutes: i32,
    ) -> ApiResult<serde_json::Value> {
        let resp = self
            .inner
            .put(format!("{}/settings/lockout", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "max_attempts": max_attempts,
                "window_minutes": window_minutes,
                "duration_minutes": duration_minutes,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn get_token_ttl(&self, tenant_id: &str) -> ApiResult<TokenTtlResponse> {
        let resp = self
            .inner
            .get(format!("{}/settings/tokens", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn put_token_ttl(
        &self,
        tenant_id: &str,
        access_token_ttl_minutes: i32,
        refresh_token_ttl_days: i32,
    ) -> ApiResult<serde_json::Value> {
        let resp = self
            .inner
            .put(format!("{}/settings/tokens", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "access_token_ttl_minutes": access_token_ttl_minutes,
                "refresh_token_ttl_days": refresh_token_ttl_days,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn get_registration_policy(
        &self,
        tenant_id: &str,
    ) -> ApiResult<RegistrationPolicyResponse> {
        let resp = self
            .inner
            .get(format!("{}/settings/registration", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn put_registration_policy(
        &self,
        tenant_id: &str,
        allow_public_registration: bool,
        require_email_verified: bool,
    ) -> ApiResult<serde_json::Value> {
        let resp = self
            .inner
            .put(format!("{}/settings/registration", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&serde_json::json!({
                "allow_public_registration": allow_public_registration,
                "require_email_verified": require_email_verified,
            }))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn get_smtp(&self, tenant_id: &str) -> ApiResult<SmtpConfigResponse> {
        let resp = self
            .inner
            .get(format!("{}/admin/smtp", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put_smtp(
        &self,
        tenant_id: &str,
        host: &str,
        port: i32,
        username: &str,
        password: Option<&str>,
        from_name: &str,
        from_email: &str,
        use_tls: bool,
        enabled: bool,
    ) -> ApiResult<serde_json::Value> {
        let mut body = serde_json::json!({
            "host": host,
            "port": port,
            "username": username,
            "from_name": from_name,
            "from_email": from_email,
            "use_tls": use_tls,
            "enabled": enabled,
        });
        if let Some(pw) = password {
            if !pw.is_empty() {
                body["password"] = serde_json::json!(pw);
            }
        }
        let resp = self
            .inner
            .put(format!("{}/admin/smtp", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .json(&body)
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn list_user_passkeys(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> ApiResult<Vec<PasskeyInfo>> {
        let resp = self
            .inner
            .get(format!("{}/admin/users/{user_id}/passkeys", self.base_url))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        self.check(resp).await
    }

    pub async fn delete_user_passkey(
        &self,
        tenant_id: &str,
        user_id: &str,
        credential_id: &str,
    ) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!(
                "{}/admin/users/{user_id}/passkeys/{credential_id}",
                self.base_url
            ))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let _ = self.check::<serde_json::Value>(resp).await?;
            Ok(())
        }
    }
}
