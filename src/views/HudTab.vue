<script setup lang="ts">
import { computed } from "vue"
import HudPreview from "../components/HudPreview.vue"
import InfoTip from "../components/InfoTip.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any], action: [action: string] }>()

function s(key: string, fb: any = null) { return props.telemetry?.settings?.[key] ?? fb }
function slider(e: Event, scale = 1) { return Number((e.target as HTMLInputElement).value) / scale }
function intVal(e: Event) { return Number((e.target as HTMLInputElement).value) }
function checked(e: Event) { return (e.target as HTMLInputElement).checked }

const throttle = computed(() => (props.telemetry?.throttle ?? 0) * 100)
const brake = computed(() => (props.telemetry?.brake ?? 0) * 100)
</script>

<template>
  <div class="hud-tab">
    <!-- Preview + TB + Controls (same layout as Dashboard) -->
    <div class="top-row">
      <div class="hud-col"><HudPreview :telemetry="telemetry" /></div>

      <div class="tb-col">
        <div class="tb-side"><span class="tb-lbl">刹车</span><span class="tb-pct">{{ brake.toFixed(0) }}%</span></div>
        <div class="tb-wrap"><div class="tb-fill" :style="{ height: brake + '%' }"></div></div>
        <div class="tb-wrap"><div class="tb-fill" :style="{ height: throttle + '%' }"></div></div>
        <div class="tb-side"><span class="tb-lbl">油门</span><span class="tb-pct">{{ throttle.toFixed(0) }}%</span></div>
      </div>

      <div class="hud-ctrl">
        <button class="btn gold" @click="emit('action','start_hud')">启动 HUD</button>
        <button class="btn red" @click="emit('action','stop_hud')">停止 HUD</button>
        <button class="btn" :class="{ 'edit-on': s('hud_edit_mode',false) }" @click="emit('action','hud_edit_mode')" @dblclick="$emit('action','reset_hud')" title="单击激活 HUD | 双击重置 HUD">{{ s('hud_edit_mode',false)?'关闭编辑':'激活 HUD' }}</button>
      </div>
    </div>

    <!-- Settings Grid -->
    <div class="settings-grid">
      <section class="card">
        <h3>档位数字</h3>
        <div class="row"><label>字号</label><input type="range" min="16" max="100" :value="s('gear_font_size',28)" @input="emit('set','gear_font_size',intVal($event))"><span class="val">{{ s('gear_font_size',28) }}px</span></div>
        <div class="row"><label>字重</label><input type="range" min="100" max="900" step="50" :value="s('gear_font_weight',500)" @input="emit('set','gear_font_weight',intVal($event))"><span class="val">{{ s('gear_font_weight',500) }}</span></div>
        <div class="row"><label>暖色起点 <InfoTip>白→黄分界点</InfoTip></label><input type="range" min="10" max="70" :value="(s('gear_yellow_start',0.5))*100" @input="emit('set','gear_yellow_start',slider($event,100))"><span class="val">{{ ((s('gear_yellow_start',0.5))*100).toFixed(0) }}%</span></div>
        <div class="toggles">
          <label class="toggle"><input type="checkbox" :checked="s('gear_italic',true)" @change="emit('set','gear_italic',checked($event))"> 斜体</label>
          <label class="toggle"><input type="checkbox" :checked="s('gear_glow',true)" @change="emit('set','gear_glow',checked($event))"> 底部打光</label>
        </div>
      </section>

      <section class="card">
        <h3>时速数字</h3>
        <div class="row"><label>字号</label><input type="range" min="16" max="100" :value="s('speed_font_size',42)" @input="emit('set','speed_font_size',intVal($event))"><span class="val">{{ s('speed_font_size',42) }}px</span></div>
        <div class="row"><label>字重</label><input type="range" min="100" max="900" step="50" :value="s('speed_font_weight',500)" @input="emit('set','speed_font_weight',intVal($event))"><span class="val">{{ s('speed_font_weight',500) }}</span></div>
        <div class="row"><label>档位-时速间距</label><input type="range" min="-10" max="30" :value="s('top_row_gap',4)" @input="emit('set','top_row_gap',intVal($event))"><span class="val">{{ s('top_row_gap',4) }}px</span></div>
        <div class="row"><label>时速-标签间距</label><input type="range" min="0" max="20" :value="s('speed_label_gap',3)" @input="emit('set','speed_label_gap',intVal($event))"><span class="val">{{ s('speed_label_gap',3) }}px</span></div>
        <label class="toggle"><input type="checkbox" :checked="s('speed_italic',true)" @change="emit('set','speed_italic',checked($event))"> 斜体</label>
      </section>

      <section class="card">
        <h3>KM/H &amp; SLIP 标签</h3>
        <div class="row"><label>字号</label><input type="range" min="8" max="24" :value="s('label_font_size',12)" @input="emit('set','label_font_size',intVal($event))"><span class="val">{{ s('label_font_size',12) }}px</span></div>
        <label class="toggle"><input type="checkbox" :checked="s('label_italic',true)" @change="emit('set','label_italic',checked($event))"> 斜体</label>
      </section>

      <section class="card">
        <h3>RPM 条</h3>
        <div class="row"><label>亮度</label><input type="range" min="20" max="100" :value="(s('rpm_bar_brightness',1))*100" @input="emit('set','rpm_bar_brightness',slider($event,100))"><span class="val">{{ ((s('rpm_bar_brightness',1))*100).toFixed(0) }}%</span></div>
        <div class="row"><label>色相 <InfoTip>RPM条色相偏移。0=默认暖色渐变，±180调色调</InfoTip></label><input type="range" min="-180" max="180" :value="s('rpm_bar_hue',0)" @input="emit('set','rpm_bar_hue',intVal($event))"><span class="val">{{ s('rpm_bar_hue',0) }}°</span></div>
        <div class="row"><label>总长</label><input type="range" min="100" max="500" :value="s('rpm_bar_width',260)" @input="emit('set','rpm_bar_width',intVal($event))"><span class="val">{{ s('rpm_bar_width',260) }}px</span></div>
        <div class="color-bar">白→<span style="color:#ffdd00">黄</span>→<span style="color:#ff8800">橙</span>→<span style="color:#ff3300">红</span>→<span style="color:#3399ff">蓝(爆闪)</span></div>
      </section>

      <section class="card">
        <h3>标记线</h3>
        <div class="row"><label>功率带阈值 <InfoTip>峰值功率百分比。高于此值的RPM区间显示白色功率带</InfoTip></label><input type="range" min="50" max="100" :value="(s('power_band_pct',0.93))*100" @input="emit('set','power_band_pct',slider($event,100))"><span class="val">{{ ((s('power_band_pct',0.93))*100).toFixed(0) }}%</span></div>
        <div class="row"><label>功率带透明度</label><input type="range" min="10" max="100" :value="(s('power_band_opacity',0.5))*100" @input="emit('set','power_band_opacity',slider($event,100))"><span class="val">{{ ((s('power_band_opacity',0.5))*100).toFixed(0) }}%</span></div>
        <div class="row"><label>爆闪触发点 <InfoTip>当前RPM超过换挡点乘以此比例时触发蓝闪。1.0=精确换挡点</InfoTip></label><input type="range" min="80" max="120" :value="(s('shift_trigger_pct',1.0))*100" @input="emit('set','shift_trigger_pct',slider($event,100))"><span class="val">{{ ((s('shift_trigger_pct',1.0))*100).toFixed(0) }}%</span></div>
        <div class="row"><label>标记线平滑 <InfoTip>换挡线/断油线/功率带的显示层EMA。越小越稳定不跳，越大越快响应</InfoTip></label><input type="range" min="2" max="50" :value="(s('marker_ema_alpha',0.06))*100" @input="emit('set','marker_ema_alpha',slider($event,100))"><span class="val">{{ (s('marker_ema_alpha',0.06)).toFixed(2) }}</span></div>
      </section>

      <section class="card">
        <h3>外观</h3>
        <div class="row"><label>透明度</label><input type="range" min="20" max="100" :value="(s('hud_opacity',0.7))*100" @input="emit('set','hud_opacity',slider($event,100))"><span class="val">{{ ((s('hud_opacity',0.7))*100).toFixed(0) }}%</span></div>
        <div class="row"><label>缩放</label><input type="range" min="30" max="300" :value="(s('hud_scale',2.0))*100" @input="emit('set','hud_scale',slider($event,100))"><span class="val">{{ ((s('hud_scale',2.0))*100).toFixed(0) }}% ({{ (s('hud_scale',2.0)).toFixed(2) }}x)</span></div>
        <div class="row"><label>G力浮动幅度 <InfoTip>整体浮动范围。0=关闭</InfoTip></label><input type="range" min="0" max="10" step="0.5" :value="s('g_float_amplitude',2)" @input="emit('set','g_float_amplitude',Number(($event.target as HTMLInputElement).value))"><span class="val">{{ (s('g_float_amplitude',2)).toFixed(1) }}px</span></div>
        <div class="row"><label>平滑度 <InfoTip>值越小越平滑。0.02=极平滑，0.15=灵敏抖动</InfoTip></label><input type="range" min="2" max="50" :value="(s('g_float_smoothing',0.08))*100" @input="emit('set','g_float_smoothing',slider($event,100))"><span class="val">{{ (s('g_float_smoothing',0.08)).toFixed(2) }}</span></div>
        <div class="row"><label>加速灵敏度 <InfoTip>加速G力方向缩放。加速G通常较小，可放大此值</InfoTip></label><input type="range" min="10" max="200" :value="(s('g_float_accel_scale',0.5))*100" @input="emit('set','g_float_accel_scale',slider($event,100))"><span class="val">{{ (s('g_float_accel_scale',0.5)).toFixed(1) }}x</span></div>
        <div class="row"><label>刹车灵敏度 <InfoTip>刹车G力方向缩放。刹车G通常1.0即可</InfoTip></label><input type="range" min="10" max="200" :value="(s('g_float_brake_scale',1.0))*100" @input="emit('set','g_float_brake_scale',slider($event,100))"><span class="val">{{ (s('g_float_brake_scale',1.0)).toFixed(1) }}x</span></div>
        <div class="row"><label>横向灵敏度 <InfoTip>左右G力方向缩放。默认1.0，过弯侧倾大时可加大</InfoTip></label><input type="range" min="10" max="200" :value="(s('g_float_lat_scale',1.0))*100" @input="emit('set','g_float_lat_scale',slider($event,100))"><span class="val">{{ (s('g_float_lat_scale',1.0)).toFixed(1) }}x</span></div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.hud-tab { padding: 16px; }

