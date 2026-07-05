<script setup lang="ts">
import { computed, ref, onMounted, watch, nextTick } from "vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any] }>()

const susp = computed(() => props.telemetry?.susp_travel ?? [0,0,0,0])
const wheelLabels = ["FL 前左","FR 前右","RL 后左","RR 后右"]

function getThresholds() {
  const s = props.telemetry?.settings
  return {
    t1: s?.susp_thr1 ?? 0.30,
    t2: s?.susp_thr2 ?? 0.55,
    t3: s?.susp_thr3 ?? 0.80,
  }
}

function travelColor(v: number): string {
  const { t1, t2, t3 } = getThresholds()
  if (v >= t3) return "#f14c4c"
  if (v >= t2) return "#d4a843"
  if (v >= t1) return "#3399ff"
  return "#4ec46b"
}

// ── Scopes ──
const maxFrames = 300
const scopeFL = ref<HTMLCanvasElement | null>(null)
const scopeFR = ref<HTMLCanvasElement | null>(null)
const scopeRL = ref<HTMLCanvasElement | null>(null)
const scopeRR = ref<HTMLCanvasElement | null>(null)

function suspColor(v: number): string {
  const { t1, t2, t3 } = getThresholds()
  if (v >= t3) return "rgba(241,76,76,0.8)"
  if (v >= t2) return "rgba(212,168,67,0.7)"
  if (v >= t1) return "rgba(51,153,255,0.6)"
  return "rgba(78,196,107,0.5)"
}

function drawScope(c: HTMLCanvasElement, wi: number) {
  const dpr = window.devicePixelRatio || 1
  const w = c.clientWidth, h = c.clientHeight
  if (w === 0 || h === 0) return
  c.width = w * dpr; c.height = h * dpr
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.fillStyle = "#121314"; ctx.fillRect(0, 0, w, h)

  const tw = window as any; if (!tw._suspHistory) return
  const hist = tw._suspHistory
  if (hist.length < 2) return

  const pad = { t: 6, r: 4, b: 14, l: 24 }
  const pw = w - pad.l - pad.r, ph = h - pad.t - pad.b

  ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.lineWidth = 0.5
  for (let i = 0; i <= 2; i++) {
    const gy = pad.t + ph * i / 2
    ctx.beginPath(); ctx.moveTo(pad.l, gy); ctx.lineTo(w - pad.r, gy); ctx.stroke()
  }
  ctx.fillStyle = "#555"; ctx.font = "7px sans-serif"; ctx.textAlign = "right"
  ctx.fillText("100%", pad.l - 3, pad.t + 5)
  ctx.fillText("0%", pad.l - 3, pad.t + ph + 3)

  ctx.save()
  ctx.beginPath(); ctx.rect(pad.l, pad.t, pw, ph); ctx.clip()

  // White glow
  ctx.beginPath()
  for (let i = 0; i < hist.length; i++) {
    const v = Math.max(0, Math.min(1, hist[i][wi] ?? 0))
    const x = pad.l + (i / maxFrames) * pw
    const y = pad.t + ph - v * ph
    i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)
  }
  ctx.strokeStyle = "rgba(255,255,255,0.08)"; ctx.lineWidth = 3; ctx.stroke()
  // Colored segments
  for (let i = 0; i < hist.length - 1; i++) {
    const v2 = Math.max(0, Math.min(1, hist[i + 1][wi] ?? 0))
    const x1 = pad.l + (i / maxFrames) * pw, x2 = pad.l + ((i + 1) / maxFrames) * pw
    const y1 = pad.t + ph - Math.max(0, Math.min(1, hist[i][wi] ?? 0)) * ph
    const y2 = pad.t + ph - v2 * ph
    ctx.beginPath(); ctx.moveTo(x1, y1); ctx.lineTo(x2, y2)
    ctx.strokeStyle = suspColor(v2); ctx.lineWidth = 1.5; ctx.stroke()
  }
  ctx.restore()
}

function drawAllScopes() {
  const tw = window as any; if (!tw._suspHistory) tw._suspHistory = []
  tw._suspHistory.push([...susp.value.slice(0, 4)])
  while (tw._suspHistory.length > maxFrames) tw._suspHistory.shift()

  if (scopeFL.value) drawScope(scopeFL.value, 0)
  if (scopeFR.value) drawScope(scopeFR.value, 1)
  if (scopeRL.value) drawScope(scopeRL.value, 2)
  if (scopeRR.value) drawScope(scopeRR.value, 3)
}

watch(susp, () => nextTick(drawAllScopes))
onMounted(() => nextTick(drawAllScopes))
</script>

