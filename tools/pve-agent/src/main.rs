//! pve-agent - CLI tool for AI agents to manage Proxmox VE

mod cli;

use clap::Parser;
use proxmox_api::{PveClient, ClientConfig};
use serde_json::Value;
use std::process::ExitCode;

use cli::{CliArgs, Commands, NodeSub, VmSub, StorageSub, ClusterSub,
         FirewallSub, HaSub, AccessSub, BackupSub, NetworkSub, PoolSub,
         SnapshotSub, DiskSub, AgentSub, TagSub, RrdSub, OutputFormat,
         RrdTimeframe, NetworkInterfaceArgs, ConsoleType};

// === Config building ===

fn make_config(args: &CliArgs) -> ClientConfig {
    let mut config = ClientConfig::from_env();
    if let Some(ref h) = args.host { config.host = h.clone(); }
    if let Some(ref p) = args.port { config.port = *p; }

    // Build token from token_id + token_secret if provided
    if let (Some(ref tid), Some(ref ts)) = (&args.token_id, &args.token_secret) {
        config.token = Some(format!("{}={}={}", args.user.as_deref().unwrap_or("root@pam"), tid, ts));
    }

    if let Some(ref pw) = args.password { config.password = Some(pw.clone()); }
    if args.verify_ssl { config.verify_ssl = true; }

    config
}

// === VM helper ===

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

fn resolve_vm_type(client: &PveClient, vmid: u32) -> String {
    // Quick type lookup - try to determine if qemu or lxc
    "qemu".to_string()
}

fn print_json(v: &Value, format: &OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(v).unwrap()),
        OutputFormat::Table => print_table(v),
        OutputFormat::Plain => println!("{}", v),
    }
}

fn print_table(v: &Value) {
    if let Some(arr) = v.pointer("/data").and_then(|a| a.as_array()) {
        if arr.is_empty() { println!("(empty)"); return; }
        // Simple key-value dump
        for item in arr {
            if let Some(obj) = item.as_object() {
                for (k, val) in obj {
                    println!("{}: {}", k, val);
                }
                println!("---");
            } else {
                println!("{}", item);
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
    }
}

fn param_vec<'a>(pairs: Vec<(&'a str, String)>) -> Vec<(String, String)> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

// === Main ===

#[tokio::main]
async fn main() -> ExitCode {
    let args = CliArgs::parse();
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
        match run(&client, cmd, &args).await {
            Ok(v) => {
                print_json(&v, &args.output);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::from(1)
            }
        }
    } else {
        println!("{{\"version\": \"0.1.0\"}}");
        ExitCode::SUCCESS
    }
}

