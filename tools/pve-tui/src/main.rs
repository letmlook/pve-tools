//! pve-tui - Interactive TUI for Proxmox VE

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap, Widget},
    Frame, Terminal,
};
use std::io;
use proxmox_api::{PveClient, ClientConfig};
use serde_json::Value;

#[derive(Copy, Clone)]
enum View {
    Dashboard, VMs, Storage, Logs,
}
impl View {
    fn next(&self) -> Self {
        match self {
            View::Dashboard => View::VMs,
            View::VMs => View::Storage,
            View::Storage => View::Logs,
            View::Logs => View::Dashboard,
        }
    }
    fn label(&self) -> &'static str {
        match self { View::Dashboard => "Dashboard", View::VMs => "VMs", View::Storage => "Storage", View::Logs => "Logs" }
    }
    fn index(&self) -> usize {
        match self { View::Dashboard => 0, View::VMs => 1, View::Storage => 2, View::Logs => 3 }
    }
}

#[derive(Clone)]
struct VmEntry {
    vmid: u32, name: String, node: String, vm_type: String, status: String,
}

struct App {
    view: View,
    loading: bool,
    version: Option<Value>,
    nodes: Option<Value>,
    resources: Option<Value>,
    storage_list: Option<Value>,
    cluster_status: Option<Value>,
    vm_list: Vec<VmEntry>,
    selected_vm: Option<usize>,
    logs: Option<Value>,
    pve_host: String,
}

