<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from "vue"
import HudPreview from "../components/HudPreview.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ action: [action: string] }>()

const s = (key: string, fb: any = null) => props.telemetry?.settings?.[key] ?? fb

const throttle = computed(() => (props.telemetry?.throttle ?? 0) * 100)
const brake = computed(() => (props.telemetry?.brake ?? 0) * 100)
const gLon = computed(() => props.telemetry?.accel_lon ?? 0)
const gLat = computed(() => props.telemetry?.accel_lat ?? 0)
const tires = computed(() => props.telemetry?.tire_temp ?? [0,0,0,0])
const slipVals = computed(() => props.telemetry?.tire_slip ?? [0,0,0,0])
const susp = computed(() => props.telemetry?.susp_travel ?? [0,0,0,0])
const boost = computed(() => props.telemetry?.boost_psi ?? 0)
const speed = computed(() => props.telemetry?.speed_kmh?.toFixed(0) ?? "0")

function tempColor(t: number): string {
  if (t <= 0) return "#555"; if (t < 60) return "#4a90d9"; if (t < 80) return "#58a6ff"
  if (t < 95) return "#4ec46b"; if (t < 110) return "#d4a843"; return "#f14c4c"
}
function slipColor(s: number): string {
  const a = Math.abs(s); if (a < 0.05) return "#4ec46b"; if (a < 0.15) return "#d4a843"; return "#f14c4c"
}

const suspOrder = [2, 0, 1, 3]
const suspNames = ["RL","FL","FR","RR"]
function travelColor(v: number): string {
  const s = props.telemetry?.settings
  if (v >= (s?.susp_thr3 ?? 0.80)) return "#f14c4c"
  if (v >= (s?.susp_thr2 ?? 0.55)) return "#d4a843"
  if (v >= (s?.susp_thr1 ?? 0.30)) return "#3399ff"
  return "#4ec46b"
}

// G-G
const ggCanvas = ref<HTMLCanvasElement | null>(null)
const SZ = 250
function drawGG() {
  const c = ggCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  c.width = SZ * dpr; c.height = SZ * dpr
  c.style.width = SZ + "px"; c.style.height = SZ + "px"
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  const cx = SZ / 2, cy = SZ / 2, r = SZ / 2 - 32

  ctx.clearRect(0, 0, SZ, SZ)
  ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2)
  ctx.fillStyle = "rgba(255,255,255,0.02)"; ctx.fill()
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1; ctx.stroke()
  ctx.strokeStyle = "rgba(255,255,255,0.06)"; ctx.lineWidth = 0.5
  ctx.beginPath(); ctx.moveTo(cx - r, cy); ctx.lineTo(cx + r, cy); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(cx, cy - r); ctx.lineTo(cx, cy + r); ctx.stroke()
  for (const p of [0.5, 1.0]) { ctx.beginPath(); ctx.arc(cx, cy, r * p, 0, Math.PI * 2); ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.stroke() }

  const scale = r / 3.0
  const dx = Math.max(-3.0, Math.min(3.0, gLat.value)) * scale
  const dy = Math.max(-3.0, Math.min(3.0, -gLon.value)) * scale
  // Trail
  const tw = window as any; if (!tw._ggTrail) tw._ggTrail = []
  const t = tw._ggTrail; t.push({ x: cx+dx, y: cy+dy }); if (t.length > 12) t.shift()
  for (let i = 0; i < t.length; i++) {
    const a = (i/t.length) * 0.2
    ctx.beginPath(); ctx.arc(t[i].x, t[i].y, 2+(i/t.length)*2, 0, Math.PI*2)
    ctx.fillStyle = `rgba(212,168,67,${a})`; ctx.fill()
  }
  // Tether + point
  if (t.length > 0) {
    const l = t[t.length-1]
    ctx.strokeStyle = "rgba(212,168,67,0.25)"; ctx.lineWidth = 1
    ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(l.x, l.y); ctx.stroke()
  }
  ctx.beginPath(); ctx.arc(cx + dx, cy + dy, 5, 0, Math.PI * 2)
  ctx.fillStyle = "#d4a843"; ctx.fill()

  const fwd = gLon.value > 0 ? gLon.value.toFixed(2) : "0.00"
  const brk = gLon.value < 0 ? Math.abs(gLon.value).toFixed(2) : "0.00"
  const lg = gLat.value < 0 ? Math.abs(gLat.value).toFixed(2) : "0.00"
  const rg = gLat.value > 0 ? gLat.value.toFixed(2) : "0.00"

  ctx.fillStyle = "#8b8f93"; ctx.font = "bold 12px sans-serif"
  ctx.textAlign = "center"
  ctx.fillText(fwd, cx, 16)
  ctx.fillText(brk, cx, SZ - 6)
  ctx.textAlign = "right"
  ctx.fillText(lg, 28, cy + 5)
  ctx.textAlign = "left"
  ctx.fillText(rg, SZ - 28, cy + 5)

  const rawG = Math.sqrt(gLon.value*gLon.value + gLat.value*gLat.value)
  // Window-persistent EMA for total G
  const w = window as any; if (!w._gTotalEma) w._gTotalEma = rawG
  const alpha = props.telemetry?.settings?.g_total_ema_alpha ?? 0.05
  w._gTotalEma += (rawG - w._gTotalEma) * alpha
  const totalG = w._gTotalEma
  ctx.fillStyle = "#fff"; ctx.font = "bold 16px sans-serif"; ctx.textAlign = "right"
  ctx.fillText(totalG.toFixed(2), cx + r, cy - r + 16)
}
watch([gLon, gLat], () => nextTick(drawGG))
onMounted(() => nextTick(drawGG))

