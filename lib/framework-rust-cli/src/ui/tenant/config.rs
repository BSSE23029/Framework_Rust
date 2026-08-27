use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_name = app.active_tenant_name().unwrap_or("?");
    let rows = app
        .config_definitions
        .iter()
        .map(|definition| {
            let value = app
                .config_values
                .get(&definition.id)
                .cloned()
                .or_else(|| definition.default_value.as_ref().map(ToString::to_string))
                .unwrap_or_else(|| "null".into());
            vec![
                definition.display_name.clone(),
                definition.config_key.clone(),
                definition.category.clone(),
                definition.value_type.clone(),
                value,
            ]
        })
        .collect();
    table.select(app.config_selected);
    table.render(
        frame,
        area,
        &format!(" Typed Configuration — {tenant_name} "),
        &["Name", "Key", "Category", "Type", "Resolved value"],
        rows,
    );
}
