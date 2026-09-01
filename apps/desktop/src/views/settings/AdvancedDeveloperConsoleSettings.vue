<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">控制台设置</div>

      <div class="setting-row">
        <div class="setting-row-title">启用开发者控制台</div>
        <a-switch v-model:checked="settings.enableConsole" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">控制台位置</div>
        <a-radio-group v-model:value="settings.consolePosition" size="small">
          <a-radio value="bottom">底部</a-radio>
          <a-radio value="right">右侧</a-radio>
          <a-radio value="separate">独立窗口</a-radio>
        </a-radio-group>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">控制台透明度</div>
        <a-slider
          v-model:value="settings.consoleOpacity"
          :min="30"
          :max="100"
          :step="5"
          style="width: 150px"
        />
        <span style="margin-left: 8px; font-size: 12px; color: var(--color-text-secondary);">
          {{ settings.consoleOpacity }}%
        </span>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">字体大小</div>
        <a-select v-model:value="settings.consoleFontSize" size="small" style="width: 100px">
          <a-select-option value="10">10px</a-select-option>
          <a-select-option value="12">12px</a-select-option>
          <a-select-option value="14">14px</a-select-option>
          <a-select-option value="16">16px</a-select-option>
        </a-select>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">显示时间戳</div>
        <a-switch v-model:checked="settings.showTimestamp" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">自动滚动到底部</div>
        <a-switch v-model:checked="settings.autoScroll" size="small" />
      </div>
    </div>

    <!-- 高级选项 -->
    <div class="setting-group">
      <div class="setting-title">高级选项</div>

      <div class="setting-row">
        <div class="setting-row-title">网络请求监控</div>
        <a-switch v-model:checked="settings.networkMonitoring" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">性能分析</div>
        <a-switch v-model:checked="settings.performanceAnalysis" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">内存使用监控</div>
        <a-switch v-model:checked="settings.memoryMonitoring" size="small" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'

const settings = reactive({
  enableConsole: false,
  consolePosition: 'bottom',
  consoleOpacity: 80,
  consoleFontSize: '12',
  showTimestamp: true,
  autoScroll: true,
  networkMonitoring: false,
  performanceAnalysis: false,
  memoryMonitoring: false
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>

// 切换开关 → 调 Tauri 开/关 DevTools
watch(() => settings.enableConsole, async (val) => {
  try {
    const opened = await invoke('toggle_devtools')
    if (opened !== val) {
      // 状态不同步（如 F12 已手动开），以 DevTools 实际状态为准
      settings.enableConsole = opened
    }
  } catch (e) {
    message.error('控制台切换失败: ' + e)
    settings.enableConsole = false
  }
})

// F12 快捷键
const onKeydown = (e) => {
  if (e.key === 'F12') {
    e.preventDefault()
    settings.enableConsole = !settings.enableConsole
  }
}
onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