async fn run(client: &PveClient, cmd: &Commands, args: &CliArgs) -> anyhow::Result<Value> {
    match cmd {
        Commands::Version => {
            Ok(client.get("/version").await?)
        }

        Commands::Node { sub } => match sub {
            NodeSub::List => Ok(client.get("/nodes").await?),
            NodeSub::Status { node } => Ok(client.get(&format!("/nodes/{}/status", node)).await?),
            NodeSub::Syslog { node, lines } => {
                Ok(client.get(&format!("/nodes/{}/syslog?limit={}", node, lines)).await?)
            }
            NodeSub::Disks { node } => Ok(client.get(&format!("/nodes/{}/disks", node)).await?),
            NodeSub::Services { node } => Ok(client.get(&format!("/nodes/{}/services", node)).await?),
            NodeSub::Capabilities { node } => Ok(client.get(&format!("/nodes/{}/capabilities", node)).await?),
            NodeSub::Tasks { node, limit } => {
                Ok(client.get(&format!("/nodes/{}/tasks?limit={}", node, limit)).await?)
            }
            NodeSub::Rrd { node, timeframe } => {
                Ok(client.get(&format!("/nodes/{}/rrd?timeframe={}", node, timeframe.as_str())).await?)
            }
            NodeSub::Network { node } => Ok(client.get(&format!("/nodes/{}/network", node)).await?),
            NodeSub::NetworkCreate { node, iface } => {
                let params = build_iface_params(iface);
                Ok(client.post_form(&format!("/nodes/{}/network", node), Some(&params)).await?)
            }
            NodeSub::NetworkUpdate { node, iface_name, iface } => {
                let params = build_iface_params(iface);
                Ok(client.put(&format!("/nodes/{}/network/{}", node, iface_name), Some(&params)).await?)
            }
            NodeSub::NetworkDelete { node, iface } => {
                Ok(client.delete(&format!("/nodes/{}/network/{}", node, iface)).await?)
            }
        },

        Commands::Vm { sub } => run_vm_cmd(client, sub, args).await,

        Commands::Storage { sub } => match sub {
            StorageSub::List { node } => {
                let url = node.as_ref().map(|n| format!("/nodes/{}/storage", n))
                    .unwrap_or_else(|| "/storage".to_string());
                Ok(client.get(&url).await?)
            }
            StorageSub::Status { storage } => {
                Ok(client.get(&format!("/storage/{}/status", storage)).await?)
            }
            StorageSub::Content { storage, type_ } => {
                let url = type_.as_ref().map(|t| format!("/storage/{}/content?type={}", storage, t))
                    .unwrap_or_else(|| format!("/storage/{}/content", storage));
                Ok(client.get(&url).await?)
            }
            StorageSub::Create { storage, type_, path, server, export, content, nodes } => {
                let mut params = vec![
                    ("storage".to_string(), storage.clone()),
                    ("type".to_string(), type_.clone()),
                ];
                if let Some(ref p) = path { params.push(("path".to_string(), p.clone())); }
                if let Some(ref s) = server { params.push(("server".to_string(), s.clone())); }
                if let Some(ref e) = export { params.push(("export".to_string(), e.clone())); }
                if let Some(ref c) = content { params.push(("content".to_string(), c.clone())); }
                if let Some(ref n) = nodes { params.push(("nodes".to_string(), n.clone())); }
                Ok(client.post_form("/storage", Some(&params)).await?)
            }
            StorageSub::Update { storage, enabled, comment } => {
                let mut params = Vec::new();
                if let Some(e) = enabled { params.push(("enabled".to_string(), if *e { "1" } else { "0" }.to_string())); }
                if let Some(ref c) = comment { params.push(("comment".to_string(), c.clone())); }
                Ok(client.put(&format!("/storage/{}", storage), if params.is_empty() { None } else { Some(&params) }).await?)
            }
            StorageSub::Delete { storage } => {
                Ok(client.delete(&format!("/storage/{}", storage)).await?)
            }
        },

        Commands::Cluster { sub } => match sub {
            ClusterSub::Status => Ok(client.get("/cluster/status").await?),
            ClusterSub::Nodes => Ok(client.get("/cluster/nodes").await?),
            ClusterSub::Resources { type_ } => {
                let url = type_.as_ref().map(|t| format!("/cluster/resources?type={}", t))
                    .unwrap_or_else(|| "/cluster/resources".to_string());
                Ok(client.get(&url).await?)
            }
            ClusterSub::Tasks { limit } => {
                Ok(client.get(&format!("/cluster/tasks?limit={}", limit)).await?)
            }
            ClusterSub::Nextid => Ok(client.get("/cluster/nextid").await?),
            ClusterSub::Log { lines } => {
                Ok(client.get(&format!("/cluster/log?limit={}", lines)).await?)
            }
        },

        Commands::Firewall { sub } => run_firewall_cmd(client, sub).await,

        Commands::Ha { sub } => run_ha_cmd(client, sub).await,

        Commands::Access { sub } => run_access_cmd(client, sub).await,

        Commands::Tasks { limit, node, r#type } => {
            let mut url = String::from("/cluster/tasks?");
            if let Some(l) = limit { url.push_str(&format!("limit={}&", l)); }
            if let Some(ref n) = node { url.push_str(&format!("node={}&", n)); }
            if let Some(ref t) = r#type { url.push_str(&format!("type={}&", t)); }
            Ok(client.get(&url.trim_end_matches('&')).await?)
        }

        Commands::Backup { sub } => run_backup_cmd(client, sub).await,

        Commands::Network { sub } => run_network_cmd(client, sub).await,

        Commands::Pool { sub } => match sub {
            PoolSub::List => Ok(client.get("/pools").await?),
            PoolSub::Create { poolid, comment } => {
                let mut params = vec![("poolid".to_string(), poolid.clone())];
                if let Some(ref c) = comment { params.push(("comment".to_string(), c.clone())); }
                Ok(client.post_form("/pools", Some(&params)).await?)
            }
            PoolSub::Delete { poolid } => {
                Ok(client.delete(&format!("/pools/{}", poolid)).await?)
            }
        },

        Commands::Rrd { sub } => match sub {
            RrdSub::Node { node, timeframe } => {
                Ok(client.get(&format!("/nodes/{}/rrd?timeframe={}", node, timeframe.as_str())).await?)
            }
            RrdSub::Vm { vmid, timeframe } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/rrd?timeframe={}", node, vmid, timeframe.as_str())).await?)
            }
            RrdSub::Storage { storage, timeframe } => {
                Ok(client.get(&format!("/storage/{}/rrd?timeframe={}", storage, timeframe.as_str())).await?)
            }
        },
    }
}

