<template>
  <div class="template-editor-fullscreen">
    <EditHeader
      :is-file-tree-visible="isFileTreeVisible"
      :is-variable-sidebar-visible="isVariableSidebarVisible"
      :has-unsaved-changes="hasUnsavedChanges"
      :current-file-name="currentFileName"
      :mode="templateType"
      @toggle-variable-sidebar="toggleVariableSidebar"
      @close-edit="closeEdit"
      @toggle-file-tree="toggleFileTree"
      @show-advanced="openAdvancedTab('engine')"
      @full-render="showFullRenderDrawer = true"
    />

    <!-- 自动保存指示器 -->
    <transition name="auto-save-fade">
      <div v-if="autoSaveIndicator" class="auto-save-indicator">
        <n-icon size="16" style="margin-right: 6px">
          <svg viewBox="0 0 24 24">
            <path
              fill="currentColor"
              d="M17 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z"
            />
          </svg>
        </n-icon>
        已自动保存
      </div>
    </transition>

    <div class="edit-main">
      <!-- 左侧栏：文件树 + 变量面板 -->
      <div v-show="shouldShowLeftSidebar" class="left-sidebar">
        <!-- 文件树面板 -->
        <TemplateExplorer
          v-show="isFileTreeVisible"
          v-model:treeData="treeData"
          :currentFile="currentFile"
          :template-name="templateInfo?.name || ''"
          @select="onSelectFile"
          @reload="onTreeReload"
          @rename="onRenameFile"
          @upload-zip="onUploadZip"
          @upload-code-file="onUploadCodeFile"
          @move="onMoveFile"
          @set-condition="onSetCondition"
          @show-releases="showReleasesModal = true"
        />

        <!-- 变量侧边栏 -->
        <VariableSidebar
          v-show="isVariableSidebarVisible"
          :template-variables="templateVariables"
          :template-syntax-categories="templateSyntaxCategories"
          :builtin-function-categories="builtinFunctionCategories"
          :loading-functions="loadingFunctions"
          :quick-variables="quickVariables"
          :template-id="route.params.id"
          :width="variableSidebarWidth"
          @insert-syntax="insertSyntax"
          @insert-function="insertFunction"
          @insert-variable="insertVariable"
          @insert-preset-variable="insertPresetVariable"
          @update:width="updateVariableSidebarWidth"
          @show-quick-design="showQuickDesign"
          @show-test-data="showTestDataFromHeader"
        />
      </div>

      <!-- 中间：编辑器区域 -->
      <div class="editor-container" :class="{ 'full-width': isEditorFullWidth }">
        <TemplateEditor
          ref="templateEditorRef"
          :currentFileName="currentFileName"
          :currentFileId="currentFileId"
          :currentFileContent="currentFileContent"
          :currentFilePath="currentFilePath"
          :templateId="route.params.id"
          @content-change="onEditorContentChange"
          @insert-variable="onInsertVariable"
          @preview="onPreview"
          @selection-change="onEditorSelectionChange"
          @save-success="onSaveSuccess"
          @reload-file="onReloadFile"
        />
      </div>

      <!-- 右侧：预览面板 -->
      <TemplatePreview
        ref="templatePreviewRef"
        :current-file="currentFileNode"
        :template-id="route.params.id"
        :file-path="currentFilePath"
        :file-content="currentFileContent"
        :variables="variableValues"
      />
    </div>

    <!-- 条件设置弹框 -->
    <ConditionModal
      ref="conditionModalRef"
      v-model:show="showConditionModal"
      :selected-file-for-condition="selectedFileForCondition"
      :user-variables="userVariables"
      @close="showConditionModal = false"
      @saved="handleConditionSave"
    />

    <!-- 编辑器设置面板 - 已移至高级面板 -->
    <!-- <EditorSettings
      v-model:show="showSettings"
      :settings="editorSettings"
      @save-settings="saveSettings"
    /> -->

    <!-- 快速设计抽屉 -->
    <QuickDesignDrawer
      ref="quickDesignDrawerRef"
      v-model:show="showQuickDesignDrawer"
      :template-id="route.params.id"
      @save="handleQuickDesignSave"
      @test-data-updated="handleTestDataUpdated"
    />

    <!-- 高级设置抽屉 -->
    <AdvancedDrawer
      ref="advancedDrawerRef"
      v-model:show="showAdvancedDrawer"
      :settings="editorSettings"
      :template-id="route.params.id"
      @save-settings="saveSettings"
      @restore-complete="handleRestoreComplete"
    />

    <!-- 全量渲染抽屉 -->
    <FullRenderDrawer
      v-model:show="showFullRenderDrawer"
      :template-id="route.params.id"
      :template-name="templateInfo?.name || ''"
      :variables="variableValues"
    />

    <!-- 版本管理对话框 -->
    <ReleaseManager
      v-model:show="showReleasesModal"
      :template-id="parseInt(route.params.id)"
      @reset="loadTree"
    />
  </div>
