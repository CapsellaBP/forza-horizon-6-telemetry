// FH6 Shift Advisor — EMA power curve sampling + fuel cut detection + EV detection
use std::collections::HashMap;

const RPM_BIN_SIZE: i32 = 100;

pub struct ShiftAdvisor {
    bins: HashMap<i32, (u32, f64, f64)>, // rpm_bin → (count, ema_torque, ema_power)
    pub(crate) sample_count: u32,
    pub(crate) ready: bool,
    peak_power_rpm: f64,
    pub(crate) locked: bool,

    // Settings (pub for server access)
    pub(crate) aggressiveness_pct: i32,
    pub(crate) limiter_threshold: f64,
    pub(crate) shift_trigger: f64,
    pub(crate) alpha: f64,
    pub(crate) throttle_min: f64,
    pub(crate) skip_frames: u32,
    pub(crate) power_drop_limit: f64,
    pub(crate) boost_stable_sample: bool,
    boost_ema: f64,
    boost_stable: bool,

    // Per-frame state
    frames_since_shift: u32,
    last_gear_sample: Option<i32>,
    last_rpm_sample: f64,
    pub(crate) idle_rpm: f64,
    pub(crate) rpm_max: f64,
    last_urgency: String,
    hysteresis_timer: u32,
    last_gear_adv: Option<i32>,

    // Fuel cut detection
    fuel_cut_rpm: f64,
    fuel_cut_lo: f64,
    fc_peak: f64,
    fc_peaks: Vec<f64>,
    fc_valleys: Vec<f64>,
    last_rpm_tick: f64,
    last_throttle: f64,
    last_gear_fc: Option<i32>,
    fc_rising: bool,
}

impl Default for ShiftAdvisor {
    fn default() -> Self {
        Self {
            bins: HashMap::new(), sample_count: 0, ready: false, peak_power_rpm: 0.0, locked: false,
            aggressiveness_pct: 0, limiter_threshold: 1.0, shift_trigger: 1.0,
            alpha: 0.25, throttle_min: 0.95, skip_frames: 8, power_drop_limit: 0.0,
            boost_stable_sample: false, boost_ema: 0.0, boost_stable: false,
            frames_since_shift: 999, last_gear_sample: None, last_rpm_sample: 0.0,
            idle_rpm: 1500.0, rpm_max: 8000.0, last_urgency: "hold".into(),
            hysteresis_timer: 0, last_gear_adv: None,
            fuel_cut_rpm: 0.0, fuel_cut_lo: 0.0, fc_peak: 0.0,
            fc_peaks: Vec::new(), fc_valleys: Vec::new(),
            last_rpm_tick: 0.0, last_throttle: 0.0, last_gear_fc: None, fc_rising: true,
        }
    }
}

fn bin(rpm: f64) -> i32 { (rpm as i32 / RPM_BIN_SIZE) * RPM_BIN_SIZE }

impl ShiftAdvisor {
    pub fn new() -> Self { Self::default() }

    pub fn reset(&mut self) {
        self.bins.clear(); self.sample_count = 0; self.peak_power_rpm = 0.0; self.ready = false;
        self.fuel_cut_rpm = 0.0; self.fuel_cut_lo = 0.0; self.fc_peaks.clear(); self.fc_valleys.clear();
        self.fc_peak = 0.0;
    }

    pub fn to_dict(&self) -> serde_json::Value {
        let mut bins_map = serde_json::Map::new();
        for (k, v) in &self.bins {
            bins_map.insert(k.to_string(), serde_json::json!([v.0, v.1, v.2]));
        }
        serde_json::json!({
            "bins": bins_map, "sample_count": self.sample_count,
            "peak_power_rpm": self.peak_power_rpm,
            "fuel_cut_rpm": self.fuel_cut_rpm, "fuel_cut_lo": self.fuel_cut_lo,
            "idle_rpm": self.idle_rpm, "rpm_max": self.rpm_max, "ready": self.ready,
            "locked": self.locked,
        })
    }

