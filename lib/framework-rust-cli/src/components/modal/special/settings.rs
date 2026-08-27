use super::*;

// ── Settings section modals ───────────────────────────────────────────────────

fn settings_hint_line<'a>() -> Line<'a> {
    Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Next field  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ])
}

fn settings_text_field(label: &str, val: &str, active: bool) -> (Block<'static>, String) {
    let border = Block::default()
        .title(label.to_string())
        .borders(Borders::ALL)
        .border_style(if active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    let display = if active {
        format!("{val}█")
    } else {
        val.to_string()
    };
    (border, display)
}

fn settings_toggle_line(label: &str, val: bool, active: bool) -> Line<'static> {
    let (bullet, color) = if val {
        ("●", Color::Cyan)
    } else {
        ("○", Color::DarkGray)
    };
    let text = if val { "enabled" } else { "disabled" };
    let bg = if active {
        Color::DarkGray
    } else {
        Color::Reset
    };
    Line::from(vec![
        Span::styled(format!("{bullet} "), Style::default().fg(color)),
        Span::styled(
            format!("{label}: {text}"),
            Style::default().fg(Color::White).bg(bg),
        ),
        Span::styled("  [Space]", Style::default().fg(Color::DarkGray)),
    ])
}

/// Password Policy modal — field: 0=min_length, 1=require_uppercase, 2=require_digit, 3=require_special
#[allow(clippy::too_many_arguments)]
pub fn render_settings_password_policy(
    frame: &mut Frame,
    min_length: &str,
    require_uppercase: bool,
    require_digit: bool,
    require_special: bool,
    field: usize,
) {
    let area = centered_rect(66, 14, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Password Policy ")
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
            Constraint::Length(3), // min_length
            Constraint::Length(1), // spacer
            Constraint::Length(1), // uppercase
            Constraint::Length(1), // digit
            Constraint::Length(1), // special
            Constraint::Min(1),
            Constraint::Length(1), // hints
        ])
        .split(inner);

    let (blk, disp) = settings_text_field("Min Length", min_length, field == 0);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[0]);
    frame.render_widget(
        Paragraph::new(settings_toggle_line(
            "Require Uppercase",
            require_uppercase,
            field == 1,
        )),
        chunks[2],
    );
    frame.render_widget(
        Paragraph::new(settings_toggle_line(
            "Require Digit",
            require_digit,
            field == 2,
        )),
        chunks[3],
    );
    frame.render_widget(
        Paragraph::new(settings_toggle_line(
            "Require Special (!@#...)",
            require_special,
            field == 3,
        )),
        chunks[4],
    );
    frame.render_widget(
        Paragraph::new(settings_hint_line()).alignment(Alignment::Center),
        chunks[6],
    );
}

/// Lockout modal — field: 0=max_attempts, 1=window_minutes, 2=duration_minutes
pub fn render_settings_lockout(
    frame: &mut Frame,
    max_attempts: &str,
    window_minutes: &str,
    duration_minutes: &str,
    field: usize,
) {
    let area = centered_rect(66, 14, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Lockout ")
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let (blk, disp) = settings_text_field("Max Attempts", max_attempts, field == 0);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[0]);
    let (blk, disp) = settings_text_field("Window (minutes)", window_minutes, field == 1);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[1]);
    let (blk, disp) =
        settings_text_field("Lockout Duration (minutes)", duration_minutes, field == 2);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[2]);
    frame.render_widget(
        Paragraph::new(settings_hint_line()).alignment(Alignment::Center),
        chunks[4],
    );
}

/// Token TTL modal — field: 0=access_token_ttl_minutes, 1=refresh_token_ttl_days
pub fn render_settings_tokens(
    frame: &mut Frame,
    access_token_ttl_minutes: &str,
    refresh_token_ttl_days: &str,
    field: usize,
) {
    let area = centered_rect(66, 12, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Token TTL ")
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let (blk, disp) = settings_text_field(
        "Access Token TTL (minutes)",
        access_token_ttl_minutes,
        field == 0,
    );
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[0]);
    let (blk, disp) = settings_text_field(
        "Refresh Token TTL (days)",
        refresh_token_ttl_days,
        field == 1,
    );
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[1]);
    frame.render_widget(
        Paragraph::new(settings_hint_line()).alignment(Alignment::Center),
        chunks[3],
    );
}

/// Registration modal — field: 0=allow_public_registration, 1=require_email_verified
pub fn render_settings_registration(
    frame: &mut Frame,
    allow_public_registration: bool,
    require_email_verified: bool,
    field: usize,
) {
    let area = centered_rect(66, 10, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Registration ")
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(settings_toggle_line(
            "Allow Public Registration",
            allow_public_registration,
            field == 0,
        )),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(settings_toggle_line(
            "Require Email Verified to Login",
            require_email_verified,
            field == 1,
        )),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(settings_hint_line()).alignment(Alignment::Center),
        chunks[3],
    );
}

/// SMTP modal — field: 0=host, 1=port, 2=username, 3=password(masked),
///              4=from_name, 5=from_email, 6=use_tls(toggle), 7=enabled(toggle)
#[allow(clippy::too_many_arguments)]
pub fn render_settings_smtp(
    frame: &mut Frame,
    host: &str,
    port: &str,
    username: &str,
    password: &str,
    from_name: &str,
    from_email: &str,
    use_tls: bool,
    enabled: bool,
    field: usize,
) {
    let area = centered_rect(66, 26, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" SMTP ")
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
            Constraint::Length(3), // host
            Constraint::Length(3), // port
            Constraint::Length(3), // username
            Constraint::Length(3), // password
            Constraint::Length(3), // from_name
            Constraint::Length(3), // from_email
            Constraint::Length(1), // use_tls
            Constraint::Length(1), // enabled
            Constraint::Min(1),
            Constraint::Length(1), // hints
        ])
        .split(inner);

    let (blk, disp) = settings_text_field("Host", host, field == 0);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[0]);
    let (blk, disp) = settings_text_field("Port", port, field == 1);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[1]);
    let (blk, disp) = settings_text_field("Username", username, field == 2);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[2]);

    // Password — masked
    let pw_display = if field == 3 {
        format!("{}█", "•".repeat(password.len()))
    } else {
        "•".repeat(password.len())
    };
    let pw_block = Block::default()
        .title("Password")
        .borders(Borders::ALL)
        .border_style(if field == 3 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    frame.render_widget(Paragraph::new(pw_display).block(pw_block), chunks[3]);

    let (blk, disp) = settings_text_field("From Name", from_name, field == 4);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[4]);
    let (blk, disp) = settings_text_field("From Email", from_email, field == 5);
    frame.render_widget(Paragraph::new(disp).block(blk), chunks[5]);

    frame.render_widget(
        Paragraph::new(settings_toggle_line("STARTTLS", use_tls, field == 6)),
        chunks[6],
    );
    frame.render_widget(
        Paragraph::new(settings_toggle_line("Enabled", enabled, field == 7)),
        chunks[7],
    );
    frame.render_widget(
        Paragraph::new(settings_hint_line()).alignment(Alignment::Center),
        chunks[9],
    );
}

/// Post-create tenant prompt: run setup wizard?
pub fn render_post_create_tenant(frame: &mut Frame, tenant_name: &str) {
    let area = centered_rect(50, 7, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Tenant created ")
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(format!("\"{}\" is ready.", tenant_name)).alignment(Alignment::Center),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new("Run the setup wizard?").alignment(Alignment::Center),
        chunks[1],
    );

    let hint = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled("  Yes     ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled("  Skip", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), chunks[3]);
}
