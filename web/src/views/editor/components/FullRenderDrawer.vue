<template>
  <n-drawer v-model:show="showDrawer" :width="'60vw'" placement="right" @update:show="handleClose">
    <n-drawer-content
      :title="drawerTitle"
      closable
      body-content-style="padding: 0; display: flex; flex-direction: column;"
    >
      <!-- 顶部工具栏 -->
      <div class="drawer-toolbar">
        <n-space>
          <n-button size="small" @click="handleRefresh" :loading="loading">
            <template #icon>
              <n-icon><Refresh /></n-icon>
            </template>
            刷新
          </n-button>
          <n-button size="small" @click="handleClearCache" :disabled="loading">
            <template #icon>
              <n-icon><FileTray /></n-icon>
            </template>
            清除缓存
          </n-button>
          <n-button
            size="small"
            @click="downloadRendered"
            :disabled="renderStats.failedFiles > 0 || loading"
          >
            <template #icon>
              <n-icon><Download /></n-icon>
            </template>
            下载ZIP
          </n-button>
        </n-space>
      </div>

      <!-- 主内容区：使用 flex 布局 -->
      <div class="drawer-content">
        <!-- 左侧：文件树 -->
        <div class="file-tree-panel" :style="{ width: `${treePanelWidth}px` }">
          <div class="panel-title">渲染文件树</div>
          <div class="tree-container">
            <n-empty
              v-if="!loading && renderedTreeData.length === 0"
              description="暂无数据"
              size="small"
            />
            <n-tree
              v-else
              :data="renderedTreeData"
              :node-props="treeNodeProps"
              :render-label="renderTreeLabel"
              :render-switcher-icon="renderSwitcherIcon"
              :selected-keys="selectedKeys"
              expand-on-click
              :default-expanded-keys="defaultExpandedKeys"
              @update:selected-keys="handleNodeSelect"
              @update:expanded-keys="handleExpandedKeysChange"
              class="file-tree"
            />
          </div>
          <!-- 拖拽调整宽度的分隔条 -->
          <div
            class="resize-handle"
            @mousedown="startResize"
            :class="{ 'is-resizing': isResizing }"
          ></div>
        </div>

        <!-- 分隔条 -->
        <div class="divider"></div>

        <!-- 右侧：编辑器 -->
        <div class="editor-panel">
          <div class="panel-title">
            <span>{{ selectedFileLabel }}</span>
            <n-button
              v-if="selectedFile && selectedFile.fileContent"
              size="small"
              @click="copyFileContent"
              quaternary
            >
              <template #icon>
                <n-icon><Copy /></n-icon>
              </template>
              复制
            </n-button>
          </div>
          <div v-if="!selectedFile" class="no-file-selected">
            <n-empty description="请从左侧选择文件查看内容" size="small" />
          </div>
          <div v-else-if="selectedFile.renderError" class="render-error-display">
            <n-result
              status="error"
              title="文件渲染失败"
              :description="selectedFile.renderError.message"
            >
              <template #footer>
                <n-space vertical>
                  <n-text v-if="selectedFile.renderError.line" code>
                    行号: {{ selectedFile.renderError.line }}
                  </n-text>
                  <n-text v-if="selectedFile.renderError.column" code>
                    列号: {{ selectedFile.renderError.column }}
                  </n-text>
                  <n-text v-if="selectedFile.renderError.suggestion" type="info">
                    💡 {{ selectedFile.renderError.suggestion }}
                  </n-text>
                </n-space>
              </template>
            </n-result>
          </div>
          <div v-else class="codemirror-container" ref="editorContainer"></div>
        </div>
      </div>

      <!-- 错误列表（如果有） -->
      <div v-if="renderErrors.length > 0" class="error-panel">
        <n-collapse>
          <n-collapse-item title="渲染错误" name="errors">
            <n-scrollbar style="max-height: 200px">
              <n-alert
                v-for="error in renderErrors"
                :key="error.id"
                type="error"
                size="small"
                class="mb-2"
              >
                <template #header>
                  <div class="error-header">
                    <n-icon class="mr-1"><FileTray /></n-icon>
                    <strong>{{ error.fileName }}</strong>
                  </div>
                </template>
                <div class="error-content">
                  <p>{{ error.message }}</p>
                  <div v-if="error.line" class="error-details mt-2">
                    <n-text code size="small">行号: {{ error.line }}</n-text>
                    <n-text v-if="error.column" code size="small" class="ml-2">
                      列号: {{ error.column }}
                    </n-text>
                  </div>
                </div>
              </n-alert>
            </n-scrollbar>
          </n-collapse-item>
        </n-collapse>
      </div>

      <template #footer>
        <n-space justify="space-between" align="center" style="width: 100%">
          <n-space align="center">
            <n-tag v-if="renderStats.failedFiles > 0" type="error" size="small">
              {{ renderStats.failedFiles }} 个错误
            </n-tag>
            <n-tag v-else-if="renderStats.totalFiles > 0" type="success" size="small">
              渲染成功
            </n-tag>
            <n-text v-if="renderStats.totalFiles > 0" depth="3" style="font-size: 12px">
              {{ renderStats.totalFiles }} 个文件 | {{ formatFileSize(renderStats.totalSize) }} |
              耗时 {{ renderTime }}ms
            </n-text>
          </n-space>
        </n-space>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup>
  import { ref, computed, h, watch, onBeforeUnmount, nextTick } from 'vue';
  import {
    NDrawer,
    NDrawerContent,
    NTree,
    NTag,
    NText,
    NButton,
    NIcon,
    NSpace,
    NAlert,
    NCollapse,
    NCollapseItem,
    NEmpty,
    NScrollbar,
    NResult,
    useMessage,
  } from 'naive-ui';
  import {
    FolderOutline,
    DocumentOutline,
    AlertCircle,
    ChevronForward,
    Refresh,
    Copy,
    Download,
    FileTray,
    FolderOpenOutline,
    Folder as FolderIcon,
    FileTrayFullOutline,
  } from '@vicons/ionicons5';
  import { previewFileTree, downloadZip, clearRenderCache } from '@/api/templateFiles';

  // CodeMirror 核心模块（参考 TemplatePreview 的简化配置）
  import {
    EditorView,
    highlightActiveLine,
    highlightActiveLineGutter,
    lineNumbers,
  } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { dracula } from '@uiw/codemirror-theme-dracula';

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
    show: {
      type: Boolean,
      default: false,
    },
    templateId: {
      type: [String, Number],
      required: true,
    },
    templateName: {
      type: String,
      default: '',
    },
    variables: {
      type: Object,
      default: () => ({}),
    },
  });

  const emit = defineEmits(['update:show']);

  const message = useMessage();

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

  function getLanguageExtension(filename) {
    const ext = filename.split('.').pop()?.toLowerCase();
    return languageMap[ext] || null;
  }

  // 状态
  const loading = ref(false);
  const rawTreeData = ref([]);
  const renderStats = ref({
    totalFiles: 0,
    totalSize: 0,
    failedFiles: 0,
  });
  const renderErrors = ref([]);
  const renderTime = ref(0);
  const selectedFile = ref(null);
  const selectedKeys = ref([]);
  const defaultExpandedKeys = ref([]);
  const expandedKeys = ref(new Set());
  const editorContainer = ref(null);
  const editorView = ref(null);

  // 拖拽调整宽度相关
  const treePanelWidth = ref(260);
  const isResizing = ref(false);
  const resizeStartX = ref(0);
  const resizeStartWidth = ref(260);

  // 计算属性：响应式转换树数据
  const renderedTreeData = computed(() => convertToNaiveUITree(rawTreeData.value));

  // 计算属性
  const showDrawer = computed({
    get: () => props.show,
    set: (val) => emit('update:show', val),
  });

  const drawerTitle = computed(() => {
    // 优先显示模板名称，如果没有则显示模板ID
    const name = props.templateName || `#${props.templateId}`;
    return `全量渲染预览 - ${name}`;
  });

  const selectedFileLabel = computed(() => {
    if (!selectedFile.value) return '文件预览';
    const label = selectedFile.value.label || selectedFile.value.fileName;
    if (selectedFile.value.renderError) {
      return `${label} (渲染失败)`;
    }
    return label;
  });

  // Tree 节点配置
  const treeNodeProps = ({ option }) => {
    return {
      class: option.renderError ? 'tree-node-error' : '',
    };
  };

  // 渲染树节点标签
  const renderTreeLabel = ({ option }) => {
    const nodes = [h('span', { class: 'tree-node-label' }, option.label || option.fileName)];

    if (option.renderError) {
      nodes.push(
        h(
          NTag,
          { type: 'error', size: 'small', class: 'ml-2' },
          {
            default: () => '渲染失败',
          }
        )
      );
    }

    if (!option.isDirectory && option.fileSize !== undefined) {
      nodes.push(
        h(
          NText,
          { depth: 3, class: 'file-size-text' },
          {
            default: () => formatFileSize(option.fileSize),
          }
        )
      );
    }

    return h('div', { class: 'tree-node-wrapper' }, nodes);
  };

  // 渲染展开箭头（只有目录有箭头）
  const renderSwitcherIcon = () => h(NIcon, null, { default: () => h(ChevronForward) });

  // 处理展开/折叠状态变化
  const handleExpandedKeysChange = (keys, option, meta) => {
    if (!meta.node) return;

    // 只对目录节点进行图标更新
    if (!meta.node.isDirectory) return;

    const nodeKey = String(meta.node.key);

    switch (meta.action) {
      case 'expand':
        expandedKeys.value.add(nodeKey);
        break;
      case 'collapse':
        expandedKeys.value.delete(nodeKey);
        break;
    }
  };

  // 格式化文件大小
  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  // 初始化CodeMirror编辑器（参考 TemplatePreview 的简化配置）
  const initCodeMirror = () => {
    if (!editorContainer.value) return;

    // 销毁旧的编辑器实例
    if (editorView.value) {
      editorView.value.destroy();
      editorView.value = null;
    }

    const content = selectedFile.value?.fileContent || '';
    const fileName = selectedFile.value?.fileName || '';

    // 根据文件扩展名获取语言支持
    const languageExt = getLanguageExtension(fileName);

    try {
      const state = EditorState.create({
        doc: content,
        extensions: [
          // Dracula 主题
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
          // 只读模式
          EditorView.editable.of(false),
          // 添加语言支持（使用展开运算符）
          ...(languageExt ? [languageExt] : []),
        ],
      });

      editorView.value = new EditorView({
        state,
        parent: editorContainer.value,
      });
    } catch (error) {
      console.error('CodeMirror初始化失败:', error);
    }
  };

  // 销毁CodeMirror编辑器
  const destroyCodeMirror = () => {
    if (editorView.value) {
      editorView.value.destroy();
      editorView.value = null;
    }
  };

  // 加载渲染数据
  const loadRenderData = async () => {
    loading.value = true;
    renderErrors.value = [];
    selectedFile.value = null;
    selectedKeys.value = [];

    const startTime = Date.now();

    try {
      // 使用新的 previewFileTree API（编辑器预览模式，从工作目录读取）
      const response = await previewFileTree({
        templateId: props.templateId,
        variables: props.variables,
      });

      // 响应拦截器返回的是 response 对象，数据在 response.data
      const apiData = response.data;

      if (apiData && apiData.code === 0) {
        const { tree, totalFiles, totalSize, failedFiles } = apiData.data;

        // 保存原始数据，computed 会自动转换
        rawTreeData.value = tree;

        // 更新统计信息
        renderStats.value = { totalFiles, totalSize, failedFiles };

        // 收集错误信息
        renderErrors.value = collectErrors(rawTreeData.value);

        // 默认展开第一层节点
        defaultExpandedKeys.value = rawTreeData.value
          .filter((node) => node.isDirectory === 1)
          .map((node) => String(node.id));

        // 初始化 expandedKeys
        expandedKeys.value = new Set(defaultExpandedKeys.value);

        // 记录渲染时间
        renderTime.value = Date.now() - startTime;

        message.success('渲染完成');
      } else {
        throw new Error(response.data?.message || '渲染失败');
      }
    } catch (error) {
      console.error('渲染失败:', error);
      message.error('渲染失败: ' + (error.response?.data?.message || error.message || '未知错误'));
      rawTreeData.value = [];
    } finally {
      loading.value = false;
    }
  };

  // 转换为 Naive UI Tree 格式
  const convertToNaiveUITree = (tree) => {
    if (!tree || !Array.isArray(tree)) return [];

    return tree.map((node) => {
      const nodeKey = String(node.id);
      const isExpanded = expandedKeys.value.has(nodeKey);
      const isDirectory = node.isDirectory === 1;

      return {
        key: nodeKey,
        label: node.fileName,
        fileName: node.fileName, // 添加 fileName 属性，用于语言高亮
        isDirectory: isDirectory,
        isLeaf: !isDirectory,
        fileSize: node.fileSize,
        filePath: node.filePath,
        fileContent: node.fileContent,
        renderError: node.renderError,
        prefix: isDirectory
          ? () =>
              h(NIcon, null, {
                default: () => h(isExpanded ? FolderOpenOutline : FolderIcon),
              })
          : () => h(NIcon, null, { default: () => h(FileTrayFullOutline) }),
        children: node.children ? convertToNaiveUITree(node.children) : undefined,
      };
    });
  };

  // 收集错误信息
  const collectErrors = (tree) => {
    const errors = [];

    function traverse(nodes) {
      if (!nodes || !Array.isArray(nodes)) return;

      for (const node of nodes) {
        if (node.renderError) {
          errors.push({
            id: node.id,
            fileName: node.fileName,
            message: node.renderError.message,
            line: node.renderError.line,
            column: node.renderError.column,
          });
        }
        if (node.children) {
          traverse(node.children);
        }
      }
    }

    traverse(tree);
    return errors;
  };

  // 处理节点选择
  const handleNodeSelect = (keys) => {
    if (!keys || keys.length === 0) {
      selectedFile.value = null;
      selectedKeys.value = [];
      destroyCodeMirror();
      return;
    }

    const key = keys[0];
    selectedKeys.value = keys;
    findNodeByKey(renderedTreeData.value, key);
  };

  const findNodeByKey = (nodes, key) => {
    for (const node of nodes) {
      if (node.key === key) {
        // 检查是否是目录
        if (node.isDirectory) {
          // 目录节点不做任何处理，只用于展开/关闭
          selectedFile.value = null;
          destroyCodeMirror();
          return true;
        }

        selectedFile.value = node;

        // 只有当有内容时才初始化编辑器
        if (node.fileContent) {
          nextTick(() => {
            initCodeMirror();
          });
        } else {
          destroyCodeMirror();
        }
        return true;
      }
      if (node.children) {
        const found = findNodeByKey(node.children, key);
        if (found) return true;
      }
    }
    return false;
  };

  // 复制文件内容
  const copyFileContent = async () => {
    if (!selectedFile.value || !selectedFile.value.fileContent) {
      message.warning('没有内容可复制');
      return;
    }

    try {
      await navigator.clipboard.writeText(selectedFile.value.fileContent);
      message.success('已复制到剪贴板');
    } catch (error) {
      message.error('复制失败');
    }
  };

  // 下载渲染后的ZIP
  const downloadRendered = async () => {
    if (renderStats.value.failedFiles > 0) {
      message.warning('存在渲染错误的文件，请先修复后再下载');
      return;
    }

    try {
      const blob = await downloadZip({
        templateId: props.templateId,
        variables: props.variables,
      });

      // 创建下载链接
      const url = window.URL.createObjectURL(new Blob([blob]));
      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', `rendered_${props.templateId}.zip`);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      message.success('下载成功');
    } catch (error) {
      console.error('下载失败:', error);
      message.error('下载失败: ' + (error.message || '未知错误'));
    }
  };

  // 刷新渲染
  const handleRefresh = () => {
    destroyCodeMirror();
    loadRenderData();
  };

  // 清除缓存
  const handleClearCache = async () => {
    try {
      await clearRenderCache({
        templateId: props.templateId,
      });
      message.success('缓存已清除');
      // 清除缓存后自动刷新
      handleRefresh();
    } catch (error) {
      console.error('清除缓存失败:', error);
      message.error(
        '清除缓存失败: ' + (error.response?.data?.message || error.message || '未知错误')
      );
    }
  };

  // 关闭抽屉
  const handleClose = () => {
    destroyCodeMirror();
    showDrawer.value = false;
  };

  // 拖拽调整宽度功能
  const startResize = (event) => {
    event.preventDefault();
    isResizing.value = true;
    resizeStartX.value = event.clientX;
    resizeStartWidth.value = treePanelWidth.value;

    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'ew-resize';
    document.body.style.userSelect = 'none';
  };

  const handleResize = (event) => {
    if (!isResizing.value) return;

    const deltaX = event.clientX - resizeStartX.value;
    const newWidth = resizeStartWidth.value + deltaX;

    // 限制最小和最大宽度
    const minWidth = 120;
    const maxWidth = 600;

    if (newWidth >= minWidth && newWidth <= maxWidth) {
      treePanelWidth.value = newWidth;
    }
  };

  const stopResize = () => {
    isResizing.value = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  };

  // 监听显示状态变化
  watch(
    () => props.show,
    (newVal) => {
      if (newVal) {
        loadRenderData();
      } else {
        destroyCodeMirror();
      }
    }
  );

  // 组件卸载前销毁编辑器
  onBeforeUnmount(() => {
    destroyCodeMirror();
    // 清理拖拽事件监听
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
  });
