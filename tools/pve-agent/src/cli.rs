//! CLI command definitions for pve-agent using clap

use clap::{Parser, Subcommand, ValueEnum, Args};
use std::path::PathBuf;

// === Global Args ===
#[derive(Parser, Debug, Clone)]
#[command(name = "pve-agent", about = "PVE CLI tool for AI agents", version)]
pub struct CliArgs {
    #[arg(long, env = "PVE_HOST")]
    pub host: Option<String>,

    #[arg(long, env = "PVE_PORT", default_value = "8006")]
    pub port: Option<u16>,

    #[arg(long, env = "PVE_USER", default_value = "root@pam")]
    pub user: Option<String>,

    #[arg(long, env = "PVE_TOKEN_ID")]
    pub token_id: Option<String>,

    #[arg(long, env = "PVE_TOKEN_SECRET")]
    pub token_secret: Option<String>,

    #[arg(long, env = "PVE_PASSWORD")]
    pub password: Option<String>,

    #[arg(long, default_value = "json")]
    pub output: OutputFormat,

    #[arg(long)]
    pub node: Option<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub yes: bool,

    #[arg(long)]
    pub wait: bool,

    #[arg(long, default_value = "60")]
    pub timeout: u32,

    #[arg(long)]
    pub verify_ssl: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
    Json, Table, Plain,
}

// === Command Tree ===
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Show PVE version info
    Version,

    /// Node management
    Node {
        #[command(subcommand)]
        sub: NodeSub,
    },

    /// VM/Container management
    Vm {
        #[command(subcommand)]
        sub: VmSub,
    },

    /// Storage management
    Storage {
        #[command(subcommand)]
        sub: StorageSub,
    },

    /// Cluster management
    Cluster {
        #[command(subcommand)]
        sub: ClusterSub,
    },

    /// Firewall management
    Firewall {
        #[command(subcommand)]
        sub: FirewallSub,
    },

    /// High Availability management
    Ha {
        #[command(subcommand)]
        sub: HaSub,
    },

    /// Access control / users
    Access {
        #[command(subcommand)]
        sub: AccessSub,
    },

    /// Backup jobs
    Backup {
        #[command(subcommand)]
        sub: BackupSub,
    },

    /// Network / SDN
    Network {
        #[command(subcommand)]
        sub: NetworkSub,
    },

    /// Resource pools
    Pool {
        #[command(subcommand)]
        sub: PoolSub,
    },

    /// Tasks
    Tasks {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
    },

    /// RRD / monitoring data
    Rrd {
        #[command(subcommand)]
        sub: RrdSub,
    },
}

