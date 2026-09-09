<template>
  <!-- 停靠式右侧 AI 栏(布局列,编辑区/预览自然收缩);头部星光图标开关 -->
  <div v-show="open" class="ai-dock" :style="{ width: width + 'px' }">
    <div class="ai-resize-handle" @mousedown="startResize"></div>

    <div class="ai-dock-head">
      <div class="ai-dock-title">
        <AiIcon :size="16" class="ai-title-icon" />
        <span>AI 助手</span>
      </div>
      <div class="ai-dock-actions">
        <a-button type="text" size="small" @click="clearChat">清空会话</a-button>
        <a-button type="text" size="small" @click="open = false">
          <template #icon><CloseOutlined /></template>
        </a-button>
      </div>
    </div>

    <div ref="messagesRef" class="ai-messages">
      <div v-if="messages.length === 0" class="ai-empty">
        <AiIcon :size="40" class="ai-empty-icon" />
        <p>问我关于当前模板、变量或代码的问题</p>
        <div class="ai-hints">
          <button v-for="h in hints" :key="h" class="ai-hint-chip" @click="send(h)">
            {{ h }}
          </button>
        </div>
      </div>
      <div v-for="(m, i) in messages" :key="i" class="ai-msg" :class="m.role">
        <div class="ai-bubble">{{ m.content }}</div>
      </div>
      <div v-if="loading" class="ai-msg assistant">
        <div class="ai-bubble ai-typing">
          <span></span><span></span><span></span>
        </div>
      </div>
    </div>

    <div class="ai-input-row">
      <a-textarea
        v-model:value="input"
        placeholder="输入问题,Enter 发送,Shift+Enter 换行"
        :auto-size="{ minRows: 2, maxRows: 6 }"
        :disabled="loading"
        @keydown.enter.exact.prevent="send()"
      />
      <a-button
        type="primary"
        :loading="loading"
        :disabled="!input.trim()"
        class="ai-send-btn"
        @click="send()"
      >
        <template #icon><SendOutlined /></template>
        <span>发送</span>
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { CloseOutlined, SendOutlined } from '@ant-design/icons-vue'
import AiIcon from '@/components/icons/AiIcon.vue'

const props = defineProps({
  currentFilePath: { type: String, default: '' },
  templateVariables: { type: Array, default: () => [] },
})

const WIDTH_KEY = 'editor_ai_dock_width'
const MIN_W = 320
const MAX_W = 640

const open = defineModel('open', { type: Boolean, default: false })
const input = ref('')
const loading = ref(false)
const messages = ref([])
const messagesRef = ref()
const width = ref(Number(localStorage.getItem(WIDTH_KEY)) || 420)

const hints = [
  '这个模板还缺什么变量?',
  '帮我优化当前文件的模板语法',
  '变量命名有什么建议?',
]

watch(messages, () => {
  nextTick(() => {
    if (messagesRef.value) {
      messagesRef.value.scrollTop = messagesRef.value.scrollHeight
    }
  })
}, { deep: true })

// 编辑上下文:当前文件 + 模板变量清单
const buildExtraContext = () => {
  const parts = []
  if (props.currentFilePath) {
    parts.push(`当前打开的文件: ${props.currentFilePath}`)
  }
  const names = props.templateVariables
    .map((v) => v.fieldName || v.name)
    .filter(Boolean)
    .slice(0, 40)
  if (names.length > 0) {
    parts.push(`模板已有变量: ${names.join(', ')}`)
  }
  return parts.join('\n')
}