// === VM subcommand handler ===

async fn run_vm_cmd(client: &PveClient, sub: &VmSub, args: &CliArgs) -> anyhow::Result<Value> {
    match sub {
        VmSub::List { node, type_, status } => {
            let mut url = String::from("/cluster/resources?");
            if let Some(ref n) = node { url.push_str(&format!("node={}&", n)); }
            if let Some(ref t) = type_ { url.push_str(&format!("type={}&", t)); }
            if let Some(ref s) = status { url.push_str(&format!("status={}&", s)); }
            Ok(client.get(url.trim_end_matches('&')).await?)
        }
        VmSub::Status { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.get(&format!("/nodes/{}/qemu/{}/status/current", node, vmid)).await?)
        }
        VmSub::Config { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await?)
        }
        VmSub::Pending { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.get(&format!("/nodes/{}/qemu/{}/pending", node, vmid)).await?)
        }

        VmSub::Start { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            let result = client.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await?;
            if args.wait {
                wait_vm(client, *vmid, "running", 60).await?;
            }
            Ok(result)
        }
        VmSub::Stop { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await?)
        }
        VmSub::Shutdown { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/shutdown", node, vmid), None).await?)
        }
        VmSub::Reboot { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/reboot", node, vmid), None).await?)
        }
        VmSub::Suspend { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/suspend", node, vmid), None).await?)
        }
        VmSub::Resume { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/status/resume", node, vmid), None).await?)
        }

        VmSub::Create { node, name, memory, cores, disk, type_, ostype, net, iso } => {
            let mut params = vec![
                ("name".to_string(), name.clone()),
                ("memory".to_string(), memory.to_string()),
                ("cores".to_string(), cores.to_string()),
                ("scsi0".to_string(), disk.clone()),
            ];
            if let Some(ref o) = ostype { params.push(("ostype".to_string(), o.clone())); }
            if let Some(ref n) = net { params.push(("net0".to_string(), n.clone())); }
            if let Some(ref i) = iso { params.push(("ide2".to_string(), format!("{},media=cdrom", i))); }
            Ok(client.post_form(&format!("/nodes/{}/qemu", node), Some(&params)).await?)
        }
        VmSub::Delete { vmid, force } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            if *force {
                Ok(client.delete(&format!("/nodes/{}/qemu/{}", node, vmid)).await?)
            } else {
                Ok(client.delete(&format!("/nodes/{}/qemu/{}", node, vmid)).await?)
            }
        }
        VmSub::Clone { vmid, newid, name, target_node, full } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            let mut params = vec![
                ("newid".to_string(), newid.to_string()),
                ("name".to_string(), name.clone()),
                ("full".to_string(), if *full { "1" } else { "0" }.to_string()),
            ];
            if let Some(ref tn) = target_node { params.push(("target".to_string(), tn.clone())); }
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/clone", node, vmid), Some(&params)).await?)
        }
        VmSub::Migrate { vmid, target_node, online, timeout } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            let mut params = vec![
                ("target".to_string(), target_node.clone()),
                ("online".to_string(), if *online { "1" } else { "0" }.to_string()),
            ];
            if let Some(t) = timeout { params.push(("timeout".to_string(), t.to_string())); }
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/migrate", node, vmid), Some(&params)).await?)
        }

        VmSub::Snapshot { sub } => match sub {
            SnapshotSub::List { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid)).await?)
            }
            SnapshotSub::Create { vmid, name, description } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let mut params = vec![("snapname".to_string(), name.clone())];
                if let Some(ref d) = description { params.push(("description".to_string(), d.clone())); }
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid), Some(&params)).await?)
            }
            SnapshotSub::Rollback { vmid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/snapshot/{}/rollback", node, vmid, name), None).await?)
            }
            SnapshotSub::Delete { vmid, name } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.delete(&format!("/nodes/{}/qemu/{}/snapshot/{}", node, vmid, name)).await?)
            }
            SnapshotSub::Update { vmid, name, description } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![("description".to_string(), description.clone())];
                Ok(client.put(&format!("/nodes/{}/qemu/{}/snapshot/{}", node, vmid, name), Some(&params)).await?)
            }
        },

        VmSub::Update { vmid, key, value } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            let params = vec![(key.clone(), value.clone())];
            Ok(client.put(&format!("/nodes/{}/qemu/{}/config", node, vmid), Some(&params)).await?)
        }

        VmSub::Disk { sub } => match sub {
            DiskSub::List { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/pending", node, vmid)).await?)
            }
            DiskSub::Resize { vmid, disk, size } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![
                    ("device".to_string(), disk.clone()),
                    ("size".to_string(), size.clone()),
                ];
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/resize", node, vmid), Some(&params)).await?)
            }
            DiskSub::Move { vmid, disk, target_storage, target_node, delete_source } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let mut params = vec![
                    ("device".to_string(), disk.clone()),
                    ("storage".to_string(), target_storage.clone()),
                    ("delete".to_string(), if *delete_source { "1" } else { "0" }.to_string()),
                ];
                if let Some(ref tn) = target_node { params.push(("target".to_string(), tn.clone())); }
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/move_disk", node, vmid), Some(&params)).await?)
            }
            DiskSub::Detach { vmid, disk } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![("device".to_string(), disk.clone())];
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/unlink", node, vmid), Some(&params)).await?)
            }
        },

        VmSub::Agent { sub } => match sub {
            AgentSub::Info { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/agent/info", node, vmid)).await?)
            }
            AgentSub::Fsinfo { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/agent/fsinfo", node, vmid)).await?)
            }
            AgentSub::Network { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/agent/networkinfo", node, vmid)).await?)
            }
            AgentSub::Hwinfo { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/agent/hwinfo", node, vmid)).await?)
            }
            AgentSub::Exec { vmid, command, args } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let mut params = vec![("command".to_string(), command.clone())];
                if let Some(ref a) = args {
                    for (i, arg) in a.iter().enumerate() {
                        params.push((format!("arg{}", i + 1), arg.clone()));
                    }
                }
                Ok(client.post_form(&format!("/nodes/{}/qemu/{}/agent/exec", node, vmid), Some(&params)).await?)
            }
            AgentSub::ExecStatus { vmid, exec_id } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/agent/exec-status?exec_id={}", node, vmid, exec_id)).await?)
            }
        },

        VmSub::Console { type_, vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            match type_ {
                ConsoleType::Vnc => Ok(client.post_form(&format!("/nodes/{}/qemu/{}/vncproxy", node, vmid), None).await?),
                ConsoleType::Spice => Ok(client.post_form(&format!("/nodes/{}/qemu/{}/vncproxy", node, vmid), None).await?),
            }
        }

        VmSub::Tag { vmid, sub } => match sub {
            TagSub::List { vmid } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                Ok(client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await?)
            }
            TagSub::Add { vmid, tags } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![("tags".to_string(), tags.join(","))];
                Ok(client.put(&format!("/nodes/{}/qemu/{}/config", node, vmid), Some(&params)).await?)
            }
            TagSub::Remove { vmid, tags } => {
                let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
                let params = vec![("tags".to_string(), tags.join(","))];
                Ok(client.put(&format!("/nodes/{}/qemu/{}/config", node, vmid), Some(&params)).await?)
            }
        },

        VmSub::Description { vmid, get, set } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            if *get {
                Ok(client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await?)
            } else if let Some(ref s) = set {
                let params = vec![("description".to_string(), s.clone())];
                Ok(client.put(&format!("/nodes/{}/qemu/{}/config", node, vmid), Some(&params)).await?)
            } else {
                Ok(client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await?)
            }
        }

        VmSub::Template { vmid } => {
            let node = resolve_vm_node(client, *vmid).await.unwrap_or_else(|| "localhost".to_string());
            Ok(client.post_form(&format!("/nodes/{}/qemu/{}/template", node, vmid), None).await?)
        }
    }
}