<template>
  <div class="tab-page">
    <h2>悬挂</h2>
    <p class="subtitle">四轮行程 · 0=全伸 1=全压</p>

    <div class="susp-grid">
      <div class="scope-box scope-fl"><div class="sl">{{ wheelLabels[0] }}</div><div class="sw"><canvas ref="scopeFL"></canvas></div></div>
      <div class="scope-box scope-rl"><div class="sl">{{ wheelLabels[2] }}</div><div class="sw"><canvas ref="scopeRL"></canvas></div></div>

      <div class="bar-col">
        <div class="bar-grid">
          <!-- Row: rear RL / RR -->
          <div class="bar-item"><div class="bar-wrap"><div class="bar" :style="{ height: ((susp[2]??0)*100).toFixed(0)+'%', background: travelColor(susp[2]??0) }"></div></div></div>
          <div class="bar-info"><span class="bar-pos">RL</span><span class="bar-val">{{ (susp[2]??0).toFixed(3) }}</span></div>
          <div class="bar-info"><span class="bar-pos">RR</span><span class="bar-val">{{ (susp[3]??0).toFixed(3) }}</span></div>
          <div class="bar-item"><div class="bar-wrap"><div class="bar" :style="{ height: ((susp[3]??0)*100).toFixed(0)+'%', background: travelColor(susp[3]??0) }"></div></div></div>
          <!-- Row: front FL / FR -->
          <div class="bar-item"><div class="bar-wrap"><div class="bar" :style="{ height: ((susp[0]??0)*100).toFixed(0)+'%', background: travelColor(susp[0]??0) }"></div></div></div>
          <div class="bar-info"><span class="bar-pos">FL</span><span class="bar-val">{{ (susp[0]??0).toFixed(3) }}</span></div>
          <div class="bar-info"><span class="bar-pos">FR</span><span class="bar-val">{{ (susp[1]??0).toFixed(3) }}</span></div>
          <div class="bar-item"><div class="bar-wrap"><div class="bar" :style="{ height: ((susp[1]??0)*100).toFixed(0)+'%', background: travelColor(susp[1]??0) }"></div></div></div>
        </div>
      </div>

      <div class="scope-box scope-fr"><div class="sl">{{ wheelLabels[1] }}</div><div class="sw"><canvas ref="scopeFR"></canvas></div></div>
      <div class="scope-box scope-rr"><div class="sl">{{ wheelLabels[3] }}</div><div class="sw"><canvas ref="scopeRR"></canvas></div></div>
    </div>

    <!-- Threshold sliders -->
    <div class="card thr-card">
      <h3>颜色阈值</h3>
      <div class="thr-row">
        <label>绿→蓝 <span class="thr-v">{{ getThresholds().t1.toFixed(2) }}</span></label>
        <input type="range" min="0.10" max="0.50" step="0.01"
          :value="getThresholds().t1"
          @input="emit('set', 'susp_thr1', Number(($event.target as HTMLInputElement).value))">
      </div>
      <div class="thr-row">
        <label>蓝→黄 <span class="thr-v">{{ getThresholds().t2.toFixed(2) }}</span></label>
        <input type="range" min="0.30" max="0.75" step="0.01"
          :value="getThresholds().t2"
          @input="emit('set', 'susp_thr2', Number(($event.target as HTMLInputElement).value))">
      </div>
      <div class="thr-row">
        <label>黄→红 <span class="thr-v">{{ getThresholds().t3.toFixed(2) }}</span></label>
        <input type="range" min="0.55" max="0.95" step="0.01"
          :value="getThresholds().t3"
          @input="emit('set', 'susp_thr3', Number(($event.target as HTMLInputElement).value))">
      </div>
    </div>
  </div>
</template>

<style scoped>
.tab-page { padding: 24px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }

/* ── Grid: = || = ── */
.susp-grid {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  grid-template-rows: 250px 250px;
  gap: 10px;
  margin-bottom: 16px;
}
.scope-box {
  background: var(--card); border: 1px solid var(--border); border-radius: 8px;
  padding: 6px 8px 4px;
  display: flex; flex-direction: column;
  min-height: 0; overflow: hidden;
}
.scope-fl { grid-column: 1; grid-row: 1; }
.scope-rl { grid-column: 1; grid-row: 2; }
.scope-fr { grid-column: 3; grid-row: 1; }
.scope-rr { grid-column: 3; grid-row: 2; }
.sl { font-size: 10px; color: var(--dim); margin-bottom: 2px; flex-shrink: 0; }
.sw { flex: 1; min-height: 0; }
.sw canvas { width: 100%; height: 100%; display: block; }

/* Center bar chart */
.bar-col {
  grid-column: 2; grid-row: 1 / 3;
  background: var(--card); border: 1px solid var(--border); border-radius: 8px;
  padding: 25px 16px 16px;
  display: flex; align-items: center; justify-content: center;
}
.bar-grid {
  display: grid;
  grid-template-columns: auto auto auto auto;
  gap: 50px 8px;
}
.bar-item { display: flex; justify-content: center; }
.bar-info { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; min-width: 30px; height: 160px; }
.bar-pos { font-size: 20px; font-weight: 700; color: var(--accent); }
.bar-val { font-size: 10px; color: var(--dim); }
.bar-wrap { width: 30px; height: 200px; background: rgba(255,255,255,0.05); border-radius: 15px; overflow: hidden; display: flex; align-items: flex-end; }
.bar { width: 100%; border-radius: 0; transition: height 0.2s; min-height: 0; }

/* Threshold card */
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 12px; }
.card h3 { font-size: 12px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 12px; }
.thr-card { }
.thr-row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
.thr-row label { font-size: 12px; color: var(--dim); min-width: 60px; flex-shrink: 0; }
.thr-row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.thr-v { font-size: 11px; color: var(--text); min-width: 36px; text-align: right; }
</style>
