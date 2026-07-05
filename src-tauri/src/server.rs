// FH6 telemetry server: UDP capture → WebSocket broadcast
// + Recording / Playback

use crate::telemetry;
use crate::shift::ShiftAdvisor;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex as StdMutex};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

const WS_ADDR: &str = "127.0.0.1:9000";
const BROADCAST_MS: u64 = 16;

/// Resolve path: dev mode → CWD-relative (cargo tauri dev sets CWD=src-tauri),
/// release → exe-relative (portable single-file distribution).
pub fn resolve_path(rel: &str) -> String {
    if cfg!(debug_assertions) {
        format!("../{rel}")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|d| d.join(rel)))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| rel.to_string())
    }
}

#[derive(Clone)]
pub struct State {
    pub raw: HashMap<String, f64>,
    pub packet_count: u64,
    pub settings: serde_json::Value,
}

pub struct SharedAdvisor {
    pub advisor: std::sync::Mutex<ShiftAdvisor>,
    pub is_ev: std::sync::Mutex<bool>,
    pub ev_check_count: std::sync::Mutex<u32>,
    pub gears_seen: std::sync::Mutex<Vec<i32>>,
    pub gear_stable: std::sync::Mutex<i32>,
    pub gear_stable_count: std::sync::Mutex<u32>,
    pub gear_stable_pending: std::sync::Mutex<i32>,
    pub last_car_id: std::sync::Mutex<i32>,
    pub car_curves: std::sync::Mutex<HashMap<i32, serde_json::Value>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            raw: HashMap::new(), packet_count: 0,
            settings: serde_json::json!({
                "hud_opacity": 0.70, "hud_scale": 2.0, "shift_aggressiveness": 0,
                "limiter_threshold": 1.0, "shift_trigger_pct": 1.0,
                "gear_yellow_start": 0.50, "rpm_bar_brightness": 1.0, "rpm_bar_hue": 0,
                "power_band_pct": 0.93, "power_band_opacity": 0.5,
                "gear_italic": true, "speed_italic": true, "label_italic": true,
                "gear_glow": true, "gear_font_weight": 500, "speed_font_weight": 500,
                "gear_font_size": 28, "speed_font_size": 42, "label_font_size": 12,
                "top_row_gap": 4, "speed_label_gap": 3, "g_float_amplitude": 2,
                "g_float_smoothing": 0.08, "g_float_accel_scale": 0.5,
                "g_float_brake_scale": 1.0, "g_float_lat_scale": 1.0,
                "marker_ema_alpha": 0.06,
                "g_total_ema_alpha": 0.05,
                "rpm_bar_width": 260,
                "boost_stable_sample": false,
                "udp_port": 5300,


                "susp_thr1": 0.30, "susp_thr2": 0.55, "susp_thr3": 0.80,
                "slip_warn": 0.10, "slip_danger": 0.50,
                "sample_throttle_min": 0.95, "sample_skip_frames": 8, "curve_alpha": 0.25,
                "power_drop_limit": 0.0,
                "ev_detect_frames": 300,
            }),
        }
    }
}

// ── Broadcast message ──

#[derive(Serialize)]
struct BroadcastMsg {
    #[serde(rename = "type")] msg_type: String,
    speed_kmh: f64, rpm: f64, rpm_max: f64, rpm_idle: f64, gear: i32,
    throttle: f64, brake: f64, power_kw: f64, torque_nm: f64, boost_psi: f64,
    accel_lon: f64, accel_lat: f64,
    car_id: i32, car_perf_index: i32, car_class: i32, drivetrain: i32, cylinders: i32,
    is_race_on: bool, is_ev: bool, hud_visible: bool, pkt_count: u64,
    tire_temp: [f64;4], tire_slip: [f64;4], susp_travel: [f64;4], max_slip: f64,
    settings: serde_json::Value,
    shift_advice: serde_json::Value,
    curve: serde_json::Value, power_band: serde_json::Value, hud_style: serde_json::Value,
    hud_running: bool,
    recording: bool, playing: bool, paused: bool, record_count: u64, play_count: u64,
}

#[inline] fn f_to_c(f: f64) -> f64 { ((f - 32.0) * 5.0 / 9.0 * 10.0).round() / 10.0 }

