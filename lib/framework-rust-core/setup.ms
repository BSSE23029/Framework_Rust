Run the core and TUI in two separate terminals.

Before the first start, expand [the core `.env`](/Users/raza/Documents/GitHub/Framework_Rust/lib/framework-rust-core/.env) with persistent secrets and local bootstrap credentials:

```env
DATABASE_URL=postgresql://raza@localhost:5432/framework_rust

FRAMEWORK_RUST_DEV_MODE=1
FRAMEWORK_RUST_BOOTSTRAP_TENANT_SLUG=master
FRAMEWORK_RUST_BOOTSTRAP_ADMIN_EMAIL=admin@example.com
FRAMEWORK_RUST_BOOTSTRAP_ADMIN_PASSWORD=Admin1234!

JWT_SECRET=<openssl rand -base64 48>
MASTER_ENCRYPTION_KEY=<openssl rand -base64 48>
TENANT_WRAP_KEY=<openssl rand -base64 48>

SERVER_HOST=127.0.0.1
SERVER_PORT=3000
ENVIRONMENT=development
CORS_ALLOWED_ORIGINS=*
```

Run `openssl rand -base64 48` three times and replace each placeholder with a different generated value. Keep these secrets stable after data is created.

Terminal 1 — core server:

```bash
cd /Users/raza/Documents/GitHub/Framework_Rust/lib/framework-rust-core
cargo run --bin framework-rust-core
```

Once running:

- API: http://localhost:3000
- Swagger UI: http://localhost:3000/docs
- OpenAPI specification: http://localhost:3000/openapi.json

Terminal 2 — admin TUI:

```bash
cd /Users/raza/Documents/GitHub/Framework_Rust/lib
cargo run -p framework-rust-cli -- serve --url http://localhost:3000
```

Log in with:

```text
Tenant:  master
Email:   admin@example.com
Password: Admin1234!
```

Press `q` to exit the TUI and `Ctrl+C` to stop the core server. The bootstrap account is only created when the database contains no tenants.