// Power
const pwCanvas = ref<HTMLCanvasElement | null>(null)
function drawPower() {
  const c = pwCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  const w = c.clientWidth, h = c.clientHeight
  c.width = w * dpr; c.height = h * dpr
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.fillStyle = "#121314"; ctx.fillRect(0, 0, w, h)
  const pad = { t: 8, r: 55, b: 18, l: 32 }, pw = w - pad.l - pad.r, ph = h - pad.t - pad.b
  const curve = props.telemetry?.curve
  const tq: [number,number][] = curve?.torque || [], pk: [number,number][] = curve?.power_kw || []
  let rLo = 0, rHi = props.telemetry?.rpm_max || 9000, tM = 500, kM = 300
  for (const [r,v] of tq) { rLo = Math.min(rLo, r); rHi = Math.max(rHi, r); tM = Math.max(tM, v) }
  for (const [,v] of pk) kM = Math.max(kM, v)
  if (rLo === 0 && pk.length === 0) { rLo = 1500; rHi = 9000; tM = 500; kM = 300 }
  const pR = (rHi - rLo) * 0.05 || 200; rLo = Math.max(0, rLo - pR); rHi += pR; tM *= 1.1; kM *= 1.1
  if (tM < 200) tM = 500; if (kM < 100) kM = 300
  const hM = kM * 1.341
  const x = (r: number) => pad.l + (r - rLo) / (rHi - rLo) * pw
  const yt = (t: number) => pad.t + ph - (t / tM) * ph
  const yp = (p: number) => pad.t + ph - (p / kM) * ph
  ctx.strokeStyle = "rgba(255,255,255,0.04)"; ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) { const gy = pad.t + ph * i / 4; ctx.beginPath(); ctx.moveTo(pad.l, gy); ctx.lineTo(w - pad.r, gy); ctx.stroke() }
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t); ctx.lineTo(pad.l, pad.t + ph); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t + ph); ctx.lineTo(w - pad.r, pad.t + ph); ctx.stroke()
  ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"; ctx.textAlign = "center"
  for (let r = Math.ceil(rLo / 1000) * 1000; r <= rHi; r += 1000) ctx.fillText((r / 1000).toFixed(0) + "k", x(r), pad.t + ph + 13)
  // Y axis labels (Nm left, hp right)
  ctx.textAlign = "right"
  ctx.fillText(tM.toFixed(0), pad.l - 4, pad.t + 8)
  ctx.fillText((tM/2).toFixed(0), pad.l - 4, pad.t + ph/2 + 4)
  ctx.fillText("0", pad.l - 4, pad.t + ph + 4)
  ctx.fillStyle = "#58a6ff"; ctx.font = "bold 8px sans-serif"; ctx.fillText("Nm", pad.l - 4, pad.t - 2)
  ctx.textAlign = "left"; ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"
  ctx.fillText(hM.toFixed(0), w - pad.r + 3, pad.t + 8)
  ctx.fillText((hM/2).toFixed(0), w - pad.r + 3, pad.t + ph/2 + 4)
  ctx.fillText("0", w - pad.r + 3, pad.t + ph + 4)
  ctx.fillStyle = "#f85149"; ctx.font = "bold 8px sans-serif"
  ctx.fillText("hp", w - pad.r + 3, pad.t - 2)

  if (tq.length > 1) { ctx.strokeStyle = "#58a6ff"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < tq.length; i++) i === 0 ? ctx.moveTo(x(tq[i][0]), yt(tq[i][1])) : ctx.lineTo(x(tq[i][0]), yt(tq[i][1])); ctx.stroke() }
  if (pk.length > 1) { ctx.setLineDash([3, 3]); ctx.strokeStyle = "#f85149"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < pk.length; i++) i === 0 ? ctx.moveTo(x(pk[i][0]), yp(pk[i][1])) : ctx.lineTo(x(pk[i][0]), yp(pk[i][1])); ctx.stroke(); ctx.setLineDash([]) }
  const sr = props.telemetry?.shift_advice?.shift_rpm
  if (sr > 0) { ctx.strokeStyle = "#d4a843"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(sr), pad.t); ctx.lineTo(x(sr), pad.t + ph); ctx.stroke() }
  const cr = props.telemetry?.rpm
  if (cr > 0) { ctx.setLineDash([2, 4]); ctx.strokeStyle = "#ccc"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(cr), pad.t); ctx.lineTo(x(cr), pad.t + ph); ctx.stroke(); ctx.setLineDash([]) }
}
watch(() => props.telemetry, () => nextTick(drawPower), { deep: true })
onMounted(() => nextTick(drawPower))
</script>

