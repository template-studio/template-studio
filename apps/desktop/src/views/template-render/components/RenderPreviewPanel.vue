<template>
  <div class="step-panel step-panel-preview">
    <div v-if="rendering" class="loading-ct">
      <a-spin size="large"><template #description>正在渲染文件预览...</template></a-spin>
    </div>
    <div v-else class="preview-layout">
      <div class="file-explorer">
        <div class="explorer-header">
          <span class="explorer-title">模板文件</span>
          <span class="explorer-count">{{ fileCount }} 个文件</span>
        </div>
        <div class="explorer-content">
          <a-tree
            v-if="fileTreeData.length > 0"
            :tree-data="fileTreeData"
            :selected-keys="[selectedFileKey]"
            :expanded-keys="expandedKeys"
            show-icon
            @select="$emit('fileSelect', $event)"
            @expand="$emit('expand', $event)"
          />
          <a-empty v-else description="暂无文件" />
        </div>
      </div>
      <div class="code-preview-pane">
        <div class="file-header">
          <div class="file-info">
            <FileTextOutlined />
            <span class="file-name">{{ currentFileName || '未选择文件' }}</span>
          </div>
          <a-button
            size="small"
            @click="$emit('copyFile')"
            :disabled="!currentFileContent"
            type="text"
          >
            <template #icon><CopyOutlined /></template>
            复制
          </a-button>
        </div>
        <div v-if="!currentFileContent && !renderError" class="no-file-selected">
          <FileTextOutlined style="font-size: 48px; color: var(--color-text-muted)" />
          <div>请选择左侧文件进行预览</div>
        </div>
        <a-alert
          v-if="renderError"
          :message="renderError"
          type="error"
          show-icon
          style="margin: 16px"
        />
        <div v-if="currentFileContent" class="code-content">
          <div ref="codeContainer" class="codemirror-container" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { FileTextOutlined, CopyOutlined } from '@ant-design/icons-vue'

defineProps({
  rendering: Boolean,
  fileTreeData: Array,
  selectedFileKey: String,
  expandedKeys: Array,
  currentFileName: String,
  currentFileContent: String,
  renderError: String,
  fileCount: Number,
})

const emit = defineEmits(['fileSelect', 'expand', 'copyFile', 'editorMounted'])

const codeContainer = ref(null)

onMounted(() => {
  emit('editorMounted', codeContainer.value)
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

.step-panel-preview {
  padding: 0;
  overflow: hidden;
}

.preview-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.file-explorer {
  width: 280px;
  background: var(--color-bg-elevated);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.explorer-header {
  height: 48px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.explorer-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}

.explorer-count {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.explorer-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.code-preview-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-header {
  height: 48px;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  flex-shrink: 0;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  font-family: 'Consolas', 'Monaco', monospace;
}

.no-file-selected {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--color-text-muted);
  font-size: 14px;
}

.code-content {
  flex: 1;
  overflow: hidden;
  background: var(--color-bg-container);
}

.codemirror-container {
  height: 100%;
}

.codemirror-container :deep(.cm-editor) {
  height: 100% !important;
}

.loading-ct {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
