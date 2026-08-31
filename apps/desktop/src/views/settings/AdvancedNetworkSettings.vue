<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">网络设置</div>

      <div class="setting-row">
        <div class="setting-row-title">代理设置</div>
        <a-select v-model:value="settings.proxyMode" size="small" style="width: 120px">
          <a-select-option value="none">不使用代理</a-select-option>
          <a-select-option value="system">系统代理</a-select-option>
          <a-select-option value="custom">自定义代理</a-select-option>
        </a-select>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">请求超时时间</div>
        <a-input v-model:value="settings.requestTimeout" size="small" style="width: 100px" placeholder="30" />
        <span style="margin-left: 8px; font-size: 12px; color: var(--color-text-secondary);">秒</span>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">并发连接数限制</div>
        <a-input-number v-model:value="settings.maxConnections" :min="1" :max="100" size="small" style="width: 100px" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { useUiSettingsStore } from '@/stores/uiSettings'

const uiSettings = useUiSettingsStore()
const settings = uiSettings.network
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>