<script setup lang="ts">
import { ref } from "vue"

const visible = ref(false)
const locked = ref(false)
let hideTimer: any = null

function show() {
  clearTimeout(hideTimer)
  visible.value = true
}
function hide() {
  if (locked.value) return
  hideTimer = setTimeout(() => { visible.value = false }, 80)
}
function toggleLock(e: MouseEvent) {
  e.stopPropagation()
  locked.value = !locked.value
  if (locked.value) {
    visible.value = true
    document.addEventListener("click", unlockOutside, { once: true })
  } else {
    visible.value = false
  }
}
function unlockOutside() {
  locked.value = false
  visible.value = false
}
</script>

<template>
  <span class="info-tip" @mouseenter="show" @mouseleave="hide" @click="toggleLock">
    <span class="info-icon">&#9432;</span>
    <span v-if="visible" class="info-pop" :class="{ locked }">
      <slot />
      <span class="info-hint">{{ locked ? '点击别处关闭' : '点击锁定' }}</span>
    </span>
  </span>
</template>

<style scoped>
.info-tip {
  position: relative; display: inline-flex; cursor: pointer; vertical-align: middle;
}
.info-icon { font-size: 11px; font-style: normal; color: var(--dim); }
.info-icon:hover { color: var(--accent); }
.info-pop {
  position: absolute; bottom: calc(100% + 6px); left: 0;
  background: #2a2b2e; color: var(--text); font-size: 11px; line-height: 1.45;
  padding: 6px 10px; border-radius: 6px; border: 1px solid var(--border);
  white-space: nowrap; z-index: 100; pointer-events: none;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
}
.info-pop.locked { border-color: var(--accent); pointer-events: auto; }
.info-hint { display: block; font-size: 9px; color: var(--dim); margin-top: 3px; }
</style>
