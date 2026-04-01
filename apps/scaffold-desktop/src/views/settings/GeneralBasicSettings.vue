<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">基础设置</div>

      <div class="setting-row">
        <div class="setting-row-title">启动时打开首页</div>
        <a-switch v-model:checked="settings.openHomePage" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">记住上次访问页面</div>
        <a-switch v-model:checked="settings.rememberLastPage" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">自动保存间隔</div>
        <a-select v-model:value="settings.autoSaveInterval" size="small" style="width: 120px">
          <a-select-option value="30">30秒</a-select-option>
          <a-select-option value="60">1分钟</a-select-option>
          <a-select-option value="300">5分钟</a-select-option>
        </a-select>
      </div>
    </div>

    <div class="setting-group">
      <div class="setting-title">关闭行为</div>

      <div class="setting-row">
        <div class="setting-row-title">
          <div>关闭窗口时的行为</div>
          <div class="setting-description">选择点击关闭按钮时应用程序的行为</div>
        </div>
        <a-radio-group v-model:value="settings.closeBehavior" size="small">
          <a-radio value="close">关闭程序</a-radio>
          <a-radio value="minimize">最小化到托盘</a-radio>
        </a-radio-group>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">
          <div>启动时显示主窗口</div>
          <div class="setting-description">如果最小化到托盘，启动时是否显示主窗口</div>
        </div>
        <a-switch v-model:checked="settings.showWindowOnStartup" size="small" />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">
          <div>最小化到系统托盘</div>
          <div class="setting-description">启用后将可以在系统托盘中找到应用程序图标</div>
        </div>
        <a-switch
          v-model:checked="settings.enableTray"
          size="small"
          @change="handleTrayToggle"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, onMounted, watch } from 'vue'

const settings = reactive({
  openHomePage: true,
  rememberLastPage: false,
  autoSaveInterval: '60',
  closeBehavior: 'close',
  showWindowOnStartup: true,
  enableTray: false
})

// 处理托盘开关变化
const handleTrayToggle = (enabled) => {
  if (enabled) {
    // 如果启用托盘，关闭行为默认设为最小化
    if (settings.closeBehavior === 'close') {
      settings.closeBehavior = 'minimize'
    }
    // TODO: Tauri 托盘功能需要额外实现
    // window.electronAPI?.tray?.enable()
    console.log('Tray feature not yet implemented in Tauri')
  } else {
    // 如果禁用托盘，关闭行为必须设为关闭程序
    settings.closeBehavior = 'close'
    // TODO: Tauri 托盘功能需要额外实现
    // window.electronAPI?.tray?.disable()
    console.log('Tray feature not yet implemented in Tauri')
  }
  saveSettings()
}

// 保存设置到本地存储
const saveSettings = () => {
  localStorage.setItem('app-settings', JSON.stringify(settings))
}

// 从本地存储加载设置
const loadSettings = () => {
  try {
    const saved = localStorage.getItem('app-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      Object.assign(settings, parsed)
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

// 监听设置变化并保存
watch(
  () => ({ ...settings }),
  saveSettings,
  { deep: true }
)

// 组件挂载时加载设置
onMounted(() => {
  loadSettings()
  // Note: Tauri doesn't need window close behavior notification
})

// 监听关闭行为变化
watch(
  () => settings.closeBehavior,
  (newBehavior) => {
    console.log('Close behavior changed to:', newBehavior)
    // Note: Tauri doesn't need window close behavior notification
  }
)
</script>

<style scoped>
@import '@/assets/styles/settings.css';
</style>