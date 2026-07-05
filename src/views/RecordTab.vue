<script setup lang="ts">
import { ref, onMounted } from "vue"

defineProps<{ telemetry: any }>()
const emit = defineEmits<{ action: [action: string] }>()

const localFiles = ref<any[]>([])
function refreshFiles() { emit('action', 'list_recordings') }
onMounted(() => {
  refreshFiles(); setInterval(refreshFiles, 5000)
  window.addEventListener('rec-files', ((e: CustomEvent) => { localFiles.value = e.detail || [] }) as EventListener)
})

const delay = ref(3)
const freeTrim = ref(3)
const freeWait = ref(true)
const timedDur = ref(30)
</script>

<template>
  <div class="tab-page">
    <h2>录制 &amp; 回放</h2>

    <div class="two-col">
      <!-- Left: Recording controls -->
      <div class="col">
        <div class="card">
          <h3>启动延迟</h3>
          <div class="row">
            <label>倒计时</label>
            <input type="range" min="0" max="10" v-model.number="delay">
            <span class="val">{{ delay === 0 ? '立即' : delay + '秒' }}</span>
          </div>
        </div>

        <div class="card">
          <h3>自由录制 <span class="tag">手动停止</span></h3>
          <div class="row">
            <label>尾部裁剪</label>
            <input type="range" min="0" max="15" v-model.number="freeTrim">
            <span class="val">{{ freeTrim === 0 ? '不裁剪' : freeTrim + '秒' }}</span>
          </div>
          <div class="row">
            <label><input type="checkbox" v-model="freeWait"> 等有效数据</label>
          </div>
          <div class="action-row">
            <button class="btn gold" v-if="!telemetry?.recording"
              @click="emit('action', JSON.stringify({ action: 'start_record', delay, wait: freeWait, trim: freeTrim }))">
              开始录制
            </button>
            <button class="btn gold" v-else
              @click="emit('action', 'stop_record')">
              停止 ({{ telemetry?.record_count ?? 0 }} 包)
            </button>
          </div>
        </div>

        <div class="card">
          <h3>定时录制 <span class="tag">自动停止</span></h3>
          <div class="row">
            <label>录制时长</label>
            <input type="range" min="5" max="120" step="5" v-model.number="timedDur">
            <span class="val">{{ timedDur }}秒</span>
          </div>
          <div class="action-row">
            <button class="btn gold" :disabled="!!telemetry?.recording"
              @click="emit('action', JSON.stringify({ action: 'start_record', delay, duration: timedDur }))">
              定时录制
            </button>
          </div>
        </div>
      </div>

      <!-- Right: File list only -->
      <div class="col">
        <div class="card">
          <h3>录制文件 <span class="tag">{{ localFiles.length }}</span></h3>
          <div v-if="localFiles.length">
            <div v-for="f in localFiles" :key="f.path" class="file-row">
              <span class="file-name">{{ f.name }}</span>
              <span class="file-info">{{ f.pkts }}包</span>
              <button class="btn sm" @click="emit('action', JSON.stringify({ action: 'playback_file', path: f.path }))">播放</button>
              <button class="btn sm" @click="emit('action', JSON.stringify({ action: 'delete_recording', path: f.path })); refreshFiles()">删除</button>
            </div>
          </div>
          <div v-else class="hint">暂无 · <a href="#" @click.prevent="refreshFiles">刷新</a></div>
        </div>
      </div>
    </div>

    <!-- Playback controls below both columns -->
<div class="card" style="margin-top:0">
  <h3>回放控制</h3>
  <div class="action-row">
    <!-- 改为切换按钮 -->
    <button class="btn gold" @click="emit('action', telemetry?.paused ? 'resume_playback' : 'pause_playback')">
      {{ telemetry?.paused ? '继续' : '暂停' }}
    </button>
    <button class="btn" @click="emit('action', 'stop_playback')">停止</button>
    <button class="btn" @click="emit('action', 'start_playback')">最新回放</button>
  </div>
  <div class="hint" v-if="telemetry?.playing">
    回放: {{ (telemetry?.pb_progress ?? 0).toFixed(1) }}s / {{ (telemetry?.pb_total ?? 0).toFixed(1) }}s · {{ telemetry?.pb_pkts ?? 0 }}/{{ telemetry?.pb_pkts_total ?? 0 }}包 · {{ telemetry?.pb_fps ?? 0 }}fps
  </div>
</div>
  </div>
</template>

<style scoped>
.tab-page { padding: 24px; max-width: 900px; }
h2 { font-size: 18px; margin-bottom: 16px; }

.two-col { display: flex; gap: 12px; align-items: flex-start; }
.col { flex: 1; min-width: 0; }

.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 14px; margin-bottom: 10px; }
.card h3 { font-size: 12px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 10px; }
.tag { font-size: 10px; color: var(--dim); font-weight: 400; margin-left: 4px; }

.row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.row label { font-size: 12px; color: var(--dim); min-width: 90px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.row input[type=checkbox] { accent-color: var(--accent); margin-right: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 50px; text-align: right; }
.action-row { display: flex; gap: 6px; margin-top: 4px; }
.btn {
  padding: 7px 14px; border: 1px solid var(--border); border-radius: 5px;
  background: var(--card); color: var(--text); font-size: 12px; cursor: pointer;
}
.btn:hover { background: #333; }
.btn:disabled { opacity: 0.4; cursor: default; }
.btn.gold { background: var(--accent); color: #181920; border-color: var(--accent); font-weight: 600; }
.btn.gold:hover { background: #e0b850; }
.btn.sm { padding: 3px 10px; font-size: 11px; }
.hint { font-size: 10px; color: var(--dim); margin-top: 6px; }
.hint a { color: var(--accent); }

.file-row { display: flex; align-items: center; gap: 6px; padding: 3px 0; border-bottom: 1px solid rgba(255,255,255,0.04); }
.file-name { flex: 1; font-size: 12px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-info { font-size: 10px; color: var(--dim); white-space: nowrap; }
</style>
