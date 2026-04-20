<template>
  <div class="simple-var-preset-editor">
    <div class="editor-layout">
      <!-- 左侧树形结构 -->
      <div
        class="tree-panel"
        :style="{ width: treePanelWidth + 'px' }"
        @contextmenu="onTreeAreaContextMenu"
      >
        <div class="tree-header">
          <h4>数据结构</h4>
        </div>
        <div class="tree-content" ref="treeContainer">
          <div v-if="treeData.length === 0" class="empty-tree" @contextmenu="onTreeAreaContextMenu">
            <p>暂无数据，右键添加根字段开始创建</p>
          </div>
          <a-tree
            v-if="treeData.length > 0"
            :tree-data="treeDataForAnt"
            :selected-keys="selectedNode ? [selectedNode.key] : []"
            :expanded-keys="Array.from(expandedKeys)"
            @select="onSelectNode"
            @expand="updateExpanded"
            draggable
            @dragstart="onDragStart"
            @dragenter="onDragEnter"
            @dragleave="onDragLeave"
            @dragover="onDragOver"
            @drop="onDrop"
            @rightClick="onRightClick"
          >
            <template #title="{ key, title, hasChildren, description }">
              <div style="display: flex; align-items: center; width: 100%;">
                <span style="font-weight: 500;">{{ title }}</span>
                <span v-if="description" style="font-size: 12px; color: #999; margin-left: 8px; font-style: italic;">{{ description }}</span>
              </div>
            </template>
          </a-tree>
        </div>

        <!-- 右键菜单 -->
        <teleport to="body">
          <div
            v-if="showDropdown"
            class="context-menu-overlay"
            @click="handleDropdownClickoutside"
            @contextmenu.prevent
          >
            <div
              class="context-menu"
              :style="{ left: dropdownX + 'px', top: dropdownY + 'px' }"
            >
              <a-menu @click="({key}) => handleDropdownSelect(key)">
                <template v-for="opt in dropdownOptions" :key="opt.key || opt.type">
                  <a-menu-divider v-if="opt.type === 'divider'" />
                  <a-menu-item v-else :key="opt.key">
                    <span v-if="opt.icon" style="margin-right: 8px"><component :is="opt.icon" /></span>
                    {{ opt.label }}
                  </a-menu-item>
                </template>
              </a-menu>
            </div>
          </div>
        </teleport>
      </div>

      <!-- 左侧拖拽分隔线 -->
      <div class="resizer tree-resizer" @mousedown="startResize('tree', $event)"></div>

      <!-- 中间详情编辑区域 -->
      <div class="detail-panel" :style="{ width: detailPanelWidth + 'px' }">
        <div class="detail-header">
          <h4>{{ selectedNode ? '编辑节点' : '选择节点' }}</h4>
          <div class="detail-actions" v-if="selectedNode">
            <a-button size="small" @click="saveCurrentNode" type="primary" :loading="nodeSaving">
              <template #icon>
                <SaveOutline />
              </template>
              保存
            </a-button>
          </div>
        </div>
        <div class="detail-content">
          <div v-if="!selectedNode" class="empty-detail">
            <a-empty description="请从左侧选择一个节点进行编辑">
              <template #image>
                <DocumentTextOutline style="font-size: 48px; color: #ccc" />
              </template>
            </a-empty>
          </div>
          <div v-else class="node-form">
            <a-form
              ref="detailFormRef"
              :model="nodeForm"
              :rules="nodeFormRules"
              layout="vertical"
            >
              <a-row :gutter="16">
                <a-col :span="12">
                  <a-form-item label="字段名" name="key">
                    <a-input v-model:value="nodeForm.key" placeholder="请输入字段名" />
                  </a-form-item>
                </a-col>
                <a-col :span="12">
                  <a-form-item label="显示名称" name="displayName">
                    <a-input v-model:value="nodeForm.displayName" placeholder="显示名称（可选）" />
                  </a-form-item>
                </a-col>
              </a-row>

              <a-form-item
                v-if="selectedNode && selectedNode.path && selectedNode.path.length === 1"
                label="分类"
                name="category"
              >
                <a-select
                  v-model:value="nodeForm.category"
                  placeholder="输入或选择分类（仅一级字段）"
                  :options="categoryOptions"
                  show-search
                  allow-clear
                  @change="handleCreateCategory"
                />
              </a-form-item>

              <a-form-item label="描述" name="description">
                <a-textarea
                  v-model:value="nodeForm.description"
                  placeholder="字段描述（可选）"
                  :rows="2"
                />
              </a-form-item>

              <a-row :gutter="16">
                <a-col :span="12">
                  <a-form-item label="示例值" name="example">
                    <a-input v-model:value="nodeForm.example" placeholder="示例值（如：张三）" />
                  </a-form-item>
                </a-col>
                <a-col :span="12">
                  <a-form-item label="插入文本" name="insertText">
                    <a-input
                      v-model:value="nodeForm.insertText"
                      placeholder="自动生成（基于字段名）"
                      readonly
                      style="background-color: #f5f5f5"
                    />
                  </a-form-item>
                </a-col>
              </a-row>

              <a-form-item label="包含子字段" name="hasChildren">
                <a-switch v-model:checked="nodeForm.hasChildren" />
                <span style="margin-left: 8px; color: #666; font-size: 12px">
                  {{ nodeForm.hasChildren ? '该字段可以包含子字段' : '该字段为普通字段' }}
                </span>
              </a-form-item>
            </a-form>
          </div>
        </div>
      </div>

      <!-- 中间拖拽分隔线 -->
      <div class="resizer detail-resizer" @mousedown="startResize('detail', $event)"></div>

      <!-- 右侧 JSON 编辑器 -->
      <div class="editor-panel">
        <div class="editor-header">
          <h4>JSON 预览</h4>
          <div class="editor-actions">
            <a-button size="small" @click="importJson" type="text">
              <template #icon>
                <CloudUploadOutline />
              </template>
              导入文件
            </a-button>
            <a-button size="small" @click="exportJson" type="text">
              <template #icon>
                <CloudDownloadOutline />
              </template>
              导出文件
            </a-button>
            <a-button size="small" @click="copyJson" type="text">
              <template #icon>
                <CopyOutline />
              </template>
              复制
            </a-button>
            <a-button size="small" @click="formatJson" type="text">
              <template #icon>
                <CodeOutline />
              </template>
              格式化
            </a-button>
            <a-button size="small" @click="syncFromJson" type="text">
              <template #icon>
                <SyncOutline />
              </template>
              同步到树
            </a-button>
          </div>
        </div>
        <div class="editor-content">
          <div class="codemirror-container" ref="editorContainer"></div>
        </div>
        <div class="editor-status">
          <span :class="['status-indicator', jsonValid ? 'valid' : 'invalid']">
            {{ jsonValid ? '✓ JSON 有效' : '✗ JSON 无效' }}
          </span>
        </div>
      </div>
    </div>

    <!-- 节点编辑弹窗 -->
    <a-modal
      v-model:open="showNodeModal"
      :title="nodeModalTitle"
      :mask-closable="false"
      :width="500"
      :footer="null"
      @cancel="closeNodeModal"
    >
      <a-form ref="nodeFormRef" :model="nodeForm" :rules="nodeFormRules" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="字段名" name="key">
              <a-input v-model:value="nodeForm.key" placeholder="请输入字段名" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="显示名称" name="displayName">
              <a-input v-model:value="nodeForm.displayName" placeholder="显示名称（可选）" />
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item v-if="isRootLevel" label="分类" name="category">
          <a-select
            v-model:value="nodeForm.category"
            placeholder="输入或选择分类（仅一级字段）"
            :options="categoryOptions"
            show-search
            allow-clear
            @change="handleCreateCategory"
          />
        </a-form-item>

        <a-form-item label="描述" name="description">
          <a-textarea
            v-model:value="nodeForm.description"
            placeholder="字段描述（可选）"
            :rows="2"
          />
        </a-form-item>

        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="示例值" name="example">
              <a-input v-model:value="nodeForm.example" placeholder="示例值（如：张三）" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="插入文本" name="insertText">
              <a-input
                v-model:value="nodeForm.insertText"
                placeholder="自动生成（基于字段名）"
                readonly
                style="background-color: #f5f5f5"
              />
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item label="包含子字段" name="hasChildren">
          <a-switch v-model:checked="nodeForm.hasChildren" />
          <span style="margin-left: 8px; color: #666; font-size: 12px">
            {{ nodeForm.hasChildren ? '该字段可以包含子字段' : '该字段为普通字段' }}
          </span>
        </a-form-item>
      </a-form>

      <div class="modal-footer">
        <a-button @click="closeNodeModal">取消</a-button>
        <a-button type="primary" @click="saveNode" :loading="nodeSaving">
          {{ editingNode ? '更新' : '添加' }}
        </a-button>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
  import { ref, reactive, computed, onMounted, onUnmounted, watch, nextTick, h } from 'vue';
  import { message } from 'ant-design-vue';
  import {
    AddOutline,
    TrashOutline,
    CodeOutline,
    SyncOutline,
    SaveOutline,
    DocumentTextOutline,
    CopyOutline,
    CloudUploadOutline,
    CloudDownloadOutline,
  } from '@/icons/ionicons5';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { json } from '@codemirror/lang-json';

  const props = defineProps({
    modelValue: {
      type: String,
      default: '',
    },
    readonly: {
      type: Boolean,
      default: false,
    },
  });

  const emit = defineEmits(['update:modelValue']);

  // 编辑器相关
  const editorContainer = ref(null);
  let editorView = null;

  // 数据状态
  const treeData = ref([]);
  const jsonContent = ref('');
  const jsonValid = ref(true);

  // 面板宽度状态
  const treePanelWidth = ref(280);
  const detailPanelWidth = ref(320);

  // 拖拽状态
  const resizing = ref(false);
  const resizeType = ref('');
  const startX = ref(0);
  const startWidth = ref(0);

  // 节点选择和编辑相关
  const selectedNode = ref(null);
  const showNodeModal = ref(false);
  const nodeFormRef = ref(null); // 弹窗表单
  const detailFormRef = ref(null); // 详情面板表单
  const editingNode = ref(null);
  const editingNodePath = ref([]);
  const nodeSaving = ref(false);
  const originalNodeData = ref(null); // 保存原始节点数据
  const isModalCanceling = ref(false); // 标识是否正在取消编辑

  // 导入导出相关 (现在使用文件选择器)

  // 右键菜单相关
  const showDropdown = ref(false);
  const dropdownX = ref(0);
  const dropdownY = ref(0);
  const dropdownNode = ref(null);
  const dropdownOptions = ref([]);

  // 树形结构相关
  const treeContainer = ref(null);
  const expandedKeys = ref(new Set());
  const draggedNode = ref(null);
  const dragOverNode = ref(null);
  const isDragging = ref(false);

  const nodeForm = reactive({
    key: '',
    displayName: '',
    description: '',
    example: '',
    insertText: '',
    category: '',
    hasChildren: false, // 是否包含子字段
  });

  const nodeFormRules = {
    key: {
      required: true,
      message: '请输入字段名',
      trigger: ['input', 'blur'],
    },
  };

  const nodeModalTitle = computed(() => {
    return editingNode.value ? '编辑节点' : '添加节点';
  });

  // 动态分类选项
  const categoryOptions = ref([]);

  // 判断当前编辑的是否为一级字段
  const isRootLevel = computed(() => {
    return editingNodePath.value.length === 0;
  });

  // 处理创建新分类
  const handleCreateCategory = (value) => {
    if (!value || value.trim() === '') return value;

    const trimmedValue = value.trim();

    // 检查是否已存在
    const exists = categoryOptions.value.some((option) => option.value === trimmedValue);

    if (!exists) {
      // 添加新分类到选项列表
      categoryOptions.value.push({
        label: trimmedValue,
        value: trimmedValue,
      });
    }

    return trimmedValue;
  };

  // 初始化分类选项（从现有数据中提取）
  const initCategoryOptions = () => {
    const categories = new Set();

    const extractCategories = (nodes) => {
      nodes.forEach((node) => {
        // 只从一级字段中提取分类
        if (node.category && node.path && node.path.length === 1) {
          categories.add(node.category);
        }
        if (node.children && Array.isArray(node.children)) {
          extractCategories(node.children);
        }
      });
    };

    if (treeData.value && treeData.value.length > 0) {
      extractCategories(treeData.value);
    }

    // 转换为选项格式
    categoryOptions.value = Array.from(categories).map((category) => ({
      label: category,
      value: category,
    }));
  };

  // 转换为 Ant Design Vue a-tree 格式的计算属性
  const treeDataForAnt = computed(() => {
    return convertToAntTreeFormat(treeData.value);
  });

  // 转换树数据为 Ant Design Vue a-tree 格式
  const convertToAntTreeFormat = (nodes) => {
    if (!Array.isArray(nodes)) return [];

    return nodes
      .map((node) => {
        if (!node || typeof node !== 'object') return null;

        try {
          const nodeKey =
            node.path?.join('.') || node.key || `node_${Math.random().toString(36).substr(2, 9)}`;

          const result = {
            title: node.displayName || node.label || node.key || 'Untitled',
            key: nodeKey,
            isLeaf: !node.children || node.children.length === 0,
            hasChildren: !!(node.children && node.children.length > 0),
            description: node.description || '',
            path: node.path || [],
            category: node.category || '',
            example: node.example || '',
            children: node.children ? convertToAntTreeFormat(node.children) : [],
          };

          return result;
        } catch (error) {
          console.error('Error converting node to tree format:', error, node);
          return null;
        }
      })
      .filter((node) => node !== null);
  };

  // 获取拖拽类名
  const getDragClass = (nodeKey) => {
    const classes = [];
    if (draggedNode.value && draggedNode.value.key === nodeKey) {
      classes.push('dragging');
    }
    if (dragOverNode.value && dragOverNode.value.key === nodeKey) {
      classes.push('drag-over');
    }
    return classes.join(' ');
  };

  // 初始化 CodeMirror 编辑器
  const initEditor = () => {
    if (!editorContainer.value) return;

    const state = EditorState.create({
      doc: props.modelValue || '{}',
      extensions: [
        basicSetup,
        json(),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const content = update.state.doc.toString();
            jsonContent.value = content;
            validateJsonContent(content);
            emit('update:modelValue', content);
          }
        }),
      ],
    });

    editorView = new EditorView({
      state,
      parent: editorContainer.value,
    });

    // 初始化时同步到树
    if (props.modelValue) {
      syncJsonToTree(props.modelValue);
    }
  };

  // JSON 处理
  const handleJsonChange = (value) => {
    jsonContent.value = value;
    validateJsonContent(value);
    emit('update:modelValue', value);
  };

  const validateJsonContent = (content) => {
    try {
      if (content.trim()) {
        JSON.parse(content);
      }
      jsonValid.value = true;
    } catch (e) {
      jsonValid.value = false;
    }
  };

  // 将 JSON 同步到树形结构
  const syncJsonToTree = (jsonStr) => {
    try {
      const data = JSON.parse(jsonStr || '{}');
      treeData.value = convertObjectToTree(data);
    } catch (e) {
      console.error('JSON 解析失败:', e);
      treeData.value = [];
    }
  };

  // 将对象转换为树形结构
  const convertObjectToTree = (obj, path = []) => {
    if (typeof obj !== 'object' || obj === null) {
      return [];
    }

    return Object.entries(obj).map(([key, value]) => {
      const currentPath = [...path, key];
      const nodeType = Array.isArray(value) ? 'array' : typeof value;

      const node = {
        key: currentPath.join('.'),
        label: key,
        nodeType,
        value,
        path: currentPath,
        description: '',
        required: false,
        children: [],
      };

      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        node.children = convertObjectToTree(value, currentPath);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object') {
        node.children = convertObjectToTree(value[0], [...currentPath, '0']);
      }

      return node;
    });
  };

  // 将树形结构转换为对象
  const convertTreeToObject = (nodes) => {
    const result = {};

    nodes.forEach((node) => {
      const { key, displayName, description, example, insertText, category, children } = node;
      const fieldKey = key || displayName || 'untitled';

      // 创建字段信息对象
      const fieldInfo = {
        displayName: displayName || '',
        description: description || '',
        example: example || '',
        insertText: insertText || `{{.${fieldKey}}}`,
      };

      // 只有一级字段才添加 category
      if (category && node.path && node.path.length === 1) {
        fieldInfo.category = category;
      }

      if (children && children.length > 0) {
        // 有子字段的节点
        fieldInfo.type = 'object';
        fieldInfo.children = convertTreeToObject(children);
      } else {
        // 普通字段
        fieldInfo.type = 'field';
      }

      result[fieldKey] = fieldInfo;
    });

    return result;
  };

  // 拖拽调整面板宽度
  const startResize = (type, event) => {
    resizing.value = true;
    resizeType.value = type;
    startX.value = event.clientX;

    if (type === 'tree') {
      startWidth.value = treePanelWidth.value;
    } else if (type === 'detail') {
      startWidth.value = detailPanelWidth.value;
    }

    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    event.preventDefault();
  };

  const handleResize = (event) => {
    if (!resizing.value) return;

    const deltaX = event.clientX - startX.value;

    if (resizeType.value === 'tree') {
      const newWidth = Math.max(200, Math.min(500, startWidth.value + deltaX));
      treePanelWidth.value = newWidth;
    } else if (resizeType.value === 'detail') {
      const newWidth = Math.max(250, Math.min(600, startWidth.value + deltaX));
      detailPanelWidth.value = newWidth;
    }
  };

  const stopResize = () => {
    resizing.value = false;
    resizeType.value = '';
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  };

  // 树形交互方法
  const onSelectNode = (keys, info) => {
    if (keys && keys.length > 0) {
      const nodeKey = keys[0];
      const node = findNodeByKey(treeData.value, nodeKey);
      if (node) {
        handleNodeSelect(node);
      }
    }
  };

  const updateExpanded = (keys) => {
    expandedKeys.value = new Set(keys);
  };

  // 右键菜单处理（Ant Design Vue 的 rightClick 事件）
  const onRightClick = ({ event, node }) => {
    event.preventDefault();
    event.stopPropagation();
    const treeDataNode = findNodeByKey(treeData.value, node.key);
    dropdownNode.value = node;
    setupContextMenu(node);
    showDropdown.value = true;
    dropdownX.value = event.clientX;
    dropdownY.value = event.clientY;
  };

  // 查找节点
  const findNodeByKey = (nodes, key) => {
    if (!nodes || !Array.isArray(nodes)) return null;

    for (const node of nodes) {
      if (node.key === key || node.path?.join('.') === key) {
        return node;
      }
      if (node.children) {
        const found = findNodeByKey(node.children, key);
        if (found) return found;
      }
    }
    return null;
  };

  // 设置右键菜单
  const setupContextMenu = (node) => {
    // 检查节点是否允许子字段
    const nodeData = findNodeByKey(treeData.value, node.key);
    const allowChildren = !nodeData || nodeData.children !== undefined;

    const menuOptions = [];

    // 只有允许子字段的节点才显示"添加子字段"选项
    if (allowChildren) {
      menuOptions.push({
        label: '添加子字段',
        key: 'addChild',
        icon: () => h(AddOutline, { style: { fontSize: '16px' } }),
      });
      menuOptions.push({ type: 'divider', key: 'divider1' });
    }

    menuOptions.push(
      { label: '复制', key: 'copy', icon: () => h(CopyOutline, { style: { fontSize: '16px' } }) },
      { type: 'divider', key: 'divider2' },
      {
        label: '删除',
        key: 'delete',
        icon: () => h(TrashOutline, { style: { fontSize: '16px' } }),
      }
    );

    dropdownOptions.value = menuOptions;
  };

  // 空白区域右键菜单
  const onTreeAreaContextMenu = (e) => {
    e.preventDefault();
    e.stopPropagation();
    dropdownNode.value = null;
    dropdownOptions.value = [
      {
        label: '添加根字段',
        key: 'addRoot',
        icon: () => h(AddOutline, { style: { fontSize: '16px' } }),
      },
    ];
    showDropdown.value = true;
    dropdownX.value = e.clientX;
    dropdownY.value = e.clientY;
  };

  // 右键菜单选择处理
  const handleDropdownSelect = (key) => {
    showDropdown.value = false;

    switch (key) {
      case 'addChild':
        if (dropdownNode.value) {
          const nodeData = findNodeByKey(treeData.value, dropdownNode.value.key);
          if (nodeData && nodeData.children !== undefined) {
            openNodeModal(null, dropdownNode.value.path);
          } else {
            message.warning('该节点不允许包含子字段');
          }
        }
        break;
      case 'addRoot':
        openNodeModal(null, []);
        break;
      case 'copy':
        if (dropdownNode.value) {
          copyNode(dropdownNode.value);
        }
        break;
      case 'delete':
        if (dropdownNode.value) {
          deleteNode(dropdownNode.value);
        }
        break;
    }
  };

  const handleDropdownClickoutside = () => {
    showDropdown.value = false;
  };

  // 节点选择
  const handleNodeSelect = (node) => {
    selectedNode.value = node;
    nodeForm.key = node.key;
    nodeForm.displayName = node.displayName || '';
    nodeForm.description = node.description || '';
    nodeForm.example = node.example || '';
    nodeForm.insertText = node.insertText || `{{.${node.key}}}`;
    nodeForm.category = node.category || '';
    nodeForm.hasChildren = node.children !== undefined;
  };

  // 保存当前编辑的节点
  const saveCurrentNode = async () => {
    if (!selectedNode.value) return;

    try {
      await detailFormRef.value?.validate();
      nodeSaving.value = true;

      // 更新选中节点的信息
      const updatedNode = {
        ...selectedNode.value,
        key: nodeForm.key,
        label: nodeForm.displayName || nodeForm.key,
        displayName: nodeForm.displayName,
        description: nodeForm.description,
        example: nodeForm.example,
        insertText: nodeForm.insertText,
        category: nodeForm.category,
        children: nodeForm.hasChildren ? selectedNode.value.children || [] : undefined,
      };

      // 更新节点路径中的key
      const newPath = [...selectedNode.value.path];
      newPath[newPath.length - 1] = nodeForm.key;
      updatedNode.path = newPath;

      // 更新树中的节点
      updateNodeInTree(updatedNode, selectedNode.value.path);
      selectedNode.value = updatedNode;

      syncTreeToJson();
      message.success('节点保存成功');
    } catch (error) {
      console.error('保存节点失败:', error);
    } finally {
      nodeSaving.value = false;
    }
  };

  // 节点操作

  // 打开节点编辑弹窗
  const openNodeModal = (node = null, parentPath = []) => {
    resetNodeForm();
    isModalCanceling.value = false;

    if (node) {
      // 编辑模式 - 保存原始数据
      const nodeData = findNodeByKey(treeData.value, node.key);
      if (nodeData) {
        originalNodeData.value = JSON.parse(JSON.stringify(nodeData));
        editingNode.value = nodeData;
        editingNodePath.value = nodeData.path;
        nodeForm.key = nodeData.key;
        nodeForm.displayName = nodeData.displayName || '';
        nodeForm.description = nodeData.description || '';
        nodeForm.example = nodeData.example || '';
        nodeForm.insertText = nodeData.insertText || `{{.${nodeData.key}}}`;
        nodeForm.category = nodeData.category || '';
        nodeForm.hasChildren = nodeData.children !== undefined;
      }
    } else {
      // 新增模式
      originalNodeData.value = null;
      editingNode.value = null;
      editingNodePath.value = parentPath;
      nodeForm.hasChildren = false;
    }

    showNodeModal.value = true;
  };

  const copyNode = (node) => {
    const copiedNode = JSON.parse(JSON.stringify(node));
    const timestamp = Date.now();
    const newKey = `${node.key}_copy_${timestamp}`;
    const newDisplayName = `${node.displayName || node.label}_副本`;

    // 更新复制节点的key和路径
    const updateCopiedNodeKeys = (n, parentPath = []) => {
      // 只更新根节点的 key 和相关字段
      if (n === copiedNode) {
        n.key = newKey;
        n.label = newDisplayName;
        n.displayName = newDisplayName;
      }
      n.path = [...parentPath, n.key];
      if (n.children) {
        n.children.forEach((child) => updateCopiedNodeKeys(child, n.path));
      }
    };

    updateCopiedNodeKeys(copiedNode, node.path.slice(0, -1));

    // 找到父节点并添加复制的节点
    if (node.path.length === 1) {
      // 根节点，直接添加到树中
      treeData.value.push(copiedNode);
    } else {
      // 子节点，添加到父节点的children中
      const parentPath = node.path.slice(0, -1);
      const parent = findNodeByPath(treeData.value, parentPath);
      if (parent && parent.children) {
        parent.children.push(copiedNode);
      }
    }

    syncTreeToJson();
    message.success('节点复制成功');
  };

  const findNodeByPath = (nodes, path) => {
    if (!path || path.length === 0) return null;

    for (const node of nodes) {
      if (node.path && node.path.join('.') === path.join('.')) {
        return node;
      }
      if (node.children) {
        const found = findNodeByPath(node.children, path);
        if (found) return found;
      }
    }
    return null;
  };

  const addChildNode = (parentNode) => {
    openNodeModal(null, parentNode.path);
  };

  const editNode = (node) => {
    openNodeModal(node);
  };

  const deleteNode = (node) => {
    // 从树中删除节点
    const deleteFromArray = (nodes, targetPath) => {
      for (let i = 0; i < nodes.length; i++) {
        if (nodes[i].path.join('.') === targetPath.join('.')) {
          nodes.splice(i, 1);
          return true;
        }
        if (nodes[i].children) {
          if (deleteFromArray(nodes[i].children, targetPath)) {
            return true;
          }
        }
      }
      return false;
    };

    deleteFromArray(treeData.value, node.path);
    syncTreeToJson();
    message.success('节点删除成功');
  };

  const resetNodeForm = () => {
    nodeForm.key = '';
    nodeForm.displayName = '';
    nodeForm.description = '';
    nodeForm.example = '';
    nodeForm.insertText = '';
    nodeForm.category = '';
    nodeForm.hasChildren = false;
  };

  const saveNode = async () => {
    try {
      await nodeFormRef.value?.validate();

      // 如果是编辑模式，检查是否要移除子字段支持
      if (editingNode.value && !nodeForm.hasChildren) {
        const currentNode = findNodeByKey(treeData.value, editingNode.value.key);
        if (currentNode && currentNode.children && currentNode.children.length > 0) {
          message.warning('该节点已包含子字段，无法设置为不包含子字段。请先删除所有子字段。');
          return;
        }
      }

      nodeSaving.value = true;

      const nodePath = [...editingNodePath.value, nodeForm.key];
      const newNode = {
        key: nodeForm.key,
        label: nodeForm.displayName || nodeForm.key,
        path: nodePath,
        displayName: nodeForm.displayName,
        description: nodeForm.description,
        example: nodeForm.example,
        insertText: nodeForm.insertText,
        category: isRootLevel.value ? nodeForm.category : undefined, // 只有一级字段才有分类
        children: nodeForm.hasChildren ? [] : undefined,
      };

      if (editingNode.value) {
        // 编辑现有节点，保留现有的子字段
        const currentNode = findNodeByKey(treeData.value, editingNode.value.key);
        if (currentNode && nodeForm.hasChildren && currentNode.children) {
          newNode.children = currentNode.children;
        }
        updateNodeInTree(newNode);
      } else {
        // 添加新节点
        addNodeToTree(newNode);
      }

      // 同步到JSON
      syncTreeToJson();

      // 保存成功消息
      const isEdit = !!editingNode.value;

      // 清空状态
      originalNodeData.value = null;
      showNodeModal.value = false;
      editingNode.value = null;
      resetNodeForm();

      message.success(isEdit ? '节点更新成功' : '节点添加成功');
    } catch (error) {
      // 处理表单验证错误
      console.log('Form validation result:', error); // 调试日志

      if (error && typeof error === 'object' && !error.message) {
        // 处理NaiveUI表单验证结果
        if (error.warnings === undefined) {
          // 验证成功的情况
          console.log('Form validation passed, but caught as error - continuing execution');
          // 这种情况下实际上验证成功了，我们应该继续执行而不是退出
          try {
            const nodePath = [...editingNodePath.value, nodeForm.key];
            const newNode = {
              key: nodeForm.key,
              label: nodeForm.displayName || nodeForm.key,
              path: nodePath,
              displayName: nodeForm.displayName,
              description: nodeForm.description,
              example: nodeForm.example,
              insertText: nodeForm.insertText,
              category: isRootLevel.value ? nodeForm.category : undefined, // 只有一级字段才有分类
              children: nodeForm.hasChildren ? [] : undefined,
            };

            if (editingNode.value) {
              // 编辑现有节点，保留现有的子字段
              const currentNode = findNodeByKey(treeData.value, editingNode.value.key);
              if (currentNode && nodeForm.hasChildren && currentNode.children) {
                newNode.children = currentNode.children;
              }
              updateNodeInTree(newNode);
            } else {
              // 添加新节点
              addNodeToTree(newNode);
            }

            // 同步到JSON
            syncTreeToJson();

            // 保存成功消息
            const isEdit = !!editingNode.value;

            // 清空状态
            originalNodeData.value = null;
            showNodeModal.value = false;
            editingNode.value = null;
            resetNodeForm();

            message.success(isEdit ? '节点更新成功' : '节点添加成功');
            return;
          } catch (innerError) {
            console.error('执行保存操作时出错:', innerError);
            message.error('保存节点失败');
            return;
          }
        } else if (error.warnings && error.warnings.length > 0) {
          // 有验证警告
          const firstWarning = error.warnings[0];
          if (firstWarning && firstWarning.message) {
            message.warning(firstWarning.message);
            return;
          }
        }
      }

      // 处理其他类型的错误
      if (
        Array.isArray(error) &&
        error.length > 0 &&
        Array.isArray(error[0]) &&
        error[0].length > 0
      ) {
        const firstError = error[0][0];
        if (firstError && firstError.message) {
          message.error(firstError.message);
          return;
        }
      }

      // 处理标准错误对象
      if (error && error.message) {
        message.error(error.message);
        return;
      }

      console.error('保存节点失败:', error);
      message.error('保存节点失败，请检查输入内容');
    } finally {
      nodeSaving.value = false;
    }
  };

  const addNodeToTree = (newNode) => {
    if (editingNodePath.value.length === 0) {
      // 添加根节点
      treeData.value.push(newNode);
    } else {
      // 添加子节点
      const findParent = (nodes, path) => {
        for (const node of nodes) {
          if (node.path.join('.') === path.join('.')) {
            return node;
          }
          if (node.children) {
            const found = findParent(node.children, path);
            if (found) return found;
          }
        }
        return null;
      };

      const parent = findParent(treeData.value, editingNodePath.value);
      if (parent) {
        if (!parent.children) parent.children = [];
        parent.children.push(newNode);
      }
    }
  };

  const updateNodeInTree = (updatedNode, originalPath = null) => {
    const targetPath = originalPath || editingNode.value?.path;
    if (!targetPath) return;

    const updateInArray = (nodes) => {
      for (let i = 0; i < nodes.length; i++) {
        if (nodes[i].path.join('.') === targetPath.join('.')) {
          // 保留子节点
          updatedNode.children = nodes[i].children;
          nodes[i] = updatedNode;
          return true;
        }
        if (nodes[i].children) {
          if (updateInArray(nodes[i].children)) {
            return true;
          }
        }
      }
      return false;
    };

    updateInArray(treeData.value);
  };

  const closeNodeModal = () => {
    // 如果是编辑模式且有原始数据，需要恢复
    if (editingNode.value && originalNodeData.value) {
      isModalCanceling.value = true;

      // 恢复原始节点数据
      const restoreNode = (nodes) => {
        for (let i = 0; i < nodes.length; i++) {
          if (nodes[i].key === editingNode.value.key) {
            nodes[i] = JSON.parse(JSON.stringify(originalNodeData.value));
            return true;
          }
          if (nodes[i].children) {
            if (restoreNode(nodes[i].children)) {
              return true;
            }
          }
        }
        return false;
      };

      restoreNode(treeData.value);
      syncTreeToJson();
    }

    showNodeModal.value = false;
    editingNode.value = null;
    originalNodeData.value = null;
    isModalCanceling.value = false;
    resetNodeForm();
  };

  // 同步树到 JSON
  const syncTreeToJson = () => {
    const obj = convertTreeToObject(treeData.value);
    const jsonStr = JSON.stringify(obj, null, 2);

    if (editorView) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: jsonStr,
        },
      });
    }

    jsonContent.value = jsonStr;
    emit('update:modelValue', jsonStr);
  };

  // 其他操作
  const formatJson = () => {
    if (!editorView) return;

    try {
      const content = editorView.state.doc.toString();
      const parsed = JSON.parse(content);
      const formatted = JSON.stringify(parsed, null, 2);

      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: formatted,
        },
      });

      message.success('JSON 格式化成功');
    } catch (e) {
      message.error('JSON 格式错误，无法格式化');
    }
  };

  const syncFromJson = () => {
    if (!editorView) return;

    try {
      const content = editorView.state.doc.toString();
      syncJsonToTree(content);
      message.success('已从 JSON 同步到树形结构');
    } catch (e) {
      message.error('JSON 格式错误，无法同步');
    }
  };

  // 监听 modelValue 变化
  watch(
    () => props.modelValue,
    (newValue) => {
      if (editorView && newValue !== editorView.state.doc.toString()) {
        editorView.dispatch({
          changes: {
            from: 0,
            to: editorView.state.doc.length,
            insert: newValue || '{}',
          },
        });
        syncJsonToTree(newValue);
      }
    }
  );

  // 监听树数据变化，自动更新分类选项
  watch(
    treeData,
    () => {
      initCategoryOptions();
    },
    { deep: true }
  );

  // 监听字段名变化，自动更新插入文本
  watch(
    () => nodeForm.key,
    (newKey) => {
      if (newKey && newKey.trim()) {
        nodeForm.insertText = `{{.${newKey.trim()}}}`;
      } else {
        nodeForm.insertText = '';
      }
    }
  );

  // 监听nodeForm变化，实时更新JSON
  watch(
    () => [
      nodeForm.key,
      nodeForm.displayName,
      nodeForm.description,
      nodeForm.example,
      nodeForm.insertText,
      nodeForm.category,
      nodeForm.hasChildren,
    ],
    () => {
      // 在取消编辑时不触发更新
      if (isModalCanceling.value) return;

      if (selectedNode.value && showNodeModal.value && editingNode.value) {
        // 实时更新编辑中的节点信息（仅在弹窗打开时）
        const updatedNode = {
          ...editingNode.value,
          key: nodeForm.key,
          label: nodeForm.displayName || nodeForm.key,
          displayName: nodeForm.displayName,
          description: nodeForm.description,
          example: nodeForm.example,
          insertText: nodeForm.insertText,
          category: isRootLevel.value ? nodeForm.category : editingNode.value.category,
          children: nodeForm.hasChildren ? editingNode.value.children || [] : undefined,
        };

        // 更新节点路径中的key
        const newPath = [...editingNode.value.path];
        if (newPath.length > 0) {
          newPath[newPath.length - 1] = nodeForm.key;
          updatedNode.path = newPath;
          updatedNode.key = newPath.join('.');
        }

        // 更新树中的节点
        updateNodeInTree(updatedNode, editingNode.value.path);
        editingNode.value = updatedNode;

        syncTreeToJson();
      }
    },
    { deep: true }
  );

  // 拖拽功能
  const onDragStart = (info) => {
    if (info.dragNode) {
      draggedNode.value = info.dragNode;
      isDragging.value = true;
    }
  };

  const onDragEnter = (info) => {
    if (info.node) {
      const nodeData = findNodeByKey(treeData.value, info.node.key);
      // 只有允许子字段的节点才能接受拖拽
      if (nodeData && nodeData.children !== undefined) {
        dragOverNode.value = info.node;
      }
    }
  };

  const onDragLeave = () => {
    dragOverNode.value = null;
  };

  const onDragOver = (info) => {
    const { event, node } = info;

    if (node) {
      const nodeData = findNodeByKey(treeData.value, node.key);
      // 检查目标节点是否允许子字段
      if (nodeData && nodeData.children !== undefined) {
        event.preventDefault();
        event.dataTransfer.dropEffect = 'move';
        dragOverNode.value = node;
      } else {
        event.dataTransfer.dropEffect = 'none';
      }
    }
  };

  const onDrop = (info) => {
    const { dragNode, node } = info;

    if (!dragNode || !node || dragNode.key === node.key) {
      clearDragState();
      return;
    }

    // 检查目标节点是否允许子字段
    const targetNodeData = findNodeByKey(treeData.value, node.key);
    if (!targetNodeData || targetNodeData.children === undefined) {
      message.warning('该节点不允许包含子字段');
      clearDragState();
      return;
    }

    // 不能拖拽到自己的子节点
    if (isDescendant(dragNode, node)) {
      clearDragState();
      return;
    }

    // 执行移动操作
    moveNode(dragNode, node);
    clearDragState();
  };

  const clearDragState = () => {
    draggedNode.value = null;
    dragOverNode.value = null;
    isDragging.value = false;
  };

  const isDescendant = (parentNode, childNode) => {
    if (!parentNode.children) return false;

    for (const child of parentNode.children) {
      if (child.key === childNode.key) return true;
      if (isDescendant(child, childNode)) return true;
    }
    return false;
  };

  const moveNode = (sourceNode, targetNode) => {
    // 从原位置移除
    const removeNode = (nodes, nodeKey) => {
      for (let i = 0; i < nodes.length; i++) {
        if (nodes[i].key === nodeKey) {
          return nodes.splice(i, 1)[0];
        }
        if (nodes[i].children) {
          const removed = removeNode(nodes[i].children, nodeKey);
          if (removed) return removed;
        }
      }
      return null;
    };

    const movedNode = removeNode(treeData.value, sourceNode.key);
    if (!movedNode) return;

    // 更新路径
    const updateNodePath = (node, newParentPath) => {
      node.path = [...newParentPath, node.label];
      node.key = node.path.join('.');
      if (node.children) {
        node.children.forEach((child) => updateNodePath(child, node.path));
      }
    };

    updateNodePath(movedNode, targetNode.path);

    // 添加到新位置
    if (!targetNode.children) {
      targetNode.children = [];
    }
    targetNode.children.push(movedNode);

    syncTreeToJson();
    message.success('节点移动成功');
  };

  // JSON文件导入导出功能

  const importJson = () => {
    // 创建隐藏的文件输入元素
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.style.display = 'none';

    input.onchange = (event) => {
      const file = event.target.files[0];
      if (file) {
        handleJsonFileImport(file);
      }
      document.body.removeChild(input);
    };

    document.body.appendChild(input);
    input.click();
  };

  const exportJson = () => {
    if (!editorView) return;

    try {
      const content = editorView.state.doc.toString();

      // 验证JSON格式
      try {
        JSON.parse(content);
      } catch (error) {
        message.error('当前JSON格式无效，无法导出');
        return;
      }

      // 创建并下载文件
      const blob = new Blob([content], { type: 'application/json' });
      const url = URL.createObjectURL(blob);

      const link = document.createElement('a');
      link.href = url;
      link.download = 'var-preset-schema.json';
      link.style.display = 'none';

      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      // 清理URL对象
      URL.revokeObjectURL(url);

      message.success('JSON文件导出成功');
    } catch (error) {
      console.error('导出失败:', error);
      message.error('导出失败');
    }
  };

  const handleJsonFileImport = (file) => {
    if (!file.name.toLowerCase().endsWith('.json')) {
      message.error('请选择.json格式的文件');
      return;
    }

    const reader = new FileReader();

    reader.onload = (event) => {
      try {
        const content = event.target.result;

        // 解析JSON内容
        let jsonObject;
        try {
          jsonObject = JSON.parse(content);
        } catch (error) {
          message.error('文件内容不是有效的JSON格式: ' + error.message);
          return;
        }

        // 转换为树结构
        const newTreeData = convertJsonToTree(jsonObject);

        // 更新树数据
        treeData.value = newTreeData;

        // 重新初始化分类选项
        initCategoryOptions();

        // 同步到编辑器
        syncTreeToJson();

        message.success(`JSON文件 "${file.name}" 导入成功`);
      } catch (error) {
        console.error('导入失败:', error);
        message.error('导入文件失败: ' + error.message);
      }
    };

    reader.onerror = () => {
      message.error('读取文件失败');
    };

    reader.readAsText(file, 'UTF-8');
  };

  const copyJson = () => {
    if (!editorView) return;

    try {
      const content = editorView.state.doc.toString();

      // 验证JSON格式
      try {
        JSON.parse(content);
      } catch (error) {
        message.error('当前JSON格式无效，无法复制');
        return;
      }

      // 复制到剪贴板
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard
          .writeText(content)
          .then(() => {
            message.success('JSON内容已复制到剪贴板');
          })
          .catch(() => {
            // 备用方法
            fallbackCopyToClipboard(content);
          });
      } else {
        // 备用方法
        fallbackCopyToClipboard(content);
      }
    } catch (error) {
      console.error('复制失败:', error);
      message.error('复制失败');
    }
  };

  const fallbackCopyToClipboard = (text) => {
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.left = '-999999px';
    textArea.style.top = '-999999px';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
      const successful = document.execCommand('copy');
      if (successful) {
        message.success('JSON内容已复制到剪贴板');
      } else {
        message.error('复制失败，请手动复制');
      }
    } catch (err) {
      message.error('复制失败，请手动复制');
      console.error('复制失败:', err);
    }

    document.body.removeChild(textArea);
  };

  const convertJsonToTree = (obj, parentPath = []) => {
    if (!obj || typeof obj !== 'object') {
      return [];
    }

    const result = [];

    for (const [key, value] of Object.entries(obj)) {
      const currentPath = [...parentPath, key];

      // 检查是否为新格式的字段信息对象
      if (
        value &&
        typeof value === 'object' &&
        !Array.isArray(value) &&
        (value.hasOwnProperty('displayName') ||
          value.hasOwnProperty('type') ||
          value.hasOwnProperty('description') ||
          value.hasOwnProperty('example'))
      ) {
        // 新格式：包含字段信息的对象
        const node = {
          key: key,
          label: value.displayName || key,
          displayName: value.displayName || '',
          path: currentPath,
          description: value.description || '',
          example: value.example || '',
          insertText: value.insertText || '',
          category: value.category || '',
        };

        if (value.type === 'object' && value.children) {
          // 包含子字段的对象
          node.children = convertJsonToTree(value.children, currentPath);
        } else {
          // 普通字段
          node.children = undefined;
        }

        result.push(node);
      } else if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
        // 旧格式：普通嵌套对象，递归处理子节点
        const node = {
          key: key,
          label: key,
          displayName: '',
          path: currentPath,
          description: '',
          example: '',
          insertText: '',
          category: '',
          children: convertJsonToTree(value, currentPath),
        };
        result.push(node);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object') {
        // 对象数组，转换为可扩展的容器节点
        const node = {
          key: key,
          label: key,
          displayName: '',
          path: currentPath,
          description: '',
          example: '',
          insertText: '',
          category: '',
          children: [],
        };
        const sampleObject = value[0];
        if (sampleObject && typeof sampleObject === 'object') {
          node.children = convertJsonToTree(sampleObject, currentPath);
        }
        result.push(node);
      } else {
        // 原始值，设为不包含子字段的节点
        const node = {
          key: key,
          label: key,
          displayName: '',
          path: currentPath,
          description: '',
          example: typeof value === 'string' ? value : '',
          insertText: '',
          category: '',
          children: undefined,
        };
        result.push(node);
      }
    }

    return result;
  };

  // 生命周期
  onMounted(async () => {
    await nextTick();
    initEditor();

    // 初始化树数据
    if (props.modelValue) {
      try {
        const jsonObject = JSON.parse(props.modelValue);
        treeData.value = convertJsonToTree(jsonObject);
      } catch (error) {
        console.error('初始化树数据失败:', error);
        treeData.value = [];
      }
    } else {
      treeData.value = [];
    }

    // 初始化分类选项
    initCategoryOptions();
  });

  onUnmounted(() => {
    if (editorView) {
      editorView.destroy();
    }
    // 清理拖拽事件监听器
    stopResize();
  });
