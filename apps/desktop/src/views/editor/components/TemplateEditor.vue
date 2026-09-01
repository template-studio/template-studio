<template>
  <div class="edit-editor">
    <!-- 文件头部 -->
    <div v-if="currentFileName" class="file-header">
      <div class="file-info">
        <span class="file-name">{{ currentFileName }}</span>
      </div>
      <div class="file-actions">
        <a-button size="small" @click="saveCurrentFile">
          <template #icon>
            <Save />
          </template>
          保存
        </a-button>
        <a-button size="small" @click="triggerPreview">
          <template #icon>
            <Eye />
          </template>
          预览
        </a-button>
      </div>
    </div>

    <!-- 编辑器容器 -->
    <div v-if="!currentFileName" class="no-file-selected">
      <div class="no-file-icon">
        <Document style="font-size: 48px; color: #ccc" />
      </div>
      <div class="no-file-text">请选择左侧文件进行编辑</div>
    </div>
    <div v-else class="codemirror-container" ref="editorContainer"></div>

    <!-- HTML 预览弹框 -->
    <a-modal v-model:open="showHtmlPreviewModal" title="HTML 预览" :style="modalStyle" :footer="null" :width="isFullscreen ? '100vw' : '90vw'">
      <template #title>
        <div class="modal-header">
          <span>HTML 预览</span>
          <div class="modal-actions">
            <a-button size="small" @click="toggleFullscreen">
              <template #icon>
                <svg
                  v-if="!isFullscreen"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"
                  />
                </svg>
                <svg
                  v-else
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"
                  />
                </svg>
              </template>
            </a-button>
          </div>
        </div>
      </template>
      <div class="html-preview-container">
        <iframe
          ref="htmlPreviewFrame"
          class="html-preview-frame"
          sandbox="allow-scripts allow-same-origin"
        ></iframe>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
  import { ref, watch, onMounted, onBeforeUnmount, nextTick, computed } from 'vue';
  import { notification } from 'ant-design-vue';
  import { editTemplateFile, restoreTemplateFile } from '@/api/editor/templateFiles';
  import { Save, Eye, Document } from '@/icons/ionicons5';

  // CodeMirror 核心模块 - 按照官方示例导入
  import {
    EditorView,
    keymap,
    highlightSpecialChars,
    drawSelection,
    highlightActiveLine,
    dropCursor,
    rectangularSelection,
    crosshairCursor,
    lineNumbers,
    highlightActiveLineGutter,
  } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { defaultKeymap, indentWithTab, history, historyKeymap } from '@codemirror/commands';
  import {
    defaultHighlightStyle,
    syntaxHighlighting,
    indentOnInput,
    bracketMatching,
    foldGutter,
    foldKeymap,
  } from '@codemirror/language';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import {
    autocompletion,
    completionKeymap,
    closeBrackets,
    closeBracketsKeymap,
  } from '@codemirror/autocomplete';

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
    currentFileName: {
      type: String,
      default: '',
    },
    currentFileId: {
      type: [String, Number],
      default: '',
    },
    currentFileContent: {
      type: String,
      default: '',
    },
    currentFilePath: {
      type: String,
      default: '',
    },
    templateId: {
      type: [String, Number],
      default: '',
    },
  });

  const emit = defineEmits(['contentChange', 'insertVariable', 'preview', 'selectionChange', 'save-success', 'reload-file']);

  const editorContainer = ref(null);
  const htmlPreviewFrame = ref(null);
  const showHtmlPreviewModal = ref(false);
  const isFullscreen = ref(false);
  let editorView = null;

  // 计算属性
  const modalStyle = computed(() => {
    if (isFullscreen.value) {
      return {
        width: '100vw',
        height: '100vh',
        margin: '0',
        borderRadius: '0',
      };
    } else {
      return {
        width: '90vw',
        height: '80vh',
      };
    }
  });

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

  async function saveCurrentFile(silent = false) {
    if (!props.currentFileId || !props.currentFilePath) {
      if (!silent) {
        notification.warning({
          title: '无法保存',
          content: '请先选择一个文件',
          duration: 2500,
        });
      }
      return false;
    }

    try {
      const content = editorView ? editorView.state.doc.toString() : props.currentFileContent;
      await editTemplateFile({
        templateId: parseInt(props.templateId),
        filePath: props.currentFilePath,
        content: content,
      });

      if (!silent) {
        notification.success({
          title: '保存成功',
          content: '文件已成功保存',
          duration: 2500,
        });
      }

      // 触发保存成功事件，通知父组件进行后续操作（如自动渲染预览）
      emit('save-success', {
        fileId: props.currentFileId,
        fileName: props.currentFileName,
        filePath: props.currentFilePath,
      });

      return true;
    } catch (e) {
      // 错误始终显示通知，即使是静默模式
      notification.error({
        title: '保存失败',
        content: '请检查网络或稍后重试',
        duration: 2500,
      });
      return false;
    }
  }

  // 插入变量到编辑器（如果有选中文本则替换）
  function insertVariable(template) {
    if (!editorView) return;

    const { state, dispatch } = editorView;
    const selection = state.selection.main;

    // 如果有选中文本，则替换；否则在光标位置插入
    const transaction = state.update({
      changes: {
        from: selection.from,
        to: selection.to,
        insert: template,
      },
      // 移动光标到插入内容之后
      selection: { anchor: selection.from + template.length },
    });

    dispatch(transaction);

    // 触发内容变化事件
    const content = editorView.state.doc.toString();
    emit('contentChange', { content });
  }

  // 触发预览
  function triggerPreview() {
    if (!props.currentFileId) {
      notification.warning({
        title: '无法预览',
        content: '请先选择一个文件',
        duration: 2500,
      });
      return;
    }

    emit('preview', { fileId: props.currentFileId, fileName: props.currentFileName });
  }

  // 获取选中文本
  function getSelectedText() {
    if (!editorView) return '';

    const selection = editorView.state.selection.main;
    if (selection.empty) return '';

    return editorView.state.doc.sliceString(selection.from, selection.to);
  }

  // 获取光标位置
  function getCursorPosition() {
    if (!editorView) return { line: 0, column: 0 };

    const selection = editorView.state.selection.main;
    const line = editorView.state.doc.lineAt(selection.head);

    return {
      line: line.number,
      column: selection.head - line.from,
      offset: selection.head,
    };
  }

  // 获取选中范围信息
  function getSelectionInfo() {
    if (!editorView) return null;

    const selection = editorView.state.selection.main;
    const selectedText = getSelectedText();
    const cursorPosition = getCursorPosition();

    return {
      hasSelection: !selection.empty,
      selectedText,
      selectionStart: selection.from,
      selectionEnd: selection.to,
      cursorPosition,
    };
  }

  // 将选中文本用 {% raw %} 包裹
  function wrapSelectionWithRaw() {
    if (!editorView) {
      notification.warning({
        title: '无法操作',
        content: '编辑器未初始化',
        duration: 2000,
      });
      return;
    }

    const selection = editorView.state.selection.main;

    // 如果没有选中文本，提示用户
    if (selection.empty) {
      notification.warning({
        title: '请先选中文本',
        content: '请选中需要原样显示的文本',
        duration: 2000,
      });
      return;
    }

    const selectedText = editorView.state.doc.sliceString(selection.from, selection.to);

    // 判断是否为多行文本（包含换行符）
    const isMultiLine = selectedText.includes('\n');

    // 根据是否多行决定是否换行包裹
    let wrappedText;
    if (isMultiLine) {
      // 多行文本，换行包裹
      wrappedText = `{% raw %}\n${selectedText}\n{% endraw %}`;
    } else {
      // 单行文本，不换行
      wrappedText = `{% raw %} ${selectedText} {% endraw %}`;
    }

    // 替换选中的文本
    const transaction = editorView.state.update({
      changes: {
        from: selection.from,
        to: selection.to,
        insert: wrappedText,
      },
    });

    editorView.dispatch(transaction);

    // 触发内容变化事件
    const content = editorView.state.doc.toString();
    emit('contentChange', { content });

    notification.success({
      title: '已添加 raw 标签',
      content: '选中文本已被 {% raw %} 标签包裹',
      duration: 2000,
    });
  }

  // 重置文件到上次提交状态（git restore）
  async function resetFile() {
    if (!props.currentFileId || !props.currentFilePath) {
      notification.warning({
        title: '无法重置',
        content: '请先选择一个文件',
        duration: 2500,
      });
      return;
    }

    try {
      await restoreTemplateFile({
        templateId: parseInt(props.templateId),
        filePath: props.currentFilePath,
      });

      // 成功后重新加载文件内容
      emit('reload-file', {
        fileId: props.currentFileId,
        filePath: props.currentFilePath,
      });

      notification.success({
        title: '重置成功',
        content: '文件已恢复到上次提交状态',
        duration: 2500,
      });
    } catch (e) {
      // 响应拦截器已处理错误提示
      console.error('重置文件失败:', e);
    }
  }

  // 暴露方法给父组件
  defineExpose({
    insertVariable,
    getSelectedText,
    getCursorPosition,
    getSelectionInfo,
    saveCurrentFile,
    resetFile,
  });

  function runHtmlFile() {
    const content = editorView ? editorView.state.doc.toString() : props.currentFileContent;
    if (!content) return;

    // 直接预览，不做校验
    showHtmlPreview(content);
  }

  function showHtmlPreview(content) {
    // 显示预览弹框
    showHtmlPreviewModal.value = true;
    isFullscreen.value = false; // 重置全屏状态

    // 等待 DOM 更新后设置 iframe 内容
    nextTick(() => {
      if (htmlPreviewFrame.value) {
        const iframe = htmlPreviewFrame.value;

        // 设置超时保护
        const timeout = setTimeout(() => {
          notification.error({
            title: '预览超时',
            content: 'HTML 渲染超时，可能存在无限循环或错误',
            duration: 3000,
          });
          showHtmlPreviewModal.value = false;
        }, 3000); // 3秒超时

        // 尝试渲染内容
        try {
          const doc = iframe.contentDocument || iframe.contentWindow.document;

          // 写入 HTML 内容
          doc.open();
          doc.write(content);
          doc.close();

          // 清除超时
          clearTimeout(timeout);

          notification.success({
            title: '预览成功',
            content: 'HTML 内容已在弹框中显示',
            duration: 2500,
          });
        } catch (e) {
          // 清除超时
          clearTimeout(timeout);

          // 如果渲染失败，显示纯文本
          try {
            const doc = iframe.contentDocument || iframe.contentWindow.document;
            doc.open();
            doc.write(
              `<html><body><pre style="white-space: pre-wrap; font-family: monospace; padding: 20px;">${content}</pre></body></html>`
            );
            doc.close();

            notification.warning({
              title: '渲染失败',
              content: 'HTML 渲染失败，已显示为纯文本',
              duration: 3000,
            });
          } catch (fallbackError) {
            // 最后的保护措施
            showHtmlPreviewModal.value = false;
            notification.error({
              title: '预览失败',
              content: '无法显示内容',
              duration: 3000,
            });
          }
        }
      }
    });
  }

  function toggleFullscreen() {
    isFullscreen.value = !isFullscreen.value;
  }

  function updateEditorContent(content, filename = '') {
    if (!editorView) {
      console.warn('编辑器未初始化，无法更新内容');
      return;
    }

    try {
      const languageExt = getLanguageExtension(filename);

      const newState = EditorState.create({
        doc: content || '',
        extensions: createEditorExtensionsWithListener(languageExt),
      });

      editorView.setState(newState);

      // 确保滚动正常工作
      setTimeout(() => {
        if (editorView && editorView.scrollDOM) {
          editorView.requestMeasure();
        }
      }, 100);

      console.log('编辑器内容更新成功:', { filename, contentLength: content?.length });
    } catch (error) {
      console.error('更新编辑器内容失败:', error);
    }
  }

  function createEditorExtensions(languageExtension = null) {
    const extensions = [
      // Dracula 主题
      dracula,
      // 启用编辑和选择功能
      EditorView.editable.of(true),
      // 行号
      lineNumbers(),
      // 代码折叠标记
      foldGutter(),
      // 高亮特殊字符
      highlightSpecialChars(),
      // 撤销历史
      history(),
      // 拖拽时的光标
      dropCursor(),
      // 输入时自动缩进
      indentOnInput(),
      // 括号匹配高亮
      bracketMatching(),
      // 自动关闭括号
      closeBrackets(),
      // 自动完成
      autocompletion(),
      // 矩形选择
      rectangularSelection(),
      // Alt+拖拽时显示十字光标
      crosshairCursor(),
      // 当前行高亮
      highlightActiveLine(),
      // 当前行行号高亮
      highlightActiveLineGutter(),
      // 高亮匹配的选中文本 - 重新添加，但使用更安全的配置
      highlightSelectionMatches(),
      // 确保滚动正常工作
      EditorView.scrollMargins.of(() => ({ top: 10, bottom: 10 })),
      // 强制启用滚动
      EditorView.theme({
        '&': { height: '100%' },
        '.cm-scroller': {
          overflow: 'auto !important',
          fontFamily: 'monospace',
          '&::-webkit-scrollbar': {
            width: '0 !important',
            height: '0 !important',
            display: 'none !important',
          },
          scrollbarWidth: 'none !important',
          msOverflowStyle: 'none !important',
        },
      }),
      // 语法高亮
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      // 快捷键
      keymap.of([
        // 保存快捷键
        {
          key: 'Ctrl-s',
          run: () => {
            saveCurrentFile();
            return true;
          },
        },
        // 预览快捷键
        {
          key: 'Ctrl-r',
          run: () => {
            triggerPreview();
            return true;
          },
        },
        // 默认快捷键
        ...defaultKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...closeBracketsKeymap,
        ...searchKeymap,
        { key: 'Tab', run: indentWithTab },
      ]),
      // 右键菜单
      EditorView.domEventHandlers({
        contextmenu: (event, view) => {
          event.preventDefault();
          showContextMenu(event, view);
          return true;
        },
        mousedown: (event, view) => {
          // 确保点击时编辑器获得焦点
          view.focus();
          return false;
        },
      }),
    ];

    // 添加语言支持
    if (languageExtension) {
      extensions.push(languageExtension);
    }

    return extensions;
  }

  function createEditorExtensionsWithListener(languageExtension = null) {
    return [
      ...createEditorExtensions(languageExtension),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const content = update.state.doc.toString();
          emit('contentChange', { content });
        }

        // 监听选择变化
        if (update.selectionSet) {
          const selection = update.state.selection.main;
          const selectedText = selection.empty
            ? ''
            : update.state.doc.sliceString(selection.from, selection.to);
          const hasSelection = !selection.empty;

          emit('selectionChange', {
            hasSelection,
            selectedText,
            selectionStart: selection.from,
            selectionEnd: selection.to,
            selectionLength: selectedText.length,
          });
        }
      }),
    ];
  }

  function showContextMenu(event, view) {
    const { clientX, clientY } = event;

    // 创建右键菜单
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.style.cssText = `
    position: fixed;
    top: ${clientY}px;
    left: ${clientX}px;
    background: #2d2d30;
    border: 1px solid #3c3c3c;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    z-index: 1000;
    min-width: 160px;
    padding: 4px 0;
  `;

    const menuItems = [
      { label: '保存 (Ctrl+S)', action: () => saveCurrentFile() },
      { type: 'separator' },
      { label: '撤销 (Ctrl+Z)', action: () => document.execCommand('undo') },
      { label: '重做 (Ctrl+Y)', action: () => document.execCommand('redo') },
      { type: 'separator' },
      { label: '剪切 (Ctrl+X)', action: () => document.execCommand('cut') },
      { label: '复制 (Ctrl+C)', action: () => document.execCommand('copy') },
      { label: '粘贴 (Ctrl+V)', action: () => document.execCommand('paste') },
      { type: 'separator' },
      {
        label: '全选 (Ctrl+A)',
        action: () => view.dispatch({ selection: { anchor: 0, head: view.state.doc.length } }),
      },
    ];

    // 添加原样显示选项（仅在选中文本时显示）
    const selection = view.state.selection.main;
    if (!selection.empty) {
      menuItems.push(
        { type: 'separator' },
        { label: '原样显示 ({% raw %})', action: () => wrapSelectionWithRaw() }
      );
    }

    // 添加预览选项
    menuItems.push(
      { type: 'separator' },
      { label: '预览 (Ctrl+R)', action: () => triggerPreview() }
    );

    // 添加重置文件选项（仅在有文件时显示）
    if (props.currentFileId && props.currentFilePath) {
      menuItems.push(
        { type: 'separator' },
        { label: '重置此文件 (Git Restore)', action: () => resetFile() }
      );
    }

    menuItems.forEach((item) => {
      if (item.type === 'separator') {
        const separator = document.createElement('div');
        separator.style.cssText = `
        height: 1px;
        background: #3c3c3c;
        margin: 4px 0;
      `;
        menu.appendChild(separator);
      } else {
        const menuItem = document.createElement('div');
        menuItem.className = 'context-menu-item';
        menuItem.textContent = item.label;
        menuItem.style.cssText = `
        padding: 8px 16px;
        cursor: pointer;
        color: #cccccc;
        font-size: 14px;
        user-select: none;
      `;

        menuItem.addEventListener('mouseenter', () => {
          menuItem.style.background = '#3c3c3c';
        });

        menuItem.addEventListener('mouseleave', () => {
          menuItem.style.background = 'transparent';
        });

        menuItem.addEventListener('click', () => {
          try {
            item.action();
          } catch (e) {
            console.warn('菜单操作失败:', e);
          }
          // 确保菜单被移除
          if (document.body.contains(menu)) {
            document.body.removeChild(menu);
          }
          // 移除事件监听器
          document.removeEventListener('click', closeMenu);
          document.removeEventListener('contextmenu', closeMenu);
        });

        menu.appendChild(menuItem);
      }
    });

    // 添加菜单到页面
    document.body.appendChild(menu);

    // 点击其他地方关闭菜单
    const closeMenu = (e) => {
      if (!menu.contains(e.target)) {
        if (document.body.contains(menu)) {
          document.body.removeChild(menu);
        }
        document.removeEventListener('click', closeMenu);
        document.removeEventListener('contextmenu', closeMenu);
      }
    };

    // 延迟添加事件监听，避免立即触发
    setTimeout(() => {
      document.addEventListener('click', closeMenu);
      document.addEventListener('contextmenu', closeMenu);
    }, 0);
  }

  // 监听文件内容变化
  watch(
    () => props.currentFileContent,
    (newContent, oldContent) => {
      console.log('文件内容变化:', {
        hasEditor: !!editorView,
        hasContainer: !!editorContainer.value,
        newContentLength: newContent?.length,
        oldContentLength: oldContent?.length,
      });

      if (
        editorView &&
        newContent !== undefined &&
        newContent !== editorView.state.doc.toString()
      ) {
        // 使用 nextTick 确保 DOM 更新完成后再更新编辑器内容
        nextTick(() => {
          if (editorView) {
            updateEditorContent(newContent, props.currentFileName);
          }
        });
      } else if (!editorView && newContent && props.currentFileName) {
        // 如果编辑器还没有创建但有内容，尝试创建编辑器
        console.log('尝试为内容创建编辑器');
        nextTick(() => {
          if (editorContainer.value && !editorView) {
            const languageExt = getLanguageExtension(props.currentFileName);
            const state = EditorState.create({
              doc: newContent || '',
              extensions: createEditorExtensionsWithListener(languageExt),
            });
            editorView = new EditorView({
              state,
              parent: editorContainer.value,
            });
            console.log('为内容创建编辑器成功');
          }
        });
      }
    },
    { immediate: false }
  );

  // 监听文件名变化，创建或更新编辑器
  watch(
    () => props.currentFileName,
    (newFileName, oldFileName) => {
      console.log('文件名变化:', {
        oldFileName,
        newFileName,
        hasEditor: !!editorView,
        hasContainer: !!editorContainer.value,
      });

      if (newFileName) {
        // 有文件名时，创建或更新编辑器
        if (editorView) {
          // 更新现有编辑器的语言支持
          const languageExt = getLanguageExtension(newFileName);
          const currentContent = editorView.state.doc.toString();
          const newState = EditorState.create({
            doc: currentContent,
            extensions: createEditorExtensionsWithListener(languageExt),
          });
          editorView.setState(newState);
        } else if (editorContainer.value) {
          // 创建新编辑器
          const languageExt = getLanguageExtension(newFileName);
          const state = EditorState.create({
            doc: props.currentFileContent || '',
            extensions: createEditorExtensionsWithListener(languageExt),
          });
          editorView = new EditorView({
            state,
            parent: editorContainer.value,
          });
          console.log('文件名变化时创建编辑器成功');
        } else {
          // 容器还没有准备好，等待 nextTick
          console.log('容器未准备好，等待 nextTick');
          nextTick(() => {
            if (editorContainer.value && !editorView) {
              const languageExt = getLanguageExtension(newFileName);
              const state = EditorState.create({
                doc: props.currentFileContent || '',
                extensions: createEditorExtensionsWithListener(languageExt),
              });
              editorView = new EditorView({
                state,
                parent: editorContainer.value,
              });
              console.log('nextTick 后创建编辑器成功');
            }
          });
        }
      } else if (oldFileName && !newFileName) {
        // 只有在从有文件名变为无文件名时才销毁编辑器
        // 避免在初始化过程中错误销毁
        if (editorView) {
          editorView.destroy();
          editorView = null;
        }
      }
    },
    { immediate: false }
  );

  onMounted(() => {
    console.log('TemplateEditor mounted:', {
      hasContainer: !!editorContainer.value,
      fileName: props.currentFileName,
      hasContent: !!props.currentFileContent,
    });

    // 使用 nextTick 确保 DOM 元素准备好
    nextTick(() => {
      console.log('TemplateEditor nextTick:', {
        hasContainer: !!editorContainer.value,
        fileName: props.currentFileName,
        hasContent: !!props.currentFileContent,
      });

      // 只有在有文件名和容器时才创建编辑器
      if (editorContainer.value && props.currentFileName) {
        const languageExt = getLanguageExtension(props.currentFileName);

        const state = EditorState.create({
          doc: props.currentFileContent || '',
          extensions: createEditorExtensionsWithListener(languageExt),
        });

        editorView = new EditorView({
          state,
          parent: editorContainer.value,
        });

        console.log('编辑器初始化成功:', { fileName: props.currentFileName });
      } else {
        console.log('编辑器初始化跳过:', {
          hasContainer: !!editorContainer.value,
          fileName: props.currentFileName,
        });
      }
    });
  });

  onBeforeUnmount(() => {
    if (editorView) {
      editorView.destroy();
    }
  });