<template>
  <div class="dashboard">
    <!-- Row 1: HUD | label/brake/throttle/label | buttons -->
    <div class="top-row">
      <div class="hud-col"><HudPreview :telemetry="telemetry" /></div>

      <!-- TB: 字符/刹车/油门/字符 -->
      <div class="tb-col">
        <div class="tb-side"><span class="tb-lbl">刹车</span><span class="tb-pct">{{ brake.toFixed(0) }}%</span></div>
        <div class="tb-wrap brk"><div class="tb-fill" :style="{ height: brake + '%' }"></div></div>
        <div class="tb-wrap thr"><div class="tb-fill" :style="{ height: throttle + '%' }"></div></div>
        <div class="tb-side"><span class="tb-lbl">油门</span><span class="tb-pct">{{ throttle.toFixed(0) }}%</span></div>
      </div>

      <div class="hud-ctrl">
        <button class="btn gold" @click="emit('action','start_hud')">启动 HUD</button>
        <button class="btn red" @click="emit('action','stop_hud')">停止 HUD</button>
        <button class="btn" :class="{ 'edit-on': s('hud_edit_mode',false) }" @click="emit('action','hud_edit_mode')" @dblclick="$emit('action','reset_hud')" title="单击激活 HUD | 双击重置 HUD">{{ s('hud_edit_mode',false) ? '激活 HUD' : '激活 HUD' }}</button>
      </div>
    </div>

    <!-- Row 2: Power + Car info -->
    <div class="mid-row">
      <div class="pw-card card"><div class="card-label">功率 / 扭矩</div><canvas ref="pwCanvas" class="pw-canvas"></canvas></div>
      <div class="info-card card">
        <div class="card-label">车况</div>
        <div class="ir"><span>速度</span><span class="in">{{ speed }}</span><span>km/h</span></div>
        <div class="ir"><span>增压</span><span class="in">{{ boost.toFixed(1) }}</span><span>psi</span></div>
        <div class="ir"><span>驱动</span><span>{{ ({0:"FWD",1:"RWD",2:"AWD"}as any)[props.telemetry?.drivetrain]??"-" }}</span></div>
        <div class="ir"><span>缸数</span><span>{{ props.telemetry?.cylinders ? props.telemetry.cylinders + '缸' : '-' }}</span></div>
        <div class="ir"><span>PI</span><span>{{ props.telemetry?.car_perf_index??"-" }}</span></div>
        <div class="ir"><span>ID</span><span>{{ props.telemetry?.car_id??"-" }}</span></div>
      </div>
    </div>

    <!-- Row 3: Temp | G-G | Slip | Susp -->
    <div class="quad-row">
      <div class="tire-box">
        <div class="card-label">胎温 °C</div>
        <div class="tire-inner">
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[0]??0)/120*100,100)+'%', background: tempColor(tires[0]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">FL</span><span class="t-num" :style="{ color: tempColor(tires[0]??0) }">{{ (tires[0]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">FR</span><span class="t-num" :style="{ color: tempColor(tires[1]??0) }">{{ (tires[1]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[1]??0)/120*100,100)+'%', background: tempColor(tires[1]??0) }"></div></div>
          </div>
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[2]??0)/120*100,100)+'%', background: tempColor(tires[2]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">RL</span><span class="t-num" :style="{ color: tempColor(tires[2]??0) }">{{ (tires[2]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">RR</span><span class="t-num" :style="{ color: tempColor(tires[3]??0) }">{{ (tires[3]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[3]??0)/120*100,100)+'%', background: tempColor(tires[3]??0) }"></div></div>
          </div>
        </div>
      </div>

      <div class="gg-box card">
        <div class="card-label">G 力</div>
        <div class="gg-wrap"><canvas ref="ggCanvas"></canvas></div>
      </div>

      <div class="tire-box">
        <div class="card-label">滑移率</div>
        <div class="tire-inner">
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[0]??0)/0.5*100,100)+'%', background: slipColor(slipVals[0]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">FL</span><span class="t-num" :style="{ color: slipColor(slipVals[0]??0) }">{{ Math.abs(slipVals[0]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">FR</span><span class="t-num" :style="{ color: slipColor(slipVals[1]??0) }">{{ Math.abs(slipVals[1]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[1]??0)/0.5*100,100)+'%', background: slipColor(slipVals[1]??0) }"></div></div>
          </div>
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[2]??0)/0.5*100,100)+'%', background: slipColor(slipVals[2]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">RL</span><span class="t-num" :style="{ color: slipColor(slipVals[2]??0) }">{{ Math.abs(slipVals[2]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">RR</span><span class="t-num" :style="{ color: slipColor(slipVals[3]??0) }">{{ Math.abs(slipVals[3]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[3]??0)/0.5*100,100)+'%', background: slipColor(slipVals[3]??0) }"></div></div>
          </div>
        </div>
      </div>

      <div class="susp-box card">
        <div class="card-label">悬挂行程</div>
        <div class="susp-inner">
          <div class="susp-row">
            <div v-for="(idx, i) in suspOrder" :key="suspNames[i]"
              class="susp-cell" :class="{ rear: i === 0 || i === 3, front: i === 1 || i === 2 }">
              <div class="susp-bar-wrap">
                <div class="susp-bar" :style="{ height: ((susp[idx]??0)*100).toFixed(0)+'%', background: travelColor(susp[idx]??0) }"></div>
              </div>
              <div class="susp-pos">{{ suspNames[i] }}</div>
              <div class="susp-val">{{ (susp[idx]??0).toFixed(3) }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard { padding: 16px; display: flex; flex-direction: column; gap: 10px; }

/* ── Row 1 ── */
.top-row { display: flex; gap: 10px; align-items: center; }
.hud-col { flex: 1; max-width: 500px; }
.hud-col > * { width: 100%; }

/* TB: char/brake/throttle/char — all in one row, same height as HUD ~140px */
.tb-col { display: flex; gap: 6px; flex-shrink: 0; height: 110px; align-items: stretch; }
.tb-side { display: flex; flex-direction: column; align-items: center; justify-content: center; width: 22px; gap: 2px; }
.tb-lbl { font-size: 11px; color: var(--dim); }
.tb-pct { font-size: 13px; color: var(--text); font-weight: 600; }
.tb-wrap { width: 45px; background: rgba(255,255,255,0.05); border-radius: 11px; overflow: hidden; display: flex; align-items: flex-end; }
.tb-fill { width: 100%; background: #fff; border-radius: 0; transition: height 0.15s; }

.hud-ctrl { display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; width: 72px; }
.btn { padding: 5px 8px; border: 1px solid var(--border); border-radius: 4px; background: var(--card); color: var(--text); font-size: 11px; cursor: pointer; text-align: center; }
.btn:hover { background: #222; }
.btn.gold { background: var(--accent); color: #121314; border-color: var(--accent); font-weight: 600; }
.btn.gold:hover { background: #e0b850; }
.btn.red { color: var(--red); border-color: var(--red); }
.btn.red:hover { background: rgba(241,76,76,0.12); }
.btn.edit-on { border: 2px dashed rgba(255,204,0,0.55); color: #ffcc00; }

/* ── Row 2 ── */
.mid-row { display: flex; gap: 10px; height: 300px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 10px; display: flex; flex-direction: column; }
.card-label { font-size: 10px; color: var(--dim); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 4px; flex-shrink: 0; }
.pw-card { flex: 1; min-width: 200px; max-width:1010px; }
.pw-canvas { flex: 1; width: 100%; min-height: 0; }
.info-card { width: 110px; flex-shrink: 0; }
.ir { display: flex; gap: 4px; color: var(--dim); font-size: 14px; margin-bottom: 3px; }
.ir span:first-child { width: 28px; flex-shrink: 0; }
.in { color: var(--text); font-weight: 600; }

/* ── Row 3 ── */
.quad-row { display: flex; gap: 10px; align-items: stretch; height: 300px; }
.tire-box { width:260px; flex-shrink: 0; background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 10px; display: flex; flex-direction: column; }
.tire-inner { flex: 1; display: flex; flex-direction: column; gap: 8px; }
.t-row { display: flex; gap: 4px; flex: 1; }
/* char / gauge / gauge / char */
.t-meta { display: flex; flex-direction: column; align-items: center; justify-content: center; width: 36px; gap: 3px; flex-shrink: 0; }
.t-pos { font-size: 16px; font-weight: 700; color: var(--text); line-height: 1; }
.t-num { font-size: 13px; font-weight: 600; font-variant-numeric: tabular-nums; }
.t-gauge { flex: 1; background: rgba(255,255,255,0.04); border-radius: 12px; overflow: hidden; display: flex; align-items: flex-end; }
.t-fill { width: 100%; border-radius: 0; transition: height 0.3s; }

.gg-box { width: 290px; flex-shrink: 0; align-items: center; display: flex; flex-direction: column; }
.gg-wrap { flex: 1; display: flex; align-items: center; justify-content: center; width: 100%; }
.gg-wrap canvas { width: 250px; height: 250px; }

.susp-box { width: 290px; flex-shrink: 0; }
.susp-inner { flex: 1; display: flex; align-items: center; justify-content: center; }
.susp-row { display: flex; gap: 28px; justify-content: center; align-items: flex-end; }
.susp-cell { display: flex; flex-direction: column; align-items: center; gap: 2px; position: relative; }
.susp-cell.front { top: -28px; }
.susp-cell.rear { top: 20px; }
.susp-pos { font-size: 12px; font-weight: 700; color: var(--accent); }
.susp-val { font-size: 9px; color: var(--dim); }
.susp-bar-wrap { width: 32px; height: 140px; background: rgba(255,255,255,0.05); border-radius: 16px; overflow: hidden; display: flex; align-items: flex-end; }
.susp-bar { width: 100%; border-radius: 0; transition: height 0.2s; min-height: 0; }
</style>
