//! pve-agent - CLI tool for AI agents to manage Proxmox VE

use anyhow::Result;
use clap::{Parser, Subcommand};
use proxmox_api::{api, PveClient, ClientConfig};
use serde_json::Value;
use std::process::ExitCode;

#[derive(Parser, Clone)]
struct Args {
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    token: Option<String>,
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Version,
    Node { #[command(subcommand)] sub: NodeCommands },
    Vm { #[command(subcommand)] sub: VmCommands },
    Storage { #[command(subcommand)] sub: StorageCommands },
}

#[derive(Subcommand, Clone)]
enum NodeCommands {
    List,
    Status { node: String },
}

#[derive(Subcommand, Clone)]
enum VmCommands {
    List,
    Status { vmid: u32 },
    Start { vmid: u32, #[arg(long)] wait: bool },
    Stop { vmid: u32 },
    Delete { vmid: u32 },
}

#[derive(Subcommand, Clone)]
enum StorageCommands {
    List,
}

fn make_config(args: &Args) -> ClientConfig {
    let mut config = ClientConfig::from_env();
    if let Some(h) = &args.host {
        config.host = h.clone();
    }
    if let Some(t) = &args.token {
        config.token = Some(t.clone());
    }
    config
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let config = make_config(&args);

    let client = match PveClient::new(&config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Connection failed: {}", e);
            return ExitCode::from(10);
        }
    };

    if let Some(cmd) = &args.cmd {
        match run_command(&client, cmd).await {
            Ok(v) => {
                println!("{}", serde_json::to_string_pretty(&v).unwrap());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::from(1)
            }
        }
    } else {
        ExitCode::SUCCESS
    }
}

async fn run_command(client: &PveClient, cmd: &Commands) -> anyhow::Result<Value> {
    match cmd {
        Commands::Version => Ok(serde_json::to_value(api::version::get_version(client).await?)?),
        Commands::Node { sub } => match sub {
            NodeCommands::List => Ok(client.get("/nodes").await?),
            NodeCommands::Status { node } => Ok(client.get(&format!("/nodes/{}/status", node)).await?),
        },
        Commands::Vm { sub } => match sub {
            VmCommands::List => Ok(client.get("/cluster/resources").await?),
            VmCommands::Status { vmid } => {
                let resources: Value = client.get("/cluster/resources").await?;
                let node = resources.pointer("/data")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.iter().find(|r| r.get("vmid").and_then(|v| v.as_u64()) == Some(*vmid as u64)))
                    .and_then(|r| r.get("node").and_then(|n| n.as_str()))
                    .unwrap_or("localhost");
                Ok(client.get(&format!("/nodes/{}/qemu/{}/status/current", node, vmid)).await?)
            }
            VmCommands::Start { vmid, wait } => {
                let resources: Value = client.get("/cluster/resources").await?;
                let node = resources.pointer("/data")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.iter().find(|r| r.get("vmid").and_then(|v| v.as_u64()) == Some(*vmid as u64)))
                    .and_then(|r| r.get("node").and_then(|n| n.as_str()))
                    .unwrap_or("localhost");
                let result = client.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await?;
                if *wait {
                    let _ = client.wait_for_vm_state(node, *vmid, "qemu", "running", 60).await;
                }
                Ok(result)
            }
            VmCommands::Stop { vmid } => {
                let resources: Value = client.get("/cluster/resources").await?;
                let node = resources.pointer("/data")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.iter().find(|r| r.get("vmid").and_then(|v| v.as_u64()) == Some(*vmid as u64)))
                    .and_then(|r| r.get("node").and_then(|n| n.as_str()))
                    .unwrap_or("localhost");
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await?)
            }
            VmCommands::Delete { vmid } => {
                let resources: Value = client.get("/cluster/resources").await?;
                let node = resources.pointer("/data")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.iter().find(|r| r.get("vmid").and_then(|v| v.as_u64()) == Some(*vmid as u64)))
                    .and_then(|r| r.get("node").and_then(|n| n.as_str()))
                    .unwrap_or("localhost");
                Ok(client.delete(&format!("/nodes/{}/qemu/{}", node, vmid)).await?)
            }
        },
        Commands::Storage { sub } => match sub {
            StorageCommands::List => Ok(client.get("/storage").await?),
        },
    }
}
