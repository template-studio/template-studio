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
import { reactive } from 'vue'

const settings = reactive({
  enableCSP: true,
  blockExternalLinks: false,
  enableSandbox: true
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>