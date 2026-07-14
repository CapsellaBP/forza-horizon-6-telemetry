// FH6 Telemetry — Tauri backend
#![recursion_limit = "256"]

mod telemetry;
mod server;
mod shift;

use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex};
use tauri::{Manager, WebviewWindowBuilder};
use serde::{Deserialize, Serialize};

pub struct PlaybackState {
    pub data: Mutex<Vec<Vec<u8>>>,
    pub idx: Mutex<usize>,
    pub total: Mutex<usize>,
}

#[derive(Serialize, Deserialize, Default)]
struct HudConfig { x: Option<f64>, y: Option<f64>, w: Option<f64>, h: Option<f64>, edit_mode: Option<bool> }

fn load_hud_config() -> HudConfig {
    std::fs::read_to_string(server::resolve_path("hud-config.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub struct HudState {
    pub window: Mutex<Option<tauri::WebviewWindow>>,
    pub cmd_rx: Mutex<std::sync::mpsc::Receiver<String>>,
    pub edit_mode: Arc<AtomicBool>,
    pub recording: Arc<AtomicBool>,
    pub playing: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub pending_rec: Arc<AtomicBool>,
    pub trim_secs: Mutex<u64>,
    pub rec_path: Mutex<String>,
    pub pb_data: Arc<Mutex<Vec<Vec<u8>>>>,
    pub pb_idx: Arc<Mutex<usize>>,
    pub pb_total: Arc<Mutex<usize>>,
    pub car_curves_save: Arc<Mutex<serde_json::Value>>,
    pub pb_sender: std::sync::mpsc::Sender<Vec<u8>>,
}

// ── Tauri commands ──

#[derive(Serialize)]
struct RecFile { name: String, path: String, size: u64, pkts: usize }

#[tauri::command]
fn list_recordings() -> Result<Vec<RecFile>, String> {
    let dir = std::fs::read_dir(server::resolve_path("tools")).map_err(|e| e.to_string())?;
    let mut files: Vec<RecFile> = dir.filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".bin"))
        .filter_map(|e| {
            let path = e.path();
            let meta = e.metadata().ok()?;
            let name = path.file_stem()?.to_string_lossy().to_string();
            let size = meta.len();
            Some(RecFile { name, path: path.to_string_lossy().to_string(), size, pkts: size as usize / 324 })
        })
        .collect();
    files.sort_by_key(|f| f.name.clone()); files.reverse();
    Ok(files)
}

