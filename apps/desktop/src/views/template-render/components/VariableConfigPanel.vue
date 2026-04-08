<template>
  <div class="step-panel">
    <div class="variables-layout">
      <div class="variables-sidebar">
        <div class="sidebar-header">
          <h3 class="sidebar-title">编辑模式</h3>
        </div>
        <div class="mode-tabs-vertical">
          <div
            class="mode-tab"
            :class="{ active: mode === 'simple' }"
            @click="$emit('update:mode', 'simple')"
          >
            <EditOutlined class="mode-icon" />
            <span class="mode-label">普通模式</span>
          </div>
          <div
            class="mode-tab"
            :class="{ active: mode === 'advanced' }"
            @click="$emit('update:mode', 'advanced')"
          >
            <CodeOutlined class="mode-icon" />
            <span class="mode-label">高级模式</span>
          </div>
        </div>

        <div class="sidebar-section">
          <h3 class="sidebar-title">上下文注入</h3>
          <div class="context-options">
            <a-checkbox
              :checked="injectProject"
              @update:checked="$emit('update:injectProject', $event)"
            >
              项目信息
            </a-checkbox>
            <a-checkbox
              :checked="injectTables"
              @update:checked="$emit('update:injectTables', $event)"
            >
              表信息
            </a-checkbox>
          </div>
          <div v-if="injectProject || injectTables" class="project-selector">
            <a-select
              :value="selectedProjectId"
              placeholder="选择项目"
              :options="projectOptions"
              allow-clear
              size="small"
              style="width: 100%"
              @change="$emit('projectChange', $event)"
            />
          </div>
          <div v-if="injectTables && projectTables.length > 0" class="table-selector">
            <a-checkbox-group
              :value="contextSelectedTables"
              :options="projectTables.map(t => ({ label: t.name, value: t.id }))"
              @change="$emit('update:contextSelectedTables', $event)"
            />
          </div>
        </div>
      </div>

      <div class="variables-content">
        <div v-show="mode === 'simple'" class="normal-mode">
          <VariableForm
            :schema="schema"
            :model-value="variables"
            @update:model-value="$emit('update:variables', $event)"
          />
        </div>
        <div v-show="mode === 'advanced'" class="advanced-mode">
          <div class="editor-wrap">
            <div class="editor-header">
              <span>JSON 编辑器</span>
              <div class="editor-actions">
                <a-button size="small" @click="$emit('formatJson')">格式化</a-button>
                <a-button size="small" type="primary" @click="$emit('syncFromSimple')">同步普通模式</a-button>
              </div>
            </div>
            <div ref="jsonEditorEl" class="json-editor" />
            <div class="editor-footer">
              <span v-if="jsonValid" class="ok">JSON 格式正确</span>
              <span v-else class="err">{{ jsonError }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { EditOutlined, CodeOutlined } from '@ant-design/icons-vue'
import VariableForm from '../VariableForm.vue'

const props = defineProps({
  mode: String,
  schema: Object,
  variables: Object,
  injectProject: Boolean,
  injectTables: Boolean,
  selectedProjectId: [String, Number],
  projectOptions: Array,
  projectTables: Array,
  contextSelectedTables: Array,
  jsonValid: Boolean,
  jsonError: String,
})

const emit = defineEmits([
  'update:mode',
  'update:variables',
  'update:injectProject',
  'update:injectTables',
  'update:contextSelectedTables',
  'projectChange',
  'formatJson',
  'syncFromSimple',
  'editorMounted',
])

const jsonEditorEl = ref(null)

onMounted(() => {
  emit('editorMounted', jsonEditorEl.value)
})
</script>

<style scoped>
.step-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.variables-layout {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.variables-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-header {
  padding: 20px 16px 12px;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.mode-tabs-vertical {
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mode-tab {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.mode-tab:hover {
  background: var(--color-bg-spotlight);
  color: var(--color-text);
}

.mode-tab.active {
  background: var(--color-primary-bg);
  color: var(--color-primary);
}

.mode-icon {
  font-size: 18px;
}

.sidebar-section {
  padding: 12px 16px;
  border-top: 1px solid var(--color-border);
}

.sidebar-section .sidebar-title {
  margin-bottom: 12px;
}

.context-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.context-options :deep(.ant-checkbox-wrapper) {
  display: flex;
  align-items: center;
  min-height: 28px;
  cursor: pointer;
  user-select: none;
}

.project-selector {
  margin-top: 10px;
}

.table-selector {
  margin-top: 8px;
  padding: 8px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  max-height: 150px;
  overflow: auto;
}

.table-selector :deep(.ant-checkbox-group) {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.variables-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
  background: var(--color-bg-container);
}

.normal-mode {
  width: 100%;
}

.advanced-mode {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.editor-wrap {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--color-bg-container);
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 400px;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  font-size: 14px;
  font-weight: 500;
  flex-shrink: 0;
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.json-editor {
  flex: 1;
  min-height: 300px;
  overflow: auto;
}

.json-editor :deep(.cm-editor) {
  height: 100%;
}

.editor-footer {
  padding: 8px 16px;
  background: var(--color-bg-elevated);
  font-size: 12px;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.editor-footer .ok {
  color: var(--color-success);
}

.editor-footer .err {
  color: var(--color-error);
}
</style>
