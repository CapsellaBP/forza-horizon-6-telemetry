"""Capture raw UDP packets from FH6 and save to binary file for Rust testing.
Usage: python tools/capture_raw.py [duration_seconds] [output_prefix]
Default: 10 seconds → tools/cap_20260702_xxxxxx.bin
"""
import socket
import os
import time
import struct
import json
from datetime import datetime

UDP_IP = "127.0.0.1"
UDP_PORT = 5300
SLED_FORMAT = "<iI15f12f4i4f4f4f4f4f5i"
SLED_NAMES = [
    "is_race_on","timestamp_ms","engine_max_rpm","engine_idle_rpm","current_engine_rpm",
    "accel_x","accel_y","accel_z","vel_x","vel_y","vel_z",
    "ang_vel_x","ang_vel_y","ang_vel_z","yaw","pitch","roll",
    "susp_travel_fl","susp_travel_fr","susp_travel_rl","susp_travel_rr",
    "tire_slip_ratio_fl","tire_slip_ratio_fr","tire_slip_ratio_rl","tire_slip_ratio_rr",
    "wheel_rot_speed_fl","wheel_rot_speed_fr","wheel_rot_speed_rl","wheel_rot_speed_rr",
    "rumble_fl","rumble_fr","rumble_rl","rumble_rr",
    "puddle_fl","puddle_fr","puddle_rl","puddle_rr",
    "surface_rumble_fl","surface_rumble_fr","surface_rumble_rl","surface_rumble_rr",
    "tire_slip_angle_fl","tire_slip_angle_fr","tire_slip_angle_rl","tire_slip_angle_rr",
    "tire_combined_slip_fl","tire_combined_slip_fr","tire_combined_slip_rl","tire_combined_slip_rr",
    "susp_travel_m_fl","susp_travel_m_fr","susp_travel_m_rl","susp_travel_m_rr",
    "car_ordinal","car_class","car_perf_index","drivetrain_type","num_cylinders",
]
DASH_FORMAT = "<3f3f4f2f5fH6B b2B"
DASH_NAMES = [
    "pos_x","pos_y","pos_z","speed","power_w","torque_nm",
    "tire_temp_fl","tire_temp_fr","tire_temp_rl","tire_temp_rr",
    "boost_psi","fuel","distance_m","best_lap","last_lap","current_lap","race_time",
    "lap","race_pos","accel","brake","clutch","handbrake","gear",
    "steer","normal_driving_line","normal_ai_brake_diff",
]

def parse_one(data: bytes) -> dict:
    pkt = {}
    if len(data) >= 232:
        for n, v in zip(SLED_NAMES, struct.unpack_from(SLED_FORMAT, data, 0)):
            pkt[n] = v
    if len(data) >= 323:
        for n, v in zip(DASH_NAMES, struct.unpack_from(DASH_FORMAT, data, 244)):
            pkt[n] = v
    return pkt

def main():
    import sys
    duration = float(sys.argv[1]) if len(sys.argv) > 1 else 10.0
    prefix = sys.argv[2] if len(sys.argv) > 2 else None

    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    prefix = prefix or f"cap_{ts}"
    bin_path = os.path.join(os.path.dirname(__file__), prefix + ".bin")
    json_path = os.path.join(os.path.dirname(__file__), prefix + ".json")

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((UDP_IP, UDP_PORT))
    sock.settimeout(0.1)

    print(f"FH6 capture → {duration}s after detecting valid data.")
    print("  Start driving (enter a car, throttle on)...")
    print(f"  Binary: {bin_path}")
    print(f"  JSON:   {json_path}")

    packets_bin = []
    packets_json = []
    t0 = None  # starts when we first see valid data
    wait_ticks = 0

    try:
        while True:
            try:
                data, _ = sock.recvfrom(4096)
            except socket.timeout:
                if t0 is not None and time.time() - t0 >= duration:
                    break
                wait_ticks += 1
                if wait_ticks % 20 == 0:
                    print("  Waiting for car data... (enter a car and drive)")
                continue

            pkt = parse_one(data)
            speed = pkt.get("speed", 0) or 0
            car_id = pkt.get("car_ordinal", 0) or 0

            # Wait for valid data before starting timer
            if t0 is None:
                if car_id > 0 and speed > 0.5:
                    t0 = time.time()
                    print(f"  Car {car_id} detected! Capturing {duration}s...")
                    packets_bin.append(data[:324])
                    packets_json.append(pkt)
                continue

            packets_bin.append(data[:324])
            packets_json.append(pkt)

            if time.time() - t0 >= duration:
                break
    except KeyboardInterrupt:
        pass
    finally:
        sock.close()

    # Save binary (concatenated 324-byte packets)
    with open(bin_path, "wb") as f:
        for p in packets_bin:
            f.write(p)

    # Save JSON (first packet only + summary) — keep it small
    summary = {
        "total_packets": len(packets_bin),
        "duration_s": time.time() - t0,
        "first": packets_json[0] if packets_json else None,
        "last": packets_json[-1] if packets_json else None,
        "all": [p for p in packets_json],  # keep all for diff testing
    }
    with open(json_path, "w") as f:
        json.dump(summary, f, indent=2, default=str)

    print(f"Done: {len(packets_bin)} packets in {time.time() - t0:.1f}s "
          f"({len(packets_bin)/(time.time()-t0):.0f} Hz)")
    if packets_json:
        p = packets_json[0]
        print(f"  Car ID: {p.get('car_ordinal')}  PI: {p.get('car_perf_index')}")
        print(f"  Max RPM: {p.get('engine_max_rpm')}  Drivetrain: {p.get('drivetrain_type')}")

if __name__ == "__main__":
    main()
