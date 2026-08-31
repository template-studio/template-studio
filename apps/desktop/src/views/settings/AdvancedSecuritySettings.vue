<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">安全设置</div>

      <div class="setting-row">
        <div class="setting-row-title">启用内容安全策略</div>
        <a-switch v-model:checked="settings.enableCSP" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">阻止外部链接</div>
        <a-switch v-model:checked="settings.blockExternalLinks" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">启用沙盒模式</div>
        <a-switch v-model:checked="settings.enableSandbox" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { useUiSettingsStore } from '@/stores/uiSettings'

const uiSettings = useUiSettingsStore()
// 嵌套 state 是 store 响应式对象的一部分，v-model 直写并经 $subscribe 自动落盘
const settings = uiSettings.security
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>