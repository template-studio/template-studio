<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">行为设置</div>

      <div class="setting-row">
        <div class="setting-row-title">双击关闭标签页</div>
        <a-switch v-model:checked="settings.doubleClickToClose" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">拖拽排序标签页</div>
        <a-switch v-model:checked="settings.dragToSort" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">自动隐藏工具栏</div>
        <a-switch v-model:checked="settings.autoHideToolbar" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive } from 'vue'

const settings = reactive({
  doubleClickToClose: false,
  dragToSort: true,
  autoHideToolbar: false
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>