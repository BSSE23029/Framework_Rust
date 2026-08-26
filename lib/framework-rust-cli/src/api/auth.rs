use super::*;

impl Client {
    pub async fn login(&self, email: &str, password: &str, slug: &str) -> ApiResult<LoginResult> {
        #[derive(Deserialize)]
        struct LoginResp {
            access_token: Option<String>,
            mfa_required: Option<bool>,
            mfa_token: Option<String>,
        }
        let resp = self
            .inner
            .post(format!("{}/auth/login", self.base_url))
            .header("x-framework-rust-tenant-slug", slug)
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await?;
        let body: LoginResp = self.check(resp).await?;
        if body.mfa_required == Some(true) {
            if let Some(mfa_token) = body.mfa_token {
                return Ok(LoginResult::MfaRequired { mfa_token });
            }
        }
        Ok(LoginResult::Token(body.access_token.unwrap_or_default()))
    }

    pub async fn login_universal(&self, email: &str, password: &str) -> ApiResult<LoginResult> {
        #[derive(Deserialize)]
        struct UniversalResp {
            access_token: Option<String>,
            mfa_required: Option<bool>,
            mfa_token: Option<String>,
            tenants: Option<Vec<TenantEntry>>,
        }
        #[derive(Deserialize)]
        struct TenantEntry {
            slug: String,
            name: String,
        }
        let resp = self
            .inner
            .post(format!("{}/auth/login/universal", self.base_url))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await?;
        let body: UniversalResp = self.check(resp).await?;
        if let Some(tenants) = body.tenants {
            return Ok(LoginResult::TenantChoice(
                tenants.into_iter().map(|t| (t.slug, t.name)).collect(),
            ));
        }
        if body.mfa_required == Some(true) {
            if let Some(mfa_token) = body.mfa_token {
                return Ok(LoginResult::MfaRequired { mfa_token });
            }
        }
        Ok(LoginResult::Token(body.access_token.unwrap_or_default()))
    }

    pub async fn mfa_challenge(
        &self,
        slug: &str,
        mfa_token: &str,
        code: &str,
    ) -> ApiResult<String> {
        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
        }
        let resp = self
            .inner
            .post(format!("{}/auth/mfa/challenge", self.base_url))
            .header("x-framework-rust-tenant-slug", slug)
            .json(&serde_json::json!({ "mfa_token": mfa_token, "code": code }))
            .send()
            .await?;
        let body: TokenResp = self.check(resp).await?;
        Ok(body.access_token)
    }

    pub async fn request_passwordless(&self, email: &str, slug: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .post(format!("{}/auth/passwordless/request", self.base_url))
            .header("x-framework-rust-tenant-slug", slug)
            .json(&serde_json::json!({ "email": email }))
            .send()
            .await?;
        self.check::<serde_json::Value>(resp).await.map(|_| ())
    }

    pub async fn verify_passwordless(
        &self,
        email: &str,
        slug: &str,
        otp: &str,
    ) -> ApiResult<LoginResult> {
        #[derive(Deserialize)]
        struct Response {
            access_token: Option<String>,
            mfa_required: Option<bool>,
            mfa_token: Option<String>,
        }
        let resp = self
            .inner
            .post(format!("{}/auth/passwordless/verify", self.base_url))
            .header("x-framework-rust-tenant-slug", slug)
            .json(&serde_json::json!({ "email": email, "otp": otp }))
            .send()
            .await?;
        let response: Response = self.check(resp).await?;
        if response.mfa_required == Some(true) {
            return response
                .mfa_token
                .map(|mfa_token| LoginResult::MfaRequired { mfa_token })
                .ok_or_else(|| ApiError::Api {
                    status: 500,
                    message: "MFA challenge token missing".into(),
                });
        }
        Ok(LoginResult::Token(
            response.access_token.unwrap_or_default(),
        ))
    }

    pub async fn admin_disable_user_mfa(&self, tenant_id: &str, user_id: &str) -> ApiResult<()> {
        let resp = self
            .inner
            .delete(format!("{}/users/{}/mfa", self.base_url, user_id))
            .headers(self.tenant_headers(tenant_id))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::Api {
                status: status.as_u16(),
                message: "disable mfa failed".into(),
            })
        }
    }
}