// === Firewall subcommand handler ===

async fn run_firewall_cmd(client: &PveClient, sub: &FirewallSub) -> anyhow::Result<Value> {
    match sub {
        FirewallSub::List { level, node, vmid } => {
            match (level.as_deref(), node.as_ref(), *vmid) {
                (Some("cluster"), None, None) => Ok(client.get("/cluster/firewall").await?),
                (Some("node"), Some(n), None) => Ok(client.get(&format!("/nodes/{}/firewall", n)).await?),
                (Some("vm"), None, Some(v)) => Ok(client.get(&format!("/qemu/{}/firewall", v)).await?),
                _ => Ok(client.get("/cluster/firewall").await?),
            }
        }
        FirewallSub::Rules { target, level, vmid } => {
            match (level.as_deref(), *vmid) {
                (Some("cluster"), None) => Ok(client.get("/cluster/firewall/rules").await?),
                (Some("node"), None) => Ok(client.get(&format!("/nodes/{}/firewall/rules", target)).await?),
                (Some("vm"), Some(vmid)) => Ok(client.get(&format!("/qemu/{}/firewall/rules", vmid)).await?),
                _ => Ok(client.get("/cluster/firewall/rules").await?),
            }
        }
        FirewallSub::Groups { level } => {
            match level.as_deref() {
                Some("cluster") => Ok(client.get("/cluster/firewall/groups").await?),
                _ => Ok(client.get("/cluster/firewall/groups").await?),
            }
        }
        FirewallSub::Options { target, level, vmid } => {
            match (level.as_deref(), *vmid) {
                (Some("cluster"), None) => Ok(client.get("/cluster/firewall/options").await?),
                (Some("node"), None) => Ok(client.get(&format!("/nodes/{}/firewall/options", target)).await?),
                (Some("vm"), Some(vmid)) => Ok(client.get(&format!("/qemu/{}/firewall/options", vmid)).await?),
                _ => Ok(client.get("/cluster/firewall/options").await?),
            }
        }
        FirewallSub::RuleAdd { target, level, vmid, direction, action, protocol, source, dest, dport, comment, enable } => {
            let mut params = vec![
                ("direction".to_string(), direction.clone()),
                ("action".to_string(), action.clone()),
                ("enable".to_string(), enable.to_string()),
            ];
            if let Some(ref p) = protocol { params.push(("proto".to_string(), p.clone())); }
            if let Some(ref s) = source { params.push(("source".to_string(), s.clone())); }
            if let Some(ref d) = dest { params.push(("dest".to_string(), d.clone())); }
            if let Some(ref dp) = dport { params.push(("dport".to_string(), dp.clone())); }
            if let Some(ref c) = comment { params.push(("comment".to_string(), c.clone())); }

            match (level.as_deref(), *vmid) {
                (Some("cluster"), None) => Ok(client.post_form("/cluster/firewall/rules", Some(&params)).await?),
                (Some("node"), None) => Ok(client.post_form(&format!("/nodes/{}/firewall/rules", target), Some(&params)).await?),
                (Some("vm"), Some(vmid)) => Ok(client.post_form(&format!("/qemu/{}/firewall/rules", vmid), Some(&params)).await?),
                _ => Ok(client.post_form("/cluster/firewall/rules", Some(&params)).await?),
            }
        }
        FirewallSub::RuleDelete { target, pos, level, vmid } => {
            match (level.as_deref(), *vmid) {
                (Some("cluster"), None) => Ok(client.delete(&format!("/cluster/firewall/rules/{}", pos)).await?),
                (Some("node"), None) => Ok(client.delete(&format!("/nodes/{}/firewall/rules/{}", target, pos)).await?),
                (Some("vm"), Some(vmid)) => Ok(client.delete(&format!("/qemu/{}/firewall/rules/{}", vmid, pos)).await?),
                _ => Ok(client.delete(&format!("/cluster/firewall/rules/{}", pos)).await?),
            }
        }
    }
}

