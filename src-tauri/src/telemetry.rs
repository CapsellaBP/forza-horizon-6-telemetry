// FH6 UDP telemetry packet parsing (324 bytes, little-endian)
//
// Sled segment:  232 bytes @ offset 0
// Dash segment:   79 bytes @ offset 244
// Total:         324 bytes

use std::collections::HashMap;

pub type TelemetryPacket = HashMap<String, f64>;

#[inline]
fn i32le(data: &[u8], pos: usize) -> i32 {
    i32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]])
}
#[inline]
fn u32le(data: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]])
}
#[inline]
fn f32le(data: &[u8], pos: usize) -> f32 {
    f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]])
}

pub fn parse(data: &[u8]) -> Option<TelemetryPacket> {
    if data.len() < 244 { return None; }
    let mut pkt = TelemetryPacket::new();
    let mut p = 0usize;

    // ── Sled: 232 bytes @ offset 0 ──
    // <i I 15f 12f 4i 4f 4f 4f 4f 4f 5i
    pkt.insert("is_race_on".into(), i32le(data, p) as f64); p += 4;
    pkt.insert("timestamp_ms".into(), u32le(data, p) as f64); p += 4;

    pkt.insert("engine_max_rpm".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("engine_idle_rpm".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("current_engine_rpm".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("accel_x".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("accel_y".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("accel_z".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("vel_x".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("vel_y".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("vel_z".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("ang_vel_x".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("ang_vel_y".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("ang_vel_z".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("yaw".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("pitch".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("roll".into(), f32le(data, p) as f64); p += 4;

    for s in &["susp_travel_fl","susp_travel_fr","susp_travel_rl","susp_travel_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["tire_slip_ratio_fl","tire_slip_ratio_fr","tire_slip_ratio_rl","tire_slip_ratio_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["wheel_rot_speed_fl","wheel_rot_speed_fr","wheel_rot_speed_rl","wheel_rot_speed_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["rumble_fl","rumble_fr","rumble_rl","rumble_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["puddle_fl","puddle_fr","puddle_rl","puddle_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["surface_rumble_fl","surface_rumble_fr","surface_rumble_rl","surface_rumble_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["tire_slip_angle_fl","tire_slip_angle_fr","tire_slip_angle_rl","tire_slip_angle_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["tire_combined_slip_fl","tire_combined_slip_fr","tire_combined_slip_rl","tire_combined_slip_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    for s in &["susp_travel_m_fl","susp_travel_m_fr","susp_travel_m_rl","susp_travel_m_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }

    pkt.insert("car_ordinal".into(), i32le(data, p) as f64); p += 4;
    pkt.insert("car_class".into(), i32le(data, p) as f64); p += 4;
    pkt.insert("car_perf_index".into(), i32le(data, p) as f64); p += 4;
    pkt.insert("drivetrain_type".into(), i32le(data, p) as f64); p += 4;
    pkt.insert("num_cylinders".into(), i32le(data, p) as f64);
    // sled total = 4+4 + 15×4 + 12×4 + 4×4 + 4×4 + 4×4 + 4×4 + 4×4 + 4×4 + 5×4 = 232 ✓

    // ── Dash: 79 bytes @ offset 244 ──
    if data.len() < 323 { return Some(pkt); }
    p = 244;

    pkt.insert("pos_x".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("pos_y".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("pos_z".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("speed".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("power_w".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("torque_nm".into(), f32le(data, p) as f64); p += 4;
    for s in &["tire_temp_fl","tire_temp_fr","tire_temp_rl","tire_temp_rr"] { pkt.insert(s.to_string(), f32le(data,p) as f64); p += 4; }
    pkt.insert("boost_psi".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("fuel".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("distance_m".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("best_lap".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("last_lap".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("current_lap".into(), f32le(data, p) as f64); p += 4;
    pkt.insert("race_time".into(), f32le(data, p) as f64); p += 4;

    pkt.insert("lap".into(), u16::from_le_bytes([data[p], data[p+1]]) as f64); p += 2;
    pkt.insert("race_pos".into(), data[p] as f64); p += 1;
    pkt.insert("accel".into(), data[p] as f64); p += 1;
    pkt.insert("brake".into(), data[p] as f64); p += 1;
    pkt.insert("clutch".into(), data[p] as f64); p += 1;
    pkt.insert("handbrake".into(), data[p] as f64); p += 1;
    pkt.insert("gear".into(), data[p] as f64); p += 1;
    pkt.insert("steer".into(), i8::from_le_bytes([data[p]]) as f64); p += 1;
    pkt.insert("normal_driving_line".into(), data[p] as f64); p += 1;
    pkt.insert("normal_ai_brake_diff".into(), data[p] as f64); // p += 1
    // dash total = 3×4 + 3×4 + 4×4 + 2×4 + 5×4 + 2 + 6×1 + 1 + 2×1 = 79 ✓

    Some(pkt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(parse(&[]).is_none());
        assert!(parse(&[0u8; 200]).is_none());
    }

    #[test]
    fn test_parse_sled() {
        let mut buf = vec![0u8; 244];
        buf[0..4].copy_from_slice(&1i32.to_le_bytes());
        buf[4..8].copy_from_slice(&12345u32.to_le_bytes());
        buf[8..12].copy_from_slice(&8000f32.to_le_bytes());
        buf[12..16].copy_from_slice(&1500f32.to_le_bytes());
        buf[16..20].copy_from_slice(&4500f32.to_le_bytes());
        buf[212..216].copy_from_slice(&1234i32.to_le_bytes());
        buf[216..220].copy_from_slice(&5i32.to_le_bytes());
        buf[220..224].copy_from_slice(&900i32.to_le_bytes());
        buf[224..228].copy_from_slice(&2i32.to_le_bytes());
        buf[228..232].copy_from_slice(&8i32.to_le_bytes());

        let pkt = parse(&buf).unwrap();
        assert_eq!(pkt["is_race_on"], 1.0);
        assert_eq!(pkt["timestamp_ms"], 12345.0);
        assert_eq!(pkt["engine_max_rpm"], 8000.0);
        assert_eq!(pkt["engine_idle_rpm"], 1500.0);
        assert_eq!(pkt["current_engine_rpm"], 4500.0);
        assert_eq!(pkt["car_ordinal"], 1234.0);
        assert_eq!(pkt["car_class"], 5.0);
        assert_eq!(pkt["car_perf_index"], 900.0);
        assert_eq!(pkt["drivetrain_type"], 2.0);
        assert_eq!(pkt["num_cylinders"], 8.0);
    }

    #[test]
    fn test_parse_full() {
        let mut buf = vec![0u8; 324];
        // Set a few dash fields
        buf[256..260].copy_from_slice(&30.0f32.to_le_bytes()); // speed = 30 m/s
        buf[260..264].copy_from_slice(&250000f32.to_le_bytes()); // power_w
        buf[264..268].copy_from_slice(&400f32.to_le_bytes()); // torque_nm
        buf[319] = 3; // gear = 3 (offset: 244+12+12+16+8+20+2+5 = 319)

        let pkt = parse(&buf).unwrap();
        assert_eq!(pkt["speed"], 30.0);
        assert_eq!(pkt["power_w"], 250000.0);
        assert_eq!(pkt["torque_nm"], 400.0);
        assert_eq!(pkt["gear"], 3.0);
    }

    /// Parse real captured data if available (run: `python tools/capture_raw.py` first)
    #[test]
    fn test_parse_captured() {
        let path = std::path::Path::new("../tools/sample.bin");
        // Also check with timestamp prefix
        let candidates = if path.exists() {
            vec![path.to_path_buf()]
        } else {
            // Search for cap_*.bin files
            let dir = std::path::Path::new("../tools");
            std::fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_name().to_string_lossy().starts_with("cap_") && e.file_name().to_string_lossy().ends_with(".bin"))
                        .map(|e| e.path())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let sample = candidates.first();
        if sample.is_none() {
            eprintln!("Skipping: no captured data. Run `python tools/capture_raw.py` while FH6 is running.");
            return;
        }

        let data = std::fs::read(sample.unwrap()).unwrap();
        assert!(data.len() >= 324, "File too small");

        let n = data.len() / 324;
        let mut errors = 0u32;
        for i in 0..n {
            let start = i * 324;
            let slice = &data[start..start + 324];
            match parse(slice) {
                Some(pkt) => {
                    // Sanity checks on real data
                    let rpm = pkt.get("current_engine_rpm").copied().unwrap_or(0.0);
                    let rpm_max = pkt.get("engine_max_rpm").copied().unwrap_or(1.0);
                    if rpm_max > 0.0 {
                        assert!(rpm >= 0.0, "Negative RPM at packet {}", i);
                        assert!(rpm <= rpm_max * 1.1, "RPM {} > max {} * 1.1 at packet {}", rpm, rpm_max, i);
                    }
                    let gear = pkt.get("gear").copied().unwrap_or(0.0);
                    assert!(gear <= 15.0, "Gear too high: {} at packet {}", gear, i);
                }
                None => errors += 1,
            }
        }
        assert_eq!(errors, 0, "{} packets failed to parse", errors);
        eprintln!("Parsed {n} real packets OK");
    }
}
