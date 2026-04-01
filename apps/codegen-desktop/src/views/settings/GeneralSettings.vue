<template>
  <div class="setting-container">
    <!-- 启动设置 -->
    <div class="setting-group">
      <div class="setting-title">启动设置</div>

      <div class="setting-row">
        <div class="setting-row-title">开机自启动</div>
        <a-switch
          v-model:checked="settings.autoStart"
          @change="handleAutoStartChange"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">启动时最小化到系统托盘</div>
        <a-switch
          v-model:checked="settings.minimizeToTray"
          @change="handleMinimizeToTrayChange"
          size="small"
        />
      </div>
    </div>

    <!-- 通知设置 -->
    <div class="setting-group">
      <div class="setting-title">通知设置</div>

      <div class="setting-row">
        <div class="setting-row-title">启用通知</div>
        <a-switch
          v-model:checked="settings.notificationsEnabled"
          @change="handleNotificationsChange"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">通知音效</div>
        <a-switch
          v-model:checked="settings.notificationSound"
          @change="handleNotificationSoundChange"
          :disabled="!settings.notificationsEnabled"
          size="small"
        />
      </div>

      <div class="setting-help-text" v-if="!settings.notificationsEnabled">
        需要先启用通知功能
      </div>
    </div>

    <!-- 更新设置 -->
    <div class="setting-group">
      <div class="setting-title">更新设置</div>

      <div class="setting-row">
        <div class="setting-row-title">自动检查更新</div>
        <a-switch
          v-model:checked="settings.autoUpdate"
          @change="handleAutoUpdateChange"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">检查更新频率</div>
        <a-select
          v-model:value="settings.updateFrequency"
          @change="handleUpdateFrequencyChange"
          :disabled="!settings.autoUpdate"
          size="small"
          style="width: 120px"
        >
          <a-select-option value="daily">每日</a-select-option>
          <a-select-option value="weekly">每周</a-select-option>
          <a-select-option value="monthly">每月</a-select-option>
        </a-select>
      </div>
    </div>

    <!-- 语言设置 -->
    <div class="setting-group">
      <div class="setting-title">语言设置</div>

      <div class="setting-row">
        <div class="setting-row-title">界面语言</div>
        <a-select
          v-model:value="settings.language"
          @change="handleLanguageChange"
          size="small"
          style="width: 120px"
        >
          <a-select-option value="zh-CN">简体中文</a-select-option>
          <a-select-option value="en-US">English</a-select-option>
          <a-select-option value="ja-JP">日本語</a-select-option>
        </a-select>
      </div>

      <div class="setting-help-text">
        语言更改将在重启应用后生效
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="setting-group">
      <div class="setting-row">
        <a-space>
          <a-button type="primary" size="small" @click="saveSettings">
            保存设置
          </a-button>
          <a-button size="small" @click="resetSettings">
            重置默认
          </a-button>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, onMounted } from 'vue'
import { message } from 'ant-design-vue'

// 响应式设置数据
const settings = reactive({
  autoStart: false,
  minimizeToTray: false,
  notificationsEnabled: true,
  notificationSound: true,
  autoUpdate: true,
  updateFrequency: 'weekly',
  language: 'zh-CN'
})

// 处理设置变更
const handleAutoStartChange = (checked) => {
  console.log('Auto start changed:', checked)
  // TODO: 实现开机自启动逻辑
}

const handleMinimizeToTrayChange = (checked) => {
  console.log('Minimize to tray changed:', checked)
  // TODO: 实现最小化到托盘逻辑
}

const handleNotificationsChange = (checked) => {
  console.log('Notifications enabled:', checked)
  if (!checked) {
    settings.notificationSound = false
  }
}

const handleNotificationSoundChange = (checked) => {
  console.log('Notification sound changed:', checked)
}

const handleAutoUpdateChange = (checked) => {
  console.log('Auto update changed:', checked)
}

const handleUpdateFrequencyChange = (value) => {
  console.log('Update frequency changed:', value)
}

const handleLanguageChange = (value) => {
  console.log('Language changed:', value)
  // TODO: 实现语言切换逻辑
}

// 保存设置
const saveSettings = () => {
  try {
    localStorage.setItem('generalSettings', JSON.stringify(settings))
    message.success('设置已保存')
  } catch (error) {
    message.error('保存设置失败')
    console.error('Save settings error:', error)
  }
}

// 重置设置
const resetSettings = () => {
  const defaultSettings = {
    autoStart: false,
    minimizeToTray: false,
    notificationsEnabled: true,
    notificationSound: true,
    autoUpdate: true,
    updateFrequency: 'weekly',
    language: 'zh-CN'
  }

  Object.assign(settings, defaultSettings)
  message.info('设置已重置为默认值')
}

// 加载设置
const loadSettings = () => {
  try {
    const saved = localStorage.getItem('generalSettings')
    if (saved) {
      const savedSettings = JSON.parse(saved)
      Object.assign(settings, savedSettings)
    }
  } catch (error) {
    console.error('Load settings error:', error)
  }
}

// 组件挂载时加载设置
onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.setting-container {
  background: transparent;
}

/* Theme adjustments now handled by global CSS variables */
</style>