impl App {
    fn new() -> Self {
        Self {
            view: View::Dashboard,
            loading: false,
            version: None, nodes: None, resources: None,
            storage_list: None, cluster_status: None,
            vm_list: Vec::new(), selected_vm: None, logs: None,
            pve_host: std::env::var("PVE_HOST").unwrap_or_else(|_| "192.168.1.100".to_string()),
        }
    }
    fn make_config(&self) -> ClientConfig {
        let mut c = ClientConfig::from_env();
        c.host = self.pve_host.clone();
        c.user = "root@pam".to_string();
        c
    }
    fn cycle(&mut self) { self.view = self.view.next(); }
    fn update_vm_list(&mut self) {
        self.vm_list.clear();
        if let Some(res) = &self.resources {
            if let Some(data) = res.pointer("/data").and_then(|d| d.as_array()) {
                for item in data {
                    let vmid = item.get("vmid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let name = item.get("name").or_else(|| item.get("id")).and_then(|v| v.as_str()).unwrap_or("-").to_string();
                    let node = item.get("node").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                    let vm_type = item.get("type").or_else(|| item.get("resource")).and_then(|v| v.as_str()).unwrap_or("qemu").to_string();
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    if vmid > 0 {
                        self.vm_list.push(VmEntry { vmid, name, node, vm_type, status });
                    }
                }
            }
        }
    }
    async fn load_all(&mut self, client: &PveClient) {
        self.loading = true;
        let (v, n, r, s, c) = tokio::join!(
            client.get("/version"), client.get("/nodes"),
            client.get("/cluster/resources"), client.get("/storage"),
            client.get("/cluster/status"),
        );
        self.version = v.ok();
        self.nodes = n.ok();
        self.resources = r.ok();
        self.storage_list = s.ok();
        self.cluster_status = c.ok();
        self.update_vm_list();
        self.loading = false;
    }
    async fn load_logs(&mut self, client: &PveClient) {
        self.loading = true;
        self.logs = client.get("/cluster/log?limit=100").await.ok();
        self.loading = false;
    }
    fn get_node(&self, vmid: u32) -> String {
        self.vm_list.iter().find(|v| v.vmid == vmid).map(|v| v.node.clone()).unwrap_or_else(|| "localhost".to_string())
    }

    fn render_frame(&self, f: &mut Frame) {
        let area = f.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(area);

        // Title bar
        let title = Paragraph::new(Line::from("  PVE Manager  ").bold().fg(Color::White))
            .style(Style::new().bg(Color::Rgb(22, 27, 34)));
        f.render_widget(title, chunks[0]);

        // Tab bar
        let tabs = ["Dashboard", "VMs", "Storage", "Logs"];
        let active = self.view.index();
        let tab_h = 3u16;
        for (i, tab) in tabs.iter().enumerate() {
            let x = chunks[1].x + (i as u16) * (chunks[1].width / 4);
            let ta = Rect::new(x, chunks[1].y, chunks[1].width / 4, tab_h);
            let style = if i == active { Style::new().bg(Color::Blue).bold().fg(Color::White) }
                else { Style::new().bg(Color::DarkGray).fg(Color::White) };
            f.render_widget(Paragraph::new(Line::from(format!(" {} ", tab))).style(style), ta);
        }

        // Content area
        let content = Rect::new(chunks[1].x, chunks[1].y + tab_h, chunks[1].width, chunks[1].height.saturating_sub(tab_h));
        match self.view {
            View::Dashboard => self.render_dashboard(f, content),
            View::VMs => self.render_vms(f, content),
            View::Storage => self.render_storage(f, content),
            View::Logs => self.render_logs(f, content),
        }

        // Status bar
        let status = format!("  Tab: Left/Right/Tab  |  R: Refresh  |  S: Start  |  X: Stop  |  Q: Quit  |  Host: {}  ", self.pve_host);
        f.render_widget(
            Paragraph::new(Line::from(status)).style(Style::new().bg(Color::Rgb(0, 43, 54)).fg(Color::White)),
            chunks[2],
        );
    }

    fn render_dashboard(&self, f: &mut Frame, area: Rect) {
        let left = Rect::new(area.x, area.y, area.width / 2, area.height);
        let right = Rect::new(area.x + area.width / 2, area.y, area.width / 2, area.height);

        // Left column: version + nodes
        let v_box = Rect::new(left.x, left.y, left.width, left.height / 2);
        let n_box = Rect::new(left.x, left.y + left.height / 2, left.width, left.height / 2);

        // Right column: cluster + storage
        let c_box = Rect::new(right.x, right.y, right.width, right.height / 2);
        let s_box = Rect::new(right.x, right.y + right.height / 2, right.width, right.height / 2);

        self.render_box(f, v_box, "  PVE Version  ", self.version.as_ref().map(|v| {
            v.get("release").or_else(|| v.get("version")).and_then(|s| s.as_str()).unwrap_or("?").to_string()
        }).unwrap_or_else(|| "Not connected".to_string()));

        self.render_node_list(f, n_box);

        self.render_json(f, c_box, self.cluster_status.as_ref(), "  Cluster  ");

        self.render_storage_list(f, s_box);
    }

    fn render_node_list(&self, f: &mut Frame, area: Rect) {
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let b = Block::default().title("  Nodes  ").borders(Borders::ALL).style(Style::new().bg(Color::Rgb(33, 37, 43)));
        f.render_widget(b, area);
        let text = if let Some(ref n) = self.nodes {
            let arr = n.pointer("/data").or_else(|| Some(n)).and_then(|d| d.as_array());
            if let Some(a) = arr {
                let mut lines = vec![format!("Total: {} nodes", a.len())];
                for node in a.iter().take(6) {
                    let name = node.get("node").or_else(|| node.get("id")).and_then(|s| s.as_str()).unwrap_or("-");
                    lines.push(format!("  {} ", name));
                }
                lines.join("\n")
            } else { "No data".to_string() }
        } else { "Not loaded".to_string() };
        f.render_widget(Paragraph::new(text).style(Style::new().fg(Color::White)), inner);
    }

    fn render_storage_list(&self, f: &mut Frame, area: Rect) {
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let b = Block::default().title("  Storage  ").borders(Borders::ALL).style(Style::new().bg(Color::Rgb(33, 37, 43)));
        f.render_widget(b, area);
        let text = if let Some(ref s) = self.storage_list {
            let arr = s.pointer("/data").or_else(|| Some(s)).and_then(|d| d.as_array());
            if let Some(a) = arr {
                let mut lines = vec![format!("Total: {} storages", a.len())];
                for st in a.iter().take(6) {
                    let id = st.get("id").or_else(|| st.get("storage")).and_then(|v| v.as_str()).unwrap_or("-");
                    lines.push(format!("  {} ", id));
                }
                lines.join("\n")
            } else { "No data".to_string() }
        } else { "Not loaded".to_string() };
        f.render_widget(Paragraph::new(text).style(Style::new().fg(Color::White)), inner);
    }

    fn render_json(&self, f: &mut Frame, area: Rect, data: Option<&Value>, title: &str) {
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let b = Block::default().title(title).borders(Borders::ALL).style(Style::new().bg(Color::Rgb(33, 37, 43)));
        f.render_widget(b, area);
        let text = if let Some(d) = data {
            serde_json::to_string_pretty(d).unwrap_or_else(|_| "Error".to_string())
        } else { "No data".to_string() };
        f.render_widget(
            Paragraph::new(text).style(Style::new().fg(Color::White)).wrap(Wrap { trim: true }),
            inner,
        );
    }

    fn render_box(&self, f: &mut Frame, area: Rect, title: &str, content: String) {
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let b = Block::default().title(title).borders(Borders::ALL).style(Style::new().bg(Color::Rgb(33, 37, 43)));
        f.render_widget(b, area);
        f.render_widget(Paragraph::new(content).style(Style::new().fg(Color::White)), inner);
    }

    fn render_vms(&self, f: &mut Frame, area: Rect) {
        let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let b = Block::default().title("  VM List  ").borders(Borders::ALL).style(Style::new().bg(Color::Rgb(33, 37, 43)));
        f.render_widget(b, area);

        if self.vm_list.is_empty() {
            f.render_widget(Paragraph::new("No VMs. Press R to refresh.").style(Style::new().fg(Color::Gray)), inner);
            return;
        }

        let header = Row::new(vec!["VMID", "Name", "Node", "Type", "Status"]).style(Style::new().bold().fg(Color::Yellow));
        let rows: Vec<Row> = self.vm_list.iter().enumerate().map(|(i, vm)| {
            let style = if Some(i) == self.selected_vm {
                Style::new().bg(Color::Blue)
            } else {
                let c = match vm.status.as_str() {
                    "running" => Color::Green, "stopped" => Color::Red, "paused" => Color::Yellow, _ => Color::White,
                };
                Style::new().fg(c)
            };
            Row::new(vec![vm.vmid.to_string(), vm.name.clone(), vm.node.clone(), vm.vm_type.clone(), vm.status.clone()]).style(style)
        }).collect();

        f.render_widget(
            Table::new(rows, [
                Constraint::Length(8), Constraint::Percentage(30),
                Constraint::Percentage(20), Constraint::Percentage(15), Constraint::Percentage(15),
            ]),
            inner,
        );
    }

    fn render_storage(&self, f: &mut Frame, area: Rect) {
        self.render_json(f, area, self.storage_list.as_ref(), "  Storage  ");
    }

    fn render_logs(&self, f: &mut Frame, area: Rect) {
        self.render_json(f, area, self.logs.as_ref(), "  Logs  ");
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let config = app.make_config();

    let runtime = tokio::runtime::Runtime::new()?;
    let client = runtime.block_on(PveClient::new(&config))?;

    runtime.block_on(app.load_all(&client));

    loop {
        terminal.draw(|f| app.render_frame(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left | KeyCode::Right | KeyCode::Char('\t') => app.cycle(),
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        match app.view {
                            View::Dashboard | View::VMs | View::Storage => runtime.block_on(app.load_all(&client)),
                            View::Logs => runtime.block_on(app.load_logs(&client)),
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(ref mut s) = app.selected_vm {
                            if *s < app.vm_list.len() - 1 { *s += 1; }
                        } else { app.selected_vm = Some(0); }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(ref mut s) = app.selected_vm {
                            if *s > 0 { *s -= 1; }
                        } else { app.selected_vm = Some(0); }
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if let Some(i) = app.selected_vm {
                            if i < app.vm_list.len() {
                                let vmid = app.vm_list[i].vmid;
                                let node = app.get_node(vmid);
                                let c = &client;
                                runtime.block_on(async move {
                                    let _ = c.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await;
                                });
                            }
                        }
                    }
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        if let Some(i) = app.selected_vm {
                            if i < app.vm_list.len() {
                                let vmid = app.vm_list[i].vmid;
                                let node = app.get_node(vmid);
                                let c = &client;
                                runtime.block_on(async move {
                                    let _ = c.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await;
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}