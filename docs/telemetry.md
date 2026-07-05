# 遥测数据格式

## UDP 数据包结构

Forza Horizon 6 使用与 FH5 相同的 324 字节格式：

| 段 | 字节偏移 | 大小 | 内容 |
|----|---------|------|------|
| Sled | 0-231 | 232B | 核心车辆物理数据 |
| Horizon Ext | 232-243 | 12B | Horizon 专属扩展 |
| Dash | 244-322 | 79B | 比赛/仪表数据 |
| Padding | 323 | 1B | 填充 |

所有值为 **little-endian**，float 为 4 字节单精度。

## Sled 段关键字段

| 偏移 | 类型 | 字段 | 说明 |
|------|------|------|------|
| 0 | s32 | is_race_on | 1=比赛中 |
| 8 | f32 | engine_max_rpm | 最大转速 |
| 12 | f32 | engine_idle_rpm | 怠速转速 |
| 16 | f32 | current_engine_rpm | 当前转速 |
| 20 | f32 | accel_x | 横向加速度 (m/s²) |
| 24 | f32 | accel_y | 垂向加速度 |
| 28 | f32 | accel_z | 纵向加速度 |
| 32-40 | f32×3 | velocity | 速度分量 (m/s) |
| 56-64 | f32×3 | yaw/pitch/roll | 姿态角 (rad) |
| 68-80 | f32×4 | susp_travel | 悬挂行程 (归一化 0~1) |
| 84-96 | f32×4 | tire_slip_ratio | 轮胎滑移率 (0=全抓地) |
| 100-112 | f32×4 | wheel_rot_speed | 轮速 (rad/s) |
| 164-176 | f32×4 | tire_slip_angle | 滑移角 |
| 180-192 | f32×4 | tire_combined_slip | 组合滑移 |
| 196-208 | f32×4 | susp_travel_meters | 悬挂行程 (米) |
| 212 | s32 | car_ordinal | 车辆唯一 ID |
| 216 | s32 | car_class | PI 等级 (0=D~7=X) |
| 220 | s32 | car_perf_index | PI 数值 |
| 224 | s32 | drivetrain_type | 0=FWD 1=RWD 2=AWD |
| 228 | s32 | num_cylinders | 气缸数 |

## Dash 段关键字段

| 偏移 | 类型 | 字段 | 说明 |
|------|------|------|------|
| 244 | f32 | position_x | 世界坐标 X |
| 248 | f32 | position_y | 世界坐标 Y |
| 252 | f32 | position_z | 世界坐标 Z |
| 256 | f32 | speed | 速度 (m/s) |
| 260 | f32 | power | 功率 (瓦特) |
| 264 | f32 | torque | 扭矩 (N·m) |
| 268-280 | f32×4 | tire_temp | 胎温 (°C) |
| 284 | f32 | boost | 涡轮增压 (psi) |
| 292 | f32 | distance | 总里程 (m) |
| 296 | f32 | best_lap | 最佳圈速 (秒) |
| 312 | u8 | race_pos | 比赛排名 |
| 313 | u8 | accel | 油门 (0-255) |
| 314 | u8 | brake | 刹车 (0-255) |
| 315 | u8 | clutch | 离合 (0-255) |
| 316 | u8 | handbrake | 手刹 (0-255) |
| 317 | u8 | gear | 档位: 0=R, 1=1挡, 2=2挡... |
| 318 | s8 | steer | 转向 (-128~127) |

## 档位编码

FH6 采用直出编码（与 FH5 不同）：
- `0` = 倒挡 (R)
- `1` = 1 挡
- `2` = 2 挡
- ...
- `10` = 10 挡 (最高)

值 >10 为偶发噪点，已被服务端过滤。

## 检测游戏格式

根据 UDP 包大小识别：
- **324 字节** → FH4 / FH5 / FH6
- **311 字节** → Forza Motorsport 7 (Dash)
- **232 字节** → FM7 (Sled only)