</template>

<script setup>
  import { useRouter, useRoute } from 'vue-router';
  import { ref, onMounted, onUnmounted, watch, computed, nextTick } from 'vue';
  import {
    getTemplateFileTree,
    addTemplateFile,
    delTemplateFile,
    getTemplateFileDetail,
    getTemplateFileContent,
    renameTemplateFile,
    uploadZipFile,
    uploadCodeFile,
    moveTemplateFile,
  } from '@/api/templateFiles';
  import { getTemplateExpose, setTemplateExpose } from '@/api/templateExpose';
  import { getBuiltinFunctions } from '@/api/builtinFunctions';
  import { getTemplateDetail } from '@/api/templates';
  import TemplateExplorer from './components/TemplateFileTree.vue';
  import TemplateEditor from './components/TemplateEditor.vue';
  import TemplatePreview from './components/TemplatePreview.vue';
  import EditHeader from './components/EditHeader.vue';
  import ConditionModal from './components/ConditionModal.vue';
  import VariableSidebar from './components/VariableSidebar.vue';
  import EditorSettings from './components/EditorSettings.vue';
  import AdvancedDrawer from './components/AdvancedDrawer.vue';
  import FullRenderDrawer from './components/FullRenderDrawer.vue';
  import ReleaseManager from './components/ReleaseManager.vue';
  import QuickDesignDrawer from './components/QuickDesignDrawer/index.vue';
  import { templateSyntaxCategories as syntaxData } from './data/templateSyntax';
  import { useTemplateFileStore } from '@/store/modules/templateFileStore';
  import { useMessage, NIcon } from 'naive-ui';

  // Props
  const props = defineProps({
    mode: {
      type: String,
      default: 'user', // 'user' | 'admin'
      validator: (value) => ['user', 'admin'].includes(value),
    },
  });

  const router = useRouter();
  const route = useRoute();
  const message = useMessage();

  // 编辑器模式
  const editorMode = computed(() => props.mode);

  // 关闭编辑器
  const closeEdit = () => {
    // 根据模式返回到不同的页面
    if (editorMode.value === 'admin') {
      router.push('/admin/templates');
    } else {
      router.push('/templates');
    }
  };

  // 以下所有代码都是从原始 templates-edit/index.vue 复制过来的
  // 保持所有原有功能不变

  const showQuickDesignDrawer = ref(false);
  const quickDesignDrawerRef = ref(null);
  const advancedDrawerRef = ref(null);

  const openAdvancedTab = (tabName) => {
    nextTick(() => {
      if (advancedDrawerRef.value) {
        advancedDrawerRef.value.openTab(tabName);
      }
    });
  };

  const showTestDataFromHeader = () => {
    showQuickDesignDrawer.value = true;
    nextTick(() => {
      if (quickDesignDrawerRef.value) {
        quickDesignDrawerRef.value.showTestDataModal();
      }
    });
  };

  // 快速设计模式处理函数
  const showQuickDesign = () => {
    showQuickDesignDrawer.value = true;
  };

  // 快速设计抽屉事件处理
  const handleQuickDesignSave = async (schema) => {
    console.log('快速设计保存:', schema);

    if (!route.params.id) {
      message.error('模板ID不存在，无法保存');
      return;
    }

    try {
      // 调用API保存到后端
      await setTemplateExpose({
        templateId: parseInt(route.params.id),
        varsSchema: schema,
        version: '1.0',
      });

      // 同时保存到本地存储作为备份
      localStorage.setItem(`template_${route.params.id}_vars_schema`, JSON.stringify(schema));

      // 保存成功后清空草稿缓存
      if (quickDesignDrawerRef.value) {
        quickDesignDrawerRef.value.clearDrafts();
      }

      message.success('变量定义保存成功');

      // 刷新用户变量
      await refreshUserVariables();
    } catch (error) {
      console.error('快速设计保存失败:', error);
      message.error('保存失败：' + (error.message || '未知错误'));
    }
  };

  const refreshUserVariables = async () => {
    await loadUserVariables();
  };

  const handleTestDataUpdated = (testData) => {
    console.log('收到测试数据更新事件:', testData);
    updateVariableValues(testData);
  };

  const updateVariableValues = (testData) => {
    console.log('更新预览变量数据:', testData);
    variableValues.value = testData || {};

    const templateId = route.params.id;
    const testDataKey = `template_studio_${templateId}_testdata`;
    localStorage.setItem(testDataKey, JSON.stringify(variableValues.value));

    // 注意：不需要手动调用 renderTemplateContent()
    // TemplatePreview 组件的 watch 会自动监听 props.variables 的变化并触发渲染
    // 删除手动调用可以避免重复渲染（之前会渲染两次）
  };

  const treeData = ref([]);
  const loadingTree = ref(true);
  const noTreeData = ref(false);
  const currentFile = ref('');
  const currentFileContent = ref('');
  const currentFileName = ref('');
  const currentFileId = ref('');
  const currentFilePath = ref('');
  const templateFileStore = useTemplateFileStore();

  const templateVariables = ref([]);
  const userVariables = ref([]);
  const templateEditorRef = ref(null);
  const conditionModalRef = ref(null);

  const variableValues = ref({});
  const currentFileNode = ref(null);
  const templatePreviewRef = ref(null);
  const previewFileId = ref(null);

  const editorSelection = ref({
    hasSelection: false,
    selectedText: '',
    selectionStart: 0,
    selectionEnd: 0,
    selectionLength: 0,
  });

  const showConditionModal = ref(false);
  const selectedFileForCondition = ref(null);

  const quickVariables = [
    { name: 'ProjectName', label: '项目名称' },
    { name: 'Author', label: '作者' },
    { name: 'PackageName', label: '包名' },
  ];

  const builtinFunctionCategories = ref([]);
  const loadingFunctions = ref(false);

  const functionDetailVisible = ref(false);
  const selectedFunction = ref(null);
  const functionDetailStyle = ref({});
  let hideTimer = null;
  let showTimer = null;

  const isVariableSidebarVisible = ref(
    localStorage.getItem('template-variable-sidebar-visible') === null
      ? false
      : localStorage.getItem('template-variable-sidebar-visible') === 'true'
  );
  const variableSidebarWidth = ref(280);

  const isFileTreeVisible = ref(
    localStorage.getItem('template-file-tree-visible') === null
      ? true
      : localStorage.getItem('template-file-tree-visible') !== 'false'
  );

  const showAdvancedDrawer = ref(false);
  const showFullRenderDrawer = ref(false);
  const showReleasesModal = ref(false);

  const editorSettings = ref({
    autoSave: {
      enabled: true,
      interval: 30,
    },
    editor: {
      fontSize: 14,
      lineNumbers: true,
      wordWrap: true,
    },
    interface: {
      theme: 'light',
      restoreLayout: true,
    },
    preview: {
      realtime: true,
      debounceDelay: 500,
    },
  });

  const templateSyntaxCategories = ref(syntaxData);

  // 模板类型相关
  const templateType = ref('');
  const templateInfo = ref(null);

  // 计算属性：根据模板类型决定是否显示文件树
  const shouldShowFileTree = computed(() => {
    return templateType.value !== 'basic' && isFileTreeVisible.value;
  });

  // 计算属性：是否显示左侧栏（文件树或变量面板至少一个显示）
  const shouldShowLeftSidebar = computed(() => {
    if (templateType.value === 'basic') {
      return false; // 基础模板不显示左侧栏
    }
    return isFileTreeVisible.value || isVariableSidebarVisible.value;
  });

  // 计算属性：编辑器是否应该占满宽度
  const isEditorFullWidth = computed(() => {
    return templateType.value === 'basic';
  });

  const toggleVariableSidebar = () => {
    isVariableSidebarVisible.value = !isVariableSidebarVisible.value;
    localStorage.setItem(
      'template-variable-sidebar-visible',
      isVariableSidebarVisible.value.toString()
    );
  };

  const updateVariableSidebarWidth = (width) => {
    variableSidebarWidth.value = width;
    localStorage.setItem('template-variable-sidebar-width', width.toString());
  };

  const toggleFileTree = () => {
    isFileTreeVisible.value = !isFileTreeVisible.value;
    localStorage.setItem('template-file-tree-visible', isFileTreeVisible.value.toString());
  };

  const loadSettings = () => {
    const savedSettings = localStorage.getItem('template-editor-settings');
    if (savedSettings) {
      try {
        const parsed = JSON.parse(savedSettings);
        editorSettings.value = { ...editorSettings.value, ...parsed };
      } catch (e) {
        console.warn('Failed to load settings:', e);
      }
    }
  };

  const saveSettings = (newSettings) => {
    editorSettings.value = { ...newSettings };
    localStorage.setItem('template-editor-settings', JSON.stringify(newSettings));
    applySettings();
  };

  // 处理恢复完成事件
  const handleRestoreComplete = async () => {
    // 重新加载文件树
    await loadTree();
    // 重新加载变量定义
    await loadUserVariables();
    // 重新加载测试数据
    loadTestData();
    message.success('模板已恢复，页面数据已刷新');
  };

  const applySettings = () => {
    setupAutoSave();
  };

  let autoSaveTimer = null;
  let isPageVisible = true;

  // 页面可见性变化处理
  const handleVisibilityChange = () => {
    isPageVisible = !document.hidden;
    if (!isPageVisible) {
      // 页面隐藏时暂停自动保存
      if (autoSaveTimer) {
        clearInterval(autoSaveTimer);
        autoSaveTimer = null;
      }
    } else {
      // 页面重新可见时恢复自动保存
      setupAutoSave();
    }
  };

  const setupAutoSave = () => {
    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
      autoSaveTimer = null;
    }

    // 只有在页面可见且启用自动保存时才设置定时器
    if (editorSettings.value.autoSave.enabled && isPageVisible) {
      const interval = editorSettings.value.autoSave.interval * 1000;
      autoSaveTimer = setInterval(() => {
        autoSaveCurrentFile();
      }, interval);
    }
  };

  const saveCurrentFile = async (silent = false) => {
    if (templateEditorRef.value && templateEditorRef.value.saveCurrentFile) {
      const success = await templateEditorRef.value.saveCurrentFile(silent);
      if (success) {
        hasUnsavedChanges.value = false;
      }
      return success;
    }
    return false;
  };

  const hasUnsavedChanges = ref(false);
  const autoSaveIndicator = ref(false);

  const autoSaveCurrentFile = async () => {
    if (currentFileId.value && hasUnsavedChanges.value) {
      try {
        const success = await saveCurrentFile(true);
        if (success) {
          console.log('Auto-saved file:', currentFileName.value);
          showAutoSaveIndicator();
        }
      } catch (error) {
        console.error('Auto-save failed:', error);
      }
    }
  };

  const showAutoSaveIndicator = () => {
    autoSaveIndicator.value = true;
    setTimeout(() => {
      autoSaveIndicator.value = false;
    }, 2000);
  };

  const handleKeyDown = (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'b') {
      event.preventDefault();
      toggleFileTree();
    }

    if ((event.ctrlKey || event.metaKey) && event.key === ',') {
      event.preventDefault();
      openAdvancedTab('editor-settings');
    }
  };

  onMounted(async () => {
    await loadTemplateInfo();
    await loadTree();
    await loadUserVariables();
    await loadBuiltinFunctions();
    loadTestData();

    loadSettings();
    applySettings();

    document.addEventListener('keydown', handleKeyDown);

    // 添加页面可见性监听
    document.addEventListener('visibilitychange', handleVisibilityChange);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);

    // 移除页面可见性监听
    document.removeEventListener('visibilitychange', handleVisibilityChange);

    if (autoSaveTimer) {
      clearInterval(autoSaveTimer);
      autoSaveTimer = null;
    }

    if (hideTimer) {
      clearTimeout(hideTimer);
    }
    if (showTimer) {
      clearTimeout(showTimer);
    }
  });

  // 加载模板信息，获取模板类型
  async function loadTemplateInfo() {
    try {
      const res = await getTemplateDetail({ id: route.params.id });
      if (res.data && res.data.data) {
        const templateData = res.data.data; // 修复：两层 data，不是三层
        templateInfo.value = templateData;
        templateType.value = templateData.templateType || templateData.template_type || 'basic';
        console.log('模板类型:', templateType.value);
      }
    } catch (error) {
      console.error('加载模板信息失败:', error);
      templateType.value = 'basic'; // 默认为基础模板
    }
  }

  async function loadTree() {
    loadingTree.value = true;
    try {
      const res = await getTemplateFileTree(route.params.id);
      const tree = res.data && res.data.data && res.data.data.tree;
      if (tree && tree.length > 0) {
        treeData.value = tree;
        noTreeData.value = false;

        // 基础模板自动选择 index.tmp 文件
        if (templateType.value === 'basic') {
          await nextTick(); // 等待DOM更新
          await autoSelectIndexFile();
        }
      } else {
        treeData.value = [];
        noTreeData.value = true;
      }
    } catch (e) {
      treeData.value = [];
      noTreeData.value = true;
    }
    loadingTree.value = false;
  }

  // 基础模板自动选择 index.tmp 文件
  async function autoSelectIndexFile() {
    if (!treeData.value || treeData.value.length === 0) {
      return;
    }

    // 查找 index.tmp 文件
    let indexFileId = null;

    function findIndexFile(nodes) {
      for (const node of nodes) {
        if (node.fileName === 'index.tmp' && node.isDirectory === 0) {
          return node.key || node.id;
        }
        if (node.children && node.children.length > 0) {
          const found = findIndexFile(node.children);
          if (found) return found;
        }
      }
      return null;
    }

    indexFileId = findIndexFile(treeData.value);

    if (indexFileId) {
      console.log('自动选择 index.tmp 文件:', indexFileId);
      await onSelectFile(indexFileId);
    } else {
      console.warn('未找到 index.tmp 文件');
    }
  }

  const convertSchemaToUserVariables = (schema, parentPath = '') => {
    const variables = [];

    if (!schema || typeof schema !== 'object') {
      return variables;
    }

    Object.keys(schema).forEach((key, index) => {
      const variable = schema[key];
      if (!variable || typeof variable !== 'object') return;

      const fullPath = parentPath ? `${parentPath}.${key}` : key;
      const userVariable = {
        id: `${route.params.id}_${fullPath}`,
        name: key,
        variableType: variable.type || 'string',
        description: variable.description || '',
        defaultValue: variable.default || '',
        isRequired: variable.required ? 1 : 0,
        sort: (variable.ui?.order || 10) + index,
        insertText: variable.insertText || `{{.${key}}}`,
        title: variable.title || key,
        path: fullPath,
        parent: parentPath || null,
        level: parentPath.split('.').length,
        ui: variable.ui || {},
      };

      variables.push(userVariable);

      if (variable.type === 'object' && variable.properties) {
        const childVariables = convertSchemaToUserVariables(variable.properties, fullPath);
        variables.push(...childVariables);
      }
    });

    return variables.sort((a, b) => a.sort - b.sort);
  };

  async function loadUserVariables() {
    try {
      const templateId = parseInt(route.params.id);
      const response = await getTemplateExpose({ templateId });

      if (response.data && response.data.data && response.data.data.templateExpose) {
        const fieldSchemaJson = response.data.data.templateExpose.fieldSchemaJson;
        if (fieldSchemaJson) {
          const schema = JSON.parse(fieldSchemaJson);
          userVariables.value = convertSchemaToUserVariables(schema);
          console.log('加载用户变量成功:', userVariables.value);
        } else {
          userVariables.value = [];
        }
      } else {
        userVariables.value = [];
      }
    } catch (error) {
      console.error('加载用户变量失败:', error);
      userVariables.value = [];
    }
  }

  async function loadBuiltinFunctions() {
    loadingFunctions.value = true;
    try {
      const res = await getBuiltinFunctions();
      if (res.data && res.data.data) {
        builtinFunctionCategories.value = res.data.data.categories || [];
      }
    } catch (error) {
      console.error('加载内置函数失败:', error);
      message.error('加载内置函数失败');
      builtinFunctionCategories.value = [];
    } finally {
      loadingFunctions.value = false;
    }
  }

  function loadTestData() {
    const templateId = route.params.id;
    const testDataKey = `template_studio_${templateId}_testdata`;
    const savedTestData = localStorage.getItem(testDataKey);
    if (savedTestData) {
      try {
        variableValues.value = JSON.parse(savedTestData);
      } catch (e) {
        variableValues.value = {};
      }
    } else {
      variableValues.value = {};
    }
  }

  function findNodeByKey(list, key) {
    for (const item of list) {
      const itemKey = String(item.key || item.id);
      const targetKey = String(key);

      if (itemKey === targetKey) {
        return item;
      }

      if (item.children) {
        const found = findNodeByKey(item.children, key);
        if (found) {
          return found;
        }
      }
    }
    return null;
  }

  // 根据文件路径查找节点（用于 loadTree 后恢复选中状态）
  function findNodeByPath(list, filePath) {
    // 规范化路径：统一使用正斜杠
    const normalizedPath = filePath.replace(/\\/g, '/');
    for (const item of list) {
      const itemPath = (item.filePath || '').replace(/\\/g, '/');
      if (itemPath === normalizedPath) {
        return item;
      }

      if (item.children) {
        const found = findNodeByPath(item.children, normalizedPath);
        if (found) {
          return found;
        }
      }
    }
    return null;
  }

  async function onSelectFile(key) {
    resetEditorSelection();
    hasUnsavedChanges.value = false;

    currentFile.value = key;
    const node = findNodeByKey(treeData.value, key);
    currentFileNode.value = node;

    if (node && node.isDirectory === 0) {
      try {
        const fileName = node.fileName || node.label || String(key);
        const filePath = node.filePath || node.key || key;
        currentFileName.value = fileName;
        currentFileId.value = String(key);
        currentFilePath.value = filePath;

        // 调用文件内容接口，传递模板ID和文件路径
        const templateId = route.params.id;
        const res = await getTemplateFileContent(templateId, filePath);
        const content = res.data.data.content || res.data.data.fileContent;
        currentFileContent.value = content;
        templateFileStore.setCurrentFileContent(content);

        if (previewFileId.value) {
          previewFileId.value = String(key);

          const templateId = route.params.id;
          const testDataKey = `template_studio_${templateId}_testdata`;
          const savedTestData = localStorage.getItem(testDataKey);
          if (savedTestData) {
            try {
              variableValues.value = JSON.parse(savedTestData);
            } catch (e) {
              console.error('解析测试数据失败:', e);
              variableValues.value = {};
            }
          }
        }
      } catch (e) {
        console.error('加载文件内容失败:', e);
        currentFileContent.value = '';
        currentFileName.value = '';
        currentFileId.value = '';
        currentFilePath.value = '';
        templateFileStore.setCurrentFileContent('');
      }
    } else {
      currentFileContent.value = '';
      currentFileName.value = '';
      currentFileId.value = '';
      currentFilePath.value = '';
      templateFileStore.setCurrentFileContent('');
    }
  }

  function onEditorContentChange({ content }) {
    currentFileContent.value = content;
    templateFileStore.setCurrentFileContent(content);
    hasUnsavedChanges.value = true;
  }

  function onEditorSelectionChange(selectionInfo) {
    editorSelection.value = selectionInfo;
  }

  function resetEditorSelection() {
    editorSelection.value = {
      hasSelection: false,
      selectedText: '',
      selectionStart: 0,
      selectionEnd: 0,
      selectionLength: 0,
    };
  }

  function onTreeReload(payload) {
    if (payload && payload.type === 'delete') {
      const templateId = parseInt(route.params.id);
      delTemplateFile({
        templateId,
        filePath: payload.filePath,
      }).then(() => {
        loadTree();
      });
      return;
    }

    const templateId = parseInt(route.params.id);
    const isDirectory = payload.type === 'folder' ? 1 : 0;

    // 获取父级路径
    let parentPath = '';
    if (payload.node && payload.node.filePath) {
      // 如果有父节点，使用父节点的路径
      parentPath = payload.node.filePath;
    }

    addTemplateFile({
      templateId,
      fileName: payload.name,
      parentPath,
      isDirectory,
    }).then(() => {
      loadTree();
    });
  }

  async function onRenameFile(payload) {
    const { id, oldName, newName, node } = payload;

    if (!newName || newName.trim() === '' || newName === oldName) {
      return;
    }

    try {
      // 构造新的文件路径（统一使用正斜杠）
      console.log('onRenameFile 原始数据:', { nodeFilePath: node.filePath, newName });
      const filePath = node.filePath.replace(/\\/g, '/');
      const pathParts = filePath.split('/');
      pathParts[pathParts.length - 1] = newName.trim();
      const newPath = pathParts.join('/');
      console.log('onRenameFile 计算后:', { filePath, newPath });

      // 记录是否为当前选中的文件
      const wasCurrentFile = currentFileId.value === String(id);
      const currentContent = currentFileContent.value;

      await renameTemplateFile({
        templateId: parseInt(route.params.id),
        filePath,
        newPath,
      });

      message.success('重命名成功');
      await loadTree();
      await nextTick(); // 等待 Vue 响应式更新完成

      // 如果重命名的是当前选中的文件，需要根据新路径找到新的节点ID并更新状态
      if (wasCurrentFile) {
        // 根据新路径在新的树数据中找到对应节点
        const newNode = findNodeByPath(treeData.value, newPath);
        console.log('重命名后查找节点:', { newPath, foundNode: newNode, treeData: treeData.value });
        if (newNode) {
          const newId = String(newNode.key || newNode.id);
          currentFileId.value = newId;
          currentFile.value = newId;
          currentFileName.value = newName.trim();
          currentFilePath.value = newPath;
          currentFileNode.value = newNode;
          currentFileContent.value = currentContent;
          templateFileStore.setCurrentFileContent(currentContent);
        }
      }
    } catch (error) {
      console.error('重命名失败:', error);
      message.error(
        '重命名失败: ' + (error.response?.data?.message || error.message || '未知错误')
      );
    }
  }

  async function onUploadZip(payload) {
    const { file } = payload;

    try {
      const res = await uploadZipFile(route.params.id, file);
      const { successCount, failedFiles, message: resultMessage } = res.data.data;

      if (failedFiles && failedFiles.length > 0) {
        message.warning(
          `${resultMessage}，成功 ${successCount} 个文件，失败 ${failedFiles.length} 个文件`
        );
        console.log('失败的文件:', failedFiles);
      } else {
        message.success(`${resultMessage}，成功解压 ${successCount} 个文件`);
      }

      await loadTree();
    } catch (error) {
      console.error('上传错误:', error);
      // 优先使用后端返回的详细错误信息
      const errorMsg = error.response?.data?.message || error.message || '未知错误';
      message.error('ZIP包上传失败：' + errorMsg);
    }
  }

  async function onUploadCodeFile(payload) {
    const { file, parentPath } = payload;
    try {
      const res = await uploadCodeFile(route.params.id, file, parentPath);
      const { fileName, isTextFile, message: resultMessage } = res.data.data;
      if (isTextFile) {
        message.success(`${resultMessage}：${fileName}（文本文件）`);
      } else {
        message.success(`${resultMessage}：${fileName}（非文本文件）`);
      }
      await loadTree();
    } catch (error) {
      message.error(
        '代码文件上传失败：' + (error.response?.data?.message || error.message || '未知错误')
      );
    }
  }

  async function onMoveFile(payload) {
    const { sourceId, targetId, sourceNode, targetNode, isRootDrop } = payload;

    if (isRootDrop && (!sourceId || sourceId === 'unknown')) {
      console.log('根目录拖拽，但缺少源节点信息，忽略此次移动');
      message.warning('拖拽移动需要明确的源文件信息');
      return;
    }

    if (!sourceId || sourceId === 'unknown') {
      console.error('移动失败：缺少源文件ID');
      message.error('移动失败：缺少源文件信息');
      return;
    }

    // 检查是否有文件路径信息
    const rawFilePath = sourceNode?.filePath;
    if (!rawFilePath) {
      message.error('移动失败：缺少源文件路径信息');
      return;
    }

    // 规范化路径（统一使用正斜杠）
    const sourceFilePath = rawFilePath.replace(/\\/g, '/');

    try {
      const templateId = parseInt(route.params.id);

      // 获取文件名（从路径中提取）
      const fileName =
        sourceFilePath.split('/').pop() || sourceNode?.fileName || sourceNode?.label || 'unknown';

      // 构建新路径（统一使用正斜杠）
      let newPath = '';
      if (targetId === '0' || !targetNode?.filePath) {
        // 移动到根目录
        newPath = fileName;
      } else {
        // 移动到文件夹内
        const targetPath = targetNode.filePath.replace(/\\/g, '/');
        newPath = targetPath + '/' + fileName;
      }

      console.log('onDropFile:', { sourceFilePath, fileName, newPath });

      await moveTemplateFile({
        templateId,
        filePath: sourceFilePath,
        newPath,
      });

      const targetName = targetId === '0' ? '根目录' : targetNode?.fileName || '未知目录';
      const sourceName = sourceNode?.fileName || sourceNode?.label || '未知文件';
      message.success(`已将 "${sourceName}" 移动到 "${targetName}"`);

      const wasCurrentFile = currentFileId.value === String(sourceId);
      const currentContent = currentFileContent.value;
      const currentName = currentFileName.value;

      await loadTree();
      await nextTick(); // 等待 Vue 响应式更新完成

      // 根据新路径找到新的节点ID并更新状态
      if (wasCurrentFile) {
        const newNode = findNodeByPath(treeData.value, newPath);
        console.log('移动后查找节点:', { newPath, foundNode: newNode });
        if (newNode) {
          const newId = String(newNode.key || newNode.id);
          currentFileId.value = newId;
          currentFile.value = newId;
          currentFileName.value = currentName;
          currentFilePath.value = newPath;
          currentFileNode.value = newNode;
          currentFileContent.value = currentContent;
          templateFileStore.setCurrentFileContent(currentContent);
        }
      }
    } catch (error) {
      console.error('移动失败:', error);

      if (error.response?.status === 404) {
        message.error('移动功能暂未实现，请联系管理员添加后端接口');
      } else {
        message.error(
          '移动失败: ' + (error.response?.data?.message || error.message || '未知错误')
        );
      }
    }
  }

  watch(treeData, (val) => {}, { deep: true });

  function onInsertVariable(template) {
    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(template);
    }
  }

  function onPresetRefresh() {
    console.log('预设变量已刷新');
  }

  function onPreview({ fileId, fileName }) {
    previewFileId.value = fileId;

    const templateId = route.params.id;
    const testDataKey = `template_studio_${templateId}_testdata`;
    const savedTestData = localStorage.getItem(testDataKey);
    if (savedTestData) {
      try {
        variableValues.value = JSON.parse(savedTestData);
      } catch (e) {
        console.error('解析测试数据失败:', e);
        variableValues.value = {};
      }
    } else {
      variableValues.value = {};
    }

    if (templatePreviewRef.value) {
      templatePreviewRef.value.expandPanel();
    }

    message.success(`正在预览: ${fileName}`);
  }

  // 保存成功后，如果预览面板处于展开状态，自动触发渲染
  function onSaveSuccess({ fileId, fileName, filePath }) {
    console.log('文件已保存:', fileName);

    // 检查预览面板是否已展开
    if (templatePreviewRef.value && !templatePreviewRef.value.isCollapsed) {
      console.log('预览面板已展开，自动触发渲染');
      templatePreviewRef.value.renderTemplateContent();
    } else {
      console.log('预览面板未展开，跳过自动渲染');
    }
  }

  // 重新加载文件内容（git restore 后调用）
  async function onReloadFile({ fileId, filePath }) {
    console.log('重新加载文件:', filePath);
    try {
      const templateId = route.params.id;
      const res = await getTemplateFileContent(templateId, filePath);
      const content = res.data.data.content || res.data.data.fileContent;
      currentFileContent.value = content;
      templateFileStore.setCurrentFileContent(content);
      hasUnsavedChanges.value = false;

      // 如果预览面板处于展开状态，自动触发渲染
      if (templatePreviewRef.value && !templatePreviewRef.value.isCollapsed) {
        console.log('预览面板已展开，自动触发渲染');
        templatePreviewRef.value.renderTemplateContent();
      }
    } catch (e) {
      console.error('重新加载文件失败:', e);
    }
  }

  function insertQuickVariable(quickVar) {
    clearShowTimer();
    clearHideTimer();
    functionDetailVisible.value = false;

    const code = `{{.${quickVar.name}}}`;

    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(code);
    }
  }

  function showFunctionDetail(func, event) {
    clearHideTimer();
    clearShowTimer();

    showTimer = setTimeout(() => {
      selectedFunction.value = func;

      const panelWidth = 300;
      const panelHeight = 200;
      const offset = 8;

      let left = event.clientX + offset;
      let top = event.clientY + offset;

      if (left + panelWidth > window.innerWidth - 10) {
        left = event.clientX - panelWidth - offset;
      }
      if (top + panelHeight > window.innerHeight - 10) {
        top = event.clientY - panelHeight - offset;
      }

      functionDetailStyle.value = {
        left: `${left}px`,
        top: `${top}px`,
      };

      functionDetailVisible.value = true;
    }, 800);
  }

  function clearHideTimer() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }

  function clearShowTimer() {
    if (showTimer) {
      clearTimeout(showTimer);
      showTimer = null;
    }
  }

  function onDetailPanelEnter() {
    clearHideTimer();
    clearShowTimer();
  }

  function hideFunctionDetail() {
    clearShowTimer();

    hideTimer = setTimeout(() => {
      functionDetailVisible.value = false;
      selectedFunction.value = null;
    }, 300);
  }

  function formatFunction(func) {
    return {
      name: func.name,
      label: func.display_name || func.name,
      code: func.insert_text || `{{ ${func.name} }}`,
      description: func.description,
    };
  }

  function insertFunction(func) {
    clearShowTimer();
    clearHideTimer();
    functionDetailVisible.value = false;

    // 从后端数据中提取 insert_text 字段
    const code = func.insert_text || func.insertText || `{{ ${func.name} }}`;

    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(code);
    }
  }

  function insertSyntax(syntax) {
    clearShowTimer();
    clearHideTimer();
    functionDetailVisible.value = false;

    const code = syntax.insertText || syntax.syntax;

    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(code);
    }
  }

  function insertVariable(variableName) {
    clearShowTimer();
    clearHideTimer();
    functionDetailVisible.value = false;

    const code = `{{.${variableName}}}`;

    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(code);
    }
  }

  function insertPresetVariable(insertText) {
    clearShowTimer();
    clearHideTimer();
    functionDetailVisible.value = false;

    if (templateEditorRef.value) {
      templateEditorRef.value.insertVariable(insertText);
    }
  }

  async function onSetCondition(fileNode) {
    // 新的 ConditionModal 组件会自动处理条件加载和设置
    selectedFileForCondition.value = fileNode;
    showConditionModal.value = true;
  }

  async function handleConditionSave() {
    // 新的 ConditionModal 组件会在内部完成所有 API 调用
    // 这里只需要刷新文件树即可
    showConditionModal.value = false;
    await loadTree();
  }
