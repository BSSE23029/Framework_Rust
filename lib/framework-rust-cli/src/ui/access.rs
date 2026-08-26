use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::app::App;

const SECTIONS: [&str; 6] = [
    "Departments",
    "Designations",
    "RDDs",
    "URDDs",
    "Groups",
    "Scopes",
];

pub fn render(frame: &mut Frame, app: &App, area: Rect, state: &mut TableState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);
    let section_title = SECTIONS
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if index == app.access_section {
                format!("[{value}]")
            } else {
                (*value).to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("  ");
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Access Organization  {section_title} ")),
        chunks[0],
    );

    let (headers, rows, widths) = rows(app);
    state.select(if rows.is_empty() {
        None
    } else {
        Some(app.access_selected.min(rows.len() - 1))
    });
    let table = Table::new(rows, widths)
        .header(
            Row::new(headers).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("▶ ")
        .block(Block::default().borders(Borders::ALL));
    frame.render_stateful_widget(table, chunks[1], state);
}

fn rows(app: &App) -> (Vec<&'static str>, Vec<Row<'static>>, Vec<Constraint>) {
    match app.access_section {
        0 => (
            vec!["Code", "Department", "Description", "ID"],
            app.departments
                .iter()
                .map(|v| Row::new(cells([&v.code, &v.name, &v.description, &v.id])))
                .collect(),
            widths(),
        ),
        1 => (
            vec!["Code", "Designation", "Description", "ID"],
            app.designations
                .iter()
                .map(|v| Row::new(cells([&v.code, &v.name, &v.description, &v.id])))
                .collect(),
            widths(),
        ),
        2 => (
            vec!["Role", "Designation", "Department", "RDD ID"],
            app.rdds
                .iter()
                .map(|v| {
                    Row::new(cells([
                        &v.role_id,
                        &v.designation_id,
                        &v.department_id,
                        &v.id,
                    ]))
                })
                .collect(),
            widths(),
        ),
        3 => (
            vec!["User", "RDD", "Attributes", "URDD ID"],
            app.urdds
                .iter()
                .map(|v| {
                    Row::new(vec![
                        Cell::from(v.user_id.clone()),
                        Cell::from(v.role_designation_department_id.clone()),
                        Cell::from(v.attributes.to_string()),
                        Cell::from(v.id.clone()),
                    ])
                })
                .collect(),
            widths(),
        ),
        4 => (
            vec!["Group", "Description", "Status", "ID"],
            app.permission_groups
                .iter()
                .map(|v| Row::new(cells([&v.name, &v.description, &v.status, &v.id])))
                .collect(),
            widths(),
        ),
        _ => (
            vec!["Scope", "Type", "Resource", "ID"],
            app.permission_scopes
                .iter()
                .map(|v| {
                    Row::new(vec![
                        Cell::from(v.name.clone()),
                        Cell::from(v.scope_type.clone()),
                        Cell::from(v.resource_type.clone().unwrap_or_default()),
                        Cell::from(v.id.clone()),
                    ])
                })
                .collect(),
            widths(),
        ),
    }
}

fn cells<const N: usize>(values: [&str; N]) -> Vec<Cell<'static>> {
    values
        .into_iter()
        .map(|value| Cell::from(value.to_owned()))
        .collect()
}

fn widths() -> Vec<Constraint> {
    vec![
        Constraint::Percentage(18),
        Constraint::Percentage(23),
        Constraint::Percentage(29),
        Constraint::Percentage(30),
    ]
}
