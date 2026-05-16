# PVE CLI & TUI 工具

> Proxmox VE 管理命令行工具，为 AI Agent 和人类用户分别提供 CLI / TUI 界面

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT%20%7C%20Apache--2.0-blue.svg)](LICENSE)

---

## 项目概述

本项目包含两个独立的工具，共享同一套 Rust 核心库：

| 工具 | 名称 | 输出格式 | 交互方式 | 目标用户 |
|------|------|---------|---------|---------|
| **CLI** | `pve-agent` | JSON（默认） | 无交互（`--yes` 全自动） | AI Agent / 自动化脚本 |
| **TUI** | `pve-tui` | 表格（默认） | 键盘/鼠标 TUI | 人类用户 |

### 兼容性

- ✅ Proxmox VE 7.x
- ✅ Proxmox VE 8.x
- ✅ Proxmox VE 9.x

---

## 快速开始

### 安装

```bash
# 从源码编译
cd pve-tools
cargo build --release

# 二进制文件位于
./target/release/pve-agent   # CLI 工具
./target/release/pve-tui     # TUI 工具

# 可选：安装到 PATH
sudo cp ./target/release/pve-agent /usr/local/bin/
sudo cp ./target/release/pve-tui /usr/local/bin/
```

### 环境变量配置

```bash
export PVE_HOST=192.168.1.100
export PVE_PORT=8006
export PVE_USER=root@pam
export PVE_TOKEN=your-api-token-here   # 推荐使用 Token
export PVE_PASSWORD=your-password       # 或使用密码
export PVE_VERIFY_SSL=false             # 跳过自签名证书
```

### 配置文件（可选）

配置文件位于 `~/.config/pve/config.toml`：

```toml
[default]
host = "192.168.1.100"
port = 8006
user = "root@pam"
token = "your-api-token-here"
verify_ssl = false

[pve1]
host = "192.168.1.101"
user = "root@pam"
token = "xxxx-xxxx-xxxx"
```

---

## TUI 界面

pve-tui 提供交互式 TUI 界面，启动后会先显示连接配置界面。

### 启动

```bash
./pve-tui
```

### 连接配置界面

首次启动时会显示配置界面，需要填写以下信息：

```
╔══════════════════════════════════════════════════════════════╗
║              PVE Manager - Connection Setup                  ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  Host: 192.168.1.100                                         ║
║  Port: 8006                                                  ║
║  User: root@pam                                              ║
║  Auth: Token (T/P)                                          ║
║  API Token: ████████████████████                            ║
║  Skip SSL: false (y/N)                                      ║
║                                                              ║
║              [ Connect ]                                     ║
║                                                              ║
║  Tab/↓: Next field  Shift+Tab/↑: Prev                       ║
║  Enter: Connect  Esc: Quit                                  ║
╚══════════════════════════════════════════════════════════════╝
```

#### 配置字段说明

| 字段 | 说明 | 默认值 |
|------|------|--------|
| Host | PVE 主机 IP 或 hostname | 环境变量 `PVE_HOST` 或 `192.168.1.100` |
| Port | API 端口 | 环境变量 `PVE_PORT` 或 `8006` |
| User | 用户名（格式：`user@pam` 或 `user@pam@realm`） | 环境变量 `PVE_USER` 或 `root@pam` |
| Auth | 认证方式：`T` = Token（推荐），`P` = Password | 优先使用 Token |
| API Token | API Token（格式：`userid=tokenid=secret`） | 环境变量 `PVE_TOKEN` |
| Password | 密码（选择 Password 认证时使用） | 环境变量 `PVE_PASSWORD` |
| Skip SSL | 跳过 SSL 验证（用于自签名证书） | 环境变量 `PVE_VERIFY_SSL` 或 `false` |

#### 快捷键

| 按键 | 功能 |
|------|------|
| `Tab` / `↓` | 下一字段 |
| `Shift+Tab` / `↑` | 上一字段 |
| `←` / `→` | 移动光标 |
| `Enter` | 连接 |
| `y` / `n` | 在 Skip SSL 字段切换 |
| `t` / `p` | 在 Auth 字段切换 Token/Password |
| `Backspace` | 删除字符 |
| `Home` | 光标移到行首 |
| `End` | 光标移到行尾 |
| `q` / `Esc` | 退出程序 |

### 主界面布局

连接成功后显示主界面：

```
╔══════════════════════════════════════════════════════════════╗
║  PVE Manager  |  192.168.1.100                                ║
╠══════════════════════════════════════════════════════════════╣
║  [Dashboard]  [VMs]  [Storage]  [Logs]  [Help]               ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║                    主视图区域                                 ║
║                                                              ║
╠══════════════════════════════════════════════════════════════╣
║  Tab: Left/Right  |  R: Refresh  |  q: Quit  |  c: Disconnect║
╚══════════════════════════════════════════════════════════════╝
```

#### 视图说明

| 视图 | 内容 |
|------|------|
| Dashboard | PVE 版本、集群状态、节点概览、存储概览 |
| VMs | 所有 VM/容器列表，显示状态、CPU、内存 |
| Storage | 存储列表及使用情况 |
| Logs | 集群日志 |
| Help | 快捷键帮助 |

