<template>
  <div class="ai-variable-panel">
    <div class="panel-header">
      <div class="header-left">
        <RobotOutlined class="ai-icon" />
        <span class="title">AI 变量填充</span>
      </div>
      <a-button
        type="primary"
        size="small"
        :loading="analyzing"
        @click="handleAnalyze"
      >
        <SearchOutlined />
        分析变量
      </a-button>
    </div>

    <!-- 变量列表 -->
    <div class="variables-section" v-if="variables.length > 0">
      <div class="section-title">
        <span>检测到 {{ variables.length }} 个变量</span>
        <a-tag :color="allFilled ? 'green' : 'orange'">
          {{ allFilled ? '已全部填充' : `${filledCount}/${variables.length} 已填充` }}
        </a-tag>
      </div>

      <div class="variable-list">
        <div
          v-for="variable in variables"
          :key="variable.name"
          class="variable-item"
          :class="{ filled: variable.value, missing: !variable.value }"
        >
          <div class="var-header">
            <code class="var-name">{{ variable.name }}</code>
            <a-tag size="small" :color="getTypeColor(variable.type)">
              {{ variable.type || 'string' }}
            </a-tag>
            <a-tag v-if="variable.required" size="small" color="red">必填</a-tag>
          </div>
          <div class="var-description" v-if="variable.description">
            {{ variable.description }}
          </div>
          <div class="var-input">
            <a-input
              v-model:value="variable.value"
              :placeholder="variable.default || `输入 ${variable.name} 的值`"
              size="small"
            >
              <template #prefix>
                <CheckCircleOutlined v-if="variable.value" style="color: #52c41a" />
                <ExclamationCircleOutlined v-else style="color: #faad14" />
              </template>
            </a-input>
          </div>
          <div class="var-confidence" v-if="variable.confidence">
            <a-progress
              :percent="Math.round(variable.confidence * 100)"
              :stroke-color="variable.confidence > 0.8 ? '#52c41a' : variable.confidence > 0.5 ? '#faad14' : '#ff4d4f'"
              size="small"
            />
            <span class="confidence-label">置信度</span>
          </div>
        </div>
      </div>
    </div>

    <!-- AI 填充按钮 -->
    <div class="ai-actions" v-if="variables.length > 0">
      <a-space>
        <a-button
          type="primary"
          :loading="filling"
          :disabled="!projectId"
          @click="handleAiFill"
        >
          <ThunderboltOutlined />
          AI 自动填充
        </a-button>
        <a-button
          :disabled="!hasValues"
          @click="handleWrite"
        >
          <SaveOutlined />
          写入 variables.json
        </a-button>
        <a-button @click="handleClear">
          清空
        </a-button>
      </a-space>
    </div>

    <!-- 空状态 -->
    <a-empty
      v-if="variables.length === 0 && !analyzing"
      description="点击「分析变量」开始"
    />

    <!-- 加载状态 -->
    <a-spin v-if="analyzing || filling" class="loading-overlay" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import {
  RobotOutlined,
  SearchOutlined,
  ThunderboltOutlined,
  SaveOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'

interface Variable {
  name: string
  type: string
  title?: string
  description?: string
  required?: boolean
  default?: string
  value?: string
  confidence?: number
}

const props = defineProps<{
  templatePath: string
  projectId?: number
}>()

const emit = defineEmits<{
  (e: 'update:variables', vars: Variable[]): void
  (e: 'write', vars: Variable[]): void
}>()

const variables = ref<Variable[]>([])
const analyzing = ref(false)
const filling = ref(false)

const allFilled = computed(() =>
  variables.value.length > 0 && variables.value.every(v => v.value)
)

const filledCount = computed(() =>
  variables.value.filter(v => v.value).length
)

const hasValues = computed(() =>
  variables.value.some(v => v.value)
)

function getTypeColor(type?: string): string {
  switch (type) {
    case 'number': return 'blue'
    case 'boolean': return 'purple'
    case 'array': return 'cyan'
    case 'object': return 'geekblue'
    default: return 'default'
  }
}

async function handleAnalyze() {
  if (!props.templatePath) {
    message.warning('请先选择模板')
    return
  }

  analyzing.value = true
  try {
    const result = await invoke<string>('ai_analyze_variables', {
      templatePath: props.templatePath,
    })
    const data = JSON.parse(result)
    variables.value = (data.variables || []).map((v: any) => ({
      name: v.name,
      type: v.type || 'string',
      title: v.title,
      description: v.description,
      required: v.required ?? true,
      default: v.default,
      value: '',
      confidence: undefined,
    }))
    emit('update:variables', variables.value)
  } catch (err: any) {
    message.error(`分析失败: ${err}`)
  } finally {
    analyzing.value = false
  }
}

async function handleAiFill() {
  if (!props.projectId) {
    message.warning('请先选择项目')
    return
  }

  filling.value = true
  try {
    const result = await invoke<string>('ai_fill_variables', {
      templatePath: props.templatePath,
      projectId: props.projectId,
    })
    const data = JSON.parse(result)

    // 合并 AI 填充结果
    for (const filled of data.filled || []) {
      const existing = variables.value.find(v => v.name === filled.name)
      if (existing) {
        existing.value = filled.value
        existing.confidence = filled.confidence
      }
    }

    emit('update:variables', variables.value)
    message.success(`AI 已填充 ${data.filled?.length || 0} 个变量`)
  } catch (err: any) {
    message.error(`AI 填充失败: ${err}`)
  } finally {
    filling.value = false
  }
}

async function handleWrite() {
  const vars: Record<string, string> = {}
  for (const v of variables.value) {
    if (v.value) {
      vars[v.name] = v.value
    }
  }

  try {
    await invoke('ai_write_variables', {
      templatePath: props.templatePath,
      variables: JSON.stringify(vars),
    })
    message.success('已写入 variables.json')
    emit('write', variables.value)
  } catch (err: any) {
    message.error(`写入失败: ${err}`)
  }
}

function handleClear() {
  for (const v of variables.value) {
    v.value = ''
    v.confidence = undefined
  }
  emit('update:variables', variables.value)
}
</script>

<style scoped>
.ai-variable-panel {
  position: relative;
  padding: 16px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ai-icon {
  font-size: 18px;
  color: var(--ant-primary-color);
}

.title {
  font-size: 16px;
  font-weight: 600;
}

.variables-section {
  margin-bottom: 16px;
}

.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  font-size: 14px;
  color: var(--ant-text-color-secondary);
}

.variable-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.variable-item {
  padding: 12px;
  border: 1px solid var(--ant-border-color);
  border-radius: 6px;
  transition: all 0.2s;
}

.variable-item.filled {
  border-color: var(--ant-success-color);
  background: var(--ant-success-bg);
}

.variable-item.missing {
  border-color: var(--ant-warning-color);
  background: var(--ant-warning-bg);
}

.var-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.var-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--ant-primary-color);
}

.var-description {
  font-size: 12px;
  color: var(--ant-text-color-secondary);
  margin-bottom: 8px;
}

.var-input {
  margin-bottom: 4px;
}

.var-confidence {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.confidence-label {
  font-size: 12px;
  color: var(--ant-text-color-secondary);
  white-space: nowrap;
}

.ai-actions {
  padding-top: 16px;
  border-top: 1px solid var(--ant-border-color);
}

.loading-overlay {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}
</style>