    pub fn from_dict(d: &serde_json::Value) -> Self {
        let mut obj = Self::new();
        obj.locked = d.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
        if let Some(bins) = d.get("bins").and_then(|b| b.as_object()) {
            for (k, v) in bins {
                if let (Ok(rpm), Some(arr)) = (k.parse::<i32>(), v.as_array()) {
                    if arr.len() == 3 {
                        obj.bins.insert(rpm, (
                            arr[0].as_u64().unwrap_or(0) as u32,
                            arr[1].as_f64().unwrap_or(0.0),
                            arr[2].as_f64().unwrap_or(0.0),
                        ));
                    }
                }
            }
        }
        obj.sample_count = d.get("sample_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        obj.peak_power_rpm = d.get("peak_power_rpm").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.fuel_cut_rpm = d.get("fuel_cut_rpm").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.fuel_cut_lo = d.get("fuel_cut_lo").and_then(|v| v.as_f64()).unwrap_or(0.0);
        obj.idle_rpm = d.get("idle_rpm").and_then(|v| v.as_f64()).unwrap_or(1500.0);
        obj.rpm_max = d.get("rpm_max").and_then(|v| v.as_f64()).unwrap_or(8000.0);
        obj.ready = d.get("ready").and_then(|v| v.as_bool()).unwrap_or(false);
        obj
    }

    pub fn get_curve(&self, band_pct: f64) -> serde_json::Value {
        let mut rpm_keys: Vec<i32> = self.bins.keys().copied().collect();
        rpm_keys.sort();
        let mut torque_pts = Vec::new();
        let mut power_pts = Vec::new();
        for b in &rpm_keys {
            if let Some((cnt, tq, pw)) = self.bins.get(b) {
                if *cnt >= 3 {
                    torque_pts.push(vec![serde_json::Value::from(*b as f64), serde_json::Value::from((tq * 10.0).round() / 10.0)]);
                    power_pts.push(vec![serde_json::Value::from(*b as f64), serde_json::Value::from((pw / 1000.0 * 10.0).round() / 10.0)]);
                }
            }
        }

        let fc_limit = if self.fuel_cut_lo > 0.0 { self.fuel_cut_lo } else if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm } else { 99999.0 };
        let mut band_lo = 0.0;
        let mut band_hi = 0.0;
        if self.ready && !power_pts.is_empty() {
            let peak_kw: f64 = power_pts.iter()
                .filter(|p| p[0].as_f64().unwrap_or(0.0) < fc_limit)
                .map(|p| p[1].as_f64().unwrap_or(0.0))
                .fold(0.0, f64::max);
            let threshold = peak_kw * band_pct;
            let in_band: Vec<f64> = power_pts.iter()
                .filter(|p| p[1].as_f64().unwrap_or(0.0) >= threshold && p[0].as_f64().unwrap_or(0.0) < fc_limit)
                .map(|p| p[0].as_f64().unwrap_or(0.0))
                .collect();
            if !in_band.is_empty() {
                band_lo = in_band.iter().fold(f64::MAX, |a, &b| a.min(b));
                band_hi = in_band.iter().fold(f64::MIN, |a, &b| a.max(b));
            }
        }

        serde_json::json!({
            "torque": torque_pts, "power_kw": power_pts,
            "optimal_rpm": self.calc_optimal(),
            "peak_power_rpm": self.peak_power_rpm,
            "fuel_cut_rpm": self.fuel_cut_rpm,
            "power_band_lo": band_lo, "power_band_hi": band_hi,
            "ready": self.ready, "samples": self.sample_count,
        })
    }

    fn calc_optimal(&self) -> f64 {
        if !self.ready { return self.rpm_max * 0.90; }
        let range = self.rpm_max - self.idle_rpm;
        let offset = self.aggressiveness_pct as f64 / 100.0 * range;
        (self.idle_rpm + 500.0).max((self.rpm_max - 200.0).min(self.peak_power_rpm + offset))
    }

    pub fn feed(&mut self, rpm: f64, torque: f64, power: f64, throttle: f64, gear: i32, boost: f64) {
        if self.locked { return; }

        self.idle_rpm = self.idle_rpm.max(500.0);
        self.rpm_max = self.rpm_max.max(1000.0);

        // Boost stability check (for turbo cars)
        if self.boost_stable_sample {
            let alpha = 0.1;
            self.boost_ema = self.boost_ema * (1.0 - alpha) + boost * alpha;
            self.boost_stable = boost > 0.1 && (boost - self.boost_ema).abs() < 0.5;
        } else {
            self.boost_stable = true; // not enabled = always sample
        }

        // Fuel cut detection
        let near_limiter = rpm > self.rpm_max * 0.85;
        if near_limiter && throttle >= self.throttle_min && Some(gear) == self.last_gear_fc {
            if rpm > self.last_rpm_tick {
                self.fc_rising = true;
                self.fc_peak = rpm;
            } else if self.fc_rising && self.last_rpm_tick - rpm > 80.0 {
                self.fc_peaks.push(self.fc_peak);
                self.fc_valleys.push(rpm);
                if self.fc_peaks.len() > 30 { self.fc_peaks.remove(0); }
                if self.fc_valleys.len() > 30 { self.fc_valleys.remove(0); }
                let mut peaks = self.fc_peaks.clone(); peaks.sort_by(|a,b| a.partial_cmp(b).unwrap());
                let mut valleys = self.fc_valleys.clone(); valleys.sort_by(|a,b| a.partial_cmp(b).unwrap());
                self.fuel_cut_rpm = peaks[peaks.len()/2];
                self.fuel_cut_lo = valleys[valleys.len()/2];
                self.fc_rising = false;
            }
        }
        if throttle > 0.9 && rpm > self.fuel_cut_rpm { self.fuel_cut_rpm = rpm; }
        self.last_rpm_tick = rpm;
        self.last_throttle = throttle;
        self.last_gear_fc = Some(gear);

        // Shift skip
        if Some(gear) != self.last_gear_sample {
            self.frames_since_shift = 0;
            self.last_gear_sample = Some(gear);
        } else { self.frames_since_shift += 1; }

        let rpm_rising = rpm >= self.last_rpm_sample - 20.0;

        if rpm >= 500.0 && throttle >= self.throttle_min && rpm_rising
            && self.frames_since_shift >= self.skip_frames
            && self.boost_stable
        {
            let b = bin(rpm);
            if let Some((cnt, ema_tq, ema_pw)) = self.bins.get_mut(&b) {
                if *cnt >= 3 && *ema_pw > 0.0 && power < *ema_pw * self.power_drop_limit {
                    // Reject dirty data
                } else {
                    let a = self.alpha;
                    *ema_tq = *ema_tq * (1.0 - a) + torque * a;
                    *ema_pw = *ema_pw * (1.0 - a) + power * a;
                    *cnt += 1;
                    self.sample_count += 1;
                    if self.sample_count % 60 == 0 { self.recalc(); }
                }
            } else {
                self.bins.insert(b, (1, torque, power));
                self.sample_count += 1;
            }
            self.last_rpm_sample = rpm;
        } else {
            self.last_rpm_sample = rpm;
        }
    }

