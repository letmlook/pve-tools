//! pve-tui - Interactive TUI for Proxmox VE

mod app;
mod ui;

use anyhow::Result;
use app::AppState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = AppState::new();
    let config = app.make_config();

    let runtime = tokio::runtime::Runtime::new()?;
    let client = runtime.block_on(proxmox_api::PveClient::new(&config))?;

    runtime.block_on(app.load_all(&client));

    loop {
        terminal.draw(|f| ui::render(&app, f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left | KeyCode::Right | KeyCode::Char('\t') => app.cycle_view(),
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        match app.view {
                            app::View::Dashboard | app::View::VMs | app::View::Storage => {
                                runtime.block_on(app.load_all(&client));
                            }
                            app::View::Logs => runtime.block_on(app.load_logs(&client)),
                            app::View::Help => {}
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
                        } else { app.selected_vm = Some(app.vm_list.len().saturating_sub(1)); }
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if let Some(i) = app.selected_vm {
                            if i < app.vm_list.len() {
                                let vmid = app.vm_list[i].vmid;
                                let node = app.get_vm_node(vmid);
                                let c = &client;
                                runtime.block_on(async move {
                                    let _ = c.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await;
                                });
                                app.view = app::View::VMs;
                            }
                        }
                    }
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        if let Some(i) = app.selected_vm {
                            if i < app.vm_list.len() {
                                let vmid = app.vm_list[i].vmid;
                                let node = app.get_vm_node(vmid);
                                let c = &client;
                                runtime.block_on(async move {
                                    let _ = c.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await;
                                });
                                app.view = app::View::VMs;
                            }
                        }
                    }
                    KeyCode::Char('1') => app.view = app::View::Dashboard,
                    KeyCode::Char('2') => app.view = app::View::VMs,
                    KeyCode::Char('3') => app.view = app::View::Storage,
                    KeyCode::Char('4') => app.view = app::View::Logs,
                    KeyCode::Char('5') => app.view = app::View::Help,
                    KeyCode::Enter => {
                        // Could show detail view here
                        if app.view == app::View::VMs && app.selected_vm.is_some() {}
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