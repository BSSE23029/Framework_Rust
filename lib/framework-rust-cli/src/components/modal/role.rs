use super::*;

/// Role editor: name, description, permissions (toggle).
/// field: 0=name, 1=description, 2=permissions section
pub fn render_edit_role(
    frame: &mut Frame,
    name: &str,
    description: &str,
    all_permissions: &[(String, String, bool)],
    field: usize,
    perm_selected: usize,
) {
    let perms_visible = all_permissions.len().clamp(1, 6) as u16;
    let height = 3 + 3 + 1 + perms_visible + 1 + 1 + 2; // name+desc+header+perms+spacer+hints+borders
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Edit Role ")
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
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(perms_visible),
        Constraint::Min(1),
        Constraint::Length(1),
    ];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Name
    let name_active = field == 0;
    let name_block = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .border_style(if name_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    frame.render_widget(
        Paragraph::new(if name_active {
            format!("{name}█")
        } else {
            name.to_string()
        })
        .block(name_block),
        chunks[0],
    );

    // Description
    let desc_active = field == 1;
    let desc_block = Block::default()
        .title("Description")
        .borders(Borders::ALL)
        .border_style(if desc_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    frame.render_widget(
        Paragraph::new(if desc_active {
            format!("{description}█")
        } else {
            description.to_string()
        })
        .block(desc_block),
        chunks[1],
    );

    // Permissions header
    let perms_active = field == 2;
    let perms_label = if perms_active {
        "── Permissions [Space=toggle] ──"
    } else {
        "── Permissions ──"
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            perms_label,
            Style::default().fg(if perms_active {
                Color::Cyan
            } else {
                Color::DarkGray
            }),
        )),
        chunks[2],
    );

    // Permissions list
    let visible_start = perm_selected.saturating_sub(perms_visible.saturating_sub(1) as usize);
    let perm_items: Vec<ListItem> = if all_permissions.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (no permissions defined yet)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        all_permissions
            .iter()
            .skip(visible_start)
            .take(perms_visible as usize)
            .enumerate()
            .map(|(i, (_id, name, assigned))| {
                let actual_idx = visible_start + i;
                let is_sel = perms_active && actual_idx == perm_selected;
                let bullet = if *assigned { "●" } else { "○" };
                let color = if *assigned {
                    Color::Yellow
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
            .collect()
    };
    let mut dummy = ListState::default();
    frame.render_stateful_widget(List::new(perm_items), chunks[3], &mut dummy);

    // Hints
    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Section  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        chunks[5],
    );
}
