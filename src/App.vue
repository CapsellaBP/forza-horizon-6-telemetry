<script setup lang="ts">
import { ref, computed } from "vue"
import DashboardTab from "./views/DashboardTab.vue"
import HudTab from "./views/HudTab.vue"
import PowerTab from "./views/PowerTab.vue"
import TiresTab from "./views/TiresTab.vue"
import SuspensionTab from "./views/SuspensionTab.vue"
import GForceTab from "./views/GForceTab.vue"
import RecordTab from "./views/RecordTab.vue"
import SettingsTab from "./views/SettingsTab.vue"

const tabs = [
  { id: "dashboard", label: "总览" },
  { id: "hud", label: "HUD" },
  { id: "power", label: "动力" },
  { id: "tires", label: "轮胎" },
  { id: "suspension", label: "悬挂" },
  { id: "gforce", label: "G值" },
  { id: "record", label: "录制" },
  { id: "settings", label: "设置" },
]

const activeTab = ref("dashboard")

const telemetry = ref<any>(null)
const connected = ref(false)
let ws: WebSocket | null = null

function connectWs() {
  ws = new WebSocket("ws://127.0.0.1:9000")
  ws.onopen = () => { connected.value = true }
  ws.onmessage = (e) => { try {
      const msg = JSON.parse(e.data)
      telemetry.value = msg
    } catch {} }
  ws.onclose = () => { connected.value = false; setTimeout(connectWs, 3000) }
}
connectWs()

// ── Status indicators ──
const sysState = computed(() => {
  if (!connected.value) return "off"       // red = disconnected
  const has = telemetry.value?.pkt_count > 0 || telemetry.value?.playing
  return has ? "sys-on" : "sys-ready"       // blue = data, green = ready
})
const sysTitle = computed(() => {
  if (!connected.value) return "断开"
  return telemetry.value?.pkt_count > 0 ? "数据接收中" : "等待数据"
})
const recState = computed(() => {
  if (telemetry.value?.recording) return "rec-blink"
  if (telemetry.value?.paused) return "play-paused"
  if (telemetry.value?.playing) return "play-on"
  return "rec-idle"
})
const recTitle = computed(() => {
  if (telemetry.value?.recording) return "录制中"
  if (telemetry.value?.paused) return "回放暂停"
  if (telemetry.value?.playing) return "回放中"
  return "录放待机"
})

function sendSetting(key: string, value: any) {
  if (ws && ws.readyState === WebSocket.OPEN)
    ws.send(JSON.stringify({ action: "set_setting", payload: { [key]: value } }))
}
function sendAction(action: string) {
  if (!ws || ws.readyState !== WebSocket.OPEN) return
  // If action is already a JSON object string, send as-is
  if (action.startsWith("{")) {
    ws.send(action)
  } else {
    ws.send(JSON.stringify({ action }))
  }
  if (action === "reset_hud") showToast("HUD 已重置")
  if (action === "reset_curve") showToast("曲线已重置")
  if (action === "reset_all") showToast("全部曲线已重置")
}

const toastMsg = ref("")
let toastTimer: any = null
function showToast(msg: string) {
  toastMsg.value = msg
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = "" }, 2000)
}

</script>

