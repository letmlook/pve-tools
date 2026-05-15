//! pve-agent - CLI tool for AI agents to manage Proxmox VE

use clap::{Parser, Subcommand};
use proxmox_api::{api, PveClient, ClientConfig};
use serde_json::Value;
use std::process::ExitCode;

#[derive(Parser, Debug, Clone)]
#[command(name = "pve-agent")]
struct Args {
    #[arg(long, env = "PVE_HOST")]
    host: Option<String>,
    #[arg(long, env = "PVE_TOKEN")]
    token: Option<String>,
    #[arg(long, env = "PVE_USER", default_value = "root@pam")]
    user: Option<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    wait: bool,
    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Version,
    Node {
        #[command(subcommand)]
        sub: NodeSub,
    },
    Vm {
        #[command(subcommand)]
        sub: VmSub,
    },
    Storage {
        #[command(subcommand)]
        sub: StorageSub,
    },
    Cluster {
        #[command(subcommand)]
        sub: ClusterSub,
    },
    Firewall {
        #[command(subcommand)]
        sub: FirewallSub,
    },
    Ha {
        #[command(subcommand)]
        sub: HaSub,
    },
    Access {
        #[command(subcommand)]
        sub: AccessSub,
    },
    Tasks {
        limit: Option<u32>,
    },
    Backup {
        #[command(subcommand)]
        sub: BackupSub,
    },
    Network {
        #[command(subcommand)]
        sub: NetworkSub,
    },
    Pool {
        #[command(subcommand)]
        sub: PoolSub,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum NodeSub {
    List,
    Status { node: String },
    Syslog { node: String },
    Disks { node: String },
}

#[derive(Subcommand, Debug, Clone)]
enum VmSub {
    List { node: Option<String> },
    Status { vmid: u32 },
    Config { vmid: u32 },
    Start { vmid: u32 },
    Stop { vmid: u32 },
    Delete { vmid: u32 },
    SnapshotList { vmid: u32 },
    SnapshotCreate { vmid: u32, name: String },
    SnapshotRollback { vmid: u32, name: String },
    SnapshotDelete { vmid: u32, name: String },
    Create { node: String, name: String, memory: u32, cores: u32, disk: String },
    Clone { vmid: u32, newid: u32, name: String },
}

#[derive(Subcommand, Debug, Clone)]
enum StorageSub {
    List,
    Status { storage: String },
    Content { storage: String, type_: Option<String> },
}

#[derive(Subcommand, Debug, Clone)]
enum ClusterSub {
    Status,
    Nodes,
    Resources,
    Tasks { limit: u32 },
    Nextid,
    Log { lines: u32 },
}

#[derive(Subcommand, Debug, Clone)]
enum FirewallSub {
    List,
    Rules { target: String },
}

#[derive(Subcommand, Debug, Clone)]
enum HaSub {
    Status,
    Resources,
    Groups,
}

#[derive(Subcommand, Debug, Clone)]
enum AccessSub {
    UserList,
    TokenList { userid: String },
    AclList,
}

#[derive(Subcommand, Debug, Clone)]
enum BackupSub {
    ScheduleList,
    Run { vmid: u32, storage: String },
}

#[derive(Subcommand, Debug, Clone)]
enum NetworkSub {
    List,
    SdnZones,
    SdnVnets,
    SdnSubnets { zone: String },
}

#[derive(Subcommand, Debug, Clone)]
enum PoolSub {
    List,
}

fn make_config(args: &Args) -> ClientConfig {
    let mut config = ClientConfig::from_env();
    if let Some(h) = &args.host { config.host = h.clone(); }
    if let Some(t) = &args.token { config.token = Some(t.clone()); }
    config
}

async fn resolve_vm_node(client: &PveClient, vmid: u32) -> Option<String> {
    let resources: Value = client.get("/cluster/resources").await.ok()?;
    let arr = resources.pointer("/data")?.as_array()?;
    for r in arr.iter() {
        if r.get("vmid")?.as_u64()? == vmid as u64 {
            return r.get("node")?.as_str().map(String::from);
        }
    }
    None
}

fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap());
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
    if args.dry_run {
        println!("{{\"dry_run\": true}}");
        return ExitCode::SUCCESS;
    }
    if let Some(ref cmd) = args.cmd {
        match run(&client, cmd, args.wait).await {
            Ok(v) => { print_json(&v); ExitCode::SUCCESS }
            Err(e) => { eprintln!("Error: {}", e); ExitCode::from(1) }
        }
    } else {
        println!("{{\"version\": \"0.1.0\"}}");
        ExitCode::SUCCESS
    }
}

