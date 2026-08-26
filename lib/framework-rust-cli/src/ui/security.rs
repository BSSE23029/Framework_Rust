use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{app::App, components::table::StatefulTable};

pub fn render(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    devices: &mut StatefulTable,
    activity: &mut StatefulTable,
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    let device_rows = app
        .trusted_devices
        .iter()
        .map(|device| {
            vec![
                device.name.clone(),
                device.ip_address.clone().unwrap_or_else(|| "—".into()),
                device
                    .last_used_at
                    .clone()
                    .unwrap_or_else(|| "never".into()),
                device.expires_at.clone(),
            ]
        })
        .collect();
    devices.select(app.trusted_device_selected);
    devices.render(
        frame,
        areas[0],
        " My trusted devices ",
        &["Name", "IP", "Last used", "Expires"],
        device_rows,
    );

    let activity_rows = app
        .request_activity
        .iter()
        .map(|row| {
            vec![
                row.created_at.clone(),
                row.http_method.clone(),
                row.request_path.clone(),
                row.response_status.to_string(),
                format!("{}ms", row.response_time_ms),
                row.platform.clone().unwrap_or_else(|| "—".into()),
            ]
        })
        .collect();
    activity.select(0);
    activity.render(
        frame,
        areas[1],
        " My request activity ",
        &["Time", "Method", "Path", "Status", "Latency", "Platform"],
        activity_rows,
    );
}