fn build_broadcast(state: &State, _connected: usize, edit_mode: bool, rec_on: bool, play_on: bool, paused: bool, rec_n: u64, adv: &SharedAdvisor) -> BroadcastMsg {
    let r = &state.raw;
    let rpm = r.get("current_engine_rpm").copied().unwrap_or(0.0);
    let rpm_max = r.get("engine_max_rpm").copied().unwrap_or(8000.0);
    let rpm_idle = r.get("engine_idle_rpm").copied().unwrap_or(1500.0);
    let speed = r.get("speed").copied().unwrap_or(0.0) * 3.6;
    let gear = *adv.gear_stable.lock().unwrap();
    let throttle = r.get("accel").copied().unwrap_or(0.0) / 255.0;

    // Shift advisor data
    let mut a = adv.advisor.lock().unwrap();
    let advice = a.advice(rpm, rpm_max, throttle, gear);
    let curve = a.get_curve(0.93);
    let is_ev = *adv.is_ev.lock().unwrap();

    BroadcastMsg {
        msg_type: "telemetry".into(),
        speed_kmh: (speed * 10.0).round() / 10.0,
        rpm: rpm.round(), rpm_max: rpm_max.round(), rpm_idle: rpm_idle.round(), gear,
        throttle: (r.get("accel").copied().unwrap_or(0.0) / 255.0 * 1000.0).round() / 1000.0,
        brake: (r.get("brake").copied().unwrap_or(0.0) / 255.0 * 1000.0).round() / 1000.0,
        power_kw: (r.get("power_w").copied().unwrap_or(0.0) / 1000.0 * 10.0).round() / 10.0,
        torque_nm: (r.get("torque_nm").copied().unwrap_or(0.0) * 10.0).round() / 10.0,
        boost_psi: (r.get("boost_psi").copied().unwrap_or(0.0) * 10.0).round() / 10.0,
        accel_lon: (-r.get("accel_z").copied().unwrap_or(0.0) / 9.80665 * 1000.0).round() / 1000.0,
        accel_lat: (-r.get("accel_x").copied().unwrap_or(0.0) / 9.80665 * 1000.0).round() / 1000.0,
        car_id: r.get("car_ordinal").copied().unwrap_or(0.0) as i32,
        car_perf_index: r.get("car_perf_index").copied().unwrap_or(0.0) as i32,
        car_class: r.get("car_class").copied().unwrap_or(0.0) as i32,
        drivetrain: r.get("drivetrain_type").copied().unwrap_or(0.0) as i32,
        cylinders: r.get("num_cylinders").copied().unwrap_or(0.0) as i32,
        is_race_on: r.get("is_race_on").copied().unwrap_or(0.0) > 0.0,
        is_ev, hud_visible: speed > 0.5, pkt_count: state.packet_count,
        tire_temp: [f_to_c(r.get("tire_temp_fl").copied().unwrap_or(0.0)), f_to_c(r.get("tire_temp_fr").copied().unwrap_or(0.0)), f_to_c(r.get("tire_temp_rl").copied().unwrap_or(0.0)), f_to_c(r.get("tire_temp_rr").copied().unwrap_or(0.0))],
        tire_slip: [(r.get("tire_slip_ratio_fl").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("tire_slip_ratio_fr").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("tire_slip_ratio_rl").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("tire_slip_ratio_rr").copied().unwrap_or(0.0)*1000.0).round()/1000.0],
        susp_travel: [(r.get("susp_travel_fl").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("susp_travel_fr").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("susp_travel_rl").copied().unwrap_or(0.0)*1000.0).round()/1000.0, (r.get("susp_travel_rr").copied().unwrap_or(0.0)*1000.0).round()/1000.0],
        max_slip: (r.get("tire_slip_ratio_fl").copied().unwrap_or(0.0).abs().max(r.get("tire_slip_ratio_fr").copied().unwrap_or(0.0).abs()).max(r.get("tire_slip_ratio_rl").copied().unwrap_or(0.0).abs()).max(r.get("tire_slip_ratio_rr").copied().unwrap_or(0.0).abs())*1000.0).round()/1000.0,
        settings: { let mut s = state.settings.clone(); if let Some(obj) = s.as_object_mut() { obj.insert("hud_edit_mode".into(), serde_json::Value::Bool(edit_mode)); } s },
        shift_advice: advice,
        curve: curve.clone(),
        power_band: serde_json::json!({ "lo": curve["power_band_lo"].as_f64().unwrap_or(0.0), "hi": curve["power_band_hi"].as_f64().unwrap_or(0.0) }),
        hud_style: serde_json::json!({ "gear_italic": true, "speed_italic": true, "label_italic": true, "label_font_size": 12, "gear_glow": true, "gear_font_weight": 500, "speed_font_weight": 500, "gear_font_size": 28, "speed_font_size": 42, "top_row_gap": 4, "g_float_amplitude": 1 }),
        hud_running: false,
        recording: rec_on,
        playing: play_on,
        paused,
        record_count: rec_n,
        play_count: 0,
    }
}