async fn run(client: &PveClient, cmd: &Commands, wait: bool) -> anyhow::Result<Value> {
    match cmd {
        Commands::Version => Ok(serde_json::to_value(api::version::get_version(client).await?)?),
        Commands::Node { sub } => match sub {
            NodeSub::List => Ok(client.get("/nodes").await?),
            NodeSub::Status { node } => Ok(client.get(&format!("/nodes/{}/status", node)).await?),
            NodeSub::Syslog { node } => Ok(client.get(&format!("/nodes/{}/syslog", node)).await?),
            NodeSub::Disks { node } => Ok(client.get(&format!("/nodes/{}/disks", node)).await?),
        },
        Commands::Vm { sub } => match sub {
            VmSub::List { node } => {
                let url = match node.as_ref() {
                    Some(n) => format!("/cluster/resources?type=vm&node={}", n),
                    None => "/cluster/resources?type=vm".to_string(),
                };
                Ok(client.get(&url).await?)
            }
            VmSub::Status { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/status/current", node, vmid)).await?)
            }
            VmSub::Config { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await?)
            }
            VmSub::Start { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let result = client.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await?;
                if wait { wait_vm(client, *vmid, "running", 60).await?; }
                Ok(result)
            }
            VmSub::Stop { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await?)
            }
            VmSub::Delete { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.delete(&format!("/nodes/{}/qemu/{}", node, vmid)).await?)
            }
            VmSub::SnapshotList { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid)).await?)
            }
            VmSub::SnapshotCreate { vmid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![("snapname".to_string(), name.clone())];
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid), Some(&params)).await?)
            }
            VmSub::SnapshotRollback { vmid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/snapshot/{}/rollback", node, vmid, name), None).await?)
            }
            VmSub::SnapshotDelete { vmid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.delete(&format!("/nodes/{}/qemu/{}/snapshot/{}", node, vmid, name)).await?)
            }
            VmSub::Create { node, name, memory, cores, disk } => {
                let params = vec![
                    ("name".to_string(), name.clone()),
                    ("memory".to_string(), memory.to_string()),
                    ("cores".to_string(), cores.to_string()),
                    ("scsi0".to_string(), disk.clone()),
                ];
                Ok(client.post_form(&format!("/nodes/{}/qemu", node), Some(&params)).await?)
            }
            VmSub::Clone { vmid, newid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![
                    ("newid".to_string(), newid.to_string()),
                    ("name".to_string(), name.clone()),
                    ("full".to_string(), "1".to_string()),
                ];
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/clone", node, vmid), Some(&params)).await?)
            }
        },
        Commands::Storage { sub } => match sub {
            StorageSub::List => Ok(client.get("/storage").await?),
            StorageSub::Status { storage } => Ok(client.get(&format!("/storage/{}/status", storage)).await?),
            StorageSub::Content { storage, type_ } => {
                let url = match type_ {
                    Some(t) => format!("/storage/{}/content?type={}", storage, t),
                    None => format!("/storage/{}/content", storage),
                };
                Ok(client.get(&url).await?)
            }
        },
        Commands::Cluster { sub } => match sub {
            ClusterSub::Status => Ok(client.get("/cluster/status").await?),
            ClusterSub::Nodes => Ok(client.get("/cluster/nodes").await?),
            ClusterSub::Resources => Ok(client.get("/cluster/resources").await?),
            ClusterSub::Tasks { limit } => Ok(client.get(&format!("/cluster/tasks?limit={}", limit)).await?),
            ClusterSub::Nextid => Ok(client.get("/cluster/nextid").await?),
            ClusterSub::Log { lines } => Ok(client.get(&format!("/cluster/log?limit={}", lines)).await?),
        },
        Commands::Firewall { sub } => match sub {
            FirewallSub::List => Ok(client.get("/cluster/firewall").await?),
            FirewallSub::Rules { target } => Ok(client.get(&format!("/nodes/{}/firewall/rules", target)).await?),
        },
        Commands::Ha { sub } => match sub {
            HaSub::Status => Ok(client.get("/cluster/ha/status").await?),
            HaSub::Resources => Ok(client.get("/cluster/ha/resources").await?),
            HaSub::Groups => Ok(client.get("/cluster/ha/groups").await?),
        },
        Commands::Access { sub } => match sub {
            AccessSub::UserList => Ok(client.get("/access/users").await?),
            AccessSub::TokenList { userid } => Ok(client.get(&format!("/access/users/{}/tokens", userid)).await?),
            AccessSub::AclList => Ok(client.get("/access/acl").await?),
        },
        Commands::Tasks { limit } => {
            Ok(client.get(&format!("/cluster/tasks?limit={}", limit.unwrap_or(50))).await?)
        }
        Commands::Backup { sub } => match sub {
            BackupSub::ScheduleList => Ok(client.get("/cluster/backup").await?),
            BackupSub::Run { vmid, storage } => {
                let params = vec![
                    ("vmid".to_string(), vmid.to_string()),
                    ("storage".to_string(), storage.clone()),
                ];
                Ok(client.post_form("/nodes/localhost/status", Some(&params)).await?)
            }
        },
        Commands::Network { sub } => match sub {
            NetworkSub::List => Ok(client.get("/nodes/localhost/network").await?),
            NetworkSub::SdnZones => Ok(client.get("/cluster/sdn/zones").await?),
            NetworkSub::SdnVnets => Ok(client.get("/cluster/sdn/vnets").await?),
            NetworkSub::SdnSubnets { zone } => Ok(client.get(&format!("/cluster/sdn/zones/{}/subnets", zone)).await?),
        },
        Commands::Pool { sub } => match sub {
            PoolSub::List => Ok(client.get("/pools").await?),
        },
    }
}

async fn wait_vm(client: &PveClient, vmid: u32, state: &str, timeout: u64) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let node = resolve_vm_node(client, vmid).await.unwrap_or_else(|| "localhost".to_string());
    while start.elapsed().as_secs() < timeout {
        let status: Value = client.get(&format!("/nodes/{}/qemu/{}/status/current", node, vmid)).await?;
        if let Some(s) = status.pointer("/data/status").and_then(|s| s.as_str()) {
            if s == state { return Ok(()); }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    anyhow::bail!("Timeout waiting for VM {} state {}", vmid, state)
}
