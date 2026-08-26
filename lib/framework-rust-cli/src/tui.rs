use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::{
    api::{ApiError, LoginResult},
    app::{App, AppMode, Focus, Modal, QuickStartState, Tab},
    components::{modal, statusbar, table::StatefulTable},
    config_file,
    events::{poll, AppEvent},
    ui,
};

mod access;
mod all_data;
mod auth;
mod client_actions;
mod content;
mod feature_loaders;
mod key_dispatch;
mod loaders;
mod modal_accounts;
mod modal_basic;
mod modal_idp;
mod modal_platform;
mod modal_rbac;
mod modal_special;
mod navigation;
mod quickstart;
mod rbac_actions;
mod render;
mod settings;
mod sidebar;
mod tenant_actions;

use all_data::*;
use auth::*;
use client_actions::*;
use content::*;
use feature_loaders::*;
use key_dispatch::*;
use loaders::*;
use quickstart::*;
use rbac_actions::*;
use settings::*;
use sidebar::*;
use tenant_actions::*;

pub async fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Restore saved session token if present.
    let saved_cfg = config_file::load();
    if let Some(token) = saved_cfg.token.clone() {
        app.client.set_token(token);
        app.mode = AppMode::Admin;
        load_tenants(&mut app).await;
        check_health(&mut app).await;
        // If tenant list loaded successfully we stay in Admin; otherwise drop back to Login.
        if app.tenants.is_empty() {
            app.mode = AppMode::Login {
                email: String::new(),
                password: String::new(),
                field: 0,
                error: None,
                tenant_choices: None,
                tenant_choice_idx: 0,
            };
        }
    }

    let mut client_table = StatefulTable::new();
    let mut user_table = StatefulTable::new();
    let mut idp_table = StatefulTable::new();
    let mut audit_table = StatefulTable::new();
    let mut domain_table = StatefulTable::new();
    let mut plan_table = StatefulTable::new();
    let mut quota_table = StatefulTable::new();
    let mut config_table = StatefulTable::new();
    let mut trusted_device_table = StatefulTable::new();
    let mut activity_table = StatefulTable::new();
    let mut session_list_state = ratatui::widgets::ListState::default();
    let mut role_list_state = ratatui::widgets::ListState::default();
    let mut permission_list_state = ratatui::widgets::ListState::default();
    let mut access_table_state = ratatui::widgets::TableState::default();

    loop {
        terminal.draw(|frame| {
            if matches!(
                &app.mode,
                AppMode::Login { .. } | AppMode::MfaChallenge { .. } | AppMode::Passwordless { .. }
            ) {
                ui::login::render(frame, &app);
                return;
            }

            let (sidebar, content, header, statusbar_area) = ui::layout::split_areas(frame);
            let (tabs_area, content_body) = ui::layout::split_content(content);

            ui::layout::render_header(frame, &app, header);
            ui::layout::render_tenant_sidebar(frame, &app, sidebar);
            ui::layout::render_tabs(frame, &app, tabs_area);

            match app.tab {
                Tab::Clients => ui::clients::render(frame, &app, content_body, &mut client_table),
                Tab::Users => ui::users::render(frame, &app, content_body, &mut user_table),
                Tab::Roles => ui::roles::render(frame, &app, content_body, &mut role_list_state),
                Tab::Permissions => {
                    ui::permissions::render(frame, &app, content_body, &mut permission_list_state)
                }
                Tab::Access => {
                    ui::access::render(frame, &app, content_body, &mut access_table_state)
                }
                Tab::Sessions => {
                    ui::sessions::render(frame, &app, content_body, &mut session_list_state)
                }
                Tab::Settings => ui::settings::render(frame, &app, content_body),
                Tab::IdentityProviders => {
                    ui::identity_providers::render(frame, &app, content_body, &mut idp_table)
                }
                Tab::Domains => ui::domains::render(frame, &app, content_body, &mut domain_table),
                Tab::Plans => {
                    ui::plans::render(frame, &app, content_body, &mut plan_table, &mut quota_table)
                }
                Tab::Config => ui::config::render(frame, &app, content_body, &mut config_table),
                Tab::Security => ui::security::render(
                    frame,
                    &app,
                    content_body,
                    &mut trusted_device_table,
                    &mut activity_table,
                ),
                Tab::AuditLog => ui::audit_log::render(frame, &app, content_body, &mut audit_table),
            }

            statusbar::render(frame, &app, statusbar_area);

            render::render_modal(frame, &app);
        })?;

        match poll()? {
            Some(AppEvent::Key(key)) => {
                if matches!(&app.mode, AppMode::Login { .. }) {
                    handle_login_key(&mut app, key.code).await;
                } else if matches!(&app.mode, AppMode::MfaChallenge { .. }) {
                    handle_mfa_key(&mut app, key.code).await;
                } else if matches!(&app.mode, AppMode::Passwordless { .. }) {
                    handle_passwordless_key(&mut app, key.code).await;
                } else {
                    handle_key(&mut app, key.code, key.modifiers).await;
                }
                if app.should_quit {
                    break;
                }
            }
            Some(AppEvent::Tick) => {}
            None => break,
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
