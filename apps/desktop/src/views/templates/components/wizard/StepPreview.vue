<template>
  <div class="step-panel step-panel-preview">
    <div v-if="loadingPreview" class="loading-ct"><a-spin size="large"><template #description>正在渲染文件预览...</template></a-spin></div>
    <div v-else-if="showPreview" class="preview-fullscreen">
      <div class="preview-main">
        <div class="file-explorer">
          <div class="explorer-header"><span class="explorer-title">模板文件</span></div>
          <div class="explorer-content">
            <a-tree v-if="fileTreeData.length > 0" :tree-data="fileTreeData" :selected-keys="[selectedFileKey]" :expanded-keys="expandedKeys" show-icon @select="$emit('file-select', $event)" @expand="$emit('expand', $event)" />
            <a-empty v-else description="暂无文件" :image="null"><template #description><span style="color:var(--color-text-muted);">暂无文件</span></template></a-empty>
          </div>
        </div>
        <div class="code-preview">
          <div class="file-header">
            <div class="file-info"><FileTextOutlined /><span class="file-name">{{ currentFileName || '未选择文件' }}</span></div>
            <a-button size="small" @click="$emit('copy-file-content')" :disabled="!currentFileContent" type="text"><template #icon><CopyOutlined /></template>复制</a-button>
          </div>
          <div v-if="!currentFileContent" class="no-file-selected">
            <div class="no-file-icon"><FileTextOutlined style="font-size:48px;color:var(--color-text-muted);" /></div>
            <div class="no-file-text">请选择左侧文件进行预览</div>
          </div>
          <div v-else class="code-content"><div class="codemirror-container" ref="codeContainer"></div></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { FileTextOutlined, CopyOutlined } from '@ant-design/icons-vue'

defineProps({
  loadingPreview: Boolean,
  showPreview: Boolean,
  fileTreeData: Array,
  selectedFileKey: String,
  expandedKeys: Array,
  currentFileName: String,
  currentFileContent: String
})

defineEmits(['file-select', 'expand', 'copy-file-content'])

const codeContainer = ref(null)

defineExpose({ codeContainer })
</script>

<style scoped>
.step-panel-preview { padding: 0; overflow: hidden; flex: 1; min-height: 0; height: 100%; display: flex; flex-direction: column; }
.loading-ct { display: flex; justify-content: center; align-items: center; min-height: 300px; }
.preview-fullscreen { flex: 1; display: flex; flex-direction: column; background: var(--color-surface); min-height: 0; height: 100%; overflow: hidden; }
.preview-main { flex: 1; display: flex; overflow: hidden; }
.file-explorer { width: 280px; background: var(--color-background); border-right: 1px solid var(--color-border); display: flex; flex-direction: column; flex-shrink: 0; height: 100%; }
.explorer-header { height: 48px; background: var(--color-surface-3); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; padding: 0 16px; flex-shrink: 0; }
.explorer-title { font-size: 14px; font-weight: 600; color: var(--color-text); }
.explorer-content { flex: 1; overflow-y: auto; padding: 8px; }
.explorer-content :deep(.ant-tree-node-content-wrapper) { padding: 4px 8px; border-radius: 4px; }
.code-preview { flex: 1; display: flex; flex-direction: column; background: var(--color-background); overflow: hidden; height: 100%; }
.file-header { height: 48px; background: var(--color-surface-3); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; justify-content: space-between; padding: 0 16px; flex-shrink: 0; }
.file-info { display: flex; align-items: center; gap: 8px; }
.file-name { font-size: 14px; font-weight: 600; color: var(--color-text); font-family: 'Consolas','Monaco',monospace; }
.file-header :deep(.ant-btn) { height: 28px; padding: 0 12px; font-size: 13px; line-height: 1; border: none; background: transparent; color: var(--color-text-secondary); }
.no-file-selected { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--color-text-muted); font-size: 16px; }
.code-content { flex: 1; overflow: hidden; background: #1e1e1e; }
.codemirror-container { height: 100%; min-height: 400px; }
:deep(.cm-editor) { height: 100% !important; font-size: 14px; outline: none !important; }
:deep(.cm-editor .cm-scroller) { font-family: 'Fira Code','Consolas','Monaco',monospace; overflow: auto !important; height: 100% !important; }
:deep(.cm-editor .cm-line) { padding: 0; }
:deep(.cm-editor .cm-cursor), :deep(.cm-editor .cm-cursor-primary) { display: none !important; }
</style>
