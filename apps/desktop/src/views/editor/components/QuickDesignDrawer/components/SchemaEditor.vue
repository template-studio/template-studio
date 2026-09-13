<template>
  <div class="schema-preview-container">
    <div class="preview-header">
      <div class="ph-left">
        <strong>Schema</strong>
        <div class="ph-fmt">
          <button
            class="sch-act fmt"
            :class="{ on: schemaFormat === 'json' }"
            @click="handleFormatChange('json')"
          >JSON</button>
          <button
            class="sch-act fmt"
            :class="{ on: schemaFormat === 'yaml' }"
            @click="handleFormatChange('yaml')"
          >YAML</button>
        </div>
      </div>
      <div class="ph-right">
        <a-tooltip title="格式化">
          <button class="sch-act" @click="handleFormat"><RefreshOutline :size="15" /></button>
        </a-tooltip>
        <a-tooltip title="复制">
          <button class="sch-act" @click="handleCopy"><CopyOutline :size="15" /></button>
        </a-tooltip>
        <!-- 低频操作收进菜单:重置(破坏性)/导入/导出 -->
        <a-dropdown trigger="click">
          <button class="sch-act" title="更多操作"><MoreOutlined /></button>
          <template #overlay>
            <a-menu @click="onMoreMenu">
              <a-menu-item key="import">
                <CloudUploadOutline :size="14" style="margin-right: 6px" />导入
              </a-menu-item>
              <a-menu-item key="export">
                <DownloadOutline :size="14" style="margin-right: 6px" />导出
              </a-menu-item>
              <a-menu-divider />
              <a-menu-item key="reset" danger>重置(丢弃未同步修改)</a-menu-item>
            </a-menu>
          </template>
        </a-dropdown>
        <a-button size="small" type="primary" @click="handleSync">
          <template #icon><SyncOutline /></template>同步
        </a-button>
      </div>
    </div>
    <div class="preview-content">
      <div ref="editorRef" class="schema-editor"></div>
    </div>
  </div>
</template>

<script setup>
  import { onMounted, onUnmounted, watch, nextTick } from 'vue';
  import {
    RefreshOutline,
    CopyOutline,
    CloudUploadOutline,
    DownloadOutline,
    SyncOutline,
  } from '@/icons/ionicons5';
  import { MoreOutlined } from '@ant-design/icons-vue';
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

  // 「更多」下拉分发
  const onMoreMenu = ({ key }) => {
    if (key === 'import') handleImport();
    else if (key === 'export') handleExport();
    else if (key === 'reset') handleReset();
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
  /* 列宽感知(容器查询):窄列时按钮不再挤压而是阶梯隐藏(规则置于基础规则之后,
     同特异性下靠源序胜出) */
  .schema-preview-container {
    container-type: inline-size;
    height: 100%;
    display: flex;
    flex-direction: column;
    border-left: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-inset-bg, #fafafa);
    min-height: 0;
    overflow: hidden;
  }

  .preview-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-panel-bg, #fff);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px 8px;
    flex-wrap: wrap; /* 列宽不足时右组换行,避免按钮溢出到栏外 */
    flex-shrink: 0;
  }

  .ph-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .ph-fmt { display: flex; gap: 2px; }

  .ph-right { display: flex;
    align-items: center;
    gap: 3px;
    margin-left: auto; /* 换行后仍靠右 */
  }

  /* 图标动作按钮(悬停提示语义) */
  .sch-act {
    height: 24px;
    min-width: 24px;
    padding: 0 4px;
    border: none;
    background: transparent;
    border-radius: 5px;
    cursor: pointer;
    color: var(--editor-muted, #64748b);
    font-size: 13px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .sch-act:hover {
    background: var(--editor-hover-bg, #f1f5f9);
    color: var(--editor-primary, #1b1c1f);
  }

  .sch-act.fmt {
    font-size: 10px;
    font-weight: 600;
    padding: 0 5px;
  }

  .sch-act.fmt.on {
    background: var(--editor-active-bg, #eef0ec);
    color: var(--editor-primary, #1b1c1f);
  }

  .preview-content {
    flex: 1;
    overflow-y: auto !important;
    overflow-x: hidden !important;
    background: var(--editor-panel-bg, #fff);
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

  /* 窄列阶梯隐藏:≤350px 藏图标动作(保同步),≤270px 整组隐藏 */
  @container (max-width: 350px) {
    .ph-right .sch-act {
      display: none;
    }
  }

  @container (max-width: 270px) {
    .ph-right {
      display: none;
    }
  }
</style>