</script>

<script>
  export default {
    name: 'FullRenderDrawer',
  };
</script>

<style scoped>
  /* 工具栏 */
  .drawer-toolbar {
    padding: 12px 16px;
    border-bottom: 1px solid #e8e8e8;
    background: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  }

  /* 主内容区 */
  .drawer-content {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  /* 文件树面板 */
  .file-tree-panel {
    min-width: 120px;
    max-width: 600px;
    background: #fff;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    position: relative;
    flex-shrink: 0;
  }

  .panel-title {
    padding: 12px 16px;
    font-size: 14px;
    font-weight: 500;
    color: #333;
    border-bottom: 1px solid #e8e8e8;
    background: #fff;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .tree-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding-right: 4px;
  }

  /* 滚动条样式 */
  .tree-container::-webkit-scrollbar {
    width: 6px;
  }

  .tree-container::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 3px;
  }

  .tree-container::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 3px;
  }

  .tree-container::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
  }

  /* 分隔条 */
  .divider {
    width: 1px;
    background: #e8e8e8;
  }

  /* 编辑器面板 */
  .editor-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .no-file-selected {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
  }

  .render-error-display {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    overflow: auto;
  }

  .codemirror-container {
    flex: 1;
    overflow: hidden;
    background: #282a36; /* Dracula theme background */
  }

  /* 错误面板 */
  .error-panel {
    border-top: 1px solid #e8e8e8;
    background: #fffaf0;
  }

  :deep(.error-panel .n-collapse) {
    border: none;
  }

  :deep(.error-panel .n-collapse-item) {
    border: none;
  }

  :deep(.error-panel .n-collapse-item__header) {
    padding: 12px 16px;
    background: #fffaf0;
    color: #d03050;
    font-weight: 500;
  }

  :deep(.error-panel .n-collapse-item__content-wrapper) {
    padding: 0 16px 12px;
  }

  :deep(.error-header) {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  :deep(.error-content) {
    font-size: 13px;
  }

  :deep(.error-details) {
    display: flex;
    gap: 8px;
  }

  /* 滚动条样式 */
  :deep(.tree-container::-webkit-scrollbar) {
    width: 6px;
    height: 6px;
  }

  :deep(.tree-container::-webkit-scrollbar-thumb) {
    background: #d0d0d0;
    border-radius: 3px;
  }

  :deep(.tree-container::-webkit-scrollbar-thumb:hover) {
    background: #b0b0b0;
  }

  :deep(.codemirror-container .cm-scroller::-webkit-scrollbar) {
    width: 10px;
    height: 10px;
  }

  :deep(.codemirror-container .cm-scroller::-webkit-scrollbar-thumb) {
    background: #4a4a4a;
    border-radius: 5px;
  }

  :deep(.codemirror-container .cm-scroller::-webkit-scrollbar-thumb:hover) {
    background: #5a5a5a;
  }

  /* 拖拽调整分隔条样式 */
  .resize-handle {
    position: absolute;
    top: 0;
    right: -8px;
    width: 16px;
    height: 100%;
    background: transparent;
    cursor: ew-resize;
    z-index: 10;
    transition: all 0.2s ease;
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
    background: rgba(24, 160, 88, 0.1);
  }

  .resize-handle:hover:before {
    background: #18a058;
    height: 60px;
  }

  .resize-handle.is-resizing {
    background: rgba(24, 160, 88, 0.2);
  }

  .resize-handle.is-resizing:before {
    background: #18a058;
    height: 80px;
  }
</style>