#[tauri::command]
fn delete_recording(path: String) -> Result<(), String> {
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn playback_file(state: tauri::State<HudState>, path: String) -> Result<(), String> {
    if state.playing.load(Ordering::SeqCst) { let _ = stop_playback(state.clone()); }
    let raw = std::fs::read(&path).map_err(|e| e.to_string())?;
    send_playback(&state, raw);
    state.playing.store(true, Ordering::SeqCst);
    state.paused.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn start_hud(app: tauri::AppHandle, state: tauri::State<HudState>) -> Result<(), String> {
    let mut hud = state.window.lock().map_err(|e| e.to_string())?;
    if hud.is_some() { return Ok(()); }
    let cfg = load_hud_config();
    let mut builder = WebviewWindowBuilder::new(&app, "hud", tauri::WebviewUrl::App("hud.html".into()))
        .title("FH6 HUD").inner_size(cfg.w.unwrap_or(800.0), cfg.h.unwrap_or(150.0))
        .transparent(true).decorations(false).always_on_top(true).skip_taskbar(true)
        .resizable(false).shadow(false).visible(true);
    if let (Some(x), Some(y)) = (cfg.x, cfg.y) { builder = builder.position(x, y); }
    let window = builder.build().map_err(|e| format!("build: {e}"))?;
    window.set_ignore_cursor_events(true).map_err(|e| format!("cursor: {e}"))?;
    if cfg.edit_mode.unwrap_or(false) {
        state.edit_mode.store(true, Ordering::SeqCst);
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_resizable(true);
        let _ = window.set_skip_taskbar(true);
        let _ = window.eval("document.body.classList.add('edit-mode')");
    }
    *hud = Some(window);
    println!("[FH6] HUD opened OK");
    Ok(())
}

fn save_hud_cfg(handle: &tauri::AppHandle) {
    if let Some(w) = handle.get_webview_window("hud") {
        if let (Ok(b), Ok(s)) = (w.outer_position(), w.inner_size()) {
            let scale = w.scale_factor().unwrap_or(1.0);
            let logical_size = s.to_logical(scale);
            let logical_pos = b.to_logical(scale);
            let cfg = HudConfig {
                x: Some(logical_pos.x), y: Some(logical_pos.y),
                w: Some(logical_size.width), h: Some(logical_size.height),
                edit_mode: Some(handle.state::<HudState>().edit_mode.load(Ordering::SeqCst)),
            };
            let _ = std::fs::write(server::resolve_path("hud-config.json"), serde_json::to_string(&cfg).unwrap_or_default());
            let _ = w.close();
        }
    }
    // Save car curves
    let state = handle.state::<HudState>();
    let curves = state.car_curves_save.lock().unwrap();
    if curves.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
        let _ = std::fs::write(server::resolve_path("car_curves.json"), serde_json::to_string(&*curves).unwrap_or_default());
    }
}

#[tauri::command]
fn stop_hud(app: tauri::AppHandle, state: tauri::State<HudState>) -> Result<(), String> {
    save_hud_cfg(&app);
    state.window.lock().map_err(|e| e.to_string())?.take();
    state.edit_mode.store(false, Ordering::SeqCst);
    println!("[FH6] HUD closed OK");
    Ok(())
}

#[tauri::command]
fn hud_edit_mode(state: tauri::State<HudState>) -> Result<bool, String> {
    let hud = state.window.lock().map_err(|e| e.to_string())?;
    if let Some(ref w) = *hud {
        let on = !state.edit_mode.load(Ordering::SeqCst);
        state.edit_mode.store(on, Ordering::SeqCst);
        w.set_ignore_cursor_events(!on).map_err(|e| e.to_string())?;
        w.set_resizable(on).map_err(|e| e.to_string())?;
        w.set_skip_taskbar(on).map_err(|e| e.to_string())?;
        let _ = w.eval(&format!("document.body.classList.toggle('edit-mode',{on})"));
        if on { let _ = w.set_focus(); }
        println!("[FH6] edit mode: {on}");
        Ok(on)
    } else { Ok(false) }
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) -> Result<(), String> {
    save_hud_cfg(&app);
    std::process::exit(0);
}

#[tauri::command]
fn reset_hud(state: tauri::State<HudState>) -> Result<(), String> {
    let _ = std::fs::remove_file(server::resolve_path("hud-config.json"));
    state.window.lock().map_err(|e| e.to_string())?.take().map(|w| w.close());
    state.edit_mode.store(false, Ordering::SeqCst);
    println!("[FH6] HUD reset");
    Ok(())
}

// ── Recording / Playback ──

#[tauri::command]
fn start_record(state: tauri::State<HudState>) -> Result<String, String> {
    if state.recording.load(Ordering::SeqCst) { return Err("Already recording".into()); }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let path = server::resolve_path(&format!("tools/rec_{}.bin", secs));
    state.recording.store(true, Ordering::SeqCst);
    *state.rec_path.lock().map_err(|e| e.to_string())? = path.clone();
    println!("[FH6] Recording → {path}");
    Ok(path)
}

fn start_record_delayed(state: tauri::State<HudState>, delay_secs: u64, wait_data: bool, duration_secs: u64, trim_secs: u64) {
    if state.recording.load(Ordering::SeqCst) || state.pending_rec.load(Ordering::SeqCst) { return; }
    state.pending_rec.store(true, Ordering::SeqCst);
    let path = {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        server::resolve_path(&format!("tools/rec_{}.bin", secs))
    };
    *state.rec_path.lock().unwrap() = path.clone();
    let rec = state.recording.clone();
    let pend = state.pending_rec.clone();
    println!("[FH6] Rec pending: delay={delay_secs}s wait_data={wait_data} dur={duration_secs}s trim={trim_secs}s → {path}");

    std::thread::spawn(move || {
        if delay_secs > 0 { std::thread::sleep(std::time::Duration::from_secs(delay_secs)); }
        pend.store(false, Ordering::SeqCst);
        rec.store(true, Ordering::SeqCst);
        println!("[FH6] Recording → {path}");

        if duration_secs > 0 {
            std::thread::sleep(std::time::Duration::from_secs(duration_secs));
            rec.store(false, Ordering::SeqCst);
            println!("[FH6] Recording done ({duration_secs}s)");
        }
    });
}

fn trim_recording(path: &str, trim_secs: u64) {
    if trim_secs == 0 { return; }
    std::thread::sleep(std::time::Duration::from_millis(200));
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
        if let Ok(meta) = file.metadata() {
            let trim_bytes = trim_secs * 60 * 324;
            let new_len = meta.len().saturating_sub(trim_bytes as u64);
            let _ = file.set_len(new_len);
            println!("[FH6] Trimmed {trim_secs}s → {} pkts", new_len / 324);
        }
    }
}

