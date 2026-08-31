<template>
  <div class="ai-assistant" :class="{ expanded: isExpanded }">
    <!-- 触发按钮 -->
    <a-button
      v-if="!isExpanded"
      type="primary"
      shape="circle"
      size="large"
      class="trigger-btn"
      @click="isExpanded = true"
    >
      <template #icon><AiIcon :size="14" /></template>
    </a-button>

    <!-- 对话面板 -->
    <div v-show="isExpanded" class="chat-panel">
      <div class="panel-header">
        <div class="header-left">
          <AiIcon :size="18" />
          <span class="title">AI 助手</span>
        </div>
        <a-space>
          <a-button type="text" size="small" @click="clearChat">
            <DeleteOutlined />
          </a-button>
          <a-button type="text" size="small" @click="isExpanded = false">
            <CloseOutlined />
          </a-button>
        </a-space>
      </div>

      <!-- 消息列表 -->
      <div class="messages" ref="messagesRef">
        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['message', msg.role]"
        >
          <div class="message-avatar">
            <AiIcon v-if="msg.role === 'assistant'" :size="14" />
            <UserOutlined v-else />
          </div>
          <div class="message-content">
            <div class="message-text" v-html="renderMarkdown(msg.content)" />
            <!-- 工具调用展示 -->
            <div v-if="msg.toolCalls && msg.toolCalls.length > 0" class="tool-calls">
              <div
                v-for="(tc, tcIndex) in msg.toolCalls"
                :key="tcIndex"
                class="tool-call"
              >
                <a-tag color="blue" size="small">{{ tc.name }}</a-tag>
                <span class="tool-status" :class="{ success: tc.success }">
                  {{ tc.success ? '成功' : '失败' }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 加载状态 -->
        <div v-if="loading" class="message assistant">
          <div class="message-avatar">
            <AiIcon :size="14" />
          </div>
          <div class="message-content">
            <a-spin size="small" />
            <span class="loading-text">思考中...</span>
          </div>
        </div>
      </div>

      <!-- 输入区 -->
      <div class="input-area">
        <a-input
          v-model:value="inputText"
          placeholder="输入问题... (Enter 发送)"
          :disabled="loading"
          @press-enter="sendMessage"
        >
          <template #suffix>
            <a-button
              type="text"
              size="small"
              :disabled="!inputText.trim() || loading"
              @click="sendMessage"
            >
              <SendOutlined />
            </a-button>
          </template>
        </a-input>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import AiIcon from '@/components/icons/AiIcon.vue'
import {

  UserOutlined,
  SendOutlined,
  CloseOutlined,
  DeleteOutlined,
} from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'

interface ToolCallInfo {
  name: string
  success: boolean
  result: string
}

interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  toolCalls?: ToolCallInfo[]
}

const props = defineProps<{
  templatePath?: string
  projectId?: number
}>()

const isExpanded = ref(false)
const inputText = ref('')
const loading = ref(false)
const messages = ref<ChatMessage[]>([])
const messagesRef = ref<HTMLElement>()

// 自动滚动到底部
watch(messages, () => {
  nextTick(() => {
    if (messagesRef.value) {
      messagesRef.value.scrollTop = messagesRef.value.scrollHeight
    }
  })
}, { deep: true })

function renderMarkdown(text: string): string {
  // 简单的 markdown 渲染
  return text
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/\n/g, '<br>')
}

async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || loading.value) return

  // 添加用户消息
  messages.value.push({ role: 'user', content: text })
  inputText.value = ''
  loading.value = true

  try {
    // 调用 Tauri AI 命令
    const result = await invoke<string>('ai_chat', {
      message: text,
      templatePath: props.templatePath || null,
      projectId: props.projectId || null,
    })

    const data = JSON.parse(result)

    // 添加 AI 响应
    const assistantMsg: ChatMessage = {
      role: 'assistant',
      content: data.response || '抱歉，我无法理解您的请求。',
    }

    // 如果有工具调用记录
    if (data.tool_calls && data.tool_calls.length > 0) {
      assistantMsg.toolCalls = data.tool_calls.map((tc: any) => ({
        name: tc.tool_name,
        success: tc.success,
        result: tc.result,
      }))
    }

    messages.value.push(assistantMsg)
  } catch (err: any) {
    messages.value.push({
      role: 'assistant',
      content: `错误: ${err}`,
    })
  } finally {
    loading.value = false
  }
}

function clearChat() {
  messages.value = []
}
</script>

<style scoped>
.ai-assistant {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
}

.trigger-btn {
  width: 48px;
  height: 48px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.chat-panel {
  width: 400px;
  height: 560px;
  background: var(--ant-component-background);
  border: 1px solid var(--ant-border-color);
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ant-border-color);
  background: var(--ant-primary-color);
  color: white;
}

.panel-header .title {
  color: white;
}

.ai-icon {
  font-size: 18px;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message {
  display: flex;
  gap: 8px;
}

.message.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.message.assistant .message-avatar {
  background: var(--ant-primary-color);
  color: white;
}

.message.user .message-avatar {
  background: var(--ant-bg-color-secondary);
  color: var(--ant-text-color);
}

.message-content {
  max-width: 80%;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.5;
}

.message.assistant .message-content {
  background: var(--ant-bg-color-secondary);
}

.message.user .message-content {
  background: var(--ant-primary-color);
  color: white;
}

.tool-calls {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--ant-border-color);
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tool-call {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}

.tool-status {
  color: var(--ant-error-color);
}

.tool-status.success {
  color: var(--ant-success-color);
}

.loading-text {
  margin-left: 8px;
  color: var(--ant-text-color-secondary);
  font-size: 12px;
}

.input-area {
  padding: 12px 16px;
  border-top: 1px solid var(--ant-border-color);
}
</style>
