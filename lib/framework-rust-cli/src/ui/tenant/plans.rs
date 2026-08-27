use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{app::App, components::table::StatefulTable};

pub fn render(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    plans: &mut StatefulTable,
    quotas: &mut StatefulTable,
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);
    let rows = app
        .plans
        .iter()
        .map(|plan| {
            vec![
                plan.name.clone(),
                plan.code.clone(),
                plan.billing_interval.clone(),
                format!("{} {:.2}", plan.currency, plan.price_cents as f64 / 100.0),
                plan.trial_days.to_string(),
            ]
        })
        .collect();
    plans.select(app.plan_selected);
    plans.render(
        frame,
        areas[0],
        " Plans ",
        &["Name", "Code", "Interval", "Price", "Trial days"],
        rows,
    );

    let quota_rows = app
        .plan_quotas
        .iter()
        .map(|quota| {
            vec![
                quota.resource.clone(),
                if quota.limit == -1 {
                    "unlimited".into()
                } else {
                    quota.limit.to_string()
                },
            ]
        })
        .collect();
    quotas.select(0);
    quotas.render(
        frame,
        areas[1],
        " Selected plan quotas ",
        &["Resource", "Limit"],
        quota_rows,
    );
}
