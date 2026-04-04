<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">调试模式</div>

      <div class="setting-row">
        <div class="setting-row-title">启用调试模式</div>
        <a-switch v-model:checked="settings.debugMode" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">显示性能监控</div>
        <a-switch v-model:checked="settings.showPerformanceMonitor" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">日志级别</div>
        <a-select v-model:value="settings.logLevel" size="small" style="width: 120px">
          <a-select-option value="error">错误</a-select-option>
          <a-select-option value="warn">警告</a-select-option>
          <a-select-option value="info">信息</a-select-option>
          <a-select-option value="debug">调试</a-select-option>
        </a-select>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">远程调试端口</div>
        <a-input v-model:value="settings.debugPort" size="small" style="width: 120px" placeholder="9222" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'

const settings = reactive({
  debugMode: false,
  showPerformanceMonitor: false,
  logLevel: 'info',
  debugPort: '9222'
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>