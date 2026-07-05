<script setup lang="ts">
import { ref, watch } from "vue"
import InfoTip from "../components/InfoTip.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any], action: [action: string] }>()

function s(key: string, fallback: any = null) {
  return props.telemetry?.settings?.[key] ?? fallback
}

const portVal = ref(s('udp_port', 5300))
watch(() => s('udp_port', 5300), (v) => { portVal.value = v })
</script>

<template>
  <div class="tab-page">
    <h2>设置</h2>
    <p class="subtitle">采样控制 · EV 检测 · 每车预设 · 数据管理</p>

    <!-- Sampling -->
    <div class="card">
      <h3>曲线采样</h3>
      <div class="row">
        <label>最低油门
          <InfoTip>低于此油门开度不采样，过滤松油/滑行数据</InfoTip>
        </label>
        <input type="range" min="50" max="100" :value="(s('sample_throttle_min', 0.95)) * 100"
          @input="emit('set', 'sample_throttle_min', Number(($event.target as HTMLInputElement).value) / 100)">
        <span class="val">{{ ((s('sample_throttle_min', 0.95)) * 100).toFixed(0) }}%</span>
      </div>
      <div class="row">
        <label>换挡跳过帧
          <InfoTip>换挡后跳过 N 帧再采样，过滤换挡瞬间的脏数据</InfoTip>
        </label>
        <input type="range" min="0" max="30" :value="s('sample_skip_frames', 8)"
          @input="emit('set', 'sample_skip_frames', Number(($event.target as HTMLInputElement).value))">
        <span class="val">{{ s('sample_skip_frames', 8) }}帧</span>
      </div>
      <div class="row">
        <label>骤降拒绝阈值 <InfoTip>同比拒绝：当前功率低于同转速EMA乘以此阈值时拒绝该样本。0=不拒绝</InfoTip></label>
        <input type="range" min="0" max="100" :value="(s('power_drop_limit', 0.0)) * 100"
          @input="emit('set', 'power_drop_limit', Number(($event.target as HTMLInputElement).value) / 100)">
        <span class="val">{{ (s('power_drop_limit', 0.0)).toFixed(2) }}</span>
      </div>
      <div class="row">
        <label>EMA 权重
          <InfoTip>新数据的权重。1/2=快速覆盖，1/16=极度平滑。值越大越敏感</InfoTip>
        </label>
        <select :value="String(s('curve_alpha', 0.25))"
          @change="emit('set', 'curve_alpha', Number(($event.target as HTMLSelectElement).value))"
          style="background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:4px;padding:3px 8px">
          <option value="0.5">1/2 (快)</option>
          <option value="0.25">1/4 (默认)</option>
          <option value="0.125">1/8</option>
          <option value="0.0625">1/16 (慢)</option>
        </select>
      </div>
      <div class="action-row" style="margin-top:12px">
        <button class="btn" @click="emit('action', 'lock_curve')">锁定曲线</button>
        <button class="btn" @click="emit('action', 'unlock_curve')">解锁曲线</button>
        <button class="btn" @click="emit('action', 'reset_curve')">重置此车</button>
        <button class="btn" @click="emit('action', 'reset_all')">重置全部</button>
      </div>
    </div>

    <!-- Boost stable sampling -->
    <div class="card">
      <h3>增压稳定采样 <InfoTip>仅涡轮车有效。增压稳定后才采样，过滤增压爬升阶段的低质量数据</InfoTip></h3>
      <div class="row">
        <label><input type="checkbox" :checked="!!s('boost_stable_sample', false)"
          @change="emit('set', 'boost_stable_sample', ($event.target as HTMLInputElement).checked)"> 启用</label>
      </div>
    </div>

    <!-- EV -->
    <div class="card">
      <h3>EV 检测</h3>
      <div class="row">
        <label>检测帧数
          <InfoTip>全油门时档位≤2 持续 N 帧判为电车（自动隐藏换挡/断油线）</InfoTip>
        </label>
        <input type="range" min="100" max="2000" step="50" :value="s('ev_detect_frames', 300)"
          @input="emit('set', 'ev_detect_frames', Number(($event.target as HTMLInputElement).value))">
        <span class="val">{{ s('ev_detect_frames', 300) }}</span>
      </div>
    </div>

    <div class="card">
      <h3>UDP 端口 <InfoTip>修改后重启应用生效。游戏内 Data Out 端口需与此一致</InfoTip></h3>
      <div class="row">
        <label>监听端口</label>
        <input type="number" min="1024" max="65535" :value="portVal"
          @input="portVal = Number(($event.target as HTMLInputElement).value)"
          @change="emit('set', 'udp_port', portVal)"
          style="width:80px;background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:4px;padding:4px 8px">
        <button class="btn" @click="emit('action', 'restart_app')" style="margin-left:8px">保存并退出</button>
        <div style="font-size:10px;color:var(--dim);margin-top:2px">修改端口后需手动重启应用生效</div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.tab-page { padding: 24px; max-width: 600px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }

.card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: 8px; padding: 16px; margin-bottom: 12px;
}
.card h3 {
  font-size: 12px; color: var(--accent); text-transform: uppercase;
  letter-spacing: 1px; margin-bottom: 12px;
}

.row {
  display: flex; align-items: center; gap: 10px; margin-bottom: 10px;
}
.row label { font-size: 12px; color: var(--dim); min-width: 110px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 60px; text-align: right; }
.action-row { display: flex; gap: 8px; }
.btn {
  padding: 5px 12px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--card); color: var(--text); font-size: 12px; cursor: pointer;
}
.btn:hover { background: #333; }
</style>