    fn recalc(&mut self) {
        if self.bins.len() < 5 { return; }
        let fc_limit = if self.fuel_cut_lo > 0.0 { self.fuel_cut_lo } else if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm } else { 99999.0 };
        let mut best_rpm = 0;
        let mut best_pw = 0.0f64;
        for (b, (cnt, _, pw)) in &self.bins {
            if *b as f64 >= fc_limit || *cnt < 3 { continue; }
            if *pw > best_pw { best_pw = *pw; best_rpm = *b; }
        }
        if best_rpm > 0 { self.peak_power_rpm = best_rpm as f64; self.ready = true; }
    }

    pub fn advice(&mut self, rpm: f64, rpm_max: f64, throttle: f64, gear: i32) -> serde_json::Value {
        if Some(gear) != self.last_gear_adv {
            self.last_urgency = "hold".into();
            self.hysteresis_timer = 0;
            self.last_gear_adv = Some(gear);
        }

        let limiter_rpm = if self.fuel_cut_rpm > 0.0 { self.fuel_cut_rpm }
            else { (rpm_max * self.limiter_threshold).round() };
        let limiter_zone = limiter_rpm > 0.0 && rpm > limiter_rpm * 0.99;

        let (optimal, pct, urgency) = if !self.ready {
            let opt = rpm_max * 0.90;
            (opt, if opt > 0.0 { rpm / opt } else { 0.0 }, "hold")
        } else {
            let opt = self.calc_optimal();
            let pct = if opt > 0.0 { rpm / opt } else { 0.0 };
            let trigger = self.shift_trigger;
            let urg = if throttle < 0.35 { "hold" }
                else if pct < trigger - 0.10 { "hold" }
                else if pct < trigger { "near" }
                else if pct < trigger + 0.10 { "shift" }
                else { "over" };
            (opt, pct, urg)
        };

        let mut urgency = urgency.to_string();
        if limiter_zone && matches!(urgency.as_str(), "shift" | "over" | "near") {
            urgency = "over".into();
        }

        // Hysteresis
        if matches!(self.last_urgency.as_str(), "over" | "shift") && urgency == "hold" {
            self.hysteresis_timer += 1;
            if self.hysteresis_timer < 15 { urgency = self.last_urgency.clone(); }
            else { self.hysteresis_timer = 0; }
        } else { self.hysteresis_timer = 0; }
        self.last_urgency = urgency.clone();

        serde_json::json!({
            "shift_rpm": (optimal as f64).round(),
            "peak_power_rpm": self.peak_power_rpm.round(),
            "limiter_rpm": limiter_rpm.round(),
            "fuel_cut_rpm": self.fuel_cut_rpm.round(),
            "urgency": urgency,
            "pct_to_shift": (pct * 1000.0).round() / 1000.0,
            "ready": self.ready,
            "samples": self.sample_count,
            "limiter": limiter_zone,
        })
    }
}

/// EV detection: if we see > 2 distinct forward gears, it's ICE, not EV
pub fn check_ev(gears_seen: &mut Vec<i32>, gear: i32, throttle: f64, check_count: &mut u32, limit: u32, is_ev: &mut bool) {
    if *is_ev && gear >= 1 { gears_seen.push(gear); }
    // If we ever see a gear > 2, it's ICE
    if *is_ev && gear > 2 { *is_ev = false; }

    if !*is_ev && *check_count < limit {
        if throttle > 0.8 {
            if gear >= 1 && !gears_seen.contains(&gear) { gears_seen.push(gear); }
            *check_count += 1;
        }
        if *check_count >= limit {
            let forward: Vec<i32> = gears_seen.iter().filter(|&&g| g >= 1).copied().collect();
            *is_ev = forward.len() <= 2;
        }
    }
}