// === HA subcommand handler ===

async fn run_ha_cmd(client: &PveClient, sub: &HaSub) -> anyhow::Result<Value> {
    match sub {
        HaSub::Status => Ok(client.get("/cluster/ha/status").await?),
        HaSub::Resources => Ok(client.get("/cluster/ha/resources").await?),
        HaSub::Groups => Ok(client.get("/cluster/ha/groups").await?),
        HaSub::ResourceCreate { resource, group, max, migration_type } => {
            let mut params = vec![
                ("sid".to_string(), resource.clone()),
                ("group".to_string(), group.clone()),
                ("max".to_string(), max.to_string()),
            ];
            if let Some(ref mt) = migration_type { params.push(("migration".to_string(), mt.clone())); }
            Ok(client.post_form("/cluster/ha/resources", Some(&params)).await?)
        }
        HaSub::ResourceDelete { resource } => {
            Ok(client.delete(&format!("/cluster/ha/resources/{}", resource)).await?)
        }
        HaSub::GroupCreate { group, nodes, type_ } => {
            let params = vec![
                ("group".to_string(), group.clone()),
                ("nodes".to_string(), nodes.clone()),
                ("type".to_string(), type_.clone()),
            ];
            Ok(client.post_form("/cluster/ha/groups", Some(&params)).await?)
        }
        HaSub::GroupDelete { group } => {
            Ok(client.delete(&format!("/cluster/ha/groups/{}", group)).await?)
        }
    }
}

