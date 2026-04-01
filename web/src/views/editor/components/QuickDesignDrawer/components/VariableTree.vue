<template>
  <div class="tree-panel">
    <div class="panel-header">
      <n-text strong>变量树</n-text>
    </div>
    <div class="tree-content" @contextmenu="onTreeAreaContextMenu">
      <n-tree
        v-if="treeData.length > 0"
        :data="treeData"
        :selected-keys="selectedKeys"
        :expanded-keys="expandedKeys"
        key-field="id"
        label-field="label"
        children-field="children"
        :node-props="nodeProps"
        :render-switcher-icon="renderSwitcherIcon"
        :render-label="renderLabel"
        selectable
        block-line
        @update:selected-keys="handleSelect"
        @update:expanded-keys="handleExpand"
      >
        <template #suffix="{ option }">
          <div class="node-actions" @click.stop>
            <n-dropdown
              :options="getNodeMenuOptions(option)"
              @select="(key) => handleNodeAction(key, option)"
              trigger="click"
              placement="bottom-end"
            >
              <n-button size="tiny" quaternary circle>
                <template #icon>
                  <n-icon><EllipsisHorizontalOutline /></n-icon>
                </template>
              </n-button>
            </n-dropdown>
          </div>
        </template>
      </n-tree>
      <n-empty v-else description="暂无变量，右键添加变量" />

      <!-- 右键上下文菜单 -->
      <n-dropdown
        to="body"
        trigger="manual"
        :x="contextMenuX"
        :y="contextMenuY"
        :options="contextMenuOptions"
        :show="showContextMenuFlag"
        @select="handleContextMenuAction"
        @clickoutside="hideContextMenu"
        placement="bottom-start"
      />
    </div>

    <!-- 删除确认对话框 -->
    <n-modal v-model:show="showDeleteConfirmModal" :mask-closable="false">
      <n-card style="width: 400px" title="确认删除" :bordered="false" size="huge">
        <template #header-extra>
          <n-button quaternary circle @click="showDeleteConfirmModal = false">
            <template #icon>
              <n-icon><CloseOutline /></n-icon>
            </template>
          </n-button>
        </template>

        <div>
          <p
            >确定要删除变量 <strong>{{ deletingNode?.label }}</strong> 吗？</p
          >
          <n-text depth="3" v-if="deletingNode?.hasChildren">
            此变量包含子变量，删除后所有子变量也会被删除。
          </n-text>
        </div>

        <template #footer>
          <div style="display: flex; justify-content: flex-end; gap: 12px">
            <n-button @click="showDeleteConfirmModal = false">取消</n-button>
            <n-button type="error" @click="confirmDelete">确定删除</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
  import { computed, ref, h, nextTick, onMounted, onUnmounted } from 'vue';
  import {
    NText,
    NTree,
    NEmpty,
    NButton,
    NIcon,
    NDropdown,
    NModal,
    NCard,
    NSpace,
    NTag,
  } from 'naive-ui';
  import {
    TextOutline,
    EllipsisHorizontalOutline,
    CloseOutline,
    CalculatorOutline,
    RadioButtonOn,
    ToggleOutline,
    ListOutline,
    LockClosedOutline,
    FolderOutline,
    CodeSlashOutline,
    AppsOutline,
    ChevronForward,
    AddOutline,
    TrashOutline,
    CreateOutline,
    PlayOutline,
  } from '@vicons/ionicons5';

  /**
   * VariableTree 组件
   * 负责以树形结构展示变量层次，支持右键菜单、节点操作、内联编辑等功能
   */

  // Props
  const props = defineProps({
    components: {
      type: Array,
      default: () => [],
    },
    selectedComponentId: {
      type: String,
      default: null,
    },
    expandedKeys: {
      type: Array,
      default: () => [],
    },
  });

  // Emits
  const emit = defineEmits([
    'update:selectedComponentId',
    'update:expandedKeys',
    'add-variable',
    'delete-variable',
    'rename-variable',
    'add-child',
  ]);

  // ==================== 状态管理 ====================
  // 右键菜单状态
  const showContextMenuFlag = ref(false);
  const contextMenuX = ref(0);
  const contextMenuY = ref(0);
  const contextMenuOptions = ref([]);
  const contextMenuTarget = ref(null);

  // 编辑状态管理
  const editingNode = ref(null);
  const renamingNode = ref(null);
  const newVariableName = ref('');

  // 对话框状态
  const showDeleteConfirmModal = ref(false);
  const deletingNode = ref(null);

  // ==================== 计算属性 ====================
  const selectedKeys = computed(() => {
    return props.selectedComponentId ? [props.selectedComponentId] : [];
  });

  /**
   * 将组件转换为树节点
   */
  const componentToTreeNode = (component, parentPath = '') => {
    const currentPath = parentPath ? `${parentPath}.${component.id}` : component.id;
    const hasChildren = component.children && component.children.length > 0;
    const isRenaming = renamingNode.value && renamingNode.value.id === component.id;

    // object 和 object_arr 类型永远不是叶子节点（可以添加子变量）
    const isContainerType = component.type === 'object' || component.type === 'object_arr';

    const node = {
      id: component.id,
      label: component.fieldName,
      fieldName: component.fieldName,
      type: component.type,
      schema: component.schema,
      path: currentPath,
      isLeaf: !isContainerType && !hasChildren, // 容器类型永远不是叶子
      isEditing: false,
      hasChildren: hasChildren,
      children: [],
    };

    // 如果有子组件，递归转换
    if (hasChildren) {
      node.children = component.children.map((child) => componentToTreeNode(child, currentPath));
    }

    return node;
  };

  const treeData = computed(() => {
    const data = props.components.map((comp) => componentToTreeNode(comp));

    // 如果有编辑节点，插入到正确位置
    if (editingNode.value) {
      insertEditingNodeToTree(data, editingNode.value);
    }

    // 如果有重命名节点，更新树节点的编辑状态
    if (renamingNode.value) {
      updateRenamingNodeInTree(data, renamingNode.value);
    }

    return data;
  });

  /**
   * 将编辑节点插入到树的正确位置
   */
  const insertEditingNodeToTree = (treeData, editingNode) => {
    const createEditingNode = () => {
      const nodeType = editingNode.type || 'string';
      // 容器类型（object, object_arr）永远不是叶子节点
      const isContainerType = nodeType === 'object' || nodeType === 'object_arr';

      const node = {
        id: editingNode.id,
        label: newVariableName.value || '',
        type: nodeType,
        path: editingNode.id,
        isLeaf: !isContainerType, // 容器类型不是叶子
        isEditing: true,
        hasChildren: false,
        children: [],
      };
      return node;
    };

    // 如果是根节点编辑
    if (editingNode.isRoot || !editingNode.parentId) {
      treeData.unshift(createEditingNode());
      return true;
    }

    // 否则查找父节点并插入
    const findAndInsert = (nodes) => {
      for (const node of nodes) {
        if (node.id === editingNode.parentId) {
          if (!node.children) node.children = [];
          node.children.unshift(createEditingNode());
          return true;
        }
        if (node.children && node.children.length > 0 && findAndInsert(node.children)) {
          return true;
        }
      }
      return false;
    };

    return findAndInsert(treeData);
  };

  /**
   * 更新重命名节点的编辑状态
   */
  const updateRenamingNodeInTree = (treeData, renamingNode) => {
    const updateNode = (nodes) => {
      for (const node of nodes) {
        if (node.id === renamingNode.id) {
          node.isEditing = true;
          return true;
        }
        if (node.children && node.children.length > 0 && updateNode(node.children)) {
          return true;
        }
      }
      return false;
    };

    updateNode(treeData);
  };

  // ==================== 节点属性配置 ====================
  const nodeProps = ({ option }) => {
    return {
      onContextmenu(e) {
        e.preventDefault();
        e.stopPropagation();

        contextMenuOptions.value = getNodeMenuOptions(option);
        contextMenuTarget.value = option;
        contextMenuX.value = e.clientX;
        contextMenuY.value = e.clientY;
        showContextMenuFlag.value = true;
      },
    };
  };

  // ==================== 图标渲染 ====================
  /**
   * 获取变量类型图标
   */
  const getVariableIconComponent = (type) => {
    const iconMap = {
      string: TextOutline,
      integer: CalculatorOutline,
      number: RadioButtonOn,
      boolean: ToggleOutline,
      enum: ListOutline,
      secret: LockClosedOutline,
      object: FolderOutline,
      array: CodeSlashOutline,
      object_arr: AppsOutline,
    };
    return iconMap[type] || TextOutline;
  };

  /**
   * 渲染展开/收起图标
   */
  const renderSwitcherIcon = () => h(NIcon, null, { default: () => h(ChevronForward) });

  /**
   * 渲染标签（支持内联编辑）
   */
  const renderLabel = ({ option }) => {
    if (option.isEditing === true) {
      const isRenaming = renamingNode.value && renamingNode.value.id === option.id;
      const placeholder = isRenaming ? '重命名变量' : '输入变量名';

      // 创建 input 元素，使用 onInput 和 onVnodeMounted 实现双向绑定
      const inputEl = h('input', {
        class: 'vscode-tree-input',
        placeholder: placeholder,
        // 阻止点击冒泡，避免触发节点选择
        onClick: (e) => {
          e.stopPropagation();
        },
        // 使用 onInput 事件监听输入
        onInput: (e) => {
          newVariableName.value = e.target.value;
        },
        // 按键事件
        onKeydown: (e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            confirmEditVariable();
          } else if (e.key === 'Escape') {
            e.preventDefault();
            cancelEdit();
          }
        },
        // 使用 DOM 更新后的生命周期钩子设置初始值和聚焦
        onVnodeMounted: (vnode) => {
          const input = vnode.el;
          if (input) {
            input.value = newVariableName.value;
            input.focus();
            input.select();
          }
        },
      });

      return inputEl;
    }

    // 正常显示标签 - 使用 span 包裹，不阻止点击事件
    return h('span', { class: 'tree-node-label' }, option.label);
  };

  // ==================== 菜单选项 ====================
  /**
   * 获取节点菜单选项
   */
  const getNodeMenuOptions = (option) => {
    const menuOptions = [];

    // 对象类型和对象数组类型都可以添加子属性
    if (option.type === 'object' || option.type === 'object_arr') {
      menuOptions.push({
        label: '添加子变量',
        key: 'add-child',
        icon: () => h(NIcon, null, { default: () => h(AddOutline) }),
      });
    }

    // 重命名变量
    menuOptions.push({
      label: '重命名变量',
      key: 'rename',
      icon: () => h(NIcon, null, { default: () => h(CreateOutline) }),
    });

    // 删除变量
    menuOptions.push({
      label: '删除变量',
      key: 'delete',
      icon: () => h(NIcon, null, { default: () => h(TrashOutline) }),
    });

    return menuOptions;
  };

  // ==================== 事件处理 ====================

  /**
   * 处理节点选择
   */
  const handleSelect = (keys) => {
    // 如果正在编辑中且点击的是编辑节点，不处理
    if (
      (editingNode.value || renamingNode.value) &&
      keys.length > 0 &&
      (keys[0].startsWith('__new__') || keys[0] === editingNode.value?.id)
    ) {
      return;
    }

    // 如果正在编辑但点击的是其他节点，先取消编辑状态
    if (editingNode.value || renamingNode.value) {
      cancelEdit();
    }

    if (keys && keys.length > 0) {
      const selectedId = keys[0];
      emit('update:selectedComponentId', selectedId);
    } else {
      emit('update:selectedComponentId', null);
    }
  };

  /**
   * 处理节点展开/折叠
   */
  const handleExpand = (keys) => {
    emit('update:expandedKeys', keys);
  };

  /**
   * 处理节点操作
   */
  const handleNodeAction = (key, option) => {
    switch (key) {
      case 'add-child':
        startAddVariable(option.id);
        break;
      case 'rename':
        startRenameVariable(option);
        break;
      case 'delete':
        startDeleteVariable(option);
        break;
    }
  };

  /**
   * 树区域右键菜单处理
   */
  const onTreeAreaContextMenu = (event) => {
    // 检查是否点击在树节点上
    if (event.target.closest('.n-tree-node')) return;

    event.preventDefault();
    event.stopPropagation();

    // 设置空白区域的右键菜单选项
    contextMenuOptions.value = [
      {
        label: '添加变量',
        key: 'add-variable',
        icon: () => h(NIcon, null, { default: () => h(AddOutline) }),
      },
    ];

    contextMenuTarget.value = null; // 标记为空白区域
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenuFlag.value = true;
  };

  /**
   * 处理右键菜单操作
   */
  const handleContextMenuAction = (key) => {
    if (contextMenuTarget.value) {
      handleNodeAction(key, contextMenuTarget.value);
    } else {
      // 空白区域的右键菜单操作
      handleEmptyAreaAction(key);
    }
    hideContextMenu();
  };

  /**
   * 空白区域右键菜单处理
   */
  const handleEmptyAreaAction = (key) => {
    switch (key) {
      case 'add-variable':
        startAddVariable('');
        break;
    }
  };

  /**
   * 隐藏右键菜单
   */
  const hideContextMenu = () => {
    showContextMenuFlag.value = false;
    contextMenuTarget.value = null;
  };

  // ==================== 变量操作 ====================
  /**
   * 开始添加变量（根变量或子变量）
   * @param {String} parentId - 父节点ID，空字符串表示根变量
   */
  const startAddVariable = (parentId) => {
    // 取消任何现有的编辑状态
    cancelEdit();

    const tempId = `__new__${parentId || 'root'}_${Date.now()}`;
    editingNode.value = {
      id: tempId,
      parentId: parentId,
      isRoot: !parentId,
      type: 'string', // 默认字符串类型
    };
    newVariableName.value = '';

    // 如果是子变量，展开父级
    if (parentId && !props.expandedKeys.includes(parentId)) {
      emit('update:expandedKeys', [...props.expandedKeys, parentId]);
    }
  };

  /**
   * 开始重命名变量
   */
  const startRenameVariable = (option) => {
    // 取消任何现有的编辑状态
    cancelEdit();

    renamingNode.value = {
      id: option.id,
      oldName: option.fieldName,
    };

    newVariableName.value = option.fieldName;
  };

  /**
   * 开始删除变量
   */
  const startDeleteVariable = (option) => {
    deletingNode.value = {
      id: option.id,
      label: option.fieldName,
      hasChildren: option.hasChildren,
    };
    showDeleteConfirmModal.value = true;
  };

  /**
   * 确认删除变量
   */
  const confirmDelete = () => {
    if (!deletingNode.value) return;

    emit('delete-variable', deletingNode.value.id);

    // 清除选择（如果删除的是当前选中的变量）
    if (selectedKeys.value.includes(deletingNode.value.id)) {
      emit('update:selectedComponentId', null);
    }

    showDeleteConfirmModal.value = false;
    deletingNode.value = null;
  };

  /**
   * 确认编辑变量（添加或重命名）
   */
  const confirmEditVariable = () => {
    if (!newVariableName.value.trim()) {
      console.warn('请输入变量名');
      return;
    }

    // 验证变量名格式
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(newVariableName.value)) {
      console.error('变量名只能包含字母、数字和下划线，且不能以数字开头');
      return;
    }

    const variableName = newVariableName.value.trim();

    // 处理重命名逻辑
    if (renamingNode.value) {
      const oldId = renamingNode.value.id;
      const oldName = renamingNode.value.oldName;

      // 如果名称没有改变，直接返回
      if (oldName === variableName) {
        cancelEdit();
        return;
      }

      emit('rename-variable', {
        id: oldId,
        newName: variableName,
      });

      // 清除重命名状态
      renamingNode.value = null;
      newVariableName.value = '';

      return;
    }

    // 处理新增变量逻辑
    if (!editingNode.value) {
      return;
    }

    if (editingNode.value.isRoot) {
      // 添加根变量
      emit('add-variable', {
        parentId: null,
        fieldName: variableName,
        type: editingNode.value.type || 'string',
      });

      // 清除编辑状态
      editingNode.value = null;
      newVariableName.value = '';
    } else {
      // 添加子变量
      emit('add-variable', {
        parentId: editingNode.value.parentId,
        fieldName: variableName,
        type: editingNode.value.type || 'string',
      });

      // 清除编辑状态
      editingNode.value = null;
      newVariableName.value = '';
    }
  };

  /**
   * 取消编辑
   */
  const cancelEdit = () => {
    if (editingNode.value || renamingNode.value) {
      editingNode.value = null;
      renamingNode.value = null;
      newVariableName.value = '';
    }
  };

  /**
   * 全局点击处理（自动确认编辑）
   */
  const handleGlobalClick = (event) => {
    const inputElement = event.target.closest('.vscode-tree-input');
    const dropdownElement = event.target.closest('.n-dropdown-menu');
    const modalElement = event.target.closest('.n-modal');

    // 如果点击的是输入框、下拉菜单或模态框，不处理
    if (inputElement || dropdownElement || modalElement) return;

    if (editingNode.value || renamingNode.value) {
      confirmEditVariable();
    }
  };

  // ==================== 生命周期 ====================
  onMounted(() => {
    document.addEventListener('click', handleGlobalClick);
  });

  onUnmounted(() => {
    document.removeEventListener('click', handleGlobalClick);
  });
