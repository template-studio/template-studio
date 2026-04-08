<template>
  <div class="system-grid">
    <div class="system-row">
      <div class="system-label">平台</div>
      <div class="system-value">{{ systemInfo.platform }}</div>
    </div>
    <div class="system-row">
      <div class="system-label">架构</div>
      <div class="system-value">{{ systemInfo.arch }}</div>
    </div>
    <div class="system-row">
      <div class="system-label">Node.js</div>
      <div class="system-value">{{ systemInfo.nodeVersion }}</div>
    </div>
    <div class="system-row">
      <div class="system-label">Tauri</div>
      <div class="system-value">{{ systemInfo.tauriVersion }}</div>
    </div>
    <div class="system-row">
      <div class="system-label">Chrome</div>
      <div class="system-value">{{ systemInfo.chromeVersion }}</div>
    </div>
    <div class="system-row">
      <div class="system-label">Vue.js</div>
      <div class="system-value">{{ systemInfo.vueVersion }}</div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  systemInfo: {
    type: Object,
    required: true
  }
})
</script>

<style scoped>
.system-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
}

.system-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  background: var(--color-background);
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

.system-label {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.system-value {
  color: var(--color-text);
  font-weight: 500;
  font-size: 13px;
}
</style>
