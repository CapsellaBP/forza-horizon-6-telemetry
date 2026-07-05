<script setup lang="ts">
import { computed } from "vue"
import InfoTip from "../components/InfoTip.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any] }>()

function s(key: string, fb: any = null) { return props.telemetry?.settings?.[key] ?? fb }

const tires = computed(() => props.telemetry?.tire_temp ?? [0,0,0,0])
const slipVals = computed(() => props.telemetry?.tire_slip ?? [0,0,0,0])
function tempColor(t: number): string {
  if (t <= 0) return "#555"; if (t < 60) return "#4a90d9"; if (t < 80) return "#58a6ff"
  if (t < 95) return "#4ec46b"; if (t < 110) return "#d4a843"; return "#f14c4c"
}
function slipColor(s: number): string {
  const a = Math.abs(s); if (a < 0.05) return "#4ec46b"; if (a < 0.15) return "#d4a843"; return "#f14c4c"
}
</script>

<template>
  <div class="tab-page">
    <h2>轮胎</h2>
    <p class="subtitle">胎温 · 滑移率</p>

    <div class="card">
      <h3>滑移阈值</h3>
      <div class="row">
        <label>黄闪 <InfoTip>滑移率超过此值显示黄色警告</InfoTip></label>
        <input type="range" min="1" max="50" :value="(s('slip_warn', 0.10)) * 100"
          @input="emit('set', 'slip_warn', Number(($event.target as HTMLInputElement).value) / 100)">
        <span class="val">{{ (s('slip_warn', 0.10)).toFixed(2) }}</span>
      </div>
      <div class="row">
        <label>红标 <InfoTip>滑移率超过此值显示红色警告（严重打滑）</InfoTip></label>
        <input type="range" min="5" max="100" :value="(s('slip_danger', 0.50)) * 100"
          @input="emit('set', 'slip_danger', Number(($event.target as HTMLInputElement).value) / 100)">
        <span class="val">{{ (s('slip_danger', 0.50)).toFixed(2) }}</span>
      </div>
    </div>

    <div class="tire-row">
      <div class="tire-box">
        <h3>胎温 °C</h3>
        <div class="tire-grid">
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
      <div class="tire-box">
        <h3>滑移率</h3>
        <div class="tire-grid">
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
    </div>
  </div>
</template>

<style scoped>
.tab-page { padding: 24px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 12px; }
.card h3, .tire-box h3 { font-size: 12px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 12px; }
.row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
.row label { font-size: 12px; color: var(--dim); min-width: 50px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 50px; text-align: right; }

.tire-row { display: flex; gap: 25px; height: calc(100vh - 225px); min-height: 200px; min-width: 300px; }
.tire-box { flex: 1; display: flex; flex-direction: column; min-height: 0; background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 14px; margin-bottom: 12px; }
.tire-grid { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 16px; }
.t-row { flex: 1; min-height: 0; display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 6px; }
.t-gauge { border-radius: 25px; overflow: hidden; display: flex; align-items: flex-end; background: rgba(255,255,255,0.04); }
.t-fill { width: 100%; border-radius: 0px; transition: height 0.3s; }
.t-meta { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 3px; }
.t-pos { font-size: 18px; font-weight: 700; color: var(--text); line-height: 1; text-align: center; }
.t-num { font-size: 14px; font-weight: 600; font-variant-numeric: tabular-nums; text-align: center; }
</style>