#### 主界面快捷键

| 按键 | 功能 |
|------|------|
| `q` | 退出程序 |
| `c` / `Esc` | 断开连接，返回配置界面 |
| `Tab` / `←` / `→` | 切换视图 |
| `1-5` | 直接切换到对应视图 |
| `↑` / `↓` 或 `j` / `k` | 选择 VM/容器 |
| `r` / `R` | 刷新当前视图 |
| `s` | 启动选中的 VM |
| `x` | 停止选中的 VM |

---

## CLI 工具

```bash
# === 查看版本/状态 ===
pve-agent version                           # PVE 版本信息
pve-agent node list                         # 节点列表
pve-agent node status <node>                # 节点状态

# === VM/容器管理 ===
pve-agent vm list                           # 列出所有 VM
pve-agent vm list --status running          # 仅运行中的 VM
pve-agent vm status <vmid>                   # VM 详细信息
pve-agent vm start <vmid>                    # 启动 VM
pve-agent vm stop <vmid>                    # 停止 VM
pve-agent vm shutdown <vmid> --wait         # 优雅关机并等待完成

# === 快照管理 ===
pve-agent vm snapshot list <vmid>           # 快照列表
pve-agent vm snapshot create <vmid> <name>  # 创建快照
pve-agent vm snapshot rollback <vmid> <name> # 回滚快照

# === 克隆/迁移 ===
pve-agent vm clone <vmid> <newid> --name <newname>
pve-agent vm migrate <vmid> <target-node>

# === 存储 ===
pve-agent storage list                      # 存储列表
pve-agent storage status <storage>         # 存储使用率

# === 集群 ===
pve-agent cluster status                   # Quorum 状态
pve-agent cluster resources                # 集群资源
pve-agent cluster tasks                    # 运行中的任务

# === 输出格式 ===
pve-agent vm list                           # JSON 输出（默认）
pve-agent vm list --table                   # 表格输出（人类友好）
```

### Agent 专用参数

```bash
# === Dry-run 模式（不实际执行）===
pve-agent vm delete 100 --dry-run

# === 全自动模式（跳过所有确认）===
pve-agent vm delete 100 --yes

# === 等待操作完成 ===
pve-agent vm start 100 --wait --timeout 120

# === 指定节点 ===
pve-agent vm list --node pve1 --type qemu

# === 输出格式 ===
pve-agent vm list --format json
```

---

## 项目结构

```
pve-tools/
├── Cargo.toml              # Workspace 配置
├── README.md
│
├── crates/
│   └── proxmox-api/        # 共享核心库
│       ├── src/
│       │   ├── lib.rs      # 库入口
│       │   ├── client/     # HTTP 客户端、认证、配置
│       │   │   ├── mod.rs
│       │   │   ├── auth.rs      # 认证（Token/Password）
│       │   │   ├── config.rs    # 配置加载
│       │   │   ├── http.rs      # HTTP 客户端
│       │   │   └── retry.rs     # 重试逻辑
│       │   ├── api/        # PVE API 端点
│       │   ├── model/      # 数据模型
│       │   └── error.rs    # 错误类型
│       └── Cargo.toml
│
└── tools/
    ├── pve-agent/          # CLI 工具
    │   └── src/
    │       ├── main.rs     # CLI 入口
    │       └── commands/   # 命令模块
    │
    └── pve-tui/            # TUI 工具
        └── src/
            ├── main.rs     # TUI 入口
            ├── app.rs      # AppState 状态机
            └── ui.rs       # UI 渲染
```

---

## 错误处理

所有错误返回结构化 JSON，便于 Agent 解析：

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "VM 999 does not exist",
    "details": {"vmid": 999}
  }
}
```

### 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 验证失败 |
| 3 | 资源不存在 |
| 4 | 认证/权限失败 |
| 5 | 资源冲突 |
| 6 | 等待超时 |
| 7 | 服务器错误 |
| 10 | 连接失败 |
| 11 | 请求超时 |

---

## 开发指南

### 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release

# 运行测试
cargo test

# Clippy 检查
cargo clippy -- -D warnings
```

### 添加新的 API 端点

在 `crates/proxmox-api/src/api/` 下添加新的模块：

```rust
use crate::client::PveClient;
use crate::{PveResult, error::PveError};

pub async fn get_new_resource(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/new-resource").await
}
```

---

## 技术栈

| 组件 | 选择 | 理由 |
|------|------|------|
| 语言 | Rust | 性能、安全、跨平台 |
| HTTP | `reqwest` | async HTTPS |
| CLI | `clap` v4 | 自动补全、help |
| TUI | `ratatui` + `crossterm` | 活跃、现代 |
| JSON | `serde_json` | 强类型 |
| 异步 | `tokio` | 事实标准 |

---

## 参考资料

- [Proxmox VE API 文档](https://pve.proxmox.com/pve-docs/api-viewer/)
- [ratatui TUI 库](https://github.com/ratatui-org/ratatui)
- [clap CLI 库](https://github.com/clap-rs/clap)

---

> 本项目由 阿彤 开发和维护