// === Access subcommand handler ===

async fn run_access_cmd(client: &PveClient, sub: &AccessSub) -> anyhow::Result<Value> {
    match sub {
        AccessSub::UserList => Ok(client.get("/access/users").await?),
        AccessSub::UserCreate { userid, email, enable } => {
            let mut params = vec![
                ("userid".to_string(), userid.clone()),
                ("enable".to_string(), enable.to_string()),
            ];
            if let Some(ref e) = email { params.push(("email".to_string(), e.clone())); }
            Ok(client.post_form("/access/users", Some(&params)).await?)
        }
        AccessSub::UserDelete { userid } => {
            Ok(client.delete(&format!("/access/users/{}", userid)).await?)
        }
        AccessSub::TokenList { userid } => {
            Ok(client.get(&format!("/access/users/{}/token", userid)).await?)
        }
        AccessSub::TokenCreate { userid, expire, description } => {
            let mut params = vec![];
            if let Some(ref e) = expire { params.push(("expire".to_string(), e.clone())); }
            if let Some(ref d) = description { params.push(("description".to_string(), d.clone())); }
            Ok(client.post_form(&format!("/access/users/{}/token", userid), if params.is_empty() { None } else { Some(&params) }).await?)
        }
        AccessSub::TokenRevoke { userid, tokenid } => {
            Ok(client.delete(&format!("/access/users/{}/token/{}", userid, tokenid)).await?)
        }
        AccessSub::AclList => Ok(client.get("/access/acl").await?),
        AccessSub::AclSet { path, role, user, group, propagate } => {
            let mut params = vec![
                ("path".to_string(), path.clone()),
                ("roles".to_string(), role.clone()),
                ("propagate".to_string(), propagate.to_string()),
            ];
            if let Some(ref u) = user { params.push(("userid".to_string(), u.clone())); }
            if let Some(ref g) = group { params.push(("groupid".to_string(), g.clone())); }
            Ok(client.put("/access/acl", Some(&params)).await?)
        }
        AccessSub::AclDelete { path, user, group } => {
            let mut params = vec![
                ("path".to_string(), path.clone()),
                ("delete".to_string(), "1".to_string()),
            ];
            if let Some(ref u) = user { params.push(("userid".to_string(), u.clone())); }
            if let Some(ref g) = group { params.push(("groupid".to_string(), g.clone())); }
            Ok(client.put("/access/acl", Some(&params)).await?)
        }
        AccessSub::Roles => Ok(client.get("/access/roles").await?),
    }
}

// === Backup subcommand handler ===

