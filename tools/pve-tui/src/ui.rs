//! pve-tui UI rendering with ratatui

use ratatui::{
    Frame,
    layout::Rect,
    layout::Constraint,
    layout::Layout,
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap, List},
};

use super::app::{AppState, View, SetupField, AppMode};

pub fn render(app: &AppState, f: &mut Frame) {
    let area = f.area();

    let header_height = 3u16;
    let status_bar_height = 2u16;

    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(0),
        Constraint::Length(status_bar_height),
    ]).split(area);

    match app.mode {
        AppMode::Setup => render_setup(app, f, chunks[1]),
        AppMode::Running => {
            render_header(app, f, chunks[0]);
            render_content(app, f, chunks[1]);
            render_status_bar(app, f, chunks[2]);
        }
    }
}

fn render_setup(app: &AppState, f: &mut Frame, area: Rect) {
    // Centered setup form
    let block = Block::default()
        .style(Style::new().bg(Color::Rgb(33, 37, 43)))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Rgb(60, 70, 85)));
    f.render_widget(block, area);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));

    // Title
    let title = Paragraph::new(
        Line::from(" PVE Manager - Connection Setup ").style(Style::new().bold().fg(Color::Cyan))
    );
    f.render_widget(title, Rect::new(inner.x, inner.y, inner.width, 1));

    // Separator line
    let sep = Paragraph::new(Line::from("─".repeat(inner.width as usize)).style(Style::new().fg(Color::Rgb(60, 70, 85))));
    f.render_widget(sep, Rect::new(inner.x, inner.y + 1, inner.width, 1));

    // Form fields
    let fields = SetupField::all();
    let line_height = 2u16;
    let form_top = inner.y + 3;

    for (i, field) in fields.iter().enumerate() {
        let y = form_top + (i as u16) * line_height;
        if y + line_height > inner.y + inner.height.saturating_sub(1) {
            break;
        }

        let is_active = *field == app.setup_field;
        let label = format!("{}: ", field.label());
        let value = app.get_value(*field);
        let cursor = app.setup_cursor;

        // Build display text with cursor
        let display = if is_active && *field != SetupField::VerifySsl && *field != SetupField::Connect {
            if cursor <= value.len() {
                let before = &value[..cursor];
                let after = &value[cursor..];
                format!("{}{}{}", before, "█", after)
            } else {
                format!("{}█", value)
            }
        } else {
            value.to_string()
        };

        let bg_color = if is_active { Color::Rgb(45, 48, 55) } else { Color::Rgb(33, 37, 43) };
        let fg_color = if is_active { Color::Yellow } else { Color::White };

        let line = Paragraph::new(Line::from(format!("{}{}", label, display)))
            .style(Style::new().fg(fg_color).bg(bg_color));
        f.render_widget(line, Rect::new(inner.x, y, inner.width, line_height));

        // For AuthMethod, show toggle hint
        if *field == SetupField::AuthMethod {
            let hint_line = Paragraph::new(Line::from(format!(
                "{}: {} (T/P)",
                field.label(),
                app.get_value(*field),
            )).style(Style::new().fg(if is_active { Color::Yellow } else { Color::Gray })));
            f.render_widget(hint_line, Rect::new(inner.x, y, inner.width, line_height));
        } else if *field == SetupField::Password {
            // Show masked password
            let masked = "*".repeat(app.setup_config.password.len());
            let display = if is_active {
                if app.setup_cursor <= masked.len() {
                    let before = &masked[..app.setup_cursor];
                    let after = &masked[app.setup_cursor..];
                    format!("{}{}{}", before, "█", after)
                } else {
                    format!("{}█", masked)
                }
            } else {
                masked
            };
            let bg_color = if is_active { Color::Rgb(45, 48, 55) } else { Color::Rgb(33, 37, 43) };
            let fg_color = if is_active { Color::Yellow } else { Color::White };
            let line = Paragraph::new(Line::from(format!("{}: {}", field.label(), display)))
                .style(Style::new().fg(fg_color).bg(bg_color));
            f.render_widget(line, Rect::new(inner.x, y, inner.width, line_height));
        } else if *field == SetupField::VerifySsl {
            let hint = if app.setup_config.verify_ssl { " (Y/n)" } else { " (y/N)" };
            let hint_line = Paragraph::new(Line::from(format!(
                "{}: {} {}",
                field.label(),
                if app.setup_config.verify_ssl { "true" } else { "false" },
                hint
            )).style(Style::new().fg(if is_active { Color::Yellow } else { Color::Gray })));
            f.render_widget(hint_line, Rect::new(inner.x, y, inner.width, line_height));
        } else if *field != SetupField::Connect {
            let line = Paragraph::new(Line::from(format!("{}{}", label, display)))
                .style(Style::new().fg(fg_color).bg(bg_color));
            f.render_widget(line, Rect::new(inner.x, y, inner.width, line_height));
        }
    }

    // Instructions
    let instr_y = inner.y + inner.height.saturating_sub(3);
    let instr = Paragraph::new(Line::from(vec![
        "  ".into(),
        "Tab/↓: Next field  ".into(),
        "Shift+Tab/↑: Prev  ".into(),
        "Enter: Select/Connect  ".into(),
        "Esc: Back  ".into(),
    ]).style(Style::new().fg(Color::Gray)));
    f.render_widget(instr, Rect::new(inner.x, instr_y, inner.width, 1));

    // Connecting indicator
    if app.connecting {
        let conn_msg = Paragraph::new(Line::from("  Connecting... ").style(Style::new().fg(Color::Yellow).bold()));
        f.render_widget(conn_msg, Rect::new(inner.x, instr_y + 1, inner.width, 1));
    }

    // Error message
    if let Some(ref err) = app.error_msg {
        let err_msg = Paragraph::new(Line::from(format!("  Error: {} ", err)).style(Style::new().fg(Color::Red)));
        f.render_widget(err_msg, Rect::new(inner.x, inner.y + inner.height.saturating_sub(1), inner.width, 1));
    }
}