</script>

<style scoped>
  .tree-panel {
    width: 240px;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    background: #fafafa;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .panel-header {
    padding: 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fff;
    display: flex;
    align-items: center;
    min-height: 56px;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .panel-header :deep(.n-button) {
    height: 28px;
    width: 28px;
  }

  .tree-content {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    background: #f9f9f9;
    position: relative;
    min-height: 0;
  }

  /* 节点操作按钮 */
  .node-actions {
    opacity: 0;
    transition: opacity 0.2s;
    margin-left: 8px;
  }

  .n-tree-node:hover .node-actions {
    opacity: 1;
  }

  /* VSCode风格的树输入框样式 */
  :deep(.vscode-tree-input) {
    width: 100%;
    height: 22px;
    padding: 1px 4px;
    border: 1px solid #d9d9d9;
    border-radius: 2px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: all 0.2s;
    box-sizing: border-box;
  }

  :deep(.vscode-tree-input:focus) {
    border-color: #007acc;
    box-shadow: 0 0 0 2px rgba(0, 122, 204, 0.25);
  }

  :deep(.vscode-tree-input::placeholder) {
    color: #999999;
    font-style: italic;
  }

  /* 树节点标签样式 */
  .tree-node-label {
    font-size: 13px;
    user-select: none;
  }

  /* 右键菜单样式 */
  :deep(.n-dropdown-menu) {
    max-height: 300px;
    overflow-y: auto;
  }
</style>
