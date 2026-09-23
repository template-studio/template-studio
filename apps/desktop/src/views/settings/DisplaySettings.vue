<template>
  <div class="setting-container">
    <!-- 主题设置 -->
    <div class="setting-group">
      <div class="setting-title">主题设置</div>

      <div class="setting-row">
        <div class="setting-row-title">主题模式</div>
        <a-radio-group
          v-model:value="settings.theme"
          @change="handleThemeChange"
          size="small"
        >
          <a-radio value="light">浅色</a-radio>
          <a-radio value="dark">深色</a-radio>
          <a-radio value="auto">跟随系统</a-radio>
        </a-radio-group>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">主题颜色</div>
        <div class="color-picker">
          <div
            v-for="color in themeColors"
            :key="color.name"
            class="color-option"
            :class="{ active: settings.primaryColor === color.value }"
            @click="handlePrimaryColorChange(color.value)"
          >
            <div class="color-preview" :style="{ backgroundColor: color.value }"></div>
            <span class="color-name">{{ color.name }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 界面设置 -->
    <div class="setting-group">
      <div class="setting-title">界面设置</div>

      <div class="setting-row">
        <div class="setting-row-title">显示侧边栏</div>
        <a-switch
          v-model:checked="settings.showSidebar"
          @change="handleSidebarToggle"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">紧凑模式</div>
        <a-switch
          v-model:checked="settings.compactMode"
          @change="handleCompactModeChange"
          size="small"
        />
      </div>

      <div class="setting-help-text">
        紧凑模式将减少界面元素间距，显示更多内容
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">字体大小</div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <a-slider
            v-model:value="settings.fontSize"
            :min="12"
            :max="20"
            :step="1"
            @change="handleFontSizeChange"
            style="width: 150px"
          />
          <span class="font-size-value">{{ settings.fontSize }}px</span>
        </div>
      </div>
    </div>

    <!-- 动画效果 -->
    <div class="setting-group">
      <div class="setting-title">动画效果</div>

      <div class="setting-row">
        <div class="setting-row-title">启用动画</div>
        <a-switch
          v-model:checked="settings.enableAnimations"
          @change="handleAnimationsToggle"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">动画速度</div>
        <a-select
          v-model:value="settings.animationSpeed"
          @change="handleAnimationSpeedChange"
          :disabled="!settings.enableAnimations"
          size="small"
          style="width: 100px"
        >
          <a-select-option value="slow">慢速</a-select-option>
          <a-select-option value="normal">正常</a-select-option>
          <a-select-option value="fast">快速</a-select-option>
        </a-select>
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
          <a-button size="small" @click="previewSettings">
            预览效果
          </a-button>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { useThemeStore } from '@/stores/theme'
import { useUiSettingsStore } from '@/stores/uiSettings'
import { useLayoutStore } from '@/stores/layout'
import { applyDisplay } from '@/composables/useDisplayPrefs'

const themeStore = useThemeStore()
const uiSettings = useUiSettingsStore()
const layoutStore = useLayoutStore()

// 主题颜色选项
const themeColors = [
  { name: '蓝色', value: '#3e7bfa' },
  { name: '绿色', value: '#52c41a' },
  { name: '紫色', value: '#722ed1' },
  { name: '橙色', value: '#fa8c16' },
  { name: '红色', value: '#f5222d' },
  { name: '青色', value: '#13c2c2' }
]

// 响应式设置数据(与 uiSettings.display 同步)
const settings = reactive({
  theme: 'light',
  primaryColor: '#3e7bfa',
  showSidebar: true,
  compactMode: false,
  fontSize: 14,
  enableAnimations: true,
  animationSpeed: 'normal'
})

// 变更即写 store + 应用(不再等"保存";保存按钮仅作确认提示)
const persistAndApply = () => {
  uiSettings.display = { ...settings }
  applyDisplay(settings)
}

// 处理设置变更
const handleThemeChange = (e) => {
  const theme = e.target.value
  if (theme === 'light') {
    themeStore.setTheme('light')
  } else if (theme === 'dark') {
    themeStore.setTheme('dark')
  } else if (theme === 'auto') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    themeStore.setTheme(prefersDark ? 'dark' : 'light')
  }
  persistAndApply()
}

const handlePrimaryColorChange = (color) => {
  settings.primaryColor = color
  persistAndApply()
}

const handleSidebarToggle = (checked) => {
  settings.showSidebar = checked
  layoutStore.setSidebarCollapsed(!checked)
  persistAndApply()
}

const handleCompactModeChange = (checked) => {
  settings.compactMode = checked
  persistAndApply()
}

const handleFontSizeChange = (value) => {
  settings.fontSize = value
  persistAndApply()
}

const handleAnimationsToggle = (checked) => {
  settings.enableAnimations = checked
  persistAndApply()
}

const handleAnimationSpeedChange = (value) => {
  settings.animationSpeed = value
  persistAndApply()
}

// 保存设置(值已实时生效并持久化,此处仅确认提示)
const saveSettings = () => {
  persistAndApply()
  message.success('显示设置已保存')
}

// 重置设置
const resetSettings = () => {
  const defaultSettings = {
    theme: 'light',
    primaryColor: '#3e7bfa',
    showSidebar: true,
    compactMode: false,
    fontSize: 14,
    enableAnimations: true,
    animationSpeed: 'normal'
  }

  Object.assign(settings, defaultSettings)
  layoutStore.setSidebarCollapsed(false)
  themeStore.setTheme('light')
  persistAndApply()
  message.info('显示设置已重置为默认值')
}

// 预览设置(实时应用后预览即所见,保留入口作说明)
const previewSettings = () => {
  applyDisplay(settings)
  message.info('设置已实时应用(本页所有开关即时生效)')
}

// 加载设置
const loadSettings = () => {
  try {
    // 恢复持久化的显示偏好
    Object.assign(settings, {
      primaryColor: uiSettings.display.primaryColor,
      showSidebar: uiSettings.display.showSidebar,
      compactMode: uiSettings.display.compactMode,
      fontSize: uiSettings.display.fontSize,
      enableAnimations: uiSettings.display.enableAnimations,
      animationSpeed: uiSettings.display.animationSpeed,
    })
    // 主题模式跟随 theme store(其自身已持久化)
    settings.theme = themeStore.currentTheme === 'dark' ? 'dark' : 'light'
    applyDisplay(settings)
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

.color-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.color-option {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
  transition: all 0.2s ease;
}

.color-option:hover {
  border-color: var(--color-border-strong);
}

.color-option.active {
  border-color: var(--color-border-strong);
  background: var(--color-active);
}

.color-preview {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.color-name {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.font-size-value {
  font-size: 12px;
  color: var(--color-text-secondary);
  min-width: 35px;
  text-align: center;
}

/* Theme adjustments now handled by global CSS variables */
</style>