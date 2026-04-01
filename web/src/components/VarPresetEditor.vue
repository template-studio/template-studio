<template>
  <div class="var-preset-editor">
    <div class="editor-layout">
      <!-- 左侧树形结构 -->
      <div class="tree-panel">
        <div class="tree-header">
          <h4>数据结构</h4>
          <div class="tree-actions">
            <n-button size="small" type="primary" @click="addRootNode">
              <template #icon>
                <n-icon><AddOutline /></n-icon>
              </template>
              添加根节点
            </n-button>
            <n-button size="small" @click="expandAll" quaternary>展开全部</n-button>
            <n-button size="small" @click="collapseAll" quaternary>收起全部</n-button>
          </div>
        </div>
        <div class="tree-content">
          <n-tree
            :data="treeData"
            :render-prefix="renderPrefix"
            :render-suffix="renderSuffix"
            :render-label="renderLabel"
            block-line
            show-irrelevant-nodes
            expand-on-click
            selectable
            @update:selected-keys="handleNodeSelect"
            ref="treeRef"
          />
        </div>
      </div>

      <!-- 右侧 CodeMirror 编辑器 -->
      <div class="editor-panel">
        <div class="editor-header">
          <h4>JSON 预览</h4>
          <div class="editor-actions">
            <n-button size="small" @click="formatJson" quaternary>
              <template #icon>
                <n-icon><CodeOutline /></n-icon>
              </template>
              格式化
            </n-button>
            <n-button size="small" @click="validateJson" quaternary>
              <template #icon>
                <n-icon><CheckmarkOutline /></n-icon>
              </template>
              验证
            </n-button>
            <n-button size="small" @click="syncFromJson" quaternary>
              <template #icon>
                <n-icon><SyncOutline /></n-icon>
              </template>
              同步到树
            </n-button>
          </div>
        </div>
        <div class="editor-content" ref="editorContainer"></div>
        <div class="editor-status">
          <span :class="['status-indicator', jsonValid ? 'valid' : 'invalid']">
            {{ jsonValid ? '✓ JSON 有效' : '✗ JSON 无效' }}
          </span>
          <span class="json-info"
            >行: {{ editorInfo.line }} | 列: {{ editorInfo.col }} | 字符:
            {{ editorInfo.length }}</span
          >
        </div>
      </div>
    </div>

    <!-- 节点编辑弹窗 -->
    <n-modal v-model:show="showNodeModal" :mask-closable="false">
      <n-card style="width: 500px" :title="nodeModalTitle" :bordered="false" size="huge">
        <template #header-extra>
          <n-button quaternary circle @click="closeNodeModal">
            <template #icon>
              <n-icon><CloseOutline /></n-icon>
            </template>
          </n-button>
        </template>

        <n-form ref="nodeFormRef" :model="nodeForm" :rules="nodeFormRules" label-placement="top">
          <n-form-item label="字段名" path="key">
            <n-input v-model:value="nodeForm.key" placeholder="请输入字段名" />
          </n-form-item>

          <n-form-item label="数据类型" path="type">
            <n-select
              v-model:value="nodeForm.type"
              placeholder="选择数据类型"
              :options="typeOptions"
              @update:value="handleTypeChange"
            />
          </n-form-item>

          <n-form-item label="描述" path="description">
            <n-input
              v-model:value="nodeForm.description"
              type="textarea"
              placeholder="字段描述"
              :rows="2"
            />
          </n-form-item>

          <n-form-item v-if="nodeForm.type === 'string'" label="默认值" path="defaultValue">
            <n-input v-model:value="nodeForm.defaultValue" placeholder="默认字符串值" />
          </n-form-item>

          <n-form-item v-if="nodeForm.type === 'number'" label="默认值" path="defaultValue">
            <n-input-number
              v-model:value="nodeForm.defaultValue"
              placeholder="默认数值"
              style="width: 100%"
            />
          </n-form-item>

          <n-form-item v-if="nodeForm.type === 'boolean'" label="默认值" path="defaultValue">
            <n-switch v-model:value="nodeForm.defaultValue" />
          </n-form-item>

          <n-form-item v-if="nodeForm.type === 'array'" label="数组元素类型" path="itemType">
            <n-select
              v-model:value="nodeForm.itemType"
              placeholder="选择数组元素类型"
              :options="typeOptions.filter((t) => t.value !== 'array')"
            />
          </n-form-item>

          <n-form-item label="是否必填" path="required">
            <n-switch v-model:value="nodeForm.required" />
          </n-form-item>
        </n-form>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="closeNodeModal">取消</n-button>
            <n-button type="primary" @click="saveNode" :loading="nodeSaving">
              {{ editingNode ? '更新' : '添加' }}
            </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
  import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch, h } from 'vue';
  import {
    NButton,
    NIcon,
    NTree,
    NModal,
    NCard,
    NForm,
    NFormItem,
    NInput,
    NInputNumber,
    NSelect,
    NSwitch,
    useMessage,
  } from 'naive-ui';
  import {
    AddOutline,
    CloseOutline,
    CreateOutline,
    TrashOutline,
    CodeOutline,
    CheckmarkOutline,
    SyncOutline,
    ChevronDownOutline,
    ChevronForwardOutline,
  } from '@vicons/ionicons5';
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

  const message = useMessage();

  // 编辑器相关
  const editorContainer = ref(null);
  const treeRef = ref(null);
  let editorView = null;

  // 数据状态
  const treeData = ref([]);
  const jsonValid = ref(true);
  const editorInfo = reactive({
    line: 1,
    col: 1,
    length: 0,
  });

  // 节点编辑相关
  const showNodeModal = ref(false);
  const nodeFormRef = ref(null);
  const editingNode = ref(null);
  const editingNodePath = ref([]);
  const nodeSaving = ref(false);

  const nodeForm = reactive({
    key: '',
    type: 'string',
    description: '',
    defaultValue: '',
    itemType: 'string',
    required: false,
  });

  const typeOptions = [
    { label: '字符串', value: 'string' },
    { label: '数字', value: 'number' },
    { label: '布尔值', value: 'boolean' },
    { label: '对象', value: 'object' },
    { label: '数组', value: 'array' },
  ];

  const nodeFormRules = {
    key: {
      required: true,
      message: '请输入字段名',
      trigger: ['input', 'blur'],
    },
    type: {
      required: true,
      message: '请选择数据类型',
      trigger: ['change'],
    },
  };

  const nodeModalTitle = computed(() => {
    return editingNode.value ? '编辑节点' : '添加节点';
  });

  // 初始化编辑器
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
            emit('update:modelValue', content);
            validateJsonContent(content);
            updateEditorInfo(update.state);
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

  // 更新编辑器信息
  const updateEditorInfo = (state) => {
    const selection = state.selection.main;
    const line = state.doc.lineAt(selection.head);
    editorInfo.line = line.number;
    editorInfo.col = selection.head - line.from + 1;
    editorInfo.length = state.doc.length;
  };

  // 验证 JSON
  const validateJsonContent = (content) => {
    try {
      JSON.parse(content);
      jsonValid.value = true;
    } catch (e) {
      jsonValid.value = false;
    }
  };

  // 将 JSON 同步到树形结构
  const syncJsonToTree = (jsonStr) => {
    try {
      const data = JSON.parse(jsonStr || '{}');
      treeData.value = convertObjectToTree(data, 'root');
    } catch (e) {
      console.error('JSON 解析失败:', e);
      treeData.value = [];
    }
  };

  // 将对象转换为树形结构
  const convertObjectToTree = (obj, parentKey = '', path = []) => {
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
      };

      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        node.children = convertObjectToTree(value, key, currentPath);
      } else if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object') {
        node.children = convertObjectToTree(value[0], key, [...currentPath, '0']);
      }

      return node;
    });
  };

  // 将树形结构转换为对象
  const convertTreeToObject = (nodes) => {
    const result = {};

    nodes.forEach((node) => {
      const { label, nodeType, value, children } = node;

      if (nodeType === 'object' && children) {
        result[label] = convertTreeToObject(children);
      } else if (nodeType === 'array') {
        if (children && children.length > 0) {
          result[label] = [convertTreeToObject(children)];
        } else {
          result[label] = [];
        }
      } else {
        // 根据类型设置默认值
        switch (nodeType) {
          case 'string':
            result[label] = value || '';
            break;
          case 'number':
            result[label] = typeof value === 'number' ? value : 0;
            break;
          case 'boolean':
            result[label] = typeof value === 'boolean' ? value : false;
            break;
          default:
            result[label] = value;
        }
      }
    });

    return result;
  };

  // 树节点渲染
  const renderPrefix = ({ node }) => {
    if (node.children && node.children.length > 0) {
      return h(
        NIcon,
        { size: 16 },
        {
          default: () => h(node.expanded ? ChevronDownOutline : ChevronForwardOutline),
        }
      );
    }
    return null;
  };

  const renderLabel = ({ node }) => {
    const typeColor = {
      string: '#52c41a',
      number: '#1890ff',
      boolean: '#722ed1',
      object: '#fa8c16',
      array: '#eb2f96',
    };

    return h('div', { class: 'tree-node-label' }, [
      h('span', { class: 'node-name' }, node.label),
      h(
        'span',
        {
          class: 'node-type',
          style: { color: typeColor[node.nodeType] || '#666' },
        },
        `[${node.nodeType}]`
      ),
      node.description && h('span', { class: 'node-desc' }, ` - ${node.description}`),
    ]);
  };

  const renderSuffix = ({ node }) => {
    if (props.readonly) return null;

    return h('div', { class: 'tree-node-actions' }, [
      h(
        NButton,
        {
          size: 'tiny',
          type: 'primary',
          secondary: true,
          onClick: (e) => {
            e.stopPropagation();
            addChildNode(node);
          },
        },
        {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(AddOutline) }),
        }
      ),
      h(
        NButton,
        {
          size: 'tiny',
          type: 'info',
          secondary: true,
          style: { marginLeft: '4px' },
          onClick: (e) => {
            e.stopPropagation();
            editNode(node);
          },
        },
        {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(CreateOutline) }),
        }
      ),
      h(
        NButton,
        {
          size: 'tiny',
          type: 'error',
          secondary: true,
          style: { marginLeft: '4px' },
          onClick: (e) => {
            e.stopPropagation();
            deleteNode(node);
          },
        },
        {
          icon: () => h(NIcon, { size: 14 }, { default: () => h(TrashOutline) }),
        }
      ),
    ]);
  };

  // 节点操作
  const addRootNode = () => {
    resetNodeForm();
    editingNode.value = null;
    editingNodePath.value = [];
    showNodeModal.value = true;
  };

  const addChildNode = (parentNode) => {
    resetNodeForm();
    editingNode.value = null;
    editingNodePath.value = parentNode.path;
    showNodeModal.value = true;
  };

  const editNode = (node) => {
    editingNode.value = node;
    editingNodePath.value = node.path;
    nodeForm.key = node.label;
    nodeForm.type = node.nodeType;
    nodeForm.description = node.description || '';
    nodeForm.required = node.required || false;

    // 设置默认值
    if (node.nodeType === 'string') {
      nodeForm.defaultValue = node.value || '';
    } else if (node.nodeType === 'number') {
      nodeForm.defaultValue = node.value || 0;
    } else if (node.nodeType === 'boolean') {
      nodeForm.defaultValue = node.value || false;
    }

    showNodeModal.value = true;
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
    nodeForm.type = 'string';
    nodeForm.description = '';
    nodeForm.defaultValue = '';
    nodeForm.itemType = 'string';
    nodeForm.required = false;
  };

  const handleTypeChange = (type) => {
    // 根据类型重置默认值
    switch (type) {
      case 'string':
        nodeForm.defaultValue = '';
        break;
      case 'number':
        nodeForm.defaultValue = 0;
        break;
      case 'boolean':
        nodeForm.defaultValue = false;
        break;
      case 'object':
      case 'array':
        nodeForm.defaultValue = '';
        break;
    }
  };

  const saveNode = async () => {
    try {
      await nodeFormRef.value?.validate();
      nodeSaving.value = true;

      const newNode = {
        key: nodeForm.key,
        label: nodeForm.key,
        nodeType: nodeForm.type,
        value: nodeForm.defaultValue,
        path: [...editingNodePath.value, nodeForm.key],
        description: nodeForm.description,
        required: nodeForm.required,
        children: nodeForm.type === 'object' || nodeForm.type === 'array' ? [] : undefined,
      };

      if (editingNode.value) {
        // 编辑现有节点
        updateNodeInTree(newNode);
      } else {
        // 添加新节点
        addNodeToTree(newNode);
      }

      syncTreeToJson();
      closeNodeModal();
      message.success(editingNode.value ? '节点更新成功' : '节点添加成功');
    } catch (error) {
      console.error('保存节点失败:', error);
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

  const updateNodeInTree = (updatedNode) => {
    const updateInArray = (nodes) => {
      for (let i = 0; i < nodes.length; i++) {
        if (nodes[i].path.join('.') === editingNode.value.path.join('.')) {
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
    showNodeModal.value = false;
    editingNode.value = null;
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

  const validateJson = () => {
    if (!editorView) return;

    try {
      const content = editorView.state.doc.toString();
      JSON.parse(content);
      message.success('JSON 格式正确');
    } catch (e) {
      message.error(`JSON 格式错误: ${e.message}`);
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

  const expandAll = () => {
    // 这里需要根据 naive-ui 的 API 来实现
    message.info('展开全部功能开发中');
  };

  const collapseAll = () => {
    // 这里需要根据 naive-ui 的 API 来实现
    message.info('收起全部功能开发中');
  };

  const handleNodeSelect = (selectedKeys) => {
    // 处理节点选择
    console.log('Selected nodes:', selectedKeys);
  };

  // 监听 modelValue 变化
  watch(
    () => props.modelValue,
    (newValue) => {
      if (editorView && newValue !== editorView.state.doc.toString()) {
        syncJsonToTree(newValue);
      }
    }
  );

  // 生命周期
  onMounted(async () => {
    await nextTick();
    initEditor();
  });

  onUnmounted(() => {
    if (editorView) {
      editorView.destroy();
    }
  });
</script>

<style scoped>
  .var-preset-editor {
    width: 100%;
    height: 600px;
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    overflow: hidden;
  }

  .editor-layout {
    display: flex;
    height: 100%;
  }

  .tree-panel {
    flex: 1;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
  }

  .editor-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .tree-header,
  .editor-header {
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fafafa;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tree-header h4,
  .editor-header h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #333;
  }

  .tree-actions,
  .editor-actions {
    display: flex;
    gap: 8px;
  }

  .tree-content {
    flex: 1;
    padding: 8px;
    overflow-y: auto;
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
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

  .json-info {
    color: #666;
  }

  .tree-node-label {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .node-name {
    font-weight: 500;
    color: #333;
  }

  .node-type {
    font-size: 11px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.05);
  }

  .node-desc {
    font-size: 12px;
    color: #666;
    font-style: italic;
  }

  .tree-node-actions {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.2s;
  }

  :deep(.n-tree-node:hover) .tree-node-actions {
    opacity: 1;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  :deep(.cm-editor) {
    height: 100%;
    font-size: 13px;
  }

  :deep(.cm-focused) {
    outline: none;
  }
</style>
