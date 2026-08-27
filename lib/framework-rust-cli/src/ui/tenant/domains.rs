use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_name = app.active_tenant_name().unwrap_or("?");
    let title = format!(" Domains — {tenant_name} ");
    let rows = app
        .domains
        .iter()
        .map(|domain| {
            vec![
                domain.domain_name.clone(),
                domain.domain_type.clone(),
                if domain.is_primary { "✓" } else { "—" }.into(),
                if domain.is_verified { "✓" } else { "pending" }.into(),
                if domain.ssl_enabled { "✓" } else { "—" }.into(),
                domain.status.clone(),
            ]
        })
        .collect();
    table.select(app.domain_selected);
    table.render(
        frame,
        area,
        &title,
        &["Domain", "Type", "Primary", "Verified", "SSL", "Status"],
        rows,
    );
}
