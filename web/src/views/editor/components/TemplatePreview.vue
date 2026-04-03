<template>
  <div
    class="template-preview"
    :class="{ collapsed: isCollapsed }"
    :style="{ width: panelWidth + 'px' }"
  >
    <div class="preview-header">
      <div class="file-info">
        <div class="file-path">{{ currentFilePath }}</div>
      </div>
      <div class="preview-meta">
        <!-- 引擎标识 -->
        <n-tooltip trigger="hover" v-if="renderServiceReady">
          <template #trigger>
            <span class="engine-tag" :class="{ wasm: isUsingWasm }">
              {{ isUsingWasm ? 'WASM' : 'Cloud' }}
            </span>
          </template>
          {{ isUsingWasm ? '本地 WASM 引擎渲染（支持离线）' : '后端服务渲染' }}
        </n-tooltip>
        <!-- 渲染耗时 -->
        <span v-if="renderStats" class="render-time"> {{ renderStats.duration }}ms </span>
      </div>
      <div class="preview-actions">
        <n-button size="small" @click="copyContent" v-if="!isCollapsed">
          <template #icon>
            <n-icon><Copy /></n-icon>
          </template>
          复制
        </n-button>
        <n-button
          size="small"
          quaternary
          @click="toggleCollapse"
          class="collapse-btn"
          :class="{ collapsed: isCollapsed }"
        >
          <template #icon>
            <n-icon>
              <ChevronForward v-if="!isCollapsed" />
              <ChevronBack v-else />
            </n-icon>
          </template>
        </n-button>
      </div>
    </div>

    <div v-show="!isCollapsed" class="preview-content" @contextmenu.prevent>
      <!-- 错误显示区域 -->
      <div v-if="renderError" class="error-display">
        <div class="error-header">
          <n-icon size="18" color="#ff4757">
            <AlertCircle />
          </n-icon>
          <span class="error-title">模板渲染错误</span>
        </div>
        <div class="error-body">
          <div class="error-type" :class="`error-type-${renderError.type}`">
            {{ getErrorTypeText(renderError.type) }}
            <span v-if="renderError.line" class="error-location">
              第 {{ renderError.line }} 行
              <span v-if="renderError.column">第 {{ renderError.column }} 列</span>
            </span>
          </div>
          <div class="error-message">{{ renderError.message }}</div>
          <div v-if="renderError.context" class="error-context">
            <div class="context-label">错误上下文:</div>
            <pre class="context-content">{{ renderError.context }}</pre>
          </div>
          <div v-if="renderError.suggestion" class="error-suggestion">
            <div class="suggestion-label">修复建议:</div>
            <div class="suggestion-content">{{ renderError.suggestion }}</div>
          </div>
        </div>
      </div>

      <!-- 正常内容显示区域 -->
      <div v-else class="file-content" ref="previewEditorRef" :key="contentKey"></div>
    </div>

    <!-- 拖动调整器 -->
    <div class="resize-handle" @mousedown="startResize" :class="{ resizing: isResizing }"></div>
  </div>
</template>