<template>
  <div class="app-shell">
    <!-- Toast notification -->
    <div v-if="toastMsg" class="toast">{{ toastMsg }}</div>
    <nav class="activity-bar">
      <div class="activity-top">
        <button v-for="t in tabs" :key="t.id"
          :class="{ active: activeTab === t.id }"
          :title="t.label" @click="activeTab = t.id"
        >{{ t.label }}</button>
      </div>
      <div class="activity-bottom">
        <span class="status-dot" :class="sysState" :title="sysTitle"></span>
        <span class="status-dot" :class="recState" :title="recTitle" style="margin-top:4px"></span>
      </div>
    </nav>

    <main class="main-content">
      <DashboardTab v-show="activeTab === 'dashboard'" :telemetry="telemetry" @action="sendAction" />
      <HudTab v-show="activeTab === 'hud'" :telemetry="telemetry" @set="sendSetting" @action="sendAction" />
      <PowerTab v-show="activeTab === 'power'" :telemetry="telemetry" @set="sendSetting" @action="sendAction" />
      <TiresTab v-show="activeTab === 'tires'" :telemetry="telemetry" @set="sendSetting" />
      <SuspensionTab v-show="activeTab === 'suspension'" :telemetry="telemetry" @set="sendSetting" />
      <GForceTab v-show="activeTab === 'gforce'" :telemetry="telemetry" @set="sendSetting" />
      <RecordTab v-show="activeTab === 'record'" :telemetry="telemetry" @action="sendAction" />
      <SettingsTab v-show="activeTab === 'settings'" :telemetry="telemetry" @set="sendSetting" @action="sendAction" />
    </main>
  </div>
</template>

<style>
:root {
  --bg: #191A1B;
  --card: #121314;
  --border: #2d2e30;
  --text: #cccccc;
  --dim: #8b8f93;
  --accent: #d4a843;
  --accent-dim: rgba(212,168,67,0.15);
  --green: #4ec46b;
  --orange: #d2991d;
  --red: #f14c4c;
}
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  background: var(--bg);
  color: var(--text);
  font-family: 'Segoe UI', 'Microsoft YaHei', -apple-system, sans-serif;
  font-size: 13px;
  overflow: hidden;
  height: 100vh;
}
#app { height: 100vh; }
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #424242; border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: #555; }
</style>

<style scoped>
.app-shell { display: flex; height: 100vh; }
.activity-bar {
  width: 48px; flex-shrink: 0;
  background: var(--card); border-right: 1px solid var(--border);
  display: flex; flex-direction: column;
  align-items: center; justify-content: space-between;
  padding: 4px 0;
}
.activity-top { display: flex; flex-direction: column; gap: 2px; }
.activity-bar button {
  width: 48px; height: 48px;
  border: none; background: transparent;
  color: var(--dim); font-size: 11px;
  cursor: pointer; position: relative;
  transition: color 0.15s;
  display: flex; align-items: center; justify-content: center;
}
.activity-bar button:hover { color: var(--text); }
.activity-bar button.active { color: var(--accent); }
.activity-bar button.active::before {
  content: ""; position: absolute; left: 0; top: 8px; bottom: 8px;
  width: 2px; background: var(--accent);
}
.activity-bottom { padding: 4px 0; }
.activity-bottom { padding: 4px 0; display: flex; flex-direction: column; align-items: center; gap: 4px; }
.status-dot { width: 10px; height: 10px; border-radius: 50%; display: block; }
.status-dot.off { background: var(--red); }
.status-dot.sys-ready { background: var(--green); }
.status-dot.sys-on { background: #3399ff; }
.status-dot.play-on { background: #fff; animation: play-blink 0.8s steps(2) infinite; }
.status-dot.play-paused { background: #fff; }
@keyframes play-blink { 0% { opacity: 1; } 100% { opacity: 0.3; } }
.status-dot.rec-idle { background: #555; }
.status-dot.rec-blink { background: var(--red); animation: rec-blink 1s steps(2) infinite; }
@keyframes rec-blink { 0% { opacity: 1; } 100% { opacity: 0.15; } }
.main-content { flex: 1; overflow-y: auto; overflow-x: hidden; }

.toast {
  position: fixed; bottom: 24px; left: 50%; transform: translateX(-50%);
  background: var(--accent); color: #181920; font-weight: 600;
  padding: 8px 20px; border-radius: 6px; font-size: 13px;
  z-index: 999; pointer-events: none;
  animation: toast-in 0.25s ease;
}
@keyframes toast-in { from { opacity:0; transform:translateX(-50%) translateY(8px); } to { opacity:1; transform:translateX(-50%) translateY(0); } }
</style>
