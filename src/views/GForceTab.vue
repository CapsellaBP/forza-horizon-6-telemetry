<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue"
import InfoTip from "../components/InfoTip.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any] }>()

const gLon = () => props.telemetry?.accel_lon ?? 0
const gLat = () => props.telemetry?.accel_lat ?? 0

function s(key: string, fb: any = null) { return props.telemetry?.settings?.[key] ?? fb }

const ggCanvas = ref<HTMLCanvasElement | null>(null)
const SZ = 350
const trail: {x:number,y:number}[] = []
const TRAIL_MAX = 15

function drawGG() {
  const c = ggCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  c.width = SZ * dpr; c.height = SZ * dpr
  c.style.width = SZ + "px"; c.style.height = SZ + "px"
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  const cx = SZ / 2, cy = SZ / 2, r = SZ / 2 - 30

  ctx.clearRect(0, 0, SZ, SZ)
  // Background
  ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2)
  ctx.fillStyle = "rgba(255,255,255,0.02)"; ctx.fill()
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1; ctx.stroke()
  ctx.strokeStyle = "rgba(255,255,255,0.06)"; ctx.lineWidth = 0.5
  ctx.beginPath(); ctx.moveTo(cx - r, cy); ctx.lineTo(cx + r, cy); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(cx, cy - r); ctx.lineTo(cx, cy + r); ctx.stroke()
  for (const p of [0.5, 1.0]) { ctx.beginPath(); ctx.arc(cx, cy, r * p, 0, Math.PI * 2); ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.stroke() }

  // Trail (fading dots)
  const maxG = 3.0
  const scale = r / maxG
  const dx = Math.max(-maxG, Math.min(maxG, gLat())) * scale
  const dy = Math.max(-maxG, Math.min(maxG, -gLon())) * scale
  trail.push({ x: cx + dx, y: cy + dy })
  if (trail.length > TRAIL_MAX) trail.shift()

  for (let i = 0; i < trail.length; i++) {
    const alpha = (i / trail.length) * 0.30
    const size = 2 + (i / trail.length) * 3
    ctx.beginPath(); ctx.arc(trail[i].x, trail[i].y, size, 0, Math.PI * 2)
    ctx.fillStyle = `rgba(212,168,67,${alpha})`
    ctx.fill()
  }

  // Current point + tether line
  if (trail.length > 0) {
    const last = trail[trail.length - 1]
    // Tether line from center
    ctx.strokeStyle = "rgba(212,168,67,0.3)"; ctx.lineWidth = 1
    ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(last.x, last.y); ctx.stroke()
    // Current point
    ctx.beginPath(); ctx.arc(last.x, last.y, 5, 0, Math.PI * 2)
    ctx.fillStyle = "#d4a843"; ctx.fill()
  }

  // Direction numbers
  const fwd = gLon() > 0 ? gLon().toFixed(2) : "0.00"
  const brk = gLon() < 0 ? Math.abs(gLon()).toFixed(2) : "0.00"
  const lg = gLat() < 0 ? Math.abs(gLat()).toFixed(2) : "0.00"
  const rg = gLat() > 0 ? gLat().toFixed(2) : "0.00"

  ctx.fillStyle = "#8b8f93"; ctx.font = "bold 12px sans-serif"
  ctx.textAlign = "center"
  ctx.fillText(fwd, cx, 16); ctx.fillText(brk, cx, SZ - 6)
  ctx.textAlign = "right"; ctx.fillText(lg, 28, cy + 5)
  ctx.textAlign = "left"; ctx.fillText(rg, SZ - 28, cy + 5)

  // Total G
  const totalG = Math.sqrt(gLon()*gLon() + gLat()*gLat())
  const w = window as any
  if (!w._gTotalEma2) w._gTotalEma2 = totalG
  w._gTotalEma2 += (totalG - w._gTotalEma2) * (props.telemetry?.settings?.g_total_ema_alpha ?? 0.05)
  ctx.fillStyle = "#fff"; ctx.font = "bold 16px sans-serif"; ctx.textAlign = "right"
  ctx.fillText(w._gTotalEma2.toFixed(2), cx + r, cy - r + 16)
}

watch(() => props.telemetry, () => nextTick(drawGG), { deep: true })
onMounted(() => nextTick(drawGG))
</script>

<template>
  <div class="tab-page">
    <h2>G 值 &amp; 姿态</h2>
    <p class="subtitle">纵向/横向加速度 · G-G 轨迹</p>

    <div class="card">
      <h3>G-G 图</h3>
      <div style="display:flex;justify-content:center">
        <canvas ref="ggCanvas"></canvas>
      </div>
    </div>

    <div class="card">
      <h3>总 G 平滑</h3>
      <div class="row">
        <label>EMA 敏感度 <InfoTip>总 G 力显示平滑度。值越小越平滑（滞后），越大越灵敏（跳变）</InfoTip></label>
        <input type="range" min="2" max="50" :value="(s('g_total_ema_alpha',0.05))*100"
          @input="emit('set','g_total_ema_alpha', Number(($event.target as HTMLInputElement).value) / 100)">
        <span class="val">{{ (s('g_total_ema_alpha',0.05)).toFixed(2) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tab-page { padding: 24px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 12px; }
.card h3 { font-size: 12px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 12px; }
.row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
.row label { font-size: 12px; color: var(--dim); min-width: 110px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 50px; text-align: right; }
</style>