</script>

<style scoped>
  .edit-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--editor-panel-bg, #fff);
  }

  /* 文件头部样式 */
  .file-header {
    height: 48px;
    background: var(--editor-panel-bg, #ffffff);
    border-bottom: 1px solid var(--editor-border, #e0e0e0);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }

  .file-info {
    display: flex;
    align-items: center;
  }

  .file-name {
    font-size: 14px;
    font-weight: bold;
    color: var(--editor-primary, #333);
  }

  .file-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* 编辑器容器 */
  .codemirror-container {
    flex: 1;
    overflow: auto;
    background: #1e1e1e;
    border-radius: 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
    min-height: 400px;
  }

  /* 空状态样式 */
  .no-file-selected {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--editor-muted, #999);
    background: var(--editor-panel-bg, #fff);
  }

  .no-file-icon {
    margin-bottom: 16px;
  }

  .no-file-text {
    font-size: 16px;
    color: var(--editor-muted, #999);
  }

  /* 模态框样式 */
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }

  .modal-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .html-preview-container {
    height: 100%;
    overflow: hidden;
  }

  .html-preview-frame {
    width: 100%;
    height: 100%;
    border: none;
    border-radius: 4px;
  }

  /* 右键菜单样式 */
  .context-menu {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .context-menu-item:hover {
    background: #3c3c3c !important;
  }
</style>
