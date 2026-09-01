<template>
  <div class="tree-panel" :style="{ width: panelWidth + 'px' }">
    <div class="panel-header">
      <strong>变量树</strong>
    </div>
    <div class="col-resize-handle" @mousedown="startResize"></div>
    <div class="tree-content" @contextmenu="onTreeAreaContextMenu">
      <a-tree
        v-if="treeData.length > 0"
        :tree-data="treeData"
        :selected-keys="selectedKeys"
        :expanded-keys="expandedKeys"
        :field-names="{ key: 'id', title: 'label', children: 'children' }"
        selectable
        block-node
        show-icon
        @select="handleSelect"
        @expand="handleExpand"
        @rightclick="onNodeRightClick"
      >
        <template #title="{ dataRef }">
          <div class="tree-node-content" style="display: flex; align-items: center; justify-content: space-between; width: 100%">
            <div style="display: flex; align-items: center; gap: 4px">
              <template v-if="dataRef.isEditing">
                <input
                  class="vscode-tree-input"
                  :placeholder="renamingNode && renamingNode.id === dataRef.id ? '重命名变量' : '输入变量名'"
                  :value="newVariableName"
                  @input="newVariableName = $event.target.value"
                  @click.stop
                  @keydown.enter.prevent="confirmEditVariable"
                  @keydown.escape.prevent="cancelEdit"
                  @focus="$event.target.select()"
                />
              </template>
              <template v-else>
                <span class="tree-node-label">{{ dataRef.label }}</span>
              </template>
            </div>
            <div class="node-actions" @click.stop>
              <a-dropdown :trigger="['click']" placement="bottomRight">
                <a-button size="small" type="text" style="padding: 0 4px">
                  <EllipsisHorizontalOutline />
                </a-button>
                <template #overlay>
                  <a-menu @click="({ key }) => handleNodeAction(key, dataRef)">
                    <a-menu-item v-if="dataRef.type === 'object' || dataRef.type === 'object_arr'" key="add-child">
                      <AddOutline style="margin-right: 8px" />
                      添加子变量
                    </a-menu-item>
                    <a-menu-item key="rename">
                      <CreateOutline style="margin-right: 8px" />
                      重命名变量
                    </a-menu-item>
                    <a-menu-item key="delete" danger>
                      <TrashOutline style="margin-right: 8px" />
                      删除变量
                    </a-menu-item>
                  </a-menu>
                </template>
              </a-dropdown>
            </div>
          </div>
        </template>
        <template #icon>
          <FolderOutline style="font-size: 14px" />
        </template>
      </a-tree>
      <a-empty v-else description="暂无变量，右键添加变量" />

      <!-- 右键上下文菜单 -->
      <div
        v-if="showContextMenuFlag"
        class="context-menu-overlay"
        @click="hideContextMenu"
        @contextmenu.prevent="hideContextMenu"
      >
        <div
          class="context-menu"
          :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
          @click.stop
        >
          <div
            v-for="option in contextMenuOptions"
            :key="option.key"
            class="context-menu-item"
            :class="{ 'context-menu-item-danger': option.key === 'delete' }"
            @click="handleContextMenuAction(option.key)"
          >
            <component :is="option.icon" style="font-size: 14px; margin-right: 8px" v-if="option.icon" />
            {{ option.label }}
          </div>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <a-modal
      v-model:open="showDeleteConfirmModal"
      title="确认删除"
      :width="400"
      :mask-closable="false"
      @cancel="showDeleteConfirmModal = false"
    >
      <div>
        <p>
          确定要删除变量 <strong>{{ deletingNode?.label }}</strong> 吗？
        </p>
        <span style="color: var(--editor-muted, #999)" v-if="deletingNode?.hasChildren">
          此变量包含子变量，删除后所有子变量也会被删除。
        </span>
      </div>
      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 12px">
          <a-button @click="showDeleteConfirmModal = false">取消</a-button>
          <a-button danger @click="confirmDelete">确定删除</a-button>
        </div>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
  import { computed, ref, h, onMounted, onUnmounted } from 'vue';

