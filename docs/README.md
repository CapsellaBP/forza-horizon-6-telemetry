# FH6 Telemetry Toolkit

> AI 辅助翻译 / AI-assisted translation

> **v0.2.0** — Tauri + Rust + Vue 3 (2026-07-05)

Forza Horizon 6 实时遥测工具套件——UDP 数据采集、透明 HUD 浮层、Web 控制面板、换挡建议、功率曲线分析、录制/回放、每车曲线记忆。

*A real-time telemetry suite for Forza Horizon 6 — UDP capture, transparent HUD overlay, web control panel, shift advisor, power curve analysis, recording/playback, per-car curve memory.*

## 快速开始 / Quick Start

```bash
# 1. Install Rust (skip if done)
#    https://rustup.rs

# 2. Install Node.js dependencies
npm install

# 3. In-game: Settings → HUD and Gameplay → Data Out: ON
#    IP: 127.0.0.1  Port: 5300

# 4. Start dev mode
cargo tauri dev

# 5. Browser: http://localhost:1420 (Tauri window included)
```

## 分发 / Distribution

```bash
cargo tauri build
# Output: src-tauri/target/release/fh6-telemetry.exe (single file, no dependencies)
```

## 项目结构 / Project Structure

```
forza_horizen/
├── src-tauri/                # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs           # Entry point (no console window)
│   │   ├── lib.rs            # Tauri commands + HUD window + recording/playback
│   │   ├── server.rs         # UDP listen + WebSocket broadcast + settings
│   │   ├── telemetry.rs      # FH6 324-byte packet parser + tests
│   │   └── shift.rs          # Shift advisor (EMA sampling/fuel-cut/EV detection)
│   ├── Cargo.toml
│   └── tauri.conf.json       # Window / bundle config
│
├── src/                      # Vue 3 frontend
│   ├── App.vue               # Main layout + sidebar + WebSocket + status
│   ├── components/
│   │   ├── HudPreview.vue    # HUD live mirror
│   │   └── InfoTip.vue       # Hover/click tooltip
│   └── views/
│       ├── DashboardTab.vue  # Overview: HUD + power + G-G + tires + suspension
│       ├── HudTab.vue        # HUD: all display params
│       ├── PowerTab.vue      # Power: curves + shift params + curve mgmt
│       ├── TiresTab.vue      # Tires: temp/slip 2x2 gauges
│       ├── SuspensionTab.vue # Suspension: scopes + bar chart + thresholds
│       ├── GForceTab.vue     # G-Force: G-G trail plot
│       ├── RecordTab.vue     # Record/playback: controls + file mgmt
│       └── SettingsTab.vue   # Settings: sampling/boost/EV/port
│
├── public/
│   └── hud.html              # HUD overlay page (transparent WebView2)
│
├── tools/
│   └── capture_raw.py        # Python capture tool (real data for testing)
│
├── docs/                     # Documentation
│   ├── README.md             # This file
│   ├── telemetry.md          # Telemetry data format (ZH)
│   ├── settings.md           # Settings reference (ZH)
│   ├── tuning.md             # Tuning guide (ZH)
│   ├── development.md        # Development guide (ZH)
│   └── user-guide.md         # User manual (ZH/EN bilingual)
│
├── server.py / capture.py    # Python original (reference, unmaintained)
├── hud/ / web/               # Old Electron HUD + web panel (deprecated)
└── hud-settings.json         # User settings (runtime, gitignored)
    hud-config.json           # HUD window position (auto-saved)
    car_curves.json           # Per-car power curves (auto-saved)
```

## 技术栈 / Tech Stack

| Layer | Tech | Notes |
|-------|------|-------|
| Capture | Rust `tokio::net::UdpSocket` | UDP port configurable, default 5300, 324-byte LE |
| Server | Rust `tokio` + `tokio-tungstenite` | 60Hz WebSocket broadcast |
| Desktop | Tauri 2.x | Rust backend + WebView2 frontend |
| Frontend | Vue 3 + TypeScript + Vite | VSCode dark theme #191A1B / #121314 |
| HUD | Tauri multi-window | Transparent / always-on-top / click-through / multi-monitor |
| Settings | JSON files | hud-settings.json synced in realtime |

## 系统要求 / Requirements

- Windows 10/11
- Forza Horizon 6 (PC)
- Rust 1.96+ / Node.js 24+ (development only)
- No dependencies required after build