<script setup>
  import { ref, computed, watch, onMounted, nextTick } from 'vue';
  import { NButton, NIcon, NTooltip, useMessage } from 'naive-ui';
  import { Copy, ChevronForward, ChevronBack, AlertCircle } from '@vicons/ionicons5';
  import {
    EditorView,
    highlightActiveLine,
    highlightActiveLineGutter,
    lineNumbers,
  } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { dracula } from '@uiw/codemirror-theme-dracula';
  import { useRenderService } from '@/composables/useRenderService';

  // 语言支持
  import { javascript } from '@codemirror/lang-javascript';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { json } from '@codemirror/lang-json';
  import { markdown } from '@codemirror/lang-markdown';
  import { python } from '@codemirror/lang-python';
  import { java } from '@codemirror/lang-java';
  import { cpp } from '@codemirror/lang-cpp';
  import { rust } from '@codemirror/lang-rust';
  import { go } from '@codemirror/lang-go';
  import { sql } from '@codemirror/lang-sql';
  import { xml } from '@codemirror/lang-xml';
  import { yaml } from '@codemirror/lang-yaml';
  import { vue } from '@codemirror/lang-vue';

  const props = defineProps({
    currentFile: {
      type: Object,
      default: null,
    },
    templateId: {
      type: [String, Number],
      default: null,
    },
    filePath: {
      type: String,
      default: null,
    },
    variables: {
      type: Object,
      default: null,
    },
    fileContent: {
      type: String,
      default: '',
    },
  });

  const message = useMessage();

  const previewEditorRef = ref(null);
  let previewEditor = null;

  // 渲染服务（WASM 引擎支持）
  const {
    isReady: renderServiceReady,
    currentEngine,
    isUsingWasm,
    render: renderWithEngine,
    renderFile: renderFileWithEngine,
  } = useRenderService();

  // 渲染相关状态
  const loading = ref(false);
  const renderedContent = ref('');
  const fileName = ref('');
  const renderError = ref(null);
  const contentKey = ref(0); // 用于强制重新渲染内容区域
  const renderStats = ref(null); // 渲染统计信息
  let renderDebounceTimer = null; // 防抖定时器
  let isRendering = false; // 渲染锁，防止重复渲染

  // 语言映射
  const languageMap = {
    js: javascript(),
    javascript: javascript(),
    ts: javascript({ typescript: true }),
    typescript: javascript({ typescript: true }),
    jsx: javascript({ jsx: true }),
    tsx: javascript({ typescript: true, jsx: true }),
    vue: vue(),
    html: html(),
    htm: html(),
    css: css(),
    scss: css(),
    sass: css(),
    less: css(),
    json: json(),
    md: markdown(),
    markdown: markdown(),
    py: python(),
    python: python(),
    java: java(),
    cpp: cpp(),
    cc: cpp(),
    cxx: cpp(),
    c: cpp(),
    rs: rust(),
    rust: rust(),
    go: go(),
    sql: sql(),
    xml: xml(),
    yaml: yaml(),
    yml: yaml(),
  };

  // 获取语言扩展
  function getLanguageExtension(filename) {
    const ext = filename.split('.').pop()?.toLowerCase();
    return languageMap[ext] || null;
  }

  // 获取错误类型文本
  function getErrorTypeText(type) {
    const typeMap = {
      parse_error: '解析错误',
      execute_error: '执行错误',
      function_error: '函数错误',
      variable_error: '变量错误',
      unknown_error: '未知错误',
    };
    return typeMap[type] || '未知错误';
  }

  // 初始化预览编辑器
  function initializePreviewEditor() {
    if (!previewEditorRef.value) return;

    const state = EditorState.create({
      doc: '',
      extensions: [
        // Dracula 主题（与主编辑器一致）
        dracula,
        // 语法高亮
        syntaxHighlighting(defaultHighlightStyle),
        // 行号
        lineNumbers(),
        // 当前行高亮
        highlightActiveLine(),
        // 当前行行号高亮
        highlightActiveLineGutter(),
        // 确保滚动正常工作
        EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
        // 强制启用滚动
        EditorView.theme({
          '&': { height: '100%' },
          '.cm-scroller': {
            overflow: 'auto !important',
            height: '100% !important',
          },
        }),
      ],
    });

    previewEditor = new EditorView({
      state,
      parent: previewEditorRef.value,
    });
  }

  // 折叠状态
  const isCollapsed = ref(true);

  // 面板宽度和拖动状态
  const panelWidth = ref(400);
  const isResizing = ref(false);
  const startX = ref(0);
  const startWidth = ref(0);

  // 从本地存储加载宽度设置
  const loadPanelWidth = () => {
    const savedWidth = localStorage.getItem('template-preview-width');
    if (savedWidth) {
      const width = parseInt(savedWidth);
      if (width >= 120 && width <= 2000) {
        panelWidth.value = width;
      }
    }
  };

  // 保存宽度设置到本地存储
  const savePanelWidth = (width) => {
    localStorage.setItem('template-preview-width', width.toString());
  };

  // 计算文件路径
  const currentFilePath = computed(() => {
    if (fileName.value) {
      return `预览: ${fileName.value}`;
    }
    return props.currentFile?.filePath || props.currentFile?.fileName || '未选择文件';
  });

  // 更新预览内容
  function updatePreviewContent() {
    if (!previewEditor) {
      return;
    }

    let content = '';
    let currentFileName = '';

    // 如果有渲染后的内容，优先显示
    if (renderedContent.value) {
      content = renderedContent.value;
      currentFileName = fileName.value;
    } else if (props.currentFile) {
      content = props.currentFile.fileContent || props.currentFile.content || '';
      currentFileName = props.currentFile.fileName || props.currentFile.name || '';
    }
    // 获取语言扩展
    const languageExt = getLanguageExtension(currentFileName);

    // 更新编辑器状态，包括语言支持
    const newState = EditorState.create({
      doc: content,
      extensions: [
        // Dracula 主题（与主编辑器一致）
        dracula,
        // 语法高亮
        syntaxHighlighting(defaultHighlightStyle),
        // 行号
        lineNumbers(),
        // 当前行高亮
        highlightActiveLine(),
        // 当前行行号高亮
        highlightActiveLineGutter(),
        // 确保滚动正常工作
        EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
        // 强制启用滚动
        EditorView.theme({
          '&': { height: '100%' },
          '.cm-scroller': {
            overflow: 'auto !important',
            height: '100% !important',
          },
        }),
        // 添加语言支持
        ...(languageExt ? [languageExt] : []),
      ],
    });

    previewEditor.setState(newState);
  }

  // 复制内容
  function copyContent() {
    if (!previewEditor) return;

    const content = previewEditor.state.doc.toString();
    navigator.clipboard.writeText(content).then(() => {
      message.success('内容已复制到剪贴板');
    });
  }

  // 切换折叠状态
  function toggleCollapse() {
    isCollapsed.value = !isCollapsed.value;
    if (isCollapsed.value) {
      panelWidth.value = 40;
    } else {
      // 展开时使用保存的宽度或默认宽度
      loadPanelWidth();

      // 如果有预览文件路径，自动渲染当前文件
      if (props.filePath) {
        console.log('预览面板展开，自动渲染当前文件');
        debouncedRender();
      }
    }
  }

  // 展开面板（如果已折叠）
  function expandPanel() {
    if (isCollapsed.value) {
      isCollapsed.value = false;
      loadPanelWidth();

      // 如果有预览文件路径，自动渲染当前文件
      if (props.filePath) {
        console.log('预览面板展开，自动渲染当前文件');
        debouncedRender();
      }
    }
  }

  // 开始拖动调整
  function startResize(e) {
    if (isCollapsed.value) return;

    e.preventDefault();
    e.stopPropagation();
    isResizing.value = true;
    startX.value = e.clientX;
    startWidth.value = panelWidth.value;

    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  // 处理拖动调整
  function handleResize(e) {
    if (!isResizing.value) return;

    e.preventDefault();
    e.stopPropagation();

    const deltaX = startX.value - e.clientX;
    const newWidth = startWidth.value + deltaX;

    // 动态计算最大宽度（基于可用空间）
    const container = document.querySelector('.edit-main');
    const maxAvailableWidth = container ? container.clientWidth - 200 : 1200; // 为编辑器保留至少200px
    const maxWidth = Math.min(Math.max(maxAvailableWidth, 400), 2000); // 最大不超过2000px，最少400px

    // 限制最小和最大宽度
    if (newWidth >= 120 && newWidth <= maxWidth) {
      panelWidth.value = newWidth;
    }
  }

  // 停止拖动调整
  function stopResize() {
    isResizing.value = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';

    // 保存当前宽度到本地存储
    savePanelWidth(panelWidth.value);
  }

  // 渲染模板（带防抖和锁机制）
  const renderTemplateContent = async () => {
    if (!props.filePath) {
      return;
    }

    // 如果正在渲染，跳过
    if (isRendering) {
      console.log('[TemplatePreview] 已有渲染任务进行中，跳过');
      return;
    }

    // 设置渲染锁
    isRendering = true;

    // 清空之前的错误状态
    renderError.value = null;
    renderedContent.value = '';
    renderStats.value = null;

    // 确保有测试变量，如果没有则使用空对象
    const variables = props.variables || {};

    // 获取当前文件内容（用于 WASM 渲染）
    // 优先使用 fileContent prop，然后是 currentFile 中的内容
    const fileContent =
      props.fileContent || props.currentFile?.fileContent || props.currentFile?.content || '';

    try {
      loading.value = true;
      const startTime = performance.now();

      // 如果使用 WASM 引擎且有文件内容，尝试 WASM 渲染
      if (isUsingWasm.value && renderServiceReady.value && fileContent) {
        try {
          const result = await renderWithEngine(fileContent, variables);

          if (result.success) {
            // WASM 渲染成功
            const wasError = renderError.value !== null;

            renderedContent.value = result.content;
            fileName.value = props.filePath.split('/').pop() || props.filePath;
            renderError.value = null;
            renderStats.value = {
              engine: currentEngine.value,
              duration: result.duration || Math.round(performance.now() - startTime),
            };

            // 如果之前是错误状态，强制重新渲染DOM
            if (wasError) {
              contentKey.value++;
            }

            // 确保预览编辑器重新初始化
            nextTick(() => {
              if (!previewEditor && previewEditorRef.value) {
                initializePreviewEditor();
              }
              setTimeout(() => {
                updatePreviewContent();
              }, 100);
            });

            return; // WASM 渲染成功，直接返回
          } else if (result.error) {
            // WASM 渲染失败，但有错误信息
            renderError.value = {
              type: result.error.type || 'unknown_error',
              message: result.error.message,
              line: result.error.line,
              column: result.error.column,
              context: result.error.context,
              suggestion: result.error.suggestion,
            };
            fileName.value = props.filePath.split('/').pop() || props.filePath;
            renderStats.value = {
              engine: currentEngine.value,
              duration: result.duration || Math.round(performance.now() - startTime),
            };

            // 销毁预览编辑器（因为要显示错误界面）
            if (previewEditor) {
              previewEditor.destroy();
              previewEditor = null;
            }

            message.error(`模板渲染错误: ${result.error.message}`);
            return;
          }
        } catch (wasmError) {
          console.warn('[TemplatePreview] WASM render failed, falling back to backend:', wasmError);
          // WASM 渲染失败，继续使用后端引擎
        }
      }

      // 使用后端引擎（通过 renderFile 方法，支持文件依赖）
      const result = await renderFileWithEngine(props.templateId, props.filePath, variables);

      if (result.success) {
        const wasError = renderError.value !== null;

        renderedContent.value = result.content;
        fileName.value = props.filePath.split('/').pop() || props.filePath;
        renderError.value = null;
        renderStats.value = {
          engine: 'Backend',
          duration: result.duration || Math.round(performance.now() - startTime),
        };

        // 如果之前是错误状态，强制重新渲染DOM
        if (wasError) {
          contentKey.value++;
        }

        // 确保预览编辑器重新初始化
        nextTick(() => {
          if (!previewEditor && previewEditorRef.value) {
            initializePreviewEditor();
          }
          setTimeout(() => {
            updatePreviewContent();
          }, 100);
        });
      } else if (result.error) {
        // 渲染失败
        renderError.value = {
          type: result.error.type || 'unknown_error',
          message: result.error.message,
          line: result.error.line,
          column: result.error.column,
          context: result.error.context,
        };
        fileName.value = props.filePath.split('/').pop() || props.filePath;
        renderStats.value = {
          engine: 'Backend',
          duration: result.duration || Math.round(performance.now() - startTime),
        };

        // 销毁预览编辑器
        if (previewEditor) {
          previewEditor.destroy();
          previewEditor = null;
        }

        message.error(`模板渲染错误: ${result.error.message}`);
      }
    } catch (error) {
      console.error('渲染模板失败:', error);
      message.error('渲染模板失败: ' + (error.message || '未知错误'));
      renderError.value = {
        type: 'unknown_error',
        message: error.message || '网络请求失败',
        line: 0,
        column: 0,
        context: '',
        suggestion: '请检查网络连接或稍后重试',
      };
    } finally {
      loading.value = false;
      isRendering = false;
    }
  };

  // 带防抖的渲染方法
  const debouncedRender = () => {
    if (renderDebounceTimer) {
      clearTimeout(renderDebounceTimer);
    }
    renderDebounceTimer = setTimeout(() => {
      renderTemplateContent();
    }, 100);
  };

  // 监听当前文件变化
  watch(
    () => props.currentFile,
    () => {
      nextTick(() => {
        updatePreviewContent();
      });
    }
  );

  // 监听文件路径和测试变量变化，仅在面板展开时自动渲染
  watch(
    [() => props.filePath, () => props.variables],
    async () => {
      if (props.filePath && !isCollapsed.value) {
        console.log('文件路径或变量变化，面板已展开，执行渲染');
        debouncedRender();
      } else if (props.filePath && isCollapsed.value) {
        console.log('文件路径或变量变化，但面板已折叠，跳过渲染');
      }
    },
    { deep: true, immediate: false }
  );

  // 初始化预览编辑器
  onMounted(() => {
    // 加载保存的宽度设置
    loadPanelWidth();

    if (previewEditorRef.value) {
      const state = EditorState.create({
        doc: '',
        extensions: [
          // Dracula 主题（与主编辑器一致）
          dracula,
          // 语法高亮
          syntaxHighlighting(defaultHighlightStyle),
          // 行号
          lineNumbers(),
          // 当前行高亮
          highlightActiveLine(),
          // 当前行行号高亮
          highlightActiveLineGutter(),
          // 确保滚动正常工作
          EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
          // 强制启用滚动
          EditorView.theme({
            '&': { height: '100%' },
            '.cm-scroller': {
              overflow: 'auto !important',
              height: '100% !important',
            },
          }),
        ],
      });

      previewEditor = new EditorView({
        state,
        parent: previewEditorRef.value,
      });

      // 初始化内容
      if (props.currentFile) {
        updatePreviewContent();
      }
    }
  });

  // 暴露方法和状态给父组件
  defineExpose({
    toggleCollapse,
    expandPanel,
    renderTemplateContent: debouncedRender, // 暴露防抖版本
    isCollapsed,
  });
</script>

<style scoped>
  .template-preview {
    background: #ffffff;
    border-left: 1px solid var(--editor-border, #e2e8f0);
    display: flex;
    flex-direction: column;
    height: 100%;
    transition: all 0.2s ease;
    position: relative;
    flex-shrink: 0;
  }

  .template-preview.collapsed {
    width: 40px !important;
  }

  .template-preview.collapsed .preview-header {
    padding: 0 6px;
    justify-content: center;
  }

  .template-preview.collapsed .preview-meta {
    display: none;
  }

  .template-preview.collapsed .file-info {
    display: none;
  }

  .template-preview.collapsed .resize-handle {
    display: none;
  }

  .preview-header {
    height: 48px;
    padding: 0 12px;
    border-bottom: 1px solid var(--editor-border, #e2e8f0);
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
    transition: padding 0.2s ease;
    background: var(--editor-panel-bg, #fafbfc);
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-path {
    font-size: 12px;
    color: var(--editor-muted, #64748b);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .preview-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-right: 8px;
  }

  .engine-tag {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: default;
    background: var(--editor-hover-bg, #f1f5f9);
    color: var(--editor-muted, #64748b);
    transition: all 0.15s;
  }

  .engine-tag:hover {
    background: #e2e8f0;
  }

  .engine-tag.wasm {
    background: #ecfdf5;
    color: var(--editor-accent, #22c55e);
  }

  .engine-tag.wasm:hover {
    background: #d1fae5;
  }

  .render-time {
    font-size: 11px;
    color: var(--editor-muted, #94a3b8);
    padding: 2px 6px;
    background: var(--editor-hover-bg, #f1f5f9);
    border-radius: 4px;
  }

  .preview-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .collapse-btn {
    transition: all 0.3s ease;
  }

  .collapse-btn.collapsed {
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 4px;
  }

  .collapse-btn.collapsed:hover {
    background-color: #f0f0f0;
  }

  .preview-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .file-content {
    flex: 1;
    overflow: hidden;
  }

  .resize-handle {
    position: absolute;
    left: -8px;
    top: 0;
    width: 16px;
    height: 100%;
    cursor: col-resize;
    background: transparent;
    transition: all 0.2s;
    z-index: 10;
    pointer-events: auto;
    user-select: none;
    touch-action: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .resize-handle:before {
    content: '';
    width: 2px;
    height: 40px;
    background: transparent;
    border-radius: 1px;
    transition: all 0.2s;
  }

  .resize-handle:hover {
    background-color: rgba(34, 197, 94, 0.08);
  }

  .resize-handle:hover:before {
    background-color: var(--editor-accent, #22c55e);
    height: 60px;
  }

  .resize-handle.resizing {
    background-color: rgba(34, 197, 94, 0.15);
  }

  .resize-handle.resizing:before {
    background-color: var(--editor-accent, #22c55e);
    height: 80px;
  }

  .template-preview.collapsed .resize-handle {
    display: none;
  }

  /* 错误显示样式 */
  .error-display {
    height: 100%;
    padding: 16px;
    background: #fff;
    overflow-y: auto;
  }

  .error-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #f0f0f0;
  }

  .error-title {
    font-size: 16px;
    font-weight: 600;
    color: #ff4757;
  }

  .error-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .error-type {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 6px;
    font-weight: 500;
    font-size: 14px;
  }

  .error-type-parse_error {
    background: #fff3cd;
    color: #856404;
    border: 1px solid #ffeaa7;
  }

  .error-type-execute_error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }

  .error-type-function_error {
    background: #d1ecf1;
    color: #0c5460;
    border: 1px solid #bee5eb;
  }

  .error-type-variable_error {
    background: #e2e3e5;
    color: #383d41;
    border: 1px solid #d6d8db;
  }

  .error-type-unknown_error {
    background: #f8f9fa;
    color: #495057;
    border: 1px solid #dee2e6;
  }

  .error-location {
    font-size: 12px;
    opacity: 0.8;
  }

  .error-message {
    padding: 12px;
    background: #f8f9fa;
    border-left: 4px solid #ff4757;
    border-radius: 4px;
    font-family: 'Courier New', monospace;
    font-size: 13px;
    color: #2d3436;
    line-height: 1.5;
  }

  .error-context {
    margin-top: 8px;
  }

  .context-label {
    font-size: 13px;
    font-weight: 500;
    color: #666;
    margin-bottom: 8px;
  }

  .context-content {
    background: #2d3436;
    color: #ddd;
    padding: 12px;
    border-radius: 4px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.4;
    overflow-x: auto;
    white-space: pre;
  }

  .error-suggestion {
    margin-top: 8px;
  }

  .suggestion-label {
    font-size: 13px;
    font-weight: 500;
    color: #0984e3;
    margin-bottom: 8px;
  }

  .suggestion-content {
    background: #e3f2fd;
    color: #1565c0;
    padding: 12px;
    border-radius: 4px;
    font-size: 13px;
    line-height: 1.5;
    border-left: 4px solid #2196f3;
  }
</style>