// ── Start server ──

pub fn start_server(
    cmd_tx: std::sync::mpsc::Sender<String>,
    edit_mode: Arc<AtomicBool>,
    recording: Arc<AtomicBool>,
    playing: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    pb_data: Arc<StdMutex<Vec<Vec<u8>>>>,
    pb_idx: Arc<StdMutex<usize>>,
    pb_total: Arc<StdMutex<usize>>,
    car_curves_save: Arc<StdMutex<serde_json::Value>>,
    pb_receiver: std::sync::mpsc::Receiver<Vec<u8>>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async { run(cmd_tx, edit_mode, recording, playing, paused, pb_data, pb_idx, pb_total, car_curves_save, pb_receiver).await; });
    });
}

async fn run(
    cmd_tx: std::sync::mpsc::Sender<String>,
    edit_mode: Arc<AtomicBool>,
    recording: Arc<AtomicBool>,
    playing: Arc<AtomicBool>,
    _paused: Arc<AtomicBool>,
    pb_data: Arc<StdMutex<Vec<Vec<u8>>>>,
    pb_idx: Arc<StdMutex<usize>>,
    pb_total: Arc<StdMutex<usize>>,
    car_curves_save: Arc<StdMutex<serde_json::Value>>,
    pb_receiver: std::sync::mpsc::Receiver<Vec<u8>>,
) {
    let saved: HashMap<i32, serde_json::Value> = std::fs::read_to_string(resolve_path("car_curves.json"))
        .ok().and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .map(|v| {
            *car_curves_save.lock().unwrap() = v.clone();
            v.as_object().map(|obj| obj.iter().filter_map(|(k, v)| k.parse().ok().map(|id| (id, v.clone()))).collect()).unwrap_or_default()
        }).unwrap_or_default();

    let state = Arc::new(RwLock::new(State::default()));
    let clients: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<Message>>>> = Arc::new(RwLock::new(Vec::new()));
    let rec_file = Arc::new(StdMutex::new(None::<std::fs::File>));
    let rec_count = Arc::new(StdMutex::new(0u64));
    let rec_path = Arc::new(StdMutex::new(String::new()));
    let adv = Arc::new(SharedAdvisor {
        advisor: StdMutex::new(ShiftAdvisor::new()),
        is_ev: StdMutex::new(false), ev_check_count: StdMutex::new(0),
        gears_seen: StdMutex::new(Vec::new()), gear_stable: StdMutex::new(1),
        gear_stable_count: StdMutex::new(0), gear_stable_pending: StdMutex::new(1),
        last_car_id: StdMutex::new(0),
        car_curves: StdMutex::new(saved),
    });

    // UDP listener
    let udp_state = state.clone();
    let udp_rec = recording.clone();
    let udp_play = playing.clone();
    let udp_adv = adv.clone();
    let udp_recfile = rec_file.clone();
    let udp_reccount = rec_count.clone();
    let ws_save = car_curves_save.clone();
    let pb_rx_shared = Arc::new(StdMutex::new(pb_receiver));
    // Load persisted settings from disk, merge into State
    if let Ok(raw) = std::fs::read_to_string(resolve_path("hud-settings.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(obj) = v.as_object() {
                let mut s = state.write().await;
                if let Some(settings) = s.settings.as_object_mut() {
                    for (k, v) in obj { settings.insert(k.clone(), v.clone()); }
                }
            }
        }
    }
    let udp_port = state.read().await.settings.get("udp_port").and_then(|v| v.as_u64()).unwrap_or(5300) as u16;
    println!("[FH6] Port from settings: {udp_port}");
    let udp_bind = format!("127.0.0.1:{udp_port}");
    tokio::spawn(async move {
        println!("[FH6] UDP spawn STARTED");
        let sock = match tokio::net::UdpSocket::bind(&udp_bind).await {
            Ok(s) => s,
            Err(e) => { eprintln!("[FH6] UDP {udp_bind} in use — is another instance running?"); eprintln!("[FH6] Error: {e}"); return; }
        };
        println!("[FH6] UDP {udp_bind} | WS {WS_ADDR}");
        let mut buf = [0u8; 4096];
        let mut was_recording = false;
        loop {
            let recv_result = match tokio::time::timeout(std::time::Duration::from_millis(100), sock.recv_from(&mut buf)).await {
                Ok(Ok((n, a))) => Ok((n, a)),
                _ => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "timeout")),
            };

            match recv_result {
                Ok((n, _)) => {
                    // Skip live UDP during playback
                    if udp_play.load(Ordering::SeqCst) { continue; }

                    // Recording: create file on start, close on stop
                    let is_rec = udp_rec.load(Ordering::SeqCst);
                    if is_rec && !was_recording {
                        let dur = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                        let path = resolve_path(&format!("tools/rec_{}.bin", dur));
                        if let Ok(f) = std::fs::File::create(&path) {
                            *udp_recfile.lock().unwrap() = Some(f);
                            *rec_path.lock().unwrap() = path.clone();
                            *udp_reccount.lock().unwrap() = 0;
                            println!("[FH6] Recording → {path}");
                        }
                    } else if !is_rec && was_recording {
                        *udp_recfile.lock().unwrap() = None;
                        let n = *udp_reccount.lock().unwrap();
                        println!("[FH6] Recording stopped ({n} pkts)");
                    }
                    was_recording = is_rec;

                    if let Some(pkt) = telemetry::parse(&buf[..n]) {
                        // Per-car curve memory (BEFORE feed)
                        let car_id = pkt.get("car_ordinal").copied().unwrap_or(0.0) as i32;
                        let throttle = pkt.get("accel").copied().unwrap_or(0.0) / 255.0;
                        // Read current settings (outside locks, before await)
                        let cur_settings = {
                            let s = udp_state.read().await;
                            (s.settings.get("sample_throttle_min").and_then(|v| v.as_f64()).unwrap_or(0.95),
                             s.settings.get("curve_alpha").and_then(|v| v.as_f64()).unwrap_or(0.25),
                             s.settings.get("sample_skip_frames").and_then(|v| v.as_u64()).unwrap_or(8) as u32,
                             s.settings.get("power_drop_limit").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        };
                        {
                            let mut last_id = udp_adv.last_car_id.lock().unwrap();
                            if car_id > 0 && *last_id == 0 {
                                // First car detected — load saved curve
                                let curves = udp_adv.car_curves.lock().unwrap();
                                if let Ok(mut a) = udp_adv.advisor.lock() {
                                    if let Some(saved) = curves.get(&car_id) {
                                        *a = ShiftAdvisor::from_dict(saved);
                                        a.throttle_min = cur_settings.0; a.alpha = cur_settings.1;
                                        a.skip_frames = cur_settings.2; a.power_drop_limit = cur_settings.3;
                                    }
                                    drop(a);
                                }
                                drop(curves);
                            }
                            if car_id > 0 && *last_id > 0 && car_id != *last_id {
                                // Always lock curves FIRST, then advisor (consistent order)
                                let mut curves = udp_adv.car_curves.lock().unwrap();
                                let mut a = udp_adv.advisor.lock().unwrap();
                                // Save old car
                                curves.insert(*last_id, a.to_dict());
                                let mut save_map = serde_json::Map::new();
                                for (k, v) in curves.iter() { save_map.insert(k.to_string(), v.clone()); }
                                *car_curves_save.lock().unwrap() = serde_json::Value::Object(save_map);
                                // Load new car (or reset)
                                if let Some(saved) = curves.get(&car_id) {
                                    *a = ShiftAdvisor::from_dict(saved);
                                } else {
                                    a.reset();
                                }
                                a.throttle_min = cur_settings.0;
                                a.alpha = cur_settings.1;
                                a.skip_frames = cur_settings.2;
                                a.power_drop_limit = cur_settings.3;
                                drop(a);
                                drop(curves);
                                *udp_adv.gears_seen.lock().unwrap() = Vec::new();
                                *udp_adv.is_ev.lock().unwrap() = false;
                                *udp_adv.ev_check_count.lock().unwrap() = 0;
                            }
                            if car_id > 0 { *last_id = car_id; }
                        }

                        // Feed shift advisor
                        let rpm = pkt.get("current_engine_rpm").copied().unwrap_or(0.0);
                        let torque = pkt.get("torque_nm").copied().unwrap_or(0.0);
                        let power = pkt.get("power_w").copied().unwrap_or(0.0);
                        let gear = pkt.get("gear").copied().unwrap_or(1.0) as i32;
                        let rpm_max = pkt.get("engine_max_rpm").copied().unwrap_or(8000.0);
                        let idle = pkt.get("engine_idle_rpm").copied().unwrap_or(1500.0);
                        if let Ok(mut a) = udp_adv.advisor.lock() {
                            a.idle_rpm = idle.max(500.0);
                            a.rpm_max = rpm_max.max(1000.0);
                            let boost = pkt.get("boost_psi").copied().unwrap_or(0.0);
                    a.feed(rpm, torque, power, throttle, gear, boost);
                        }
                        // Gear debounce + EV
                        let raw_gear = gear;
                        if raw_gear >= 0 && raw_gear <= 10 {
                            let mut pending = udp_adv.gear_stable_pending.lock().unwrap();
                            let mut cnt = udp_adv.gear_stable_count.lock().unwrap();
                            if raw_gear == *pending { *cnt += 1; } else { *pending = raw_gear; *cnt = 1; }
                            if *cnt >= 3 { *udp_adv.gear_stable.lock().unwrap() = raw_gear; }
                        }
                        let stable_gear = *udp_adv.gear_stable.lock().unwrap();
                        crate::shift::check_ev(
                            &mut udp_adv.gears_seen.lock().unwrap(), stable_gear, throttle,
                            &mut udp_adv.ev_check_count.lock().unwrap(),
                            300, &mut udp_adv.is_ev.lock().unwrap(),
                        );

                        let mut s = udp_state.write().await;
                        s.raw = pkt;
                        s.packet_count += 1;
                        // Write to recording file
                        if is_rec {
                            if let Ok(mut f) = udp_recfile.lock() {
                                if let Some(ref mut file) = *f {
                                    let _ = file.write_all(&buf[..324]);
                                    *udp_reccount.lock().unwrap() += 1;
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
    });

    // WebSocket listener
    let ws_state = state.clone();
    let ws_clients = clients.clone();
    let ws_cmd_tx = cmd_tx.clone();
    let ws_adv = adv.clone();
    let br_save = ws_save.clone();
    tokio::spawn(async move {
        let listener = match TcpListener::bind(WS_ADDR).await {
            Ok(l) => l,
            Err(e) => { eprintln!("[FH6] WS bind failed: {e}"); return; }
        };
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let ws = tokio_tungstenite::accept_async(stream).await;
                if let Ok(ws) = ws {
                    let (mut tx, mut rx) = ws.split();
                    let (send, mut recv) = tokio::sync::mpsc::unbounded_channel::<Message>();
                    ws_clients.write().await.push(send);
                    let cl = ws_clients.clone();
                    let tx_cmd = ws_cmd_tx.clone();
                    let st = ws_state.clone();
                    let adv2 = ws_adv.clone();
                    let save2 = ws_save.clone();
                    tokio::spawn(async move {
                        loop {
                            tokio::select! {
                                msg = rx.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            // set_setting handled here directly
                                            if text.contains("set_setting") {
                                                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                                    if let Some(payload) = msg.get("payload") {
                                                        let mut s = st.write().await;
                                                        if let Some(obj) = s.settings.as_object_mut() {
                                                            if let Some(p) = payload.as_object() {
                                                                for (k,v) in p { obj.insert(k.clone(), v.clone()); }
                                                            }
                                                        }
                                                        // Persist settings to disk
                                                        let json = serde_json::to_string(&s.settings).unwrap_or_default();
                                                        match std::fs::write(resolve_path("hud-settings.json"), &json) {
                                                            Ok(_) => {},
                                                            Err(e) => eprintln!("[FH6] Failed to save settings: {e}"),
                                                        }
                                                        // Sync to advisor
                                                        if let Some(v) = payload.get("power_drop_limit").and_then(|v| v.as_f64()) {
                                                            if let Ok(mut a) = adv2.advisor.lock() { a.power_drop_limit = v; }
                                                        }
                                                        if let Some(v) = payload.get("boost_stable_sample").and_then(|v| v.as_bool()) {
                                                            if let Ok(mut a) = adv2.advisor.lock() { a.boost_stable_sample = v; }
                                                        }
                                                        if let Some(v) = payload.get("curve_alpha").and_then(|v| v.as_f64()) {
                                                            if let Ok(mut a) = adv2.advisor.lock() { a.alpha = v; }
                                                        }
                                                        if let Some(v) = payload.get("sample_throttle_min").and_then(|v| v.as_f64()) {
                                                            if let Ok(mut a) = adv2.advisor.lock() { a.throttle_min = v; }
                                                        }
                                                        if let Some(v) = payload.get("sample_skip_frames").and_then(|v| v.as_u64()) {
                                                            if let Ok(mut a) = adv2.advisor.lock() { a.skip_frames = v as u32; }
                                                        }
                                                    }
                                                }
                                            }
                                            else if text.contains("reset_curve") {
                                                if let Ok(mut a) = adv2.advisor.lock() { a.reset(); }
                                                *adv2.is_ev.lock().unwrap() = false; *adv2.ev_check_count.lock().unwrap() = 0;
                                                adv2.gears_seen.lock().unwrap().clear();
                                                let car_id = *adv2.last_car_id.lock().unwrap();
                                                adv2.car_curves.lock().unwrap().remove(&car_id);
                                                let curves = adv2.car_curves.lock().unwrap();
                                                let mut m = serde_json::Map::new();
                                                for (k, v) in curves.iter() { m.insert(k.to_string(), v.clone()); }
                                                *save2.lock().unwrap() = serde_json::Value::Object(m);
                                            }
                                            else if text.contains("reset_all") {
                                                if let Ok(mut a) = adv2.advisor.lock() { a.reset(); }
                                                adv2.car_curves.lock().unwrap().clear();
                                                *adv2.is_ev.lock().unwrap() = false; *adv2.ev_check_count.lock().unwrap() = 0;
                                                adv2.gears_seen.lock().unwrap().clear();
                                                *save2.lock().unwrap() = serde_json::json!({});
                                                let _ = std::fs::write(resolve_path("car_curves.json"), "{}");
                                            }
                                            // JSON payloads with action params → background thread
                                            else if text.starts_with("{") {
                                                let _ = tx_cmd.send(text);
                                            }
                                            else if text.contains("lock_curve") {
                                                if let Ok(mut a) = adv2.advisor.lock() { a.locked = true; }
                                            }
                                            else if text.contains("unlock_curve") {
                                                if let Ok(mut a) = adv2.advisor.lock() { a.locked = false; }
                                            }
                                            else if text.contains("restart_app") { let _ = tx_cmd.send("restart_app".into()); }
                                            else if text.contains("reset_hud") { let _ = tx_cmd.send("reset_hud".into()); }
                                            else if text.contains("start_hud") { let _ = tx_cmd.send("start_hud".into()); }
                                            else if text.contains("stop_hud") { let _ = tx_cmd.send("stop_hud".into()); }
                                            else if text.contains("hud_edit_mode") { let _ = tx_cmd.send("hud_edit_mode".into()); }
                                            else if text.contains("start_record") { let _ = tx_cmd.send("start_record".into()); }
                                            else if text.contains("stop_record") { let _ = tx_cmd.send("stop_record".into()); }
                                            else if text.contains("start_playback") { let _ = tx_cmd.send("start_playback".into()); }
                                            else if text.contains("stop_playback") { let _ = tx_cmd.send("stop_playback".into()); }
                                            else if text.contains("pause_playback") { let _ = tx_cmd.send("pause_playback".into()); }
                                            else if text.contains("resume_playback") { let _ = tx_cmd.send("resume_playback".into()); }
                                            else if text.contains("list_recordings") {
                                                println!("[FH6] list_recordings requested");
                                                match std::fs::read_dir(resolve_path("tools")) {
                                                    Ok(dir) => {
                                                        let mut files: Vec<serde_json::Value> = dir.filter_map(|e| e.ok())
                                                            .filter(|e| e.file_name().to_string_lossy().ends_with(".bin"))
                                                            .filter_map(|e| {
                                                                let path = e.path();
                                                                let meta = e.metadata().ok()?;
                                                                let size = meta.len();
                                                                Some(serde_json::json!({
                                                                    "name": path.file_stem()?.to_string_lossy(),
                                                                    "path": path.to_string_lossy(),
                                                                    "size": size, "pkts": size as u64 / 324
                                                                }))
                                                            }).collect();
                                                        files.sort_by_key(|f| f["name"].as_str().unwrap_or("").to_string());
                                                        files.reverse();
                                                        println!("[FH6] Found {} recording files", files.len());
                                                        let msg = serde_json::json!({"type":"rec_files","files":files}).to_string();
                                                        let _ = tx.send(Message::Text(msg.into())).await;
                                                    }
                                                    Err(e) => eprintln!("[FH6] list_recordings error: {e}"),
                                                }
                                            }
                                            else if text.contains("\"delete_recording\"") { let _ = tx_cmd.send(text); }
                                            else if text.contains("\"playback_file\"") { let _ = tx_cmd.send(text); }
                                        }
                                        Some(Err(_)) | None => break,
                                        _ => {}
                                    }
                                }
                                msg = recv.recv() => {
                                    match msg { Some(m) => { let _ = tx.send(m).await; } None => break, }
                                }
                            }
                        }
                        cl.write().await.retain(|c| !c.is_closed());
                    });
                }
            }
        }
    });

    // Broadcast loop
    // Playback state for broadcast loop
    let pb_brx = pb_rx_shared.clone();
    let pb_data2 = pb_data.clone();
    let pb_idx2 = pb_idx.clone();
    let pb_total2 = pb_total.clone();

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(BROADCAST_MS));
    let mut save_tick: u64 = 0;
    // Local playback buffer (populated from channel)
    let mut pb_buf: Vec<Vec<u8>> = Vec::new();
    let mut pb_i = 0usize;
    let mut pb_paused = false;
    let mut pb_first_ts: u64 = 0;
    let mut pb_prev_ts: u64 = 0;
    loop {
        interval.tick().await;

        // Check playback channel
        while let Ok(raw) = pb_brx.lock().unwrap().try_recv() {
            let text = String::from_utf8_lossy(&raw[..raw.len().min(20)]);
            if text == "STOP" { pb_buf.clear(); pb_i = 0; pb_paused = false; state.write().await.packet_count = 0; }
            else if text == "PAUSE" { pb_paused = true; }
            else if text == "RESUME" { pb_paused = false; }
            else if let Some(data_start) = raw.iter().position(|&b| b == b':')
                .and_then(|i| raw[i+1..].iter().position(|&b| b == b':').map(|j| i+j+2))
            {
                pb_buf = raw[data_start..].chunks(324).map(|c| c.to_vec()).collect();
                pb_i = 0; pb_paused = false;
                println!("[FH6] PB loaded: {} pkts", pb_buf.len());
                // Sync to shared arcs so frontend can see progress
                *pb_data2.lock().unwrap() = pb_buf.clone();
                *pb_idx2.lock().unwrap() = pb_i;
                *pb_total2.lock().unwrap() = pb_buf.len();
            }
        }

        // Advance playback frame & feed advisor (with timestamp-based timing)
        if !pb_buf.is_empty() && !pb_paused && pb_i < pb_buf.len() {
            if let Some(ref pkt) = telemetry::parse(&pb_buf[pb_i]) {
                // Per-car curve memory (same logic as live UDP)
                let car_id = pkt.get("car_ordinal").copied().unwrap_or(0.0) as i32;
                let mut last_id = adv.last_car_id.lock().unwrap();
                // First car: load saved curve
                if car_id > 0 && *last_id == 0 {
                    let curves = adv.car_curves.lock().unwrap();
                    if let Ok(mut a) = adv.advisor.lock() {
                        if let Some(saved) = curves.get(&car_id) { *a = ShiftAdvisor::from_dict(saved); }
                    }
                }
                // Car change
                if car_id > 0 && *last_id > 0 && car_id != *last_id {
                    let mut curves = adv.car_curves.lock().unwrap();
                    if let Ok(a) = adv.advisor.lock() {
                        curves.insert(*last_id, a.to_dict());
                    }
                    if let Ok(mut a) = adv.advisor.lock() {
                        if let Some(saved) = curves.get(&car_id) { *a = ShiftAdvisor::from_dict(saved); }
                        else { a.reset(); }
                    }
                    *adv.gears_seen.lock().unwrap() = Vec::new();
                    *adv.is_ev.lock().unwrap() = false;
                    *adv.ev_check_count.lock().unwrap() = 0;
                }
                if car_id > 0 { *last_id = car_id; }
                drop(last_id);

                // Read timestamp for timing
                let ts = pkt.get("timestamp_ms").copied().unwrap_or(0.0) as u64;
                if pb_i == 0 {
                    // First frame: record baseline
                    pb_first_ts = ts;
                    pb_prev_ts = ts;
                } else {
                    let prev = pb_prev_ts;
                    let delta_ms = if ts > prev { ts - prev } else { 16 };
                    let delta_ms = delta_ms.min(200); // cap at 200ms (5Hz min)
                    if delta_ms > 0 {
                        tokio::time::sleep(std::time::Duration::from_millis(delta_ms)).await;
                    }
                    pb_prev_ts = ts;
                }
                // Feed advisor and update state
                let rpm = pkt.get("current_engine_rpm").copied().unwrap_or(0.0);
                let torque = pkt.get("torque_nm").copied().unwrap_or(0.0);
                let power = pkt.get("power_w").copied().unwrap_or(0.0);
                let throttle = pkt.get("accel").copied().unwrap_or(0.0) / 255.0;
                let gear = pkt.get("gear").copied().unwrap_or(1.0) as i32;
                let idle = pkt.get("engine_idle_rpm").copied().unwrap_or(1500.0);
                let rmax = pkt.get("engine_max_rpm").copied().unwrap_or(8000.0);
                let mut s = state.write().await;
                s.raw = pkt.clone();
                s.packet_count += 1;
                if let Ok(mut a) = adv.advisor.lock() {
                    a.idle_rpm = idle.max(500.0);
                    a.rpm_max = rmax.max(1000.0);
                    let boost = pkt.get("boost_psi").copied().unwrap_or(0.0);
                    a.feed(rpm, torque, power, throttle, gear, boost);
                }
            }
            pb_i += 1;
            *pb_idx2.lock().unwrap() = pb_i;
            // End of playback reached
            if pb_i >= pb_buf.len() { pb_buf.clear(); pb_i = 0; state.write().await.packet_count = 0; }
        }

        let pb_playing = !pb_buf.is_empty();
        let st = state.read().await;
        let cl = clients.read().await;
        let _pb_n = *pb_idx2.lock().unwrap();
        let pb_t = *pb_total2.lock().unwrap();
        let msg = build_broadcast(&st, cl.len(),
            edit_mode.load(Ordering::SeqCst),
            recording.load(Ordering::SeqCst),
            pb_playing,
            pb_paused,
            *rec_count.lock().unwrap(),
            &adv,
        );
        if let Ok(mut json) = serde_json::to_string(&msg) {
            if pb_t > 0 && pb_i > 0 {
                let elapsed = (pb_prev_ts - pb_first_ts) as f64 / 1000.0;
                let total = elapsed * pb_t as f64 / pb_i.max(1) as f64;
                let fps = if elapsed > 0.0 { pb_i as f64 / elapsed } else { 0.0 };
                let prog = format!(", \"pb_progress\": {:.1}, \"pb_total\": {:.1}, \"pb_pkts\": {}, \"pb_pkts_total\": {}, \"pb_fps\": {:.0}",
                    elapsed, total, pb_i, pb_t, fps);
                json.pop();
                json.push_str(&prog);
                json.push('}');
            }
            let ws_msg = Message::Text(json.into());
            for c in cl.iter() { let _ = c.send(ws_msg.clone()); }

            // Periodic curve save: sync current car to car_curves_save (~1s)
            save_tick += 1;
            if save_tick % 60 == 0 {
                if let Ok(a) = adv.advisor.lock() {
                    if a.ready {
                        let car_id = *adv.last_car_id.lock().unwrap();
                        if car_id > 0 {
                            let mut curves = adv.car_curves.lock().unwrap();
                            curves.insert(car_id, a.to_dict());
                            let mut m = serde_json::Map::new();
                            for (k, v) in curves.iter() { m.insert(k.to_string(), v.clone()); }
                            *br_save.lock().unwrap() = serde_json::Value::Object(m);
                        }
                    }
                }
            }
        }
    }
}