fn render_header(app: &AppState, f: &mut Frame, area: Rect) {
    let inner = area;

    let block = Block::default().style(Style::new().bg(Color::Rgb(33, 37, 43)));
    f.render_widget(block, area);

    let title = format!(" PVE Manager  |  {}  ", app.pve_host);
    let title_len = title.len() as u16;
    f.render_widget(
        Paragraph::new(Line::from(title.as_str()).style(Style::new().bold().fg(Color::White))),
        Rect::new(inner.x, inner.y, title_len.min(inner.width), inner.height),
    );

    let tabs = ["Dashboard", "VMs", "Storage", "Logs", "Help"];
    let tab_count = tabs.len() as u16;
    let available_width = inner.width.saturating_sub(title_len + 2).max(1);
    let tab_width = available_width / tab_count;

    for (i, tab) in tabs.iter().enumerate() {
        let x = inner.x + title_len + 1 + (i as u16) * tab_width;
        let ta = Rect::new(x, inner.y, tab_width.saturating_sub(1), inner.height);
        let active = app.view.index() == i;
        let style = if active {
            Style::new().bg(Color::Blue).bold().fg(Color::White)
        } else {
            Style::new().bg(Color::Rgb(45, 48, 55)).fg(Color::Gray)
        };
        f.render_widget(
            Paragraph::new(Line::from(format!(" {} ", tab))).style(style),
            ta,
        );
    }
}

fn render_content(app: &AppState, f: &mut Frame, area: Rect) {
    let b = Block::default()
        .style(Style::new().bg(Color::Rgb(33, 37, 43)))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Rgb(60, 70, 85)));
    f.render_widget(b, area);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

    match app.view {
        View::Dashboard => render_dashboard(app, f, inner),
        View::VMs => render_vm_list(app, f, inner),
        View::Storage => render_storage(app, f, inner),
        View::Logs => render_logs(app, f, inner),
        View::Help => render_help(f, inner),
    }
}

fn render_dashboard(app: &AppState, f: &mut Frame, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(area);

    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(rows[0]);

    // Top left: PVE Version
    render_panel(f, cols[0], "Version", || {
        app.version.as_ref()
            .map(|j| j.get("version").or_else(|| j.get("release")).and_then(|s| s.as_str()).unwrap_or("?"))
            .unwrap_or("Not connected")
            .to_string()
    });

    // Top right: Cluster status
    render_panel(f, cols[1], "Cluster", || {
        app.cluster_status.as_ref()
            .map(|cs| serde_json::to_string_pretty(cs).unwrap_or_else(|_| "?".to_string()))
            .unwrap_or_else(|| "No data".to_string())
    });

    // Bottom left: Nodes
    render_panel(f, rows[1], "Nodes", || {
        if let Some(ref n) = app.nodes {
            let data = n.pointer("/data").unwrap_or(n);
            if let Some(arr) = data.as_array() {
                let mut lines = vec![format!("Total: {} nodes", arr.len())];
                for node in arr.iter().take(8) {
                    let name = node.get("node").or_else(|| node.get("id")).and_then(|s| s.as_str()).unwrap_or("-");
                    let status = node.get("status").and_then(|s| s.as_str()).unwrap_or("-");
                    let s_icon = if status == "online" { "●" } else { "○" };
                    let _color = if status == "online" { Color::Green } else { Color::Red };
                    lines.push(format!(" {} {} ", s_icon, name));
                }
                return lines.join("\n");
            }
        }
        "No data".to_string()
    });

    // Bottom right: Storage
    render_panel(f, rows[1], "Storage", || {
        if let Some(ref s) = app.storage_list {
            let data = s.pointer("/data").unwrap_or(s);
            if let Some(arr) = data.as_array() {
                let mut lines = vec![format!("Total: {} storages", arr.len())];
                for st in arr.iter().take(8) {
                    let id = st.get("id").or_else(|| st.get("storage")).and_then(|v| v.as_str()).unwrap_or("-");
                    lines.push(format!(" {} ", id));
                }
                return lines.join("\n");
            }
        }
        "No data".to_string()
    });
}

