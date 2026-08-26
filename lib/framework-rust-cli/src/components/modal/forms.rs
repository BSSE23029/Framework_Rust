use super::*;

/// (role_id, role_name, is_assigned)
pub fn render_user_roles(
    frame: &mut Frame,
    email: &str,
    all_roles: &[(String, String, bool)],
    selected: usize,
) {
    let height = (all_roles.len() as u16 + 6).max(8);
    let area = centered_rect(56, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" Roles — {} ", email))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let mut constraints: Vec<ratatui::layout::Constraint> = all_roles
        .iter()
        .map(|_| ratatui::layout::Constraint::Length(1))
        .collect();
    constraints.push(ratatui::layout::Constraint::Min(1));
    constraints.push(ratatui::layout::Constraint::Length(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, (_id, name, assigned)) in all_roles.iter().enumerate() {
        let is_sel = i == selected;
        let bullet = if *assigned { "●" } else { "○" };
        let color = if *assigned {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let bg = if is_sel {
            Color::DarkGray
        } else {
            Color::Reset
        };
        let line = Line::from(vec![
            Span::styled(format!("{bullet} "), Style::default().fg(color)),
            Span::styled(name.as_str(), Style::default().fg(Color::White).bg(bg)),
        ]);
        frame.render_widget(Paragraph::new(line), chunks[i]);
    }

    let hint_idx = all_roles.len() + 1;
    let hints = Line::from(vec![
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::styled(" Toggle   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[hint_idx],
    );
}

/// Create-client modal with text fields + visual type toggle.
pub fn render_create_client(
    frame: &mut Frame,
    name: &str,
    redirect_uri: &str,
    scopes: &str,
    client_type: u8,
    field: usize,
) {
    let area = centered_rect(62, 17, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" New Client ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Redirect URI
            Constraint::Length(3), // Scopes
            Constraint::Length(3), // Type toggle
            Constraint::Min(1),
            Constraint::Length(1), // hints
        ])
        .split(inner);

    let border = |active: bool| {
        if active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    // Name
    let a = field == 0;
    frame.render_widget(
        Paragraph::new(if a {
            format!("{name}█")
        } else {
            name.to_string()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Name")
                .border_style(border(a)),
        ),
        chunks[0],
    );

    // Redirect URI
    let a = field == 1;
    frame.render_widget(
        Paragraph::new(if a {
            format!("{redirect_uri}█")
        } else {
            redirect_uri.to_string()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Redirect URI")
                .border_style(border(a)),
        ),
        chunks[1],
    );

    // Scopes
    let a = field == 2;
    frame.render_widget(
        Paragraph::new(if a {
            format!("{scopes}█")
        } else {
            scopes.to_string()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Scopes")
                .border_style(border(a)),
        ),
        chunks[2],
    );

    // Type toggle — shows all 3 options, selected one in cyan bold
    let type_active = field == 3;
    let title = if type_active { "Type  ←/→" } else { "Type" };
    let labels = ["Confidential", "SPA/Mobile", "Machine (M2M)"];
    let mut spans: Vec<Span> = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        if i as u8 == client_type {
            spans.push(Span::styled(
                format!("● {label}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("○ {label}"),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border(type_active)),
        ),
        chunks[3],
    );

    // Hints
    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Next   ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::Cyan)),
        Span::styled(" Type   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Create   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[5],
    );
}

/// Render a simple form modal with labelled fields.
/// `fields`: list of `(label, value, placeholder)` triples.
/// `active_field`: index of the currently focused input.
pub fn render_form(
    frame: &mut Frame,
    title: &str,
    fields: &[(&str, &str, &str)],
    active_field: usize,
) {
    let height = (fields.len() as u16) * 3 + 5;
    let area = centered_rect(62, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let mut constraints = vec![];
    for _ in fields {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, (label, value, placeholder)) in fields.iter().enumerate() {
        let is_active = i == active_field;
        let border_style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_block = Block::default()
            .title(*label)
            .borders(Borders::ALL)
            .border_style(border_style);

        let para = if is_active {
            Paragraph::new(format!("{value}█")).block(input_block)
        } else if value.is_empty() && !placeholder.is_empty() {
            Paragraph::new(Span::styled(
                *placeholder,
                Style::default().fg(Color::DarkGray),
            ))
            .block(input_block)
        } else {
            Paragraph::new(value.to_string()).block(input_block)
        };

        frame.render_widget(para, chunks[i]);
    }

    let hint_area = chunks[fields.len()];
    let hint = Paragraph::new("[Tab] Next field   [Enter] Submit   [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, hint_area);
}

/// Edit client modal: text fields + type toggle (field 5).
#[allow(clippy::too_many_arguments)]
pub fn render_edit_client(
    frame: &mut Frame,
    name: &str,
    redirect_uris: &str,
    scopes: &str,
    access_token_ttl: &str,
    refresh_token_ttl: &str,
    client_type: u8,
    field: usize,
) {
    let area = centered_rect(64, 23, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Edit Client ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Redirect URIs
            Constraint::Length(3), // Scopes
            Constraint::Length(3), // Access token TTL
            Constraint::Length(3), // Refresh token TTL
            Constraint::Length(3), // Type toggle
            Constraint::Min(1),
            Constraint::Length(1), // hints
        ])
        .split(inner);

    let border = |active: bool| {
        if active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let text_field = |val: &str, placeholder: &str, active: bool| -> Paragraph<'_> {
        if active {
            Paragraph::new(format!("{val}█"))
        } else if val.is_empty() {
            Paragraph::new(Span::styled(
                placeholder.to_string(),
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Paragraph::new(val.to_string())
        }
    };

    frame.render_widget(
        text_field(name, "e.g. My App", field == 0).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Name")
                .border_style(border(field == 0)),
        ),
        chunks[0],
    );
    frame.render_widget(
        text_field(
            redirect_uris,
            "https://app.example.com/callback",
            field == 1,
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Redirect URIs (comma-separated)")
                .border_style(border(field == 1)),
        ),
        chunks[1],
    );
    frame.render_widget(
        text_field(scopes, "openid email profile", field == 2).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Scopes")
                .border_style(border(field == 2)),
        ),
        chunks[2],
    );
    frame.render_widget(
        text_field(access_token_ttl, "blank = tenant default", field == 3).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Access Token TTL (minutes, blank = default)")
                .border_style(border(field == 3)),
        ),
        chunks[3],
    );
    frame.render_widget(
        text_field(refresh_token_ttl, "blank = tenant default", field == 4).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Refresh Token TTL (days, blank = default)")
                .border_style(border(field == 4)),
        ),
        chunks[4],
    );

    let type_active = field == 5;
    let title = if type_active {
        "Type  ←/→ or Space"
    } else {
        "Type"
    };
    let labels = ["Confidential", "SPA/Mobile", "Machine (M2M)"];
    let mut spans: Vec<Span> = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        if i as u8 == client_type {
            spans.push(Span::styled(
                format!("● {label}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("○ {label}"),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border(type_active)),
        ),
        chunks[5],
    );

    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Next   ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::Cyan)),
        Span::styled(" Type   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[7],
    );
}
