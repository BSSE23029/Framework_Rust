---
title: SMTP & Email Delivery
description: Configure per-tenant SMTP so FRAMEWORK RUST can send verification and password-reset emails.
---

FRAMEWORK RUST sends mail directly through each tenant's configured SMTP server using the Rust `lettre` library. It has no required email-vendor integration or bundled delivery service: use any standards-compatible provider such as Amazon SES, Postmark, SendGrid, Mailgun, Resend SMTP, or a self-hosted SMTP server.

When enabled, the server sends transactional emails for three flows:

- **Email verification** — triggered on registration when the tenant's `require_email_verified` setting is `true`
- **Password reset** — triggered when a user calls `POST /auth/forgot-password`
- **Passwordless login** — sends a one-time code from `POST /auth/passwordless/request`

Email delivery is best-effort: if the SMTP connection fails, the error is logged and the originating API call still succeeds. Users are not shown delivery errors.

---

## Configure via TUI

<Steps>
  <Step title="Open the Settings tab">
    In the TUI sidebar, select the tenant you want to configure, then navigate to the **Settings** tab. Use the left/right arrow keys to cycle between **General**, **Security**, and **SMTP** sections (SMTP is section index 4).
  </Step>

  <Step title="Fill in the SMTP fields">
    Use `Tab` to move between fields. The available fields are:

    | Field | Default | Description |
    |-------|---------|-------------|
    | Host | — | SMTP server hostname (e.g. `smtp.sendgrid.net`) |
    | Port | `587` | SMTP port |
    | Username | — | SMTP auth username |
    | Password | — | SMTP auth password (stored encrypted) |
    | From Name | — | Display name in the `From` header |
    | From Email | — | Sender address (e.g. `no-reply@yourapp.com`) |
    | STARTTLS | `true` | Toggle with `Space` — enables STARTTLS on the connection |
    | Enabled | `false` | Toggle with `Space` — must be `true` for emails to be sent |
  </Step>

  <Step title="Save">
    Press `Enter` to save. The TUI calls `PUT /admin/smtp` on your behalf. A status bar message confirms success or reports an error.

    <Note>
      Leave the **Password** field blank to keep the existing stored password. Submitting a non-empty value overwrites it.
    </Note>
  </Step>
</Steps>

---

## Configure via API

Both endpoints require two headers:

```
X-FRAMEWORK-RUST-Admin-Key: <your-admin-key>
X-FRAMEWORK-RUST-Tenant-ID: <tenant-uuid>
```

<Tabs>
  <Tab title="GET /admin/smtp">
    Retrieve the current SMTP configuration. The password is **never** returned; `password_set` indicates whether one is stored.

    ```bash
    curl http://localhost:3000/admin/smtp \
      -H "X-FRAMEWORK-RUST-Admin-Key: your-admin-key" \
      -H "X-FRAMEWORK-RUST-Tenant-ID: 018e1234-0000-7000-8000-000000000001"
    ```

    ```json
    {
      "host": "smtp.sendgrid.net",
      "port": 587,
      "username": "apikey",
      "password_set": true,
      "from_name": "Acme Auth",
      "from_email": "no-reply@acme.com",
      "use_tls": true,
      "enabled": true
    }
    ```
  </Tab>

  <Tab title="PUT /admin/smtp">
    Upsert the SMTP configuration. Omit `password` to leave the stored password unchanged.

    ```bash
    curl -X PUT http://localhost:3000/admin/smtp \
      -H "X-FRAMEWORK-RUST-Admin-Key: your-admin-key" \
      -H "X-FRAMEWORK-RUST-Tenant-ID: 018e1234-0000-7000-8000-000000000001" \
      -H "Content-Type: application/json" \
      -d '{
        "host": "smtp.sendgrid.net",
        "port": 587,
        "username": "apikey",
        "password": "SG.xxxxxxxxxxxxxxxx",
        "from_name": "Acme Auth",
        "from_email": "no-reply@acme.com",
        "use_tls": true,
        "enabled": true
      }'
    ```

    Returns `200 OK` with `{ "message": "SMTP config saved" }`.

    **Disable without wiping config:** resend the non-secret configuration fields with `enabled: false` and omit `password`:

    ```bash
    curl -X PUT http://localhost:3000/admin/smtp \
      -H "X-FRAMEWORK-RUST-Admin-Key: your-admin-key" \
      -H "X-FRAMEWORK-RUST-Tenant-ID: 018e1234-0000-7000-8000-000000000001" \
      -H "Content-Type: application/json" \
      -d '{
        "host": "smtp.sendgrid.net",
        "port": 587,
        "username": "apikey",
        "from_name": "Acme Auth",
        "from_email": "no-reply@acme.com",
        "use_tls": true,
        "enabled": false
      }'
    ```

    <Note>
      `host`, `username`, `from_name`, and `from_email` are required on every PUT. `password` may be omitted to retain the existing encrypted password; `port`, `use_tls`, and `enabled` use their documented defaults when omitted.
    </Note>
  </Tab>