// === Node Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum NodeSub {
    List,
    Status { node: String },
    Syslog { node: String, #[arg(long, default_value = "50")] lines: u32 },
    Disks { node: String },
    Services { node: String },
    Capabilities { node: String },
    Tasks { node: String, #[arg(long, default_value = "50")] limit: u32 },
    Rrd { node: String, #[arg(long, default_value = "hour")] timeframe: RrdTimeframe },
    Network { node: String },
    NetworkCreate { node: String, #[command(flatten)] iface: NetworkInterfaceArgs },
    NetworkUpdate { node: String, iface_name: String, #[command(flatten)] iface: NetworkInterfaceArgs },
    NetworkDelete { node: String, iface: String },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum RrdTimeframe {
    Hour, Day, Week, Month, Year,
}

impl RrdTimeframe {
    pub fn as_str(&self) -> &'static str {
        match self {
            RrdTimeframe::Hour => "hour",
            RrdTimeframe::Day => "day",
            RrdTimeframe::Week => "week",
            RrdTimeframe::Month => "month",
            RrdTimeframe::Year => "year",
        }
    }
}

// === VM Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum VmSub {
    List {
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        type_: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    Status { vmid: u32 },
    Config { vmid: u32 },
    Pending { vmid: u32 },

    Start { vmid: u32 },
    Stop { vmid: u32 },
    Shutdown { vmid: u32 },
    Reboot { vmid: u32 },
    Suspend { vmid: u32 },
    Resume { vmid: u32 },

    Create {
        node: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        memory: u32,
        #[arg(long)]
        cores: u32,
        #[arg(long)]
        disk: String,
        #[arg(long, default_value = "qemu")]
        type_: String,
        #[arg(long)]
        ostype: Option<String>,
        #[arg(long)]
        net: Option<String>,
        #[arg(long)]
        iso: Option<String>,
    },
    Delete { vmid: u32, #[arg(long)] force: bool },

    Clone { vmid: u32, newid: u32, #[arg(long)] name: String, #[arg(long)] target_node: Option<String>, #[arg(long)] full: bool },
    Migrate { vmid: u32, target_node: String, #[arg(long)] online: bool, #[arg(long)] timeout: Option<u32> },

    Snapshot {
        #[command(subcommand)]
        sub: SnapshotSub,
    },

    Update { vmid: u32, #[arg(long)] key: String, #[arg(long)] value: String },

    Disk {
        #[command(subcommand)]
        sub: DiskSub,
    },

    Agent {
        #[command(subcommand)]
        sub: AgentSub,
    },

    Console {
        #[arg(value_enum)]
        type_: ConsoleType,
        vmid: u32,
    },

    Tag { vmid: u32, #[command(subcommand)] sub: TagSub },
    Description { vmid: u32, #[arg(long)] get: bool, #[arg(long)] set: Option<String> },
    Template { vmid: u32 },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SnapshotSub {
    List { vmid: u32 },
    Create { vmid: u32, name: String, #[arg(long)] description: Option<String> },
    Rollback { vmid: u32, name: String },
    Delete { vmid: u32, name: String },
    Update { vmid: u32, name: String, #[arg(long)] description: String },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DiskSub {
    List { vmid: u32 },
    Resize { vmid: u32, disk: String, size: String },
    Move { vmid: u32, disk: String, #[arg(long)] target_storage: String, #[arg(long)] target_node: Option<String>, #[arg(long)] delete_source: bool },
    Detach { vmid: u32, disk: String },
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentSub {
    Info { vmid: u32 },
    Fsinfo { vmid: u32 },
    Network { vmid: u32 },
    Hwinfo { vmid: u32 },
    Exec { vmid: u32, command: String, #[arg(long)] args: Option<Vec<String>> },
    ExecStatus { vmid: u32, exec_id: u32 },
}

#[derive(Subcommand, Debug, Clone)]
pub enum TagSub {
    List { vmid: u32 },
    Add { vmid: u32, tags: Vec<String> },
    Remove { vmid: u32, tags: Vec<String> },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ConsoleType {
    Vnc, Spice,
}

#[derive(Subcommand, Debug, Clone)]
pub enum RrdSub {
    Node { node: String, #[arg(long, default_value = "hour")] timeframe: RrdTimeframe },
    Vm { vmid: u32, #[arg(long, default_value = "hour")] timeframe: RrdTimeframe },
    Storage { storage: String, #[arg(long, default_value = "hour")] timeframe: RrdTimeframe },
}

// === Storage Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum StorageSub {
    List {
        #[arg(long)]
        node: Option<String>,
    },
    Status { storage: String },
    Content { storage: String, #[arg(long)] type_: Option<String> },
    Create {
        storage: String,
        #[arg(long)]
        type_: String,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        server: Option<String>,
        #[arg(long)]
        export: Option<String>,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        nodes: Option<String>,
    },
    Update { storage: String, #[arg(long)] enabled: Option<bool>, #[arg(long)] comment: Option<String> },
    Delete { storage: String },
}

// === Cluster Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum ClusterSub {
    Status,
    Nodes,
    Resources {
        #[arg(long)]
        type_: Option<String>,
    },
    Tasks {
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    Nextid,
    Log {
        #[arg(long, default_value = "50")]
        lines: u32,
    },
}

// === Firewall Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum FirewallSub {
    List {
        #[arg(long)]
        level: Option<String>,
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        vmid: Option<u32>,
    },
    Rules {
        target: String,
        #[arg(long)]
        level: Option<String>,
        #[arg(long)]
        vmid: Option<u32>,
    },
    Groups {
        #[arg(long)]
        level: Option<String>,
    },
    Options {
        target: String,
        #[arg(long)]
        level: Option<String>,
        #[arg(long)]
        vmid: Option<u32>,
    },
    RuleAdd {
        target: String,
        #[arg(long)]
        level: Option<String>,
        #[arg(long)]
        vmid: Option<u32>,
        #[arg(long)]
        direction: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        protocol: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        dest: Option<String>,
        #[arg(long)]
        dport: Option<String>,
        #[arg(long)]
        comment: Option<String>,
        #[arg(long, default_value = "1")]
        enable: u32,
    },
    RuleDelete {
        target: String,
        pos: u32,
        #[arg(long)]
        level: Option<String>,
        #[arg(long)]
        vmid: Option<u32>,
    },
}

// === HA Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum HaSub {
    Status,
    Resources,
    Groups,
    ResourceCreate {
        resource: String,
        #[arg(long)]
        group: String,
        #[arg(long, default_value = "1")]
        max: u32,
        #[arg(long)]
        migration_type: Option<String>,
    },
    ResourceDelete { resource: String },
    GroupCreate {
        group: String,
        #[arg(long)]
        nodes: String,
        #[arg(long, default_value = "standard")]
        type_: String,
    },
    GroupDelete { group: String },
}

// === Access Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum AccessSub {
    UserList,
    UserCreate {
        userid: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long, default_value = "1")]
        enable: u32,
    },
    UserDelete { userid: String },
    TokenList { userid: String },
    TokenCreate {
        userid: String,
        #[arg(long)]
        expire: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    TokenRevoke { userid: String, tokenid: String },
    AclList,
    AclSet {
        path: String,
        role: String,
        #[arg(long)]
        user: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long, default_value = "1")]
        propagate: u32,
    },
    AclDelete {
        path: String,
        #[arg(long)]
        user: Option<String>,
        #[arg(long)]
        group: Option<String>,
    },
    Roles,
}

// === Backup Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum BackupSub {
    ScheduleList,
    ScheduleCreate {
        id: String,
        #[arg(long)]
        schedule: String,
        #[arg(long)]
        storage: String,
        #[arg(long)]
        selection: String,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        enabled: Option<bool>,
    },
    ScheduleUpdate {
        id: String,
        #[arg(long)]
        schedule: Option<String>,
        #[arg(long)]
        storage: Option<String>,
        #[arg(long)]
        selection: Option<String>,
    },
    ScheduleDelete { id: String },
    Run { vmid: u32, #[arg(long)] storage: String },
}

// === Network / SDN Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum NetworkSub {
    List { node: String },
    SdnZones,
    SdnZoneCreate { zone: String, #[arg(long)] type_: String, #[arg(long)] cidr: Option<String> },
    SdnVnets { #[arg(long)] zone: Option<String> },
    SdnVnetCreate { vnet: String, #[arg(long)] zone: String },
    SdnSubnets { vnet: String, #[arg(long)] zone: String },
    SdnSubnetCreate { vnet: String, zone: String, #[arg(long)] cidr: String },
}

#[derive(Args, Debug, Clone)]
pub struct NetworkInterfaceArgs {
    #[arg(long)]
    pub type_: Option<String>,
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub bridge: Option<String>,
    #[arg(long)]
    pub address: Option<String>,
    #[arg(long)]
    pub netmask: Option<String>,
    #[arg(long)]
    pub gateway: Option<String>,
    #[arg(long)]
    pub autostart: Option<bool>,
    #[arg(long)]
    pub comments: Option<String>,
}

// === Pool Subcommands ===
#[derive(Subcommand, Debug, Clone)]
pub enum PoolSub {
    List,
    Create { poolid: String, #[arg(long)] comment: Option<String> },
    Delete { poolid: String },
}