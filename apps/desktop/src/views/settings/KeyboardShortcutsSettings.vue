<template>
  <div class="setting-container">
    <div class="setting-group">
      <div class="setting-title">快捷键配置</div>
      <div class="setting-help-text">
        自定义全局快捷键，提高工作效率
      </div>

      <div class="shortcuts-list">
        <div v-for="shortcut in shortcuts" :key="shortcut.id" class="shortcut-item">
          <div class="shortcut-info">
            <span class="shortcut-name">{{ shortcut.name }}</span>
            <span class="shortcut-desc">{{ shortcut.description }}</span>
          </div>
          <div class="shortcut-key">
            <a-input
              v-model:value="shortcut.key"
              size="small"
              readonly
              @click="startRecording(shortcut)"
              :class="{ recording: recordingId === shortcut.id }"
              style="width: 150px; text-align: center;"
            />
            <a-button
              v-if="shortcut.key !== shortcut.defaultKey"
              type="link"
              size="small"
              @click="resetShortcut(shortcut)"
            >
              重置
            </a-button>
          </div>
        </div>
      </div>
    </div>

    <div class="setting-group">
      <div class="setting-row">
        <a-space>
          <a-button type="primary" size="small" @click="saveShortcuts">
            保存设置
          </a-button>
          <a-button size="small" @click="resetAllShortcuts">
            恢复默认
          </a-button>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { message } from 'ant-design-vue'

// 快捷键列表
const shortcuts = ref([
  { id: 'newProject', name: '新建项目', description: '创建新项目', key: 'Ctrl+N', defaultKey: 'Ctrl+N' },
  { id: 'openProject', name: '打开项目', description: '打开现有项目', key: 'Ctrl+O', defaultKey: 'Ctrl+O' },
  { id: 'save', name: '保存', description: '保存当前配置', key: 'Ctrl+S', defaultKey: 'Ctrl+S' },
  { id: 'search', name: '全局搜索', description: '打开搜索面板', key: 'Ctrl+K', defaultKey: 'Ctrl+K' },
  { id: 'toggleSidebar', name: '切换侧边栏', description: '显示/隐藏侧边栏', key: 'Ctrl+B', defaultKey: 'Ctrl+B' },
  { id: 'newTable', name: '新建表', description: '在项目中创建新表', key: 'Ctrl+T', defaultKey: 'Ctrl+T' },
  { id: 'aiGenerate', name: 'AI 生成', description: '使用 AI 生成代码', key: 'Ctrl+G', defaultKey: 'Ctrl+G' },
  { id: 'refresh', name: '刷新', description: '刷新当前页面数据', key: 'F5', defaultKey: 'F5' },
  { id: 'settings', name: '打开设置', description: '打开设置页面', key: 'Ctrl+,', defaultKey: 'Ctrl+,' },
  { id: 'closeTab', name: '关闭标签', description: '关闭当前标签页', key: 'Ctrl+W', defaultKey: 'Ctrl+W' }
])

// 正在录制的快捷键 ID
const recordingId = ref(null)

// 开始录制快捷键
const startRecording = (shortcut) => {
  recordingId.value = shortcut.id
}

// 处理键盘按下事件
const handleKeyDown = (e) => {
  if (!recordingId.value) return

  e.preventDefault()
  e.stopPropagation()

  // 构建快捷键字符串
  const parts = []
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
  if (e.shiftKey) parts.push('Shift')
  if (e.altKey) parts.push('Alt')

  // 获取按键名称
  let key = e.key
  if (key === ' ') key = 'Space'
  else if (key === 'Escape') key = 'Esc'
  else if (key === 'Delete') key = 'Del'
  else if (key === 'ArrowUp') key = 'Up'
  else if (key === 'ArrowDown') key = 'Down'
  else if (key === 'ArrowLeft') key = 'Left'
  else if (key === 'ArrowRight') key = 'Right'

  // 忽略单独的修饰键
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(key)) return

  parts.push(key)
  const shortcutKey = parts.join('+')

  // 更新快捷键
  const shortcut = shortcuts.value.find(s => s.id === recordingId.value)
  if (shortcut) {
    shortcut.key = shortcutKey
  }

  recordingId.value = null
}

// 重置单个快捷键
const resetShortcut = (shortcut) => {
  shortcut.key = shortcut.defaultKey
}

// 保存快捷键设置
const saveShortcuts = () => {
  try {
    localStorage.setItem('keyboardShortcuts', JSON.stringify(shortcuts.value))
    message.success('快捷键设置已保存')
  } catch (error) {
    message.error('保存失败')
  }
}

// 恢复所有默认快捷键
const resetAllShortcuts = () => {
  shortcuts.value.forEach(s => {
    s.key = s.defaultKey
  })
  message.info('已恢复默认快捷键')
}

// 加载设置
const loadShortcuts = () => {
  try {
    const saved = localStorage.getItem('keyboardShortcuts')
    if (saved) {
      const savedShortcuts = JSON.parse(saved)
      savedShortcuts.forEach(saved => {
        const shortcut = shortcuts.value.find(s => s.id === saved.id)
        if (shortcut) {
          shortcut.key = saved.key
        }
      })
    }
  } catch (error) {
    console.error('Load shortcuts error:', error)
  }
}

onMounted(() => {
  loadShortcuts()
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.setting-container {
  background: transparent;
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--color-surface);
  border-radius: var(--border-radius-md);
  border: 1px solid var(--color-border);
}

.shortcut-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.shortcut-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text);
}

.shortcut-desc {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.shortcut-key {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shortcut-key :deep(.ant-input) {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  cursor: pointer;
}

.shortcut-key :deep(.ant-input.recording) {
  border-color: var(--color-border-strong);
  box-shadow: 0 0 0 2px rgba(28, 29, 31, 0.14);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 2px rgba(28, 29, 31, 0.14);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(24, 144, 255, 0.4);
  }
}
</style>