</script>

<style scoped>
  .simple-var-preset-editor {
    width: 100%;
    height: 100%;
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    overflow: hidden;
  }

  .editor-layout {
    display: flex;
    height: 100%;
  }

  .tree-panel {
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .detail-panel {
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .editor-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .resizer {
    width: 4px;
    background: #f0f0f0;
    cursor: col-resize;
    flex-shrink: 0;
    position: relative;
    transition: background-color 0.2s ease;
  }

  .resizer:hover {
    background: #d0d0d0;
  }

  .resizer:active {
    background: #1890ff;
  }

  .resizer::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 2px;
    height: 20px;
    background: #bbb;
    border-radius: 1px;
  }

  .resizer:hover::before {
    background: #999;
  }

  .resizer:active::before {
    background: #fff;
  }

  .tree-header,
  .detail-header,
  .editor-header {
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fafafa;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tree-header h4,
  .detail-header h4,
  .editor-header h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #333;
  }

  .detail-actions,
  .editor-actions {
    display: flex;
    gap: 8px;
  }

  .tree-content {
    flex: 1;
    padding: 8px;
    overflow-y: auto;
  }

  .detail-content {
    flex: 1;
    overflow-y: auto;
  }

  .empty-detail {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
  }

  .node-form {
    padding: 16px;
  }

  .empty-tree {
    text-align: center;
    padding: 40px 20px;
    color: #999;
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
  }

  .codemirror-container {
    height: 100%;
    width: 100%;
  }

  .editor-status {
    padding: 8px 16px;
    border-top: 1px solid #e0e0e0;
    background: #fafafa;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }

  .status-indicator {
    font-weight: 500;
  }

  .status-indicator.valid {
    color: #52c41a;
  }

  .status-indicator.invalid {
    color: #ff4d4f;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  /* CodeMirror 样式 */
  :deep(.cm-editor) {
    height: 100%;
    font-size: 13px;
    border: none;
  }

  :deep(.cm-focused) {
    outline: none;
  }

  :deep(.cm-content) {
    padding: 8px;
  }

  :deep(.cm-scroller) {
    font-family: 'Courier New', 'Consolas', 'Monaco', monospace;
  }

  /* 树形结构样式 */
  /* 右键菜单 */
  .context-menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9999;
  }

  .context-menu {
    position: fixed;
    z-index: 10000;
    min-width: 160px;
    background: #fff;
    border-radius: 8px;
    box-shadow: 0 6px 16px 0 rgba(0, 0, 0, 0.08), 0 3px 6px -4px rgba(0, 0, 0, 0.12), 0 9px 28px 8px rgba(0, 0, 0, 0.05);
    padding: 4px 0;
  }

  .context-menu :deep(.ant-menu) {
    border-right: none;
    box-shadow: none;
  }

  /* 拖拽样式 */
  :deep(.ant-tree-treenode.dragging) {
    opacity: 0.5;
    background: rgba(24, 144, 255, 0.1);
  }

  :deep(.ant-tree-treenode.drag-over) {
    background: rgba(24, 144, 255, 0.2);
    border: 2px dashed #1890ff;
    border-radius: 4px;
  }

  :deep(.ant-tree-treenode[draggable='true']) {
    cursor: grab;
  }

  :deep(.ant-tree-treenode[draggable='true']:active) {
    cursor: grabbing;
  }

  /* 树节点样式优化 */
  :deep(.ant-tree-node-content-wrapper) {
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  :deep(.ant-tree-node-content-wrapper:hover) {
    background: #f5f5f5;
  }

  :deep(.ant-tree-treenode-selected .ant-tree-node-content-wrapper) {
    background: #e6f7ff;
  }

  :deep(.ant-tree-treenode-selected .ant-tree-node-content-wrapper:hover) {
    background: #e6f7ff;
  }
</style>