#[tauri::command]
fn stop_record(state: tauri::State<HudState>) -> Result<(), String> {
    state.recording.store(false, Ordering::SeqCst);
    state.pending_rec.store(false, Ordering::SeqCst);
    // Trim tail from free recording
    let path = state.rec_path.lock().map_err(|e| e.to_string())?.clone();
    let trim = *state.trim_secs.lock().map_err(|e| e.to_string())?;
    if trim > 0 { trim_recording(&path, trim); }
    println!("[FH6] Recording stopped");
    Ok(())
}

fn send_playback(state: &HudState, raw: Vec<u8>) {
    let total = raw.len() / 324;
    let mut msg = format!("PLAY:{total}:").into_bytes();
    msg.extend_from_slice(&raw);
    match state.pb_sender.send(msg) {
        Ok(_) => println!("[FH6] Playback SENT via channel ({total} pkts)"),
        Err(e) => eprintln!("[FH6] Playback channel send FAILED: {e}"),
    }
}

#[tauri::command]
fn start_playback(state: tauri::State<HudState>) -> Result<(), String> {
    if state.playing.load(Ordering::SeqCst) { return Err("Already playing".into()); }
    let dir = std::fs::read_dir(server::resolve_path("tools")).map_err(|e| e.to_string())?;
    let mut files: Vec<_> = dir.filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("rec_") && e.file_name().to_string_lossy().ends_with(".bin"))
        .collect();
    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));
    if let Some(f) = files.last() {
        let path = f.path();
        let raw = std::fs::read(&path).map_err(|e| e.to_string())?;
        send_playback(&state, raw);
        state.playing.store(true, Ordering::SeqCst);
        state.paused.store(false, Ordering::SeqCst);
        Ok(())
    } else {
        Err("No recordings found. Record first.".into())
    }
}

#[tauri::command]
fn stop_playback(state: tauri::State<HudState>) -> Result<(), String> {
    state.playing.store(false, Ordering::SeqCst);
    state.paused.store(false, Ordering::SeqCst);
    state.pb_data.lock().map_err(|e| e.to_string())?.clear();
    let _ = state.pb_sender.send(b"STOP".to_vec());
    println!("[FH6] Playback stopped");
    Ok(())
}

#[tauri::command]
fn pause_playback(state: tauri::State<HudState>) -> Result<(), String> {
    state.paused.store(true, Ordering::SeqCst);
    let _ = state.pb_sender.send(b"PAUSE".to_vec());
    println!("[FH6] Playback paused");
    Ok(())
}

#[tauri::command]
fn resume_playback(state: tauri::State<HudState>) -> Result<(), String> {
    state.paused.store(false, Ordering::SeqCst);
    let _ = state.pb_sender.send(b"RESUME".to_vec());
    println!("[FH6] Playback resumed");
    Ok(())
}

