---
title: Getting Started
description: Run FRAMEWORK RUST in under 5 minutes.
---

<Steps>
  <Step title="Start the server">
    The fastest way is Docker. On first run, FRAMEWORK RUST auto-generates the three cryptographic secrets and prints them to logs.

    ```bash
    docker run -d --name framework-rust -p 3000:3000 \
      -e DATABASE_URL=postgresql://user:pass@host:5432/framework-rust \
      -e FRAMEWORK_RUST_ADMIN_KEY=your-admin-key \
      -e FRAMEWORK_RUST_BOOTSTRAP_ADMIN_EMAIL=admin@example.com \
      -e FRAMEWORK_RUST_BOOTSTRAP_ADMIN_PASSWORD=Admin1234! \
      ghcr.io/shrpp/framework-rust-core:latest
    ```

    Check the logs immediately:

    ```bash
    docker logs framework-rust 2>&1 | grep -A5 "SECRETS GENERATED"
    ```

    You'll see:

    ```
    ╔══════════════════════════════════════════════════════╗
    ║           FRAMEWORK RUST — FIRST RUN: SECRETS GENERATED       ║
    ║  JWT_SECRET=<base64>                                 ║
    ║  MASTER_ENCRYPTION_KEY=<base64>                      ║
    ║  TENANT_WRAP_KEY=<base64>                            ║
    ╚══════════════════════════════════════════════════════╝
    ```

    <Warning>
      Save these three values immediately. Losing them makes all encrypted data permanently unrecoverable.
    </Warning>
  </Step>

  <Step title="Pin your secrets and restart">
    Stop the container and re-run with the secrets pinned:

    ```bash
    docker stop framework-rust && docker rm framework-rust

    docker run -d --name framework-rust -p 3000:3000 \
      -e DATABASE_URL=postgresql://user:pass@host:5432/framework-rust \
      -e JWT_SECRET=<value-from-logs> \
      -e MASTER_ENCRYPTION_KEY=<value-from-logs> \
      -e TENANT_WRAP_KEY=<value-from-logs> \
      -e FRAMEWORK_RUST_ADMIN_KEY=your-admin-key \
      -e FRAMEWORK_RUST_BOOTSTRAP_ADMIN_EMAIL=admin@example.com \
      -e FRAMEWORK_RUST_BOOTSTRAP_ADMIN_PASSWORD=Admin1234! \
      ghcr.io/shrpp/framework-rust-core:latest
    ```

    Verify it's healthy:

    ```bash
    curl http://localhost:3000/health
    # {"status":"ok","version":"x.y.z"}
    ```
  </Step>

  <Step title="Docker Compose (recommended for local dev)">
    Use this `docker-compose.yml` to get Postgres + FRAMEWORK RUST in one command:

    ```yaml
    version: "3.9"
    services:
      postgres:
        image: postgres:16
        environment:
          POSTGRES_USER: framework-rust
          POSTGRES_PASSWORD: framework-rust
          POSTGRES_DB: framework-rust
        volumes:
          - pg_data:/var/lib/postgresql/data

      framework-rust:
        image: ghcr.io/shrpp/framework-rust-core:latest
        ports:
          - "3000:3000"
        environment:
          DATABASE_URL: postgresql://framework-rust:framework-rust@postgres:5432/framework-rust
          FRAMEWORK_RUST_ADMIN_KEY: change-me
          FRAMEWORK_RUST_BOOTSTRAP_ADMIN_EMAIL: admin@example.com
          FRAMEWORK_RUST_BOOTSTRAP_ADMIN_PASSWORD: Admin1234!
          # Paste generated secrets here after first run:
          # JWT_SECRET:
          # MASTER_ENCRYPTION_KEY:
          # TENANT_WRAP_KEY:
        depends_on:
          postgres:
            condition: service_started

    volumes:
      pg_data:
    ```

    ```bash
    docker compose up -d
    docker compose logs framework-rust   # grab the generated secrets
    ```
  </Step>

  <Step title="Install the Admin TUI">
    The `framework-rust` binary is a terminal UI to manage tenants, users, clients, roles, and permissions.

    <Tabs>
      <Tab title="macOS (Apple Silicon)">
        ```bash
        curl -Lo framework-rust https://github.com/shrpp/framework-rust/releases/latest/download/framework-rust-aarch64-apple-darwin
        xattr -dr com.apple.quarantine framework-rust
        chmod +x framework-rust && sudo mv framework-rust /usr/local/bin/
        ```

        <Note>
          The `xattr` command is required because the binary is unsigned in alpha. macOS Gatekeeper blocks unsigned binaries by default.
        </Note>
      </Tab>
      <Tab title="macOS (Intel)">
        ```bash
        curl -Lo framework-rust https://github.com/shrpp/framework-rust/releases/latest/download/framework-rust-x86_64-apple-darwin
        xattr -dr com.apple.quarantine framework-rust
        chmod +x framework-rust && sudo mv framework-rust /usr/local/bin/
        ```
      </Tab>
      <Tab title="Linux (x86_64)">
        ```bash
        curl -Lo framework-rust https://github.com/shrpp/framework-rust/releases/latest/download/framework-rust-x86_64-unknown-linux-gnu
        chmod +x framework-rust && sudo mv framework-rust /usr/local/bin/
        ```
      </Tab>
      <Tab title="Linux (ARM64)">
        ```bash
        curl -Lo framework-rust https://github.com/shrpp/framework-rust/releases/latest/download/framework-rust-aarch64-unknown-linux-gnu
        chmod +x framework-rust && sudo mv framework-rust /usr/local/bin/
        ```
      </Tab>
      <Tab title="Windows">
        ```powershell
        curl -Lo framework-rust.exe https://github.com/shrpp/framework-rust/releases/latest/download/framework-rust-x86_64-pc-windows-msvc.exe
        .\framework-rust.exe --url http://localhost:3000
        ```

        <Note>
          Windows SmartScreen will warn about the unsigned binary. Click **More info → Run anyway**.
        </Note>
      </Tab>
    </Tabs>
  </Step>

  <Step title="Connect and log in">
    ```bash
    framework-rust --url http://localhost:3000
    # or: FRAMEWORK_RUST_URL=http://localhost:3000 framework-rust
    ```

    When prompted for the **Admin Key**, enter the value you set in `FRAMEWORK_RUST_ADMIN_KEY`.

    A **master** tenant is created automatically on first startup using your bootstrap credentials. Navigate with arrow keys or `j`/`k`. Press `?` for the full key reference.
  </Step>

  <Step title="Test the API directly">
    Use `x-framework-rust-tenant-slug` to identify the tenant — no UUID needed during development.

    ```bash
    # Login
    curl -s -X POST http://localhost:3000/auth/login \
      -H 'x-framework-rust-tenant-slug: master' \
      -H 'Content-Type: application/json' \
      -d '{"email":"admin@example.com","password":"Admin1234!"}' | jq

    # Get current user (use the access_token from above)
    curl -s http://localhost:3000/users/me \
      -H 'x-framework-rust-tenant-slug: master' \
      -H 'Authorization: Bearer <access_token>' | jq
    ```

    <Tip>
      Use `x-framework-rust-tenant-slug: master` in development. In production, prefer `x-framework-rust-tenant-id: <uuid>` to skip the slug lookup.
    </Tip>
  </Step>
</Steps>

## Next steps

<CardGroup cols={2}>
  <Card title="Configuration" icon="sliders" href="/docs/configuration">
    All environment variables and production checklist
  </Card>
  <Card title="Admin TUI" icon="terminal" href="/docs/admin-tui">
    Full keyboard reference and tab guide
  </Card>
  <Card title="M2M / Client Credentials" icon="server" href="/docs/m2m">
    Service-to-service auth with embedded roles
  </Card>
  <Card title="API Reference" icon="rectangle-terminal" href="/docs/api-reference">
    All HTTP endpoints
  </Card>
</CardGroup>
