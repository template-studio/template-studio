<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">实验性功能</div>

      <div class="setting-row">
        <div class="setting-row-title">Beta功能测试</div>
        <a-switch v-model:checked="settings.betaFeatures" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">新UI预览</div>
        <a-switch v-model:checked="settings.newUIPreview" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">性能优化模式</div>
        <a-switch v-model:checked="settings.performanceMode" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'

const settings = reactive({
  betaFeatures: false,
  newUIPreview: false,
  performanceMode: false
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>