</script>

<style scoped>
  .template-editor-fullscreen {
    position: fixed;
    inset: 0;
    background: var(--editor-bg, #f5f5f5);
    display: flex;
    flex-direction: column;
  }

  .edit-main {
    flex: 1;
    display: flex;
    min-height: 0;
    transition: all 0.2s ease;
  }

  .left-sidebar {
    display: flex;
    min-height: 0;
    border-right: 1px solid var(--editor-border, #e2e8f0);
    background: var(--editor-panel-bg, #fafbfc);
    transition: all 0.2s ease;
  }

  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
    transition: all 0.2s ease;
  }

  /* 基础模板编辑器占满宽度 */
  .editor-container.full-width {
    flex: 1;
    width: 100%;
  }

  .auto-save-indicator {
    position: fixed;
    top: 80px;
    right: 24px;
    background: var(--editor-accent, #22c55e);
    color: #fff;
    padding: 8px 16px;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
    font-size: 14px;
    font-weight: 500;
    z-index: 2000;
    display: flex;
    align-items: center;
  }

  .auto-save-fade-enter-active,
  .auto-save-fade-leave-active {
    transition: all 0.2s ease;
  }

  .auto-save-fade-enter-from {
    opacity: 0;
    transform: translateY(-10px) scale(0.95);
  }

  .auto-save-fade-leave-to {
    opacity: 0;
    transform: translateY(-10px) scale(0.95);
  }
</style>