const send = async (preset) => {
  const text = (preset || input.value).trim()
  if (!text || loading.value) return

  messages.value.push({ role: 'user', content: text })
  input.value = ''
  loading.value = true

  try {
    const history = messages.value
      .slice(0, -1)
      .slice(-20)
      .map((m) => ({ role: m.role, content: m.content }))

    const result = await invoke('ai_chat', {
      message: text,
      templatePath: null,
      projectId: null,
      extraContext: buildExtraContext(),
      history,
    })
    const data = JSON.parse(result)
    messages.value.push({
      role: 'assistant',
      content: data.response || '抱歉,没有拿到有效回复。',
    })
  } catch (e) {
    messages.value.push({
      role: 'assistant',
      content: `调用失败: ${e.message || e}`,
    })
  } finally {
    loading.value = false
  }
}

const clearChat = () => {
  messages.value = []
}

// ---- 左缘拖宽(与编辑器各列手柄同款交互) ----
let dragging = false
let startX = 0
let startW = 0

const onDragMove = (e) => {
  if (!dragging) return
  const next = Math.min(MAX_W, Math.max(MIN_W, startW + (startX - e.clientX)))
  width.value = next
}

const onDragEnd = () => {
  if (!dragging) return
  dragging = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  localStorage.setItem(WIDTH_KEY, String(width.value))
}

const startResize = (e) => {
  dragging = true
  startX = e.clientX
  startW = width.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  e.preventDefault()
}

onMounted(() => {
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
})

onUnmounted(() => {
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
})
</script>

<style scoped>
.ai-dock {
  position: relative;
  height: 100%;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--editor-panel-bg, #fff);
  border-left: 1px solid var(--editor-border, #e0e0e6);
  overflow: hidden;
}

.ai-resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  width: 5px;
  height: 100%;
  cursor: col-resize;
  z-index: 10;
  transition: background-color 0.15s ease;
}

.ai-resize-handle:hover {
  background: var(--editor-accent, #16a34a);
}

.ai-dock-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 12px 16px;
  border-bottom: 1px solid var(--editor-border, #e0e0e6);
  flex-shrink: 0;
}

.ai-dock-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--editor-primary, #1b1c1f);
}

.ai-title-icon {
  color: var(--editor-accent, #16a34a);
}

.ai-dock-actions {
  display: flex;
  gap: 4px;
}

.ai-messages {
  flex: 1;
  overflow-y: auto;
  padding: 18px 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ai-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--editor-muted, #999);
}

.ai-empty-icon {
  opacity: 0.45;
}

.ai-empty p {
  margin: 0;
  font-size: 13px;
}

.ai-hints {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: stretch;
  width: 88%;
}

.ai-hint-chip {
  border: 1px solid var(--editor-border, #e0e0e6);
  background: var(--editor-inset-bg, #f8f9fa);
  color: var(--editor-primary, #333);
  border-radius: 8px;
  padding: 8px 14px;
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s ease;
}

.ai-hint-chip:hover {
  border-color: var(--editor-accent, #16a34a);
}

.ai-msg {
  display: flex;
}

.ai-msg.user {
  justify-content: flex-end;
}

.ai-bubble {
  max-width: 86%;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 13.5px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-word;
}

.ai-msg.user .ai-bubble {
  background: var(--editor-primary, #1b1c1f);
  color: var(--editor-bg, #fff);
  border-bottom-right-radius: 4px;
}

.ai-msg.assistant .ai-bubble {
  background: var(--editor-inset-bg, #f4f4f2);
  color: var(--editor-primary, #333);
  border-bottom-left-radius: 4px;
}

.ai-typing {
  display: flex;
  gap: 4px;
  align-items: center;
  padding: 14px 16px;
}

.ai-typing span {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--editor-muted, #999);
  animation: ai-dot 1.2s infinite ease-in-out;
}

.ai-typing span:nth-child(2) { animation-delay: 0.15s; }
.ai-typing span:nth-child(3) { animation-delay: 0.3s; }

@keyframes ai-dot {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.4; }
  30% { transform: translateY(-4px); opacity: 1; }
}

.ai-input-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px 14px;
  border-top: 1px solid var(--editor-border, #e0e0e6);
  flex-shrink: 0;
}

.ai-send-btn {
  align-self: flex-end;
}
</style>