fn render_vm_list(app: &AppState, f: &mut Frame, area: Rect) {
    if app.vm_list.is_empty() {
        f.render_widget(Paragraph::new("No VMs. Press R to refresh.").style(Style::new().fg(Color::Gray)), area);
        return;
    }

    let rows: Vec<Row> = app.vm_list.iter().enumerate().map(|(i, vm)| {
        let sel = Some(i) == app.selected_vm;
        let bg = if sel { Color::Blue } else { Color::Rgb(33, 37, 43) };
        let fg = match vm.status.as_str() {
            "running" => Color::Green,
            "stopped" => Color::Red,
            "paused" => Color::Yellow,
            _ => Color::White,
        };
        Row::new(vec![
            vm.vmid.to_string(),
            vm.name.clone(),
            vm.node.clone(),
            vm.vm_type.clone(),
            format!("● {}", vm.status),
            format!("{:.0}%", vm.cpu * 100.0),
            app.format_mem(vm.mem),
        ]).style(Style::new().fg(fg).bg(bg))
    }).collect();

    f.render_widget(
        Table::new(rows, [
            Constraint::Length(6),
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(8),
            Constraint::Percentage(12),
            Constraint::Percentage(8),
            Constraint::Percentage(15),
        ]),
        area,
    );
}

fn render_storage(app: &AppState, f: &mut Frame, area: Rect) {
    if let Some(ref s) = app.storage_list {
        let data = s.pointer("/data").unwrap_or(s);
        if let Some(arr) = data.as_array() {
            let rows: Vec<Row> = arr.iter().map(|st| {
                let id = st.get("id").or_else(|| st.get("storage")).and_then(|v| v.as_str()).unwrap_or("-");
                let t = st.get("type").and_then(|v| v.as_str()).unwrap_or("-");
                let c = st.get("content").and_then(|v| v.as_str()).unwrap_or("-");
                let enabled = st.get("enabled").and_then(|v| v.as_u64()).unwrap_or(1) != 0;
                let color = if enabled { Color::Green } else { Color::Red };
                Row::new(vec![id, t, c, if enabled { "active" } else { "disabled" }])
                    .style(Style::new().fg(color))
            }).collect();

            f.render_widget(Table::new(rows, [
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ]), area);
            return;
        }
    }
    f.render_widget(Paragraph::new("No storage data. Press R to refresh.").style(Style::new().fg(Color::Gray)), area);
}

fn render_logs(app: &AppState, f: &mut Frame, area: Rect) {
    if let Some(ref logs) = app.logs {
        let data = logs.pointer("/data").unwrap_or(logs);
        if let Some(arr) = data.as_array() {
            let lines: Vec<Text> = arr.iter().rev().take(50).map(|entry| {
                let msg = entry.as_str().unwrap_or("?").to_string();
                Text::from(Line::from(msg))
            }).collect();
            f.render_widget(List::new(lines), area);
            return;
        }
    }
    f.render_widget(Paragraph::new("No logs. Press R to refresh.").style(Style::new().fg(Color::Gray)), area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = r#"
    Keybindings:

      q / Esc     Quit
      1-5 / Tab   Switch tab
      ↑ / j       Move up
      ↓ / k       Move down
      Enter       Select / confirm
      r           Refresh
      s           Start VM
      x           Stop VM
      c           Disconnect (return to setup)

    Colors:
      ● green     Running
      ● red       Stopped
      ● yellow    Paused
    "#;
    f.render_widget(
        Paragraph::new(help_text)
            .style(Style::new().fg(Color::White))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_status_bar(app: &AppState, f: &mut Frame, area: Rect) {
    let block = Block::default().style(Style::new().bg(Color::Rgb(0, 43, 54)).fg(Color::White));
    f.render_widget(block, area);

    let msg = if app.loading {
        "  Loading...  ".to_string()
    } else if let Some(ref e) = app.error_msg {
        format!("  Error: {}  ", e)
    } else {
        format!(
            "  Tab: Left/Right  |  R: Refresh  |  S: Start  |  x: Stop  |  q: Quit  |  c: Disconnect  |  VMs: {}  ",
            app.vm_list.len()
        )
    };

    f.render_widget(Paragraph::new(Line::from(msg)), area);
}

fn render_panel<F: Fn() -> String>(f: &mut Frame, area: Rect, title: &str, content_fn: F) {
    let b = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Rgb(40, 44, 52)).fg(Color::White));
    f.render_widget(b, area);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));
    let content = content_fn();
    f.render_widget(
        Paragraph::new(content)
            .style(Style::new().fg(Color::White))
            .wrap(Wrap { trim: true }),
        inner,
    );
}