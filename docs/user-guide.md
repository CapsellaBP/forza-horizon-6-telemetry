# FH6 Telemetry Toolkit — 用户手册 / User Guide

> **注意 / Note**: 英文部分由 AI 翻译，如有歧义以中文为准。The English section is AI-translated; the Chinese version takes precedence in case of ambiguity.

---

## 中文

### 简介

FH6 Telemetry Toolkit 是《极限竞速 地平线 6》的实时遥测工具套件。通过读取游戏 UDP 数据流，提供 HUD 浮层、功率曲线分析、换挡建议、轮胎/悬挂监控、录制回放等功能。

### 系统要求

- Windows 10/11
- Forza Horizon 6 (PC)
- 游戏内 Data Out 功能已开启

### 安装与运行

1. 下载 `FH6 Telemetry.exe`（单文件，无需 Python 或 Node.js）
2. 双击运行，主窗口自动打开
3. 首次使用需配置游戏内 Data Out（见下节）

### 游戏内设置

1. 进入游戏 → 设置 → HUD and Gameplay
2. **Data Out**: ON
3. **IP 地址**: `127.0.0.1`
4. **端口**: `5300`（与工具的 UDP 端口一致，可在设置页修改）

设置完成后进入一辆车并起步，控制面板即开始接收数据。

### 界面概览

主窗口左侧为标签栏，共 8 个页面：

| 标签 | 功能 |
|------|------|
| 总览 | HUD 预览 + 功率曲线 + G-G 图 + 胎温/滑移/悬挂 + 油门刹车 |
| HUD | HUD 浮层全部显示参数（字号/配色/标记线/G力浮动等） |
| 动力 | 功率/扭矩曲线 + 换挡激进程度 + 断油阈值 + 曲线管理 |
| 轮胎 | 胎温量杯 + 滑移率量杯 + 滑移阈值 |
| 悬挂 | 四轮行程示波器 + 胶囊柱状图 + 颜色阈值 |
| G值 | G-G 拖尾轨迹图 + 总G平滑 |
| 录制 | 自由录制/定时录制 + 文件管理 + 回放控制 |
| 设置 | 曲线采样参数 + 增压稳定 + EV检测 + UDP端口 |

### HUD 浮层

- 点击"启动 HUD"打开透明浮层窗口
- 浮层显示：档位、时速、RPM 条、换挡线（蓝）、断油线（红）、功率带（白）
- 爆闪提示：达到换挡点时档位和 RPM 条闪烁蓝光
- G 力浮动：加速/刹车/过弯时浮层整体漂移
- 编辑模式：点击"激活 HUD"可拖动窗口、调整大小，黄色虚线框表示编辑中
- 停车自动隐藏，起步恢复显示
- 点击"停止 HUD"关闭浮层，位置自动保存

### 换挡建议

- 自动学习当前车辆的功率曲线（EMA 采样）
- 在 HUD 和总览页显示换挡建议：hold（保持）/ near（接近）/ shift（换挡）/ over（超转）
- 自动检测断油转速
- 自动识别电动车（隐藏换挡/断油线）
- 换车时自动保存旧车曲线、加载新车曲线
- 可锁定曲线防止继续采样（双击功率图 / 按钮 / 总览车况卡片）
- 增压稳定采样：仅涡轮车有效，增压稳定后才采，避免爬升段低质量数据。动力页可调参数

### 录制与回放

- **自由录制**：点击"开始录制"，手动停止
- **定时录制**：设定倒计时 + 录制时长，自动停止
- **尾部裁剪**：录制停止后自动裁掉末尾 N 秒
- **回放**：选择文件播放，支持暂停/继续/停止
- 录制文件格式为 `.bin`（324 字节原始 UDP 包拼接），存放在 `tools/` 目录

### 参数说明

所有可调参数均配有信息提示（点击 ⓘ 图标）。悬停即显示说明，点击可锁定提示方便对照调节。详见设置页面。

### 数据文件

| 文件 | 说明 |
|------|------|
| `hud-settings.json` | 所有用户设置，删除恢复默认 |
| `hud-config.json` | HUD 窗口位置和大小 |
| `car_curves.json` | 每车功率曲线数据 |

---

## English

### Introduction

FH6 Telemetry Toolkit is a real-time telemetry suite for Forza Horizon 6. It reads the game's UDP data stream and provides a HUD overlay, power curve analysis, shift advisor, tire/suspension monitoring, recording/playback, and more.

### Requirements

- Windows 10/11
- Forza Horizon 6 (PC)
- Data Out enabled in-game

### Installation

1. Download `FH6 Telemetry.exe` (single file, no dependencies required)
2. Double-click to launch — the main window opens automatically
3. Configure in-game Data Out on first use (see below)

### In-Game Setup

1. In-game → Settings → HUD and Gameplay
2. **Data Out**: ON
3. **IP Address**: `127.0.0.1`
4. **Port**: `5300` (must match the tool's UDP port, configurable in Settings tab)

Enter a car and start driving — the control panel will begin receiving data.

### Interface Overview

The left sidebar has 8 tabs:

| Tab | Function |
|-----|----------|
| Dashboard | HUD preview + power curve + G-G plot + tire temp/slip/suspension + throttle/brake |
| HUD | Full HUD display parameters (fonts, colors, markers, G-force float, etc.) |
| Power | Power/torque curves + shift aggressiveness + limiter threshold + curve management |
| Tires | Tire temp gauges + slip ratio gauges + slip thresholds |
| Suspension | 4-wheel oscilloscopes + staggered bar chart + color thresholds |
| G-Force | G-G trail plot + total G smoothing |
| Record | Free/timed recording + file management + playback controls |
| Settings | Curve sampling + boost stable + EV detection + UDP port |

### HUD Overlay

- Click "Start HUD" to open the transparent overlay window
- Displays: gear, speed, RPM bar, shift line (blue), limiter line (red), power band (white)
- Strobe flash: gear and RPM bar flash blue when reaching shift point
- G-force float: overlay drifts with acceleration/braking/cornering
- Edit mode: click "Activate HUD" to drag and resize the window (yellow dashed border)
- Auto-hides when parked, reappears when moving
- Click "Stop HUD" to close — position is saved automatically

### Shift Advisor

- Automatically learns the current car's power curve (EMA sampling)
- Displays shift advice on HUD and Dashboard: hold / near / shift / over
- Auto-detects fuel cut RPM
- Auto-detects electric vehicles (hides shift/limiter lines)
- Saves curve on car change, loads saved curve on return
- Lock curve to prevent further sampling (double-click power chart / button / dashboard info card)
- Boost stable sampling: turbo cars only, waits for boost to stabilize before sampling. Parameters on Power page

### Recording & Playback

- **Free Record**: click "Start Recording", stop manually
- **Timed Record**: set countdown + duration, auto-stop
- **Tail Trim**: auto-trim last N seconds after recording stops
- **Playback**: select a file to play, with pause/resume/stop
- Recordings saved as `.bin` files (raw 324-byte UDP packets) in `tools/`

### Parameters

All adjustable parameters have info tooltips (click the ⓘ icon). Hover for instant display, click to lock the tooltip for reference while adjusting.

### Data Files

| File | Description |
|------|-------------|
| `hud-settings.json` | All user settings; delete to reset defaults |
| `hud-config.json` | HUD window position and size |
| `car_curves.json` | Per-car power curve data |