/* ── Top row (sync with Dashboard) ── */
.top-row { display: flex; gap: 10px; align-items: center; margin-bottom: 16px; }
.hud-col { flex: 1; max-width: 500px; }
.hud-col > * { width: 100%; }

.tb-col { display: flex; gap: 6px; flex-shrink: 0; height: 110px; align-items: stretch; }
.tb-side { display: flex; flex-direction: column; align-items: center; justify-content: center; width: 22px; gap: 2px; }
.tb-lbl { font-size: 11px; color: var(--dim); }
.tb-pct { font-size: 13px; color: var(--text); font-weight: 600; }
.tb-wrap { width: 45px; background: rgba(255,255,255,0.05); border-radius: 11px; overflow: hidden; display: flex; align-items: flex-end; }
.tb-fill { width: 100%; background: #fff; border-radius: 8px; transition: height 0.15s; }

.hud-ctrl { display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; width: 72px; }
.btn { padding: 5px 8px; border: 1px solid var(--border); border-radius: 4px; background: var(--card); color: var(--text); font-size: 11px; cursor: pointer; text-align: center; }
.btn:hover { background: #222; }
.btn.gold { background: var(--accent); color: #121314; border-color: var(--accent); font-weight: 600; }
.btn.gold:hover { background: #e0b850; }
.btn.red { color: var(--red); border-color: var(--red); }
.btn.red:hover { background: rgba(241,76,76,0.12); }
.btn.edit-on { border: 2px dashed rgba(255,204,0,0.55); color: #ffcc00; }

/* ── Settings grid ── */
.settings-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 14px; }
.card h3 { font-size: 11px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 10px; }
.row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.row label { font-size: 12px; color: var(--dim); min-width: 80px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 52px; text-align: right; }
.toggles { display: flex; gap: 12px; margin-top: 4px; }
.toggle { font-size: 12px; color: var(--dim); display: flex; align-items: center; gap: 4px; cursor: pointer; }
.toggle input { accent-color: var(--accent); }
.color-bar { font-size: 11px; display: flex; gap: 4px; margin-top: 2px; }
</style>