// 面板宽度（右缘拖拽调节，范围 160–420）
const panelWidth = ref(190)
const startResize = (e) => {
  e.preventDefault()
  const startX = e.clientX
  const startW = panelWidth.value
  const onMove = (ev) => {
    panelWidth.value = Math.min(420, Math.max(160, startW + (ev.clientX - startX)))
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

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
  } from '@/icons/ionicons5';

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

    // object 和 object_arr 类型永远不是叶子节点（可以添加子变量）
    const isContainerType = component.type === 'object' || component.type === 'object_arr';

    const node = {
      id: component.id,
      label: component.fieldName,
      fieldName: component.fieldName,
      type: component.type,
      schema: component.schema,
      path: currentPath,
      isLeaf: !isContainerType && !hasChildren,
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
      const isContainerType = nodeType === 'object' || nodeType === 'object_arr';

      const node = {
        id: editingNode.id,
        label: newVariableName.value || '',
        type: nodeType,
        path: editingNode.id,
        isLeaf: !isContainerType,
        isEditing: true,
        hasChildren: false,
        children: [],
      };
      return node;
    };

    if (editingNode.isRoot || !editingNode.parentId) {
      treeData.unshift(createEditingNode());
      return true;
    }

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

  // ==================== 节点右键处理 ====================
  const onNodeRightClick = ({ event, node }) => {
    event.preventDefault();
    event.stopPropagation();

    contextMenuOptions.value = getNodeMenuOptions(node.dataRef);
    contextMenuTarget.value = node.dataRef;
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenuFlag.value = true;
  };

  // ==================== 菜单选项 ====================
  const getNodeMenuOptions = (option) => {
    const menuOptions = [];

    if (option.type === 'object' || option.type === 'object_arr') {
      menuOptions.push({
        label: '添加子变量',
        key: 'add-child',
        icon: AddOutline,
      });
    }

    menuOptions.push({
      label: '重命名变量',
      key: 'rename',
      icon: CreateOutline,
    });

    menuOptions.push({
      label: '删除变量',
      key: 'delete',
      icon: TrashOutline,
    });

    return menuOptions;
  };

  // ==================== 事件处理 ====================

  const handleSelect = (keys, { node }) => {
    if (editingNode.value || renamingNode.value) {
      cancelEdit();
    }

    if (keys && keys.length > 0) {
      emit('update:selectedComponentId', keys[0]);
    } else {
      emit('update:selectedComponentId', null);
    }
  };

  const handleExpand = (keys) => {
    emit('update:expandedKeys', keys);
  };

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

  const onTreeAreaContextMenu = (event) => {
    if (event.target.closest('.ant-tree-treenode')) return;

    event.preventDefault();
    event.stopPropagation();

    contextMenuOptions.value = [
      {
        label: '添加变量',
        key: 'add-variable',
        icon: AddOutline,
      },
    ];

    contextMenuTarget.value = null;
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenuFlag.value = true;
  };

  const handleContextMenuAction = (key) => {
    if (contextMenuTarget.value) {
      handleNodeAction(key, contextMenuTarget.value);
    } else {
      handleEmptyAreaAction(key);
    }
    hideContextMenu();
  };

  const handleEmptyAreaAction = (key) => {
    switch (key) {
      case 'add-variable':
        startAddVariable('');
        break;
    }
  };

  const hideContextMenu = () => {
    showContextMenuFlag.value = false;
    contextMenuTarget.value = null;
  };

  // ==================== 变量操作 ====================
  const startAddVariable = (parentId) => {
    cancelEdit();

    const tempId = `__new__${parentId || 'root'}_${Date.now()}`;
    editingNode.value = {
      id: tempId,
      parentId: parentId,
      isRoot: !parentId,
      type: 'string',
    };
    newVariableName.value = '';

    if (parentId && !props.expandedKeys.includes(parentId)) {
      emit('update:expandedKeys', [...props.expandedKeys, parentId]);
    }
  };

  const startRenameVariable = (option) => {
    cancelEdit();

    renamingNode.value = {
      id: option.id,
      oldName: option.fieldName,
    };

    newVariableName.value = option.fieldName;
  };

  const startDeleteVariable = (option) => {
    deletingNode.value = {
      id: option.id,
      label: option.fieldName,
      hasChildren: option.hasChildren,
    };
    showDeleteConfirmModal.value = true;
  };

  const confirmDelete = () => {
    if (!deletingNode.value) return;

    emit('delete-variable', deletingNode.value.id);

    if (selectedKeys.value.includes(deletingNode.value.id)) {
      emit('update:selectedComponentId', null);
    }

    showDeleteConfirmModal.value = false;
    deletingNode.value = null;
  };

  const confirmEditVariable = () => {
    if (!newVariableName.value.trim()) {
      console.warn('请输入变量名');
      return;
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(newVariableName.value)) {
      console.error('变量名只能包含字母、数字和下划线，且不能以数字开头');
      return;
    }

    const variableName = newVariableName.value.trim();

    if (renamingNode.value) {
      const oldId = renamingNode.value.id;
      const oldName = renamingNode.value.oldName;

      if (oldName === variableName) {
        cancelEdit();
        return;
      }

      emit('rename-variable', {
        id: oldId,
        newName: variableName,
      });

      renamingNode.value = null;
      newVariableName.value = '';
      return;
    }

    if (!editingNode.value) return;

    if (editingNode.value.isRoot) {
      emit('add-variable', {
        parentId: null,
        fieldName: variableName,
        type: editingNode.value.type || 'string',
      });
      editingNode.value = null;
      newVariableName.value = '';
    } else {
      emit('add-variable', {
        parentId: editingNode.value.parentId,
        fieldName: variableName,
        type: editingNode.value.type || 'string',
      });
      editingNode.value = null;
      newVariableName.value = '';
    }
  };

  const cancelEdit = () => {
    if (editingNode.value || renamingNode.value) {
      editingNode.value = null;
      renamingNode.value = null;
      newVariableName.value = '';
    }
  };

  const handleGlobalClick = (event) => {
    const inputElement = event.target.closest('.vscode-tree-input');
    const menuElement = event.target.closest('.ant-dropdown-menu');
    const modalElement = event.target.closest('.ant-modal');

    if (inputElement || menuElement || modalElement) return;

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
    .col-resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 5px;
    cursor: col-resize;
    z-index: 10;
    background: transparent;
    transition: background 0.15s ease;
  }

  .col-resize-handle:hover {
    background: var(--editor-accent, #18a058);
    opacity: 0.5;
  }

  .tree-panel {
    flex-shrink: 0;
    position: relative;
    border-right: 1px solid var(--editor-border, #e0e0e0);
    display: flex;
    flex-direction: column;
    background: var(--editor-inset-bg, #fafafa);
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-panel-bg, #fff);
    display: flex;
    align-items: center;
    min-height: 56px;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .tree-content {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--editor-inset-bg, #f9f9f9);
    position: relative;
    min-height: 0;
  }

  /* 节点操作按钮 */
  .node-actions {
    opacity: 0;
    transition: opacity 0.2s;
    margin-left: 8px;
  }

  .ant-tree-treenode:hover .node-actions {
    opacity: 1;
  }

  /* VSCode风格的树输入框样式 */
  :deep(.vscode-tree-input) {
    width: 100%;
    height: 22px;
    padding: 1px 4px;
    border: 1px solid var(--editor-border, #d9d9d9);
    border-radius: 2px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: all 0.2s;
    box-sizing: border-box;
  }

  :deep(.vscode-tree-input:focus) {
    border-color: var(--color-info, #007acc);
    box-shadow: 0 0 0 2px rgba(0, 122, 204, 0.25);
  }

  :deep(.vscode-tree-input::placeholder) {
    color: var(--editor-muted, #999999);
    font-style: italic;
  }

  /* 树节点标签样式 */
  .tree-node-label {
    font-size: 13px;
    user-select: none;
  }

  /* 右键菜单 */
  .context-menu-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
  }

  .context-menu {
    position: fixed;
    background: var(--editor-panel-bg, #fff);
    border: 1px solid var(--editor-border, #e8e8e8);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    padding: 4px 0;
    min-width: 140px;
    z-index: 1001;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    cursor: pointer;
    font-size: 13px;
    color: var(--editor-primary, #333);
    transition: background 0.2s;
  }

  .context-menu-item:hover {
    background: var(--editor-inset-bg, #f5f5f5);
  }

  .context-menu-item-danger {
    color: #ff4d4f;
  }

  .context-menu-item-danger:hover {
    background: rgba(229, 72, 77, 0.09);
  }
</style>
