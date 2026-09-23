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
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { useUiSettingsStore } from '@/stores/uiSettings'

const uiSettings = useUiSettingsStore()

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
// 开机自启(#717):直接调 autostart 插件写系统注册表,成功后记忆偏好
const handleAutoStartChange = async (checked) => {
  try {
    if (checked) {
      await enable()
    } else {
      await disable()
    }
    uiSettings.general = { ...(uiSettings.general || {}), autoStart: checked }
    message.success(checked ? '已开启开机自启动' : '已关闭开机自启动')
  } catch (error) {
    // 失败回滚开关状态
    settings.autoStart = !checked
    message.error('设置开机自启失败: ' + (error?.message || error))
  }
}

// 启动最小化到托盘(#717):记忆偏好,main 窗口创建参数由前端启动时应用
const handleMinimizeToTrayChange = (checked) => {
  uiSettings.general = { ...(uiSettings.general || {}), minimizeToTray: checked }
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
  // 开机自启即时生效(插件已写系统),其余偏好统一入 uiSettings 持久化
  uiSettings.general = {
    minimizeToTray: settings.minimizeToTray,
    notificationsEnabled: settings.notificationsEnabled,
    notificationSound: settings.notificationSound,
    autoUpdate: settings.autoUpdate,
    updateFrequency: settings.updateFrequency,
    language: settings.language,
  }
  message.success('设置已保存')
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

// 加载设置:偏好从 uiSettings 恢复;开机自启以系统注册表实际状态为准(插件读真值)
const loadSettings = async () => {
  try {
    const g = uiSettings.general || {}
    Object.assign(settings, {
      minimizeToTray: g.minimizeToTray ?? false,
      notificationsEnabled: g.notificationsEnabled ?? true,
      notificationSound: g.notificationSound ?? true,
      autoUpdate: g.autoUpdate ?? true,
      updateFrequency: g.updateFrequency ?? 'weekly',
      language: g.language ?? 'zh-CN',
    })
    settings.autoStart = await isEnabled()
  } catch (error) {
    console.warn('读取自启状态失败(非 Tauri 环境为预期):', error?.message || error)
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