// ── Entry ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let edit_mode = Arc::new(AtomicBool::new(false));
    let recording = Arc::new(AtomicBool::new(false));
    let playing = Arc::new(AtomicBool::new(false));
    let paused = Arc::new(AtomicBool::new(false));
    let pending_rec = Arc::new(AtomicBool::new(false));
    let pb_data = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
    let pb_idx = Arc::new(Mutex::new(0usize));
    let pb_total = Arc::new(Mutex::new(0usize));
    let car_curves_save = Arc::new(Mutex::new(serde_json::json!({})));
    let (pb_sender, pb_receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    println!("[FH6] PB channel created");
    // Test: send a message immediately to verify channel direction
    pb_sender.send(b"TEST".to_vec()).ok();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<String>();

    server::start_server(cmd_tx, edit_mode.clone(), recording.clone(), playing.clone(), paused.clone(),
        pb_data.clone(), pb_idx.clone(), pb_total.clone(), car_curves_save.clone(), pb_receiver);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(HudState {
            window: Mutex::new(None), cmd_rx: Mutex::new(cmd_rx),
            edit_mode, recording: recording.clone(), playing: playing.clone(),
            paused: paused.clone(), pending_rec: pending_rec.clone(), trim_secs: Mutex::new(0), rec_path: Mutex::new(String::new()),
            pb_data: pb_data.clone(), pb_idx: pb_idx.clone(), pb_total: pb_total.clone(),
            car_curves_save: car_curves_save.clone(),
            pb_sender: pb_sender.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            start_hud, stop_hud, hud_edit_mode, reset_hud,
            start_record, stop_record, start_playback, stop_playback, pause_playback, resume_playback,
            list_recordings, delete_recording, playback_file, restart_app,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let handle2 = handle.clone();
            if let Some(main_win) = app.get_webview_window("main") {
                main_win.on_window_event(move |ev| {
                    if let tauri::WindowEvent::Destroyed = ev { save_hud_cfg(&handle); }
                });
            }
            std::thread::spawn(move || {
                let state: tauri::State<HudState> = handle2.state();
                loop {
                    if let Ok(cmd) = state.cmd_rx.lock().unwrap().recv() {
                        match cmd.as_str() {
                            "start_hud" => { let _ = start_hud(handle2.clone(), state.clone()); }
                            "stop_hud" => { let _ = stop_hud(handle2.clone(), state.clone()); }
                            "hud_edit_mode" => { let _ = hud_edit_mode(state.clone()); }
                            "reset_hud" => { let _ = reset_hud(state.clone()); }
                            "restart_app" => { let _ = restart_app(handle2.clone()); }
                            "start_record" => { let _ = start_record(state.clone()); }
                            "stop_record" => { let _ = stop_record(state.clone()); }
                            "start_playback" => { let _ = start_playback(state.clone()); }
                            "stop_playback" => { let _ = stop_playback(state.clone()); }
                            "pause_playback" => { let _ = pause_playback(state.clone()); }
                            "resume_playback" => { let _ = resume_playback(state.clone()); }
                            "list_recordings" => {}
                            other if other.starts_with("{") => {
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(other) {
                                    let act = v["action"].as_str().unwrap_or("");
                                    match act {
                                        "start_record" => {
                                            let d = v["delay"].as_u64().unwrap_or(0);
                                            let w = v["wait"].as_bool().unwrap_or(false);
                                            let dur = v["duration"].as_u64().unwrap_or(0);
                                            let t = v["trim"].as_u64().unwrap_or(0);
                                            *state.trim_secs.lock().unwrap() = t;
                                            start_record_delayed(state.clone(), d, w, dur, t);
                                        }
                                        "stop_record" => { let _ = stop_record(state.clone()); }
                                        "start_playback" => { let _ = start_playback(state.clone()); }
                                        "stop_playback" => { let _ = stop_playback(state.clone()); }
                                        "pause_playback" => { let _ = pause_playback(state.clone()); }
                                        "resume_playback" => { let _ = resume_playback(state.clone()); }
                                        "start_hud" => { let _ = start_hud(handle2.clone(), state.clone()); }
                                        "stop_hud" => { let _ = stop_hud(handle2.clone(), state.clone()); }
                                        "hud_edit_mode" => { let _ = hud_edit_mode(state.clone()); }
                                        "reset_hud" => { let _ = reset_hud(state.clone()); }
                                        "restart_app" => { let _ = restart_app(handle2.clone()); }
                                        "list_recordings" => {
                                            if let Ok(files) = list_recordings() {
                                                if let Some(w) = handle2.get_webview_window("main") {
                                                    let json = serde_json::to_string(&files).unwrap_or_default();
                                                    let _ = w.eval(&format!("window.dispatchEvent(new CustomEvent('rec-files',{{detail:{json}}}))"));
                                                }
                                            }
                                        }
                                        "delete_recording" => {
                                            if let Some(p) = v["path"].as_str() {
                                                let _ = delete_recording(p.to_string());
                                            }
                                        }
                                        "playback_file" => {
                                            if let Some(p) = v["path"].as_str() {
                                                let _ = playback_file(state.clone(), p.to_string());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
