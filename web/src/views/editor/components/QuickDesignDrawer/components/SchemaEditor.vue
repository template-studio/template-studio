<template>
  <div class="schema-preview-container">
    <div class="preview-header">
      <div style="display: flex; align-items: center; gap: 12px">
        <n-text strong>Schema</n-text>
        <!-- 格式切换 -->
        <n-button-group size="small">
          <n-button
            :type="schemaFormat === 'json' ? 'primary' : 'default'"
            @click="handleFormatChange('json')"
          >
            JSON
          </n-button>
          <n-button
            :type="schemaFormat === 'yaml' ? 'primary' : 'default'"
            @click="handleFormatChange('yaml')"
          >
            YAML
          </n-button>
        </n-button-group>
      </div>
      <n-space :size="8">
        <n-button size="small" @click="handleReset" quaternary> 重置 </n-button>
        <n-button size="small" @click="handleFormat" quaternary>
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          格式化
        </n-button>
        <n-button size="small" @click="handleCopy" quaternary>
          <template #icon>
            <n-icon><CopyOutline /></n-icon>
          </template>
          复制
        </n-button>
        <n-button size="small" @click="handleImport" quaternary>
          <template #icon>
            <n-icon><CloudUploadOutline /></n-icon>
          </template>
          导入
        </n-button>
        <n-button size="small" @click="handleExport" quaternary>
          <template #icon>
            <n-icon><DownloadOutline /></n-icon>
          </template>
          导出
        </n-button>
        <n-button size="small" @click="handleSync" quaternary type="primary">
          <template #icon>
            <n-icon><SyncOutline /></n-icon>
          </template>
          同步
        </n-button>
      </n-space>
    </div>
    <div class="preview-content">
      <div ref="editorRef" class="schema-editor"></div>
    </div>
  </div>
</template>

<script setup>
  import { onMounted, onUnmounted, watch, nextTick } from 'vue';
  import { NText, NSpace, NButton, NButtonGroup, NIcon } from 'naive-ui';
  import {
    RefreshOutline,
    CopyOutline,
    CloudUploadOutline,
    DownloadOutline,
    SyncOutline,
  } from '@vicons/ionicons5';
  import { useSchemaEditor } from '../composables/useSchemaEditor';

  /**
   * SchemaEditor 组件
   * 负责显示和编辑 JSON/YAML Schema
   */

  // Props
  const props = defineProps({
    schema: {
      type: String,
      default: '{}',
    },
    show: {
      type: Boolean,
      default: true,
    },
    templateId: {
      type: [String, Number],
      default: '',
    },
  });

  // Emits
  const emit = defineEmits(['update:schema', 'sync', 'import', 'reset']);

  // 使用 composable
  const {
    editorRef,
    schemaFormat,
    initEditor,
    updateContent,
    format,
    copy,
    import: importSchema,
    export: exportSchema,
    syncToCanvas,
    destroy,
  } = useSchemaEditor(props, emit);

  // 监听显示状态变化
  watch(
    () => props.show,
    (newVal) => {
      if (newVal) {
        // 延迟初始化，确保 DOM 已渲染
        nextTick(() => {
          if (!editorRef.value) return;
          initEditor();
        });
      }
    }
  );

  // 监听 Schema 变化（不需要deep，因为schema是字符串）
  watch(
    () => props.schema,
    (newSchema, oldSchema) => {
      if (props.show && newSchema !== oldSchema) {
        updateContent();
      }
    }
  );

  // 事件处理
  const handleFormat = () => {
    format();
  };

  const handleCopy = () => {
    copy();
  };

  const handleImport = () => {
    importSchema();
  };

  const handleExport = () => {
    exportSchema();
  };

  const handleSync = () => {
    syncToCanvas();
  };

  const handleReset = () => {
    emit('reset');
  };

  const handleFormatChange = (format) => {
    if (schemaFormat.value !== format) {
      schemaFormat.value = format;
      // 格式切换后重新初始化编辑器
      nextTick(() => {
        initEditor();
      });
    }
  };

  // 生命周期
  onMounted(() => {
    if (props.show) {
      nextTick(() => {
        initEditor();
      });
    }
  });

  onUnmounted(() => {
    destroy();
  });

  // 暴露方法给父组件
  defineExpose({
    syncToCanvas,
  });
</script>

<style scoped>
  .schema-preview-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    border-left: 1px solid #e0e0e0;
    background: #fafafa;
    min-height: 0;
    overflow: hidden;
  }

  .preview-header {
    padding: 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fff;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .preview-content {
    flex: 1;
    overflow-y: auto !important;
    overflow-x: hidden !important;
    background: #fff;
    min-height: 0;
  }

  .schema-editor {
    height: 100%;
    font-family: Monaco, Menlo, 'Ubuntu Mono', Consolas, monospace;
  }

  /* 隐藏 CodeMirror 编辑器的滚动条 */
  :deep(.CodeMirror-vscrollbar),
  :deep(.CodeMirror-hscrollbar) {
    display: none !important;
  }

  :deep(.CodeMirror) {
    scrollbar-width: none !important;
  }

  :deep(.CodeMirror::-webkit-scrollbar) {
    width: 0 !important;
    height: 0 !important;
  }
</style>