</Tabs>

---

## How emails are used

### Email verification

When `require_email_verified` is `true` in tenant settings and SMTP is enabled, FRAMEWORK RUST sends a verification link immediately after `POST /auth/register`.

```
Subject: Verify your email address

Hi,

Click the link below to verify your email address. The link expires in 24 hours.

https://auth.yourapp.com/auth/verify-email?token=<one-time-token>

If you did not create an account, you can ignore this email.
```

The token maps to a row in `one_time_tokens` with `purpose = verify_email`. Once clicked, `email_verified` is set to `true` on the user record and the token is consumed.

### Password reset

`POST /auth/forgot-password` (body: `{ "email": "user@example.com" }`) sends a reset link when SMTP is enabled. The response is always `200 OK` regardless of whether the email exists, to prevent user enumeration.

```
Subject: Reset your password

Hi,

A password reset was requested for this address. Click the link below to set a new password. The link expires in 1 hour.

https://auth.yourapp.com/reset-password?token=<one-time-token>

If you did not request this, you can safely ignore it.
```

The token maps to `one_time_tokens` with `purpose = reset_password`. Submit it to `POST /auth/reset-password` along with the new password.

### Passwordless login

`POST /auth/passwordless/request` creates and emails a short-lived one-time code when passwordless login is enabled for the tenant. The code is then submitted to `POST /auth/passwordless/verify`. As with password reset, the request response does not disclose whether an account exists.

<Warning>
  If SMTP is disabled or not configured, forgot-password still returns `200 OK` but no email is sent. Ensure SMTP is enabled and tested before enabling the `require_email_verified` tenant setting in production.
</Warning>

---

## Security

SMTP passwords are encrypted at rest using FRAMEWORK RUST's double-envelope AES-256-GCM key hierarchy (via [`hefesto`](https://crates.io/crates/hefesto)):

1. **Master key** (`MASTER_ENCRYPTION_KEY`) wraps a per-tenant data key
2. The per-tenant data key is additionally wrapped by `TENANT_WRAP_KEY`
3. The SMTP password ciphertext (+ nonce) is stored in `tenant_smtp_config`; the plaintext never touches the database

The password is decrypted in-process only at send time and is never returned by any API endpoint.

## Delivery characteristics

- Messages contain both plain-text and HTML alternatives.
- `use_tls: true` negotiates STARTTLS; disabling it uses the SMTP relay without STARTTLS.
- Sending is synchronous but best-effort from the authentication flow: configuration, connection, and delivery failures are logged and are not returned to the caller.
- There is currently no durable queue, automatic retry schedule, bounce processing, delivery-status tracking, or provider webhook handling.

<Check>Use a dedicated SMTP credential (API key) rather than your primary mail account password.</Check>
<Check>Ensure your SMTP provider enforces TLS — keep `use_tls: true` unless your provider explicitly requires otherwise.</Check>
<Check>Rotate the SMTP password via `PUT /admin/smtp` with a new `password` field; the old ciphertext is overwritten immediately.</Check>