async fn run_backup_cmd(client: &PveClient, sub: &BackupSub) -> anyhow::Result<Value> {
    match sub {
        BackupSub::ScheduleList => Ok(client.get("/cluster/backup").await?),
        BackupSub::ScheduleCreate { id, schedule, storage, selection, mode, enabled } => {
            let mut params = vec![
                ("id".to_string(), id.clone()),
                ("schedule".to_string(), schedule.clone()),
                ("storage".to_string(), storage.clone()),
                ("selection".to_string(), selection.clone()),
            ];
            if let Some(ref m) = mode { params.push(("mode".to_string(), m.clone())); }
            if let Some(ref e) = enabled { params.push(("enabled".to_string(), if *e { "1" } else { "0" }.to_string())); }
            Ok(client.post_form("/cluster/backup", Some(&params)).await?)
        }
        BackupSub::ScheduleUpdate { id, schedule, storage, selection } => {
            let mut params = vec![];
            if let Some(ref s) = schedule { params.push(("schedule".to_string(), s.clone())); }
            if let Some(ref s) = storage { params.push(("storage".to_string(), s.clone())); }
            if let Some(ref s) = selection { params.push(("selection".to_string(), s.clone())); }
            Ok(client.put(&format!("/cluster/backup/{}", id), if params.is_empty() { None } else { Some(&params) }).await?)
        }
        BackupSub::ScheduleDelete { id } => {
            Ok(client.delete(&format!("/cluster/backup/{}", id)).await?)
        }
        BackupSub::Run { vmid, storage } => {
            let params = vec![
                ("vmid".to_string(), vmid.to_string()),
                ("storage".to_string(), storage.clone()),
            ];
            // Trigger backup via node-specific endpoint
            Ok(client.post_form(&format!("/nodes/self/qemu/{}/backup", vmid), Some(&params)).await?)
        }
    }
}

// === Network subcommand handler ===

async fn run_network_cmd(client: &PveClient, sub: &NetworkSub) -> anyhow::Result<Value> {
    match sub {
        NetworkSub::List { node } => Ok(client.get(&format!("/nodes/{}/network", node)).await?),
        NetworkSub::SdnZones => Ok(client.get("/cluster/sdn/zones").await?),
        NetworkSub::SdnZoneCreate { zone, type_, cidr } => {
            let mut params = vec![
                ("zone".to_string(), zone.clone()),
                ("type".to_string(), type_.clone()),
            ];
            if let Some(ref c) = cidr { params.push(("ipam".to_string(), c.clone())); }
            Ok(client.post_form("/cluster/sdn/zones", Some(&params)).await?)
        }
        NetworkSub::SdnVnets { zone } => {
            let url = zone.as_ref().map(|z| format!("/cluster/sdn/vnets?zone={}", z))
                .unwrap_or_else(|| "/cluster/sdn/vnets".to_string());
            Ok(client.get(&url).await?)
        }
        NetworkSub::SdnVnetCreate { vnet, zone } => {
            let params = vec![
                ("vnet".to_string(), vnet.clone()),
                ("zone".to_string(), zone.clone()),
            ];
            Ok(client.post_form("/cluster/sdn/vnets", Some(&params)).await?)
        }
        NetworkSub::SdnSubnets { vnet, zone } => {
            Ok(client.get(&format!("/cluster/sdn/vnets/{}/subnets?zone={}", vnet, zone)).await?)
        }
        NetworkSub::SdnSubnetCreate { vnet, zone, cidr } => {
            let params = vec![
                ("zone".to_string(), zone.clone()),
                ("subnet".to_string(), cidr.clone()),
            ];
            Ok(client.post_form(&format!("/cluster/sdn/vnets/{}/subnets", vnet), Some(&params)).await?)
        }
    }
}

// === Helpers ===

fn build_iface_params(iface: &NetworkInterfaceArgs) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(ref t) = iface.type_ { params.push(("type".to_string(), t.clone())); }
    if let Some(ref d) = iface.device { params.push(("device".to_string(), d.clone())); }
    if let Some(ref b) = iface.bridge { params.push(("bridge".to_string(), b.clone())); }
    if let Some(ref a) = iface.address { params.push(("address".to_string(), a.clone())); }
    if let Some(ref n) = iface.netmask { params.push(("netmask".to_string(), n.clone())); }
    if let Some(ref g) = iface.gateway { params.push(("gateway".to_string(), g.clone())); }
    if let Some(ref a) = iface.autostart {
        params.push(("autostart".to_string(), if *a { "1" } else { "0" }.to_string()));
    }
    if let Some(ref c) = iface.comments { params.push(("comments".to_string(), c.clone())); }
    params
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