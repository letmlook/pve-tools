//! pve-tui - Interactive TUI for Proxmox VE

mod app;
mod ui;

use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use app::{AppState, AppMode, SetupField, AuthMethod};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let runtime = tokio::runtime::Runtime::new()?;
    let client = Arc::new(Mutex::new(None::<proxmox_api::PveClient>));

    let mut app = AppState::new();

    loop {
        terminal.draw(|f| ui::render(&app, f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let disconnect = match app.mode {
                AppMode::Setup => {
                    handle_setup_key(&mut app, key.code);
                    false
                }
                AppMode::Running => {
                    let d = handle_running_key(&mut app, key.code);
                    d
                }
            };

            if disconnect {
                app.mode = AppMode::Setup;
                app.pve_host.clear();
                app.version = None;
                app.nodes = None;
                app.resources = None;
                app.storage_list = None;
                app.cluster_status = None;
                app.vm_list.clear();
                app.selected_vm = None;
                app.error_msg = None;
                let mut locked = runtime.block_on(client.lock());
                *locked = None;
                continue;
            }

            // Handle pending connect after key press
            if app.connecting {
                app.connecting = false;
                let config = app.to_client_config();
                let host = config.host.clone();
                let client_clone = Arc::clone(&client);

                match runtime.block_on(proxmox_api::PveClient::new(&config)) {
                    Ok(c) => {
                        {
                            let mut locked = runtime.block_on(client_clone.lock());
                            *locked = Some(c);
                        }
                        app.pve_host = host.clone();
                        app.mode = app::AppMode::Running;
                        app.error_msg = None;
                        // Load initial data
                        let locked = runtime.block_on(client_clone.lock());
                        if let Some(ref c) = *locked {
                            runtime.block_on(app.load_all(c));
                        }
                    }
                    Err(e) => {
                        app.error_msg = Some(format!("Connection failed: {}", e));
                        app.setup_field = SetupField::Host;
                    }
                }
            }

            // Handle refresh in running mode
            if app.mode == AppMode::Running && matches!(key.code, KeyCode::Char('r') | KeyCode::Char('R')) {
                let locked = runtime.block_on(client.lock());
                if let Some(ref c) = *locked {
                    runtime.block_on(app.load_all(c));
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn handle_setup_key(app: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('q') => {
            std::process::exit(0);
        }
        KeyCode::Tab | KeyCode::Down => {
            app.setup_field = app.setup_field.next();
            app.setup_cursor = 0;
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.setup_field = app.setup_field.prev();
            app.setup_cursor = 0;
        }
        KeyCode::Enter => {
            if app.setup_field == SetupField::Connect {
                app.connecting = true;
                app.error_msg = None;
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') if app.setup_field == SetupField::VerifySsl => {
            app.setup_config.verify_ssl = true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') if app.setup_field == SetupField::VerifySsl => {
            app.setup_config.verify_ssl = false;
        }
        KeyCode::Char('t') | KeyCode::Char('T') if app.setup_field == SetupField::AuthMethod => {
            app.setup_config.auth_method = AuthMethod::Token;
            app.setup_cursor = 0;
        }
        KeyCode::Char('p') | KeyCode::Char('P') if app.setup_field == SetupField::AuthMethod => {
            app.setup_config.auth_method = AuthMethod::Password;
            app.setup_cursor = 0;
        }
        KeyCode::Char(c) => {
            if !matches!(app.setup_field, SetupField::Connect | SetupField::VerifySsl | SetupField::AuthMethod) {
                app.setup_type(c);
            }
        }
        KeyCode::Backspace => {
            app.setup_backspace();
        }
        KeyCode::Home => {
            app.setup_cursor = 0;
        }
        KeyCode::End => {
            app.setup_cursor = app.get_value(app.setup_field).len();
        }
        KeyCode::Left | KeyCode::Right => {
            app.setup_cursor = 0;
        }
        _ => {}
    }
}

fn handle_running_key(app: &mut AppState, key: KeyCode) -> bool {
    // Returns true if should disconnect
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            std::process::exit(0);
        }
        KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
            return true;
        }
        KeyCode::Char('\t') | KeyCode::Right => {
            app.cycle_view();
        }
        KeyCode::Left => {
            app.cycle_view();
        }
        KeyCode::Char('1') => app.view = app::View::Dashboard,
        KeyCode::Char('2') => app.view = app::View::VMs,
        KeyCode::Char('3') => app.view = app::View::Storage,
        KeyCode::Char('4') => app.view = app::View::Logs,
        KeyCode::Char('5') => app.view = app::View::Help,
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref mut s) = app.selected_vm {
                if *s < app.vm_list.len() - 1 {
                    *s += 1;
                }
            } else if !app.vm_list.is_empty() {
                app.selected_vm = Some(0);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref mut s) = app.selected_vm {
                if *s > 0 {
                    *s -= 1;
                }
            } else if !app.vm_list.is_empty() {
                app.selected_vm = Some(app.vm_list.len().saturating_sub(1));
            }
        }
        KeyCode::Enter => {
            // Detail view placeholder
        }
        _ => {}
    }
    false
}