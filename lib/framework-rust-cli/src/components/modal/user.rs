use super::*;

/// Comprehensive user editor: email, password, is_active, roles (toggle), permissions (read-only).
/// field: 0=email, 1=password, 2=is_active, 3=roles section
#[allow(clippy::too_many_arguments)]
pub fn render_edit_user(
    frame: &mut Frame,
    email: &str,
    password: &str,
    is_active: bool,
    all_roles: &[(String, String, bool)],
    permissions: &[String],
    passkeys: &[crate::api::PasskeyInfo],
    field: usize,
    role_selected: usize,
    passkey_selected: usize,
) {
    let roles_visible = all_roles.len().clamp(1, 5) as u16;
    let perms_visible = permissions.len().clamp(1, 4) as u16;
    let pk_visible = passkeys.len().clamp(1, 3) as u16;
    let height = 3 + 3 + 2 + 1 + roles_visible + 1 + perms_visible + 1 + pk_visible + 1 + 2;
    let area = centered_rect(66, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Edit User ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let constraints = vec![
        Constraint::Length(3),             // email field
        Constraint::Length(3),             // password field
        Constraint::Length(2),             // is_active + separator
        Constraint::Length(1),             // roles header
        Constraint::Length(roles_visible), // roles list
        Constraint::Length(1),             // permissions header
        Constraint::Length(perms_visible), // permissions
        Constraint::Length(1),             // passkeys header
        Constraint::Length(pk_visible),    // passkeys list
        Constraint::Length(1),             // hint
    ];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Email
    let email_active = field == 0;
    let email_block = Block::default()
        .title("Email")
        .borders(Borders::ALL)
        .border_style(if email_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    let email_display = if email_active {
        format!("{email}█")
    } else {
        email.to_string()
    };
    frame.render_widget(Paragraph::new(email_display).block(email_block), chunks[0]);

    // Password
    let pw_active = field == 1;
    let pw_block = Block::default()
        .title("Password (leave blank to keep)")
        .borders(Borders::ALL)
        .border_style(if pw_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    let pw_display = if pw_active {
        format!("{}█", "•".repeat(password.len()))
    } else {
        "•".repeat(password.len())
    };
    frame.render_widget(Paragraph::new(pw_display).block(pw_block), chunks[1]);

    // is_active + separator
    let status_active = field == 2;
    let status_color = if is_active { Color::Green } else { Color::Red };
    let status_text = if is_active {
        "● active"
    } else {
        "○ inactive"
    };
    let active_indicator = if status_active { " ◀ focused" } else { "" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Status  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  [Space]{active_indicator}"),
                Style::default().fg(Color::DarkGray),
            ),
        ])),
        chunks[2],
    );

    // Roles header
    let roles_active = field == 3;
    let roles_label = if roles_active {
        "── Roles [Space=toggle] ──"
    } else {
        "── Roles ──"
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            roles_label,
            Style::default().fg(if roles_active {
                Color::Cyan
            } else {
                Color::DarkGray
            }),
        )),
        chunks[3],
    );

    // Roles list
    let visible_start = role_selected.saturating_sub(roles_visible.saturating_sub(1) as usize);
    let role_items: Vec<ListItem> = all_roles
        .iter()
        .skip(visible_start)
        .take(roles_visible as usize)
        .enumerate()
        .map(|(i, (_id, name, assigned))| {
            let actual_idx = visible_start + i;
            let is_sel = roles_active && actual_idx == role_selected;
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
            ListItem::new(Line::from(vec![
                Span::styled(format!("{bullet} "), Style::default().fg(color)),
                Span::styled(name.as_str(), Style::default().fg(Color::White).bg(bg)),
            ]))
        })
        .collect();
    let roles_list = List::new(role_items);
    let mut dummy_state = ListState::default();
    frame.render_stateful_widget(roles_list, chunks[4], &mut dummy_state);

    // Permissions header
    frame.render_widget(
        Paragraph::new(Span::styled(
            "── Permissions (from roles, read-only) ──",
            Style::default().fg(Color::DarkGray),
        )),
        chunks[5],
    );

    // Permissions list
    let perm_items: Vec<ListItem> = if permissions.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        permissions
            .iter()
            .take(perms_visible as usize)
            .map(|p| {
                ListItem::new(Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Yellow)),
                    Span::styled(p.as_str(), Style::default().fg(Color::White)),
                ]))
            })
            .collect()
    };
    let mut dummy_state2 = ListState::default();
    frame.render_stateful_widget(List::new(perm_items), chunks[6], &mut dummy_state2);

    // Passkeys header
    let pk_active = field == 4;
    let pk_label = if pk_active {
        "── Passkeys [d=delete] ──"
    } else {
        "── Passkeys ──"
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            pk_label,
            Style::default().fg(if pk_active {
                Color::Cyan
            } else {
                Color::DarkGray
            }),
        )),
        chunks[7],
    );

    // Passkeys list
    let pk_items: Vec<ListItem> = if passkeys.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (none registered)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        let visible_start = passkey_selected.saturating_sub(pk_visible.saturating_sub(1) as usize);
        passkeys
            .iter()
            .enumerate()
            .skip(visible_start)
            .take(pk_visible as usize)
            .map(|(i, pk)| {
                let is_sel = pk_active && i == passkey_selected;
                let last = pk
                    .last_used_at
                    .as_deref()
                    .map(|s| &s[..10])
                    .unwrap_or("never");
                let bg = if is_sel {
                    Color::DarkGray
                } else {
                    Color::Reset
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        if is_sel { "▶ " } else { "  " },
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(pk.name.as_str(), Style::default().fg(Color::White).bg(bg)),
                    Span::styled(
                        format!("  last: {last}"),
                        Style::default().fg(Color::DarkGray).bg(bg),
                    ),
                ]))
            })
            .collect()
    };
    let mut dummy_pk = ListState::default();
    frame.render_stateful_widget(List::new(pk_items), chunks[8], &mut dummy_pk);

    // Hints
    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Section  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("d", Style::default().fg(Color::Cyan)),
        Span::styled(" Del passkey  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[9],
    );
}
