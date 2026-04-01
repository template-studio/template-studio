<template>
  <div class="preview-fullscreen">
    <div class="preview-header">
      <div class="header-left">
        <span class="preview-title">预览确认</span>
        <span class="template-name">{{ templateInfo?.name }}</span>
      </div>
      <div class="header-actions">
        <n-button size="small" @click="$emit('prev')">
          <template #icon>
            <n-icon><ArrowBack /></n-icon>
          </template>
          返回配置
        </n-button>
        <n-button
          v-if="props.templateInfo?.templateType === 'basic'"
          size="small"
          @click="copyToClipboard"
          style="margin-right: 8px"
        >
          <template #icon>
            <n-icon><CopyOutline /></n-icon>
          </template>
          复制
        </n-button>
        <n-button type="primary" size="small" @click="generateProject">
          <template #icon>
            <n-icon><Download /></n-icon>
          </template>
          下载
        </n-button>
      </div>
    </div>

    <div class="preview-main">
      <!-- 左侧：模板资源管理器 - 基础模板时隐藏 -->
      <div v-if="showFileTree" class="file-explorer">
        <div class="explorer-header">
          <span class="explorer-title">模板文件</span>
        </div>
        <div class="explorer-content">
          <div v-if="loading" class="loading-container">
            <n-spin size="small">
              <template #description> 加载中... </template>
            </n-spin>
          </div>
          <div v-else-if="treeData.length === 0" class="empty-container">
            <div class="empty-text">暂无文件</div>
          </div>
          <div v-else class="file-tree">
            <NTree
              :data="treeDataComputed"
              :selected-keys="[currentFile]"
              :render-label="renderLabel"
              :render-switcher-icon="renderSwitcherIcon"
              @update:selected-keys="onSelectFile"
              @update:expanded-keys="updateExpandedKeys"
            />
          </div>
        </div>
      </div>

      <!-- 右侧：预览区域 -->
      <div class="preview-container">
        <div class="preview-content">
          <div v-if="!currentFile" class="no-file-selected">
            <div class="no-file-icon">
              <n-icon size="48" color="#ccc">
                <Document />
              </n-icon>
            </div>
            <div class="no-file-text">请选择左侧文件进行预览</div>
          </div>
          <div v-else class="file-preview">
            <div class="file-header">
              <div class="file-info">
                <span class="file-name">{{ currentFile }}</span>
              </div>
            </div>
            <div class="file-content">
              <div class="code-preview">
                <div class="codemirror-container" ref="codeContainer"></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, onMounted, computed, h, watch, nextTick, onBeforeUnmount } from 'vue';
  import { useMessage, NIcon, NTree, NSelect, NTag } from 'naive-ui';
  import {
    ArrowBack,
    Download,
    Close,
    Document,
    Folder,
    FolderOpenOutline,
    FileTrayFullOutline,
    ChevronForward,
    CopyOutline,
  } from '@vicons/ionicons5';
  import { generateFileTree, generateZip } from '@/api/templateFiles';
  import { getTemplateExpose } from '@/api/templateExpose';

  // CodeMirror 核心模块
  import { EditorView, lineNumbers, highlightActiveLineGutter, keymap } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { defaultKeymap } from '@codemirror/commands';

  // Dracula 主题
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
    templateInfo: {
      type: Object,
      default: null,
    },
    selectedVersion: {
      type: String,
      default: '',
    },
    variables: {
      type: Object,
      default: () => ({}),
    },
  });

  const emit = defineEmits(['prev', 'next']);

  const message = useMessage();

  // 文件树数据
  const treeData = ref([]);
  const loading = ref(false);
  const currentFile = ref('');
  const currentFilePath = ref('');
  const currentFileContent = ref('');

  // 渲染后的文件数据
  const renderedFilesMap = ref(new Map());

  // 计算属性：是否显示文件树（基础模板时隐藏）
  const showFileTree = computed(() => {
    return props.templateInfo?.templateType !== 'basic';
  });

  // 模板变量信息
  const templateVariables = ref([]);

  // CodeMirror 相关
  const codeContainer = ref(null);
  let editorView = null;

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

  // 展开状态
  const expandedKeys = ref(new Set());

  // 转换树数据为NTree格式
  const treeDataComputed = computed(() => {
    return treeToNaive(treeData.value);
  });

  function treeToNaive(tree) {
    if (!Array.isArray(tree)) return [];

    // 排序逻辑：目录在前，文件在后，同类型按名称排序
    const customSort = (a, b) => {
      // 首先按目录/文件排序：目录在前，文件在后
      if ((b.isDirectory || 0) - (a.isDirectory || 0) !== 0) {
        return (b.isDirectory || 0) - (a.isDirectory || 0);
      }
      // 同类型按名称排序
      const nameA = (a.fileName || a.label || '').toLowerCase();
      const nameB = (b.fileName || b.label || '').toLowerCase();
      return nameA.localeCompare(nameB);
    };

    const sorted = [...tree].sort(customSort);

    return sorted.map((node) => {
      const nodeKey = node.key || node.id;
      const isExpanded = expandedKeys.value.has(String(nodeKey));

      // 获取显示名称：直接使用fileName，后端已提供正确的文件名
      const getDisplayName = (node) => {
        return node.fileName || node.name || '';
      };

      return {
        label: getDisplayName(node),
        key: nodeKey,
        isLeaf: node.isDirectory !== 1, // 目录永远不是叶子节点，文件永远是叶子节点
        filePath: node.filePath,
        fileName: node.fileName,
        prefix: () =>
          h(NIcon, null, {
            default: () =>
              h(
                node.isDirectory === 1
                  ? isExpanded
                    ? FolderOpenOutline
                    : Folder
                  : FileTrayFullOutline
              ),
          }),
        children: node.children ? treeToNaive(node.children) : [],
      };
    });
  }

  // 获取模板变量信息 - 直接使用expose API
  const loadTemplateVariables = async () => {
    if (!props.templateInfo?.id) {
      return;
    }

    try {
      // 直接调用expose API获取模板变量定义
      const res = await getTemplateExpose({
        templateId: props.templateInfo.id,
      });

      if (res.data?.code === 0 && res.data?.data?.templateExpose) {
        const fieldSchemaJson = res.data.data.templateExpose.fieldSchemaJson;
        if (fieldSchemaJson) {
          try {
            const parsedSchema = JSON.parse(fieldSchemaJson);
            // 将schema转换为变量列表格式
            templateVariables.value = convertSchemaToVariablesList(parsedSchema);
          } catch (parseError) {
            console.error('解析变量定义失败:', parseError);
            templateVariables.value = [];
          }
        } else {
          templateVariables.value = [];
        }
      } else {
        templateVariables.value = [];
      }
    } catch (error) {
      console.error('获取模板变量失败:', error);
      templateVariables.value = [];
    }
  };

  // 将schema格式转换为变量列表格式
  const convertSchemaToVariablesList = (schema) => {
    const variablesList = [];

    if (!schema || typeof schema !== 'object') {
      return variablesList;
    }

    Object.entries(schema).forEach(([key, variable]) => {
      if (variable && typeof variable === 'object') {
        variablesList.push({
          name: key,
          variableType: variable.type || 'string',
          label: variable.title || key,
          description: variable.description || '',
          defaultValue: variable.default,
          required: variable.required || false,
        });
      }
    });

    return variablesList;
  };

  // 根据变量类型转换变量值
  const convertVariableTypes = (variables) => {
    const converted = { ...variables };
    const variableTypeMap = {};

    // 创建变量类型映射
    templateVariables.value.forEach((v) => {
      variableTypeMap[v.name] = v.variableType;
    });

    // 转换变量类型
    Object.keys(converted).forEach((key) => {
      const variableType = variableTypeMap[key];
      if (!variableType) return;

      const value = converted[key];
      const valueStr = String(value);

      switch (variableType) {
        case 'string':
          converted[key] = valueStr;
          break;
        case 'boolean':
          // 字符串 "false" 转换为 false，其他非空值转换为 true
          converted[key] = valueStr === 'false' ? false : Boolean(valueStr && valueStr !== '0');
          break;
        case 'number':
          // 尝试转换为数字
          const numValue = Number(valueStr);
          converted[key] = isNaN(numValue) ? 0 : numValue;
          break;
        case 'list':
          // 如果是JSON格式的数组字符串，尝试解析
          if (valueStr.startsWith('[') && valueStr.endsWith(']')) {
            try {
              converted[key] = JSON.parse(valueStr);
            } catch {
              // 如果解析失败，按逗号分割
              converted[key] = valueStr
                .slice(1, -1)
                .split(',')
                .map((s) => s.trim());
            }
          } else {
            // 按逗号分割字符串
            converted[key] = valueStr.split(',').map((s) => s.trim());
          }
          break;
        case 'object':
          // 如果是JSON格式的对象字符串，尝试解析
          if (
            (valueStr.startsWith('{') && valueStr.endsWith('}')) ||
            (valueStr.startsWith('[') && valueStr.endsWith(']'))
          ) {
            try {
              converted[key] = JSON.parse(valueStr);
            } catch {
              converted[key] = valueStr;
            }
          } else {
            converted[key] = valueStr;
          }
          break;
        default:
          // 未知类型，保持原值
          break;
      }
    });

    return converted;
  };

  // 加载渲染后的文件树
  const loadTree = async () => {
    if (!props.templateInfo?.id) {
      console.log('templateInfo.id 不存在，跳过加载');
      return;
    }
    loading.value = true;
    try {
      // 先获取模板变量信息
      await loadTemplateVariables();

      // 转换变量类型
      const convertedVariables = convertVariableTypes(props.variables || {});

      // 调用用户模式渲染接口（从发布版本读取）
      const renderRes = await generateFileTree({
        templateId: props.templateInfo.id,
        variables: convertedVariables,
        version: props.selectedVersion || undefined, // 空字符串不传递，默认使用latest
      });
      const tree = renderRes.data?.data?.tree || [];
      treeData.value = tree;

      // 构建文件映射，方便快速查找
      renderedFilesMap.value.clear();
      const flattenFiles = (nodes) => {
        nodes.forEach((node) => {
          renderedFilesMap.value.set(node.id, node);
          if (node.children && node.children.length > 0) {
            flattenFiles(node.children);
          }
        });
      };
      flattenFiles(tree);

      // 根据模板类型自动选择文件
      if (props.templateInfo?.templateType === 'basic') {
        // 基础模板：优先选择index.tmp
        const indexFile = findFileByName(tree, 'index.tmp');
        if (indexFile) {
          // 自动选择index.tmp文件
          setTimeout(() => {
            onSelectFile([indexFile.id]);
          }, 100);
        } else {
          // 如果没有index.tmp，选择第一个文件
          const firstFile = findFirstFile(tree);
          if (firstFile) {
            setTimeout(() => {
              onSelectFile([firstFile.id]);
            }, 100);
          }
        }
      } else {
        // 非基础模板：选择第一个文件（保持原有行为）
        const firstFile = findFirstFile(tree);
        if (firstFile) {
          setTimeout(() => {
            onSelectFile([firstFile.id]);
          }, 100);
        }
      }
    } catch (error) {
      console.error('加载文件树失败:', error);
      treeData.value = [];
    } finally {
      loading.value = false;
    }
  };

  // 在树数据中按文件名查找文件的辅助函数
  const findFileByName = (nodes, fileName) => {
    for (const node of nodes) {
      if (node.fileName === fileName && node.isDirectory !== 1) {
        return node;
      }
      if (node.children && node.children.length > 0) {
        const found = findFileByName(node.children, fileName);
        if (found) return found;
      }
    }
    return null;
  };

  // 查找第一个文件（用于默认选择）
  const findFirstFile = (nodes) => {
    for (const node of nodes) {
      if (node.isDirectory !== 1) {
        return node;
      }
      if (node.children && node.children.length > 0) {
        const found = findFirstFile(node.children);
        if (found) return found;
      }
    }
    return null;
  };

  // 选择文件
  const onSelectFile = async (keys) => {
    if (!keys || keys.length === 0) return;

    const selectedKey = keys[0];

    // 从树数据中查找对应的节点
    const findNodeByKey = (nodes, key) => {
      for (const node of nodes) {
        if (node.key === key || node.id === key) {
          return node;
        }
        if (node.children) {
          const found = findNodeByKey(node.children, key);
          if (found) return found;
        }
      }
      return null;
    };

    const selectedNode = findNodeByKey(treeData.value, selectedKey);
    if (!selectedNode || selectedNode.isDirectory === 1) {
      // 如果是文件夹，不处理
      return;
    }

    const fileId = selectedNode.id || selectedKey;

    // 从渲染后的文件映射中获取内容
    const renderedFile = renderedFilesMap.value.get(fileId);
    if (renderedFile) {
      currentFile.value = renderedFile.fileName;
      currentFilePath.value = renderedFile.filePath;
      currentFileContent.value = renderedFile.fileContent;

      // 等待DOM更新后创建或更新CodeMirror编辑器
      await nextTick();
      createOrUpdateEditor();
    } else {
      console.error('未找到渲染后的文件:', fileId);
      currentFileContent.value = '文件未找到';
    }
  };

  // 创建右键菜单功能
  const createContextMenu = () => {
    const menuHandler = (event) => {
      // 阻止默认右键菜单
      event.preventDefault();
      console.log('右键菜单被触发'); // 调试信息

      try {
        const selection = editorView.state.selection.main;
        const selectedText = editorView.state.sliceDoc(selection.from, selection.to);

        console.log('选中的文本:', selectedText); // 调试信息

        // 无论是否选中文本都显示菜单
        const menu = document.createElement('div');
        menu.style.cssText = `
        position: fixed;
        background: #282c34;
        border: 1px solid #3c4049;
        border-radius: 6px;
        padding: 4px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        z-index: 10000;
        min-width: 80px;
      `;

        // 复制菜单项
        const copyItem = document.createElement('div');
        copyItem.textContent = '复制';
        copyItem.style.cssText = `
        padding: 6px 12px;
        cursor: pointer;
        font-size: 13px;
        color: #abb2bf;
        transition: background 0.2s;
        border-radius: 3px;
      `;

        copyItem.addEventListener('mouseenter', () => {
          copyItem.style.background = '#3c4049';
        });
        copyItem.addEventListener('mouseleave', () => {
          copyItem.style.background = 'transparent';
        });

        copyItem.addEventListener('click', async () => {
          try {
            const textToCopy = selectedText || editorView.state.doc.toString();
            await navigator.clipboard.writeText(textToCopy);

            // 显示复制成功提示
            const toast = document.createElement('div');
            toast.textContent = '已复制到剪贴板';
            toast.style.cssText = `
            position: fixed;
            background: #18a058;
            color: white;
            padding: 8px 16px;
            border-radius: 4px;
            font-size: 14px;
            z-index: 10001;
            pointer-events: none;
          `;
            toast.style.left = `${event.clientX}px`;
            toast.style.top = `${event.clientY - 40}px`;
            document.body.appendChild(toast);

            setTimeout(() => {
              if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
              }
            }, 2000);
          } catch (err) {
            console.error('复制失败:', err);
            // 降级到传统方法
            try {
              const textToCopy = selectedText || editorView.state.doc.toString();
              const textArea = document.createElement('textarea');
              textArea.value = textToCopy;
              textArea.style.position = 'fixed';
              textArea.style.left = '-999999px';
              document.body.appendChild(textArea);
              textArea.select();
              document.execCommand('copy');
              document.body.removeChild(textArea);
            } catch (fallbackErr) {
              console.error('降级复制也失败:', fallbackErr);
            }
          } finally {
            if (menu.parentNode) {
              menu.parentNode.removeChild(menu);
            }
          }
        });

        menu.appendChild(copyItem);
        document.body.appendChild(menu);

        // 设置菜单位置，确保不会超出视窗
        const rect = menu.getBoundingClientRect();
        let left = event.clientX;
        let top = event.clientY;

        if (left + rect.width > window.innerWidth) {
          left = window.innerWidth - rect.width - 10;
        }
        if (top + rect.height > window.innerHeight) {
          top = window.innerHeight - rect.height - 10;
        }

        menu.style.left = `${left}px`;
        menu.style.top = `${top}px`;

        // 点击其他地方关闭菜单
        const closeMenu = (e) => {
          if (!menu.contains(e.target)) {
            if (menu.parentNode) {
              menu.parentNode.removeChild(menu);
            }
            document.removeEventListener('click', closeMenu);
            document.removeEventListener('contextmenu', closeMenu);
          }
        };

        // 延迟添加事件监听，避免立即触发
        setTimeout(() => {
          document.addEventListener('click', closeMenu);
          document.addEventListener('contextmenu', closeMenu);
        }, 100);
      } catch (error) {
        console.error('右键菜单处理错误:', error);
      }
    };

    return EditorView.domEventHandlers({
      contextmenu: menuHandler,
    });
  };

  // 创建或更新CodeMirror编辑器
  const createOrUpdateEditor = async () => {
    if (!codeContainer.value) return;

    // 销毁现有编辑器
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }

    // 获取语言扩展
    const languageExt = currentFile.value ? getLanguageExtension(currentFile.value) : null;

    // 创建编辑器扩展
    const extensions = [
      // Dracula 主题
      dracula,
      // 只读模式
      EditorView.editable.of(false),
      // 行号
      lineNumbers(),
      // 当前行行号高亮
      highlightActiveLineGutter(),
      // 语法高亮
      syntaxHighlighting(defaultHighlightStyle),
      // 默认键盘映射（包含复制快捷键）
      keymap.of(defaultKeymap),
      // 右键菜单
      createContextMenu(),
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
    ];

    // 添加语言支持
    if (languageExt) {
      extensions.push(languageExt);
    }

    // 创建编辑器状态
    const state = EditorState.create({
      doc: currentFileContent.value,
      extensions,
    });

    // 创建编辑器视图
    editorView = new EditorView({
      state,
      parent: codeContainer.value,
    });

    // 确保编辑器可以滚动
    setTimeout(() => {
      if (editorView && editorView.scrollDOM) {
        editorView.requestMeasure();
      }
    }, 100);
  };

  // 更新展开状态
  const updateExpandedKeys = (keys) => {
    expandedKeys.value = new Set(keys);
  };

  // 渲染标签
  const renderLabel = ({ option }) => {
    return option.label;
  };

  // 渲染切换图标 - 使用默认的展开/折叠箭头
  const renderSwitcherIcon = () => h(NIcon, null, { default: () => h(ChevronForward) });

  // 复制到剪贴板
  const copyToClipboard = async () => {
    if (!currentFileContent.value) {
      message.error('没有可复制的内容');
      return;
    }

    try {
      await navigator.clipboard.writeText(currentFileContent.value);
      message.success('内容已复制到剪贴板');
    } catch (err) {
      console.error('复制失败:', err);
      message.error('复制失败，请手动复制');
    }
  };

  // 生成项目
  const generateProject = async () => {
    try {
      message.loading('正在生成项目...', { duration: 0 });

      // 转换变量类型
      const convertedVariables = convertVariableTypes(props.variables || {});

      const response = await generateZip({
        templateId: props.templateInfo.id,
        variables: convertedVariables,
        fileName: props.templateInfo.name,
        version: props.selectedVersion || undefined, // 空字符串不传递，默认使用latest
      });

      // 创建下载链接
      const blob = new Blob([response.data], { type: 'application/zip' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `${props.templateInfo.name}.zip`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      message.destroyAll();
      message.success('项目生成成功！');
    } catch (error) {
      message.destroyAll();
      console.error('生成项目失败:', error);
      message.error('生成项目失败，请重试');
    }
  };

  // 监听templateInfo变化，当有数据时加载文件树
  watch(
    () => props.templateInfo,
    (newTemplateInfo) => {
      if (newTemplateInfo?.id) {
        // 延迟加载，避免在第一步就加载
        setTimeout(() => {
          loadTree();
        }, 100);
      }
    },
    { immediate: false }
  );

  // 监听版本变化，重新加载文件树
  watch(
    () => props.selectedVersion,
    () => {
      if (props.templateInfo?.id) {
        loadTree();
      }
    }
  );

  onMounted(() => {
    // 如果已经有 templateInfo，则加载数据
    if (props.templateInfo?.id) {
      console.log('StepPreview 组件挂载时直接加载文件树');
      loadTree();
    }
  });

  onBeforeUnmount(() => {
    if (editorView) {
      editorView.destroy();
    }
  });
</script>

<style scoped>
  .preview-fullscreen {
    position: fixed;
    inset: 0;
    background: #f5f5f5;
    display: flex;
    flex-direction: column;
  }

  .preview-header {
    height: 56px;
    background: #fff;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 32px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
    flex: 1;
  }

  .preview-title {
    font-size: 1.2rem;
    font-weight: bold;
    color: #18a058;
  }

  .template-name {
    font-size: 1rem;
    color: #666;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    justify-content: flex-end;
  }

  .preview-main {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  /* 文件资源管理器 */
  .file-explorer {
    width: 280px;
    background: #fff;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
  }

  .explorer-header {
    height: 48px;
    background: #f8f9fa;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    padding: 0 16px;
  }

  .explorer-title {
    font-size: 14px;
    font-weight: bold;
    color: #333;
  }

  .explorer-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .loading-container {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .empty-container {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .empty-text {
    color: #999;
    font-size: 14px;
  }

  /* 文件树样式 */
  .file-tree {
    height: 100%;
  }

  /* 预览容器 */
  .preview-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: #fff;
  }

  .preview-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .no-file-selected {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #999;
  }

  .no-file-icon {
    margin-bottom: 16px;
  }

  .no-file-text {
    font-size: 16px;
  }

  .file-preview {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .file-header {
    height: 48px;
    background: #f8f9fa;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    padding: 0 16px;
  }

  .file-info {
    display: flex;
    align-items: center;
  }

  .file-name {
    font-size: 14px;
    font-weight: bold;
    color: #333;
  }

  .file-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .code-preview {
    flex: 1;
    overflow: auto;
    background: #1e1e1e;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
  }

  .codemirror-container {
    height: 100%;
    min-height: 400px;
  }

  /* CodeMirror 样式覆盖 - 确保与编辑页面一致 */
  :deep(.cm-editor) {
    height: 100% !important;
    font-size: 15px;
    outline: none !important;
  }

  :deep(.cm-editor .cm-scroller) {
    font-family: 'Fira Code', 'Consolas', 'Monaco', monospace;
    overflow: auto !important;
    height: 100% !important;
    max-height: none !important;
  }

  :deep(.cm-editor .cm-line) {
    padding: 0;
  }

  /* 只读模式下的光标隐藏 */
  :deep(.cm-editor .cm-cursor) {
    display: none !important;
  }

  :deep(.cm-editor .cm-cursor-primary) {
    display: none !important;
  }
</style>
