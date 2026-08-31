<template>
  <div
    class="template-explorer"
    :style="{ width: `${panelWidth}px` }"
    @contextmenu="onTreeAreaContextMenu"
  >
    <div class="explorer-title">
      <span class="title-text">模板资源</span>
      <div class="title-actions">
        <a-tooltip>
          <template #title>下载模板</template>
          <button class="action-icon" @click="handleExport">
            <DownloadOutline style="font-size: 16px" />
          </button>
        </a-tooltip>
        <a-tooltip>
          <template #title>版本管理</template>
          <button class="action-icon" @click="emit('show-releases')">
            <GitBranchOutline style="font-size: 16px" />
          </button>
        </a-tooltip>
      </div>
    </div>
    <div
      class="explorer-container"
      :class="{ 'drag-over-root': isDragOverRoot }"
      ref="explorerContainer"
      @dragover.capture="onContainerDragOver"
      @drop.capture="onContainerDrop"
      @dragenter.capture="onContainerDragEnter"
      @click="onContainerClick"
    >
      <a-tree
        :tree-data="treeDataComputed"
        :selected-keys="[currentFile]"
        :field-names="{ key: 'key', title: 'label', children: 'children' }"
        @select="onSelectFile"
        @expand="updatePrefixWithExpanded"
        @right-click="onNodeRightClick"
        draggable
        @dragstart="onDragStart"
        @dragenter="onDragEnter"
        @dragleave="onDragLeave"
        @dragover="onDragOver"
        @drop="onDrop"
      >
        <template #title="option">
          <span v-if="option.isEditing" class="edit-node-container">
            <input
              class="vscode-tree-input"
              :value="newName"
              autofocus
              style="flex: 1; min-width: 100px; padding: 2px 6px; border: 1px solid #22c55e; border-radius: 4px; outline: none; font-size: 13px;"
              @input="newName = $event.target.value"
              @keydown.enter.prevent="confirmAddNode"
              @keydown.escape.prevent="cancelAddNode"
              @blur="setTimeout(() => { if (editingNode || renamingNode) confirmAddNode() }, 100)"
            />
            <button class="edit-confirm-btn" @click.stop="confirmAddNode" title="确认 (Enter)">✓</button>
            <button class="edit-cancel-btn" @click.stop="cancelAddNode" title="取消 (Esc)">✗</button>
          </span>
          <span v-else style="display: flex; align-items: center; gap: 4px;">
            {{ option.fileName || option.label }}
            <span v-if="option.hasCondition" style="font-size: 14px; margin-left: 4px; color: #22c55e; font-weight: bold;" :title="'文件生成条件：\n' + (option.conditionSummary || '已设置条件') + '\n点击右键菜单可编辑条件'">⚡</span>
          </span>
        </template>
        <template #switcherIcon="{ expanded }">
          <ChevronForward v-if="!expanded" style="font-size: 14px" />
          <ChevronDown v-else style="font-size: 14px" />
        </template>
        <template #icon="{ data }">
          <FolderOpenOutline v-if="data.isDirectory && expandedKeys.has(String(data.key))" style="font-size: 14px" />
          <Folder v-else-if="data.isDirectory" style="font-size: 14px" />
          <FileTrayFullOutline v-else style="font-size: 14px" />
        </template>
      </a-tree>
      <div
        v-if="!treeData || treeData.length === 0"
        style="
          padding: 32px;
          color: #888;
          text-align: center;
          user-select: none;
          cursor: context-menu;
        "
        @contextmenu="onTreeAreaContextMenu"
        >暂无数据（右键新建）</div
      >
      <!-- 根目录拖放区域 - 始终显示 -->
      <div
        class="root-drop-zone"
        :class="{ 'drag-over': isDragOverRoot }"
        @dragover.prevent="onRootZoneDragOver"
        @dragleave="onRootZoneDragLeave"
        @drop.prevent="onRootZoneDrop"
      >
        <div class="root-drop-zone-content">
          <svg viewBox="0 0 24 24" style="width: 16px; height: 16px">
            <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
          </svg>
          <span>拖放文件到此处移动到根目录</span>
        </div>
      </div>
    </div>
    <!-- 自定义右键菜单 -->
    <div v-if="showDropdown" class="context-menu-overlay" @click="showDropdown = false" @contextmenu.prevent="showDropdown = false">
      <div class="context-menu" :style="{ left: dropdownX + 'px', top: dropdownY + 'px' }">
        <template v-for="item in dropdownOptions" :key="item.key">
          <div v-if="item.type === 'divider'" class="context-menu-divider"></div>
          <div v-else class="context-menu-item" @click="handleDropdownSelect(item.key)">
            <component v-if="item.icon" :is="item.icon" style="font-size: 14px; margin-right: 8px" />
            {{ item.label }}
          </div>
        </template>
      </div>
    </div>
    <input
      type="file"
      ref="fileInput"
      accept=".zip"
      style="display: none"
      @change="handleFileSelect"
    />
    <input
      type="file"
      ref="codeFileInput"
      accept=".js,.ts,.py,.go,.java,.c,.cpp,.json,.md,.txt,.html,.css,.sh,.php,.rb,.rs,.cs,.xml,.yml,.yaml,.ini,.cfg,.conf,.log,.vue,.jsx,.tsx,.less,.scss,.sass,.bat,.ps1,.swift,.dart,.r,.m,.mm,.pl,.lua,.groovy,.gradle,.pom,.lock,.toml,.env,.gitignore,.dockerfile,.makefile,.cmake,.rst,.tex,.bib,.wiki,.adoc,.asciidoc"
      style="display: none"
      @change="handleCodeFileSelect"
    />
    <!-- 拖拽调整宽度的分隔条 -->
    <div
      class="resize-handle"
      @mousedown="startResize"
      :class="{ 'is-resizing': isResizing }"
    ></div>
  </div>
</template>

<script setup>
  import { ref, watch, h, computed, onMounted, onUnmounted } from 'vue';
  import { message } from 'ant-design-vue';
  import {
    ChevronForward,
    ChevronDown,
    FileTrayFullOutline,
    Folder,
    FolderOpenOutline,
    Trash,
    CreateOutline as Edit,
    DownloadOutline,
    GitBranchOutline,
  } from '@/icons/ionicons5';
  import { exportTemplate } from '@/api/editor/templates';
  import { useRoute } from 'vue-router';

  const props = defineProps({
    treeData: {
      type: Array,
      default: () => [],
    },
    currentFile: {
      type: [String, Number],
      default: '',
    },
    templateName: {
      type: String,
      default: '',
    },
  });

  const route = useRoute();
  const templateId = route.params.id;
  const emit = defineEmits([
    'select',
    'reload',
    'update:treeData',
    'rename',
    'uploadZip',
    'uploadCodeFile',
    'move',
    'setCondition',
    'show-releases',
  ]);

  const showDropdown = ref(false);
  const dropdownOptions = ref([
    {
      label: '新增文件',
      key: 'addFile',
      icon: FileTrayFullOutline,
    },
    {
      label: '新增文件夹',
      key: 'addFolder',
      icon: Folder,
    },
    { type: 'divider', key: 'divider1' },
    {
      label: '删除节点',
      key: 'deleteNode',
      icon: Trash,
    },
  ]);
  const dropdownX = ref(0);
  const dropdownY = ref(0);
  const dropdownNode = ref(null);
  const addType = ref('file');
  const editingNode = ref(null);
  const newName = ref('');
  const renamingNode = ref(null);
  const fileInput = ref(null);
  const codeFileInput = ref(null);
  const localTreeData = ref(JSON.parse(JSON.stringify(props.treeData)));
  // 维护展开状态的映射
  const expandedKeys = ref(new Set());

  // 拖拽调整宽度相关
  const panelWidth = ref(260);
  const isResizing = ref(false);
  const resizeStartX = ref(0);
  const resizeStartWidth = ref(260);

  // 拖拽移动相关
  const draggedNode = ref(null);
  const dragOverNode = ref(null);
  const isDragging = ref(false);
  const isDragOverRoot = ref(false);
  const lastMousePosition = ref({ x: 0, y: 0 });
  const explorerContainer = ref(null);

  // 全局拖拽事件监听
  let globalDragOverHandler = null;
  let globalDragEndHandler = null;

  // 拖拽调整宽度功能
  function startResize(event) {
    event.preventDefault();
    isResizing.value = true;
    resizeStartX.value = event.clientX;
    resizeStartWidth.value = panelWidth.value;

    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'ew-resize';
    document.body.style.userSelect = 'none';
  }

  function handleResize(event) {
    if (!isResizing.value) return;

    const deltaX = event.clientX - resizeStartX.value;
    const newWidth = resizeStartWidth.value + deltaX;

    // 动态计算最大宽度（基于可用空间）
    const minWidth = 120;
    const container = document.querySelector('.edit-main');
    const maxAvailableWidth = container ? container.clientWidth - 300 : 800; // 为编辑器+预览面板保留至少300px
    const maxWidth = Math.min(Math.max(maxAvailableWidth, 400), 1000); // 最大不超过1000px，最少400px

    if (newWidth >= minWidth && newWidth <= maxWidth) {
      panelWidth.value = newWidth;
    }
  }

  function stopResize() {
    isResizing.value = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  // 全局点击事件处理
  function handleGlobalClick(event) {
    // 检查是否点击在输入框外
    const inputElement = event.target.closest('.vscode-tree-input');
    const isTreeNode = event.target.closest('.ant-tree-treenode');

    // 如果有正在编辑的节点，且点击在输入框外，则确认编辑
    if ((editingNode.value || renamingNode.value) && !inputElement) {
      confirmAddNode();
    }
  }

  watch(
    () => props.treeData,
    (val) => {
      localTreeData.value = JSON.parse(JSON.stringify(val));
    },
    { deep: true }
  );

  // 组件挂载时添加全局点击监听
  onMounted(() => {
    document.addEventListener('click', handleGlobalClick, true);
  });

  // 组件卸载时移除全局点击监听
  onUnmounted(() => {
    document.removeEventListener('click', handleGlobalClick, true);
    // 清理拖拽相关事件监听
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    // 清理拖拽状态和监听器
    clearDragState();
  });

  // 计算属性，确保展开状态变化时重新渲染
  const treeDataComputed = computed(() => {
    return convertToAntTree(localTreeData.value);
  });

  function convertToAntTree(tree) {
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

      return {
        label: node.isEditing === true ? undefined : node.fileName || node.label,
        key: nodeKey,
        isLeaf: node.isDirectory === 0,
        isEditing: node.isEditing === true,
        filePath: node.filePath,
        fileName: node.fileName,
        isDirectory: node.isDirectory === 1,
        hasCondition: node.hasCondition,
        generateCondition: node.generateCondition,
        children: node.children ? convertToAntTree(node.children) : [],
        // 添加拖拽状态的类名
        class: getDragClass(nodeKey),
      };
    });
  }

  // 更新展开/闭合时的图标
  function updatePrefixWithExpanded(keys, _option, meta) {
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
  }

  function onSelectFile(keys) {
    if (keys && keys.length > 0) {
      emit('select', keys[0]);
    }
  }

  // 检查是否为子节点
  function isDescendant(parentNode, childNode) {
    if (!parentNode.children || parentNode.children.length === 0) return false;

    for (const child of parentNode.children) {
      if (child.key === childNode.key) return true;
      if (isDescendant(child, childNode)) return true;
    }
    return false;
  }

  // 根据 key 查找节点
  function findNodeByKey(nodes, key) {
    if (!nodes || !Array.isArray(nodes)) return null;

    for (const node of nodes) {
      if (String(node.key) === String(key) || String(node.id) === String(key)) {
        return node;
      }

      if (node.children && node.children.length > 0) {
        const found = findNodeByKey(node.children, key);
        if (found) return found;
      }
    }

    return null;
  }

  // 获取拖拽状态的类名
  function getDragClass(nodeKey) {
    const classes = [];

    if (draggedNode.value && String(draggedNode.value.key) === String(nodeKey)) {
      classes.push('dragging');
    }

    if (dragOverNode.value && String(dragOverNode.value.key) === String(nodeKey)) {
      classes.push('drag-over');
    }

    return classes.join(' ');
  }

  function nodeProps({ option }) {
    return {
      onContextmenu(e) {
        e.preventDefault();
        dropdownNode.value = option;

        // 检查是否在子目录中（有父路径）
        const isInSubdirectory = option.filePath && option.filePath.includes('/');

        if (option.isLeaf) {
          dropdownOptions.value = [
            {
              label: '重命名',
              key: 'renameNode',
              icon: Edit,
            },
            isInSubdirectory ? { type: 'divider', key: 'divider0' } : null,
            isInSubdirectory
              ? {
                  label: '移动到根目录',
                  key: 'moveToRoot',
                  icon: Folder,
                }
              : null,
            { type: 'divider', key: 'divider1' },
            {
              label: '设置生成条件',
              key: 'setCondition',
              icon: Edit,
            },
            { type: 'divider', key: 'divider2' },
            {
              label: '删除节点',
              key: 'deleteNode',
              icon: Trash,
            },
          ].filter(Boolean);
        } else {
          dropdownOptions.value = [
            {
              label: '新增文件',
              key: 'addFile',
              icon: FileTrayFullOutline,
            },
            {
              label: '新增文件夹',
              key: 'addFolder',
              icon: Folder,
            },
            { type: 'divider', key: 'divider1' },
            {
              label: '上传文件',
              key: 'uploadCodeFile',
              icon: FileTrayFullOutline,
            },
            {
              label: '重命名',
              key: 'renameNode',
              icon: Edit,
            },
            isInSubdirectory ? { type: 'divider', key: 'divider2' } : null,
            isInSubdirectory
              ? {
                  label: '移动到根目录',
                  key: 'moveToRoot',
                  icon: Folder,
                }
              : null,
            { type: 'divider', key: 'divider3' },
            {
              label: '设置生成条件',
              key: 'setCondition',
              icon: Edit,
            },
            { type: 'divider', key: 'divider4' },
            {
              label: '删除节点',
              key: 'deleteNode',
              icon: Trash,
            },
          ].filter(Boolean);
        }
        showDropdown.value = true;
        dropdownX.value = e.clientX;
        dropdownY.value = e.clientY;
      },
    };
  }

  // Ant Design Vue Tree 右键点击事件处理
  function onNodeRightClick(info) {
    const { event, node } = info;
    event.preventDefault();
    event.stopPropagation();

    // 从 localTreeData 中查找原始节点数据
    const originalNode = findNodeByKey(localTreeData.value, node.key);
    dropdownNode.value = originalNode || node;

    // 检查是否在子目录中（有父路径）
    const nodeData = originalNode || node;
    const isInSubdirectory = nodeData.filePath && (nodeData.filePath.includes('/') || nodeData.filePath.includes('\\'));

    const isLeaf = nodeData.isLeaf || nodeData.isDirectory === 0;

    if (isLeaf) {
      dropdownOptions.value = [
        {
          label: '重命名',
          key: 'renameNode',
          icon: Edit,
        },
        isInSubdirectory ? { type: 'divider', key: 'divider0' } : null,
        isInSubdirectory
          ? {
              label: '移动到根目录',
              key: 'moveToRoot',
              icon: Folder,
            }
          : null,
        { type: 'divider', key: 'divider1' },
        {
          label: '设置生成条件',
          key: 'setCondition',
          icon: Edit,
        },
        { type: 'divider', key: 'divider2' },
        {
          label: '删除节点',
          key: 'deleteNode',
          icon: Trash,
        },
      ].filter(Boolean);
    } else {
      dropdownOptions.value = [
        {
          label: '新增文件',
          key: 'addFile',
          icon: FileTrayFullOutline,
        },
        {
          label: '新增文件夹',
          key: 'addFolder',
          icon: Folder,
        },
        { type: 'divider', key: 'divider1' },
        {
          label: '上传文件',
          key: 'uploadCodeFile',
          icon: FileTrayFullOutline,
        },
        {
          label: '重命名',
          key: 'renameNode',
          icon: Edit,
        },
        isInSubdirectory ? { type: 'divider', key: 'divider2' } : null,
        isInSubdirectory
          ? {
              label: '移动到根目录',
              key: 'moveToRoot',
              icon: Folder,
            }
          : null,
        { type: 'divider', key: 'divider3' },
        {
          label: '设置生成条件',
          key: 'setCondition',
          icon: Edit,
        },
        { type: 'divider', key: 'divider4' },
        {
          label: '删除节点',
          key: 'deleteNode',
          icon: Trash,
        },
      ].filter(Boolean);
    }

    showDropdown.value = true;
    dropdownX.value = event.clientX;
    dropdownY.value = event.clientY;
  }

  // 设置全局拖拽监听
  function setupGlobalDragListeners() {
    // 先清理之前的监听器，避免重复添加
    clearGlobalDragListeners();

    globalDragEndHandler = () => {
      console.log('🔚 globalDragEnd - clearing drag state');
      clearDragState();
    };

    document.addEventListener('dragend', globalDragEndHandler);
  }

  // 清理全局拖拽监听
  function clearGlobalDragListeners() {
    if (globalDragEndHandler) {
      document.removeEventListener('dragend', globalDragEndHandler);
      globalDragEndHandler = null;
    }
  }

  // 清理拖拽状态
  function clearDragState() {
    draggedNode.value = null;
    dragOverNode.value = null;
    isDragging.value = false;
    isDragOverRoot.value = false;
    clearGlobalDragListeners();
  }

  // NTree 拖拽事件处理
  function onDragStart(info) {
    console.log('🚀 drag start, raw info:', info);

    // 尝试多种方式获取节点数据，处理 Vue 响应式代理问题
    let rawNode = null;

    // 方式1: 直接访问 node
    if (info.node) {
      rawNode = info.node;
      console.log('✅ Got node from info.node');
    }
    // 方式2: 访问 dragNode
    else if (info.dragNode) {
      rawNode = info.dragNode;
      console.log('✅ Got node from info.dragNode');
    }
    // 方式3: 从 event.target 获取（某些浏览器）
    else if (info.event && info.event.target) {
      const key = info.event.target.getAttribute('data-key');
      if (key) {
        rawNode = findNodeByKey(localTreeData.value, key);
        console.log('✅ Got node from event.target data-key:', key);
      }
    }

    console.log('🚀 rawNode:', rawNode);

    if (!rawNode) {
      console.log('❌ No node found in info');
      return;
    }

    // 使用原始数据创建新对象，避免响应式代理问题
    // 使用 JSON 序列化/反序列化来解除所有响应式代理
    let dragNode;
    try {
      dragNode = JSON.parse(JSON.stringify(rawNode));
    } catch (e) {
      // 如果序列化失败，使用展开运算符
      dragNode = { ...rawNode };
    }

    console.log('🚀 dragNode after clone:', dragNode);

    // 获取原始节点数据（从localTreeData中查找，确保有完整的filePath）
    const originalNode = findNodeByKey(localTreeData.value, dragNode.key);
    console.log('🚀 originalNode from tree:', originalNode);

    // 创建节点数据，优先使用原始节点数据
    const nodeData = {
      ...(originalNode || dragNode),
      key: String(dragNode.key || originalNode?.key || originalNode?.id || ''),
      id: String(dragNode.key || originalNode?.key || originalNode?.id || ''),
      // 确保 filePath 存在
      filePath: originalNode?.filePath || dragNode.filePath || dragNode.label || '',
      fileName: originalNode?.fileName || dragNode.fileName || dragNode.label || '',
    };

    console.log('✅ Final nodeData:', nodeData);

    // 先设置状态
    draggedNode.value = nodeData;
    isDragging.value = true;

    // 使用 nextTick 确认状态已更新
    setTimeout(() => {
      console.log('✅ draggedNode.value after set:', draggedNode.value);
    }, 0);

    // 将拖拽数据存储到 DataTransfer 中
    if (info.event && info.event.dataTransfer) {
      const jsonData = JSON.stringify(nodeData);
      try {
        info.event.dataTransfer.setData('application/json', jsonData);
        info.event.dataTransfer.setData('text/plain', jsonData);
        info.event.dataTransfer.effectAllowed = 'move';
        console.log('✅ Saved to dataTransfer');
      } catch (e) {
        console.error('❌ Failed to set dataTransfer:', e);
      }
    }

    setupGlobalDragListeners();
  }

  function onDragEnter(info) {
    console.log('🎯 drag enter:', info);

    // 尝试多种方式获取节点数据
    let rawNode = null;
    if (info.node) {
      rawNode = info.node;
    } else if (info.dragNode) {
      rawNode = info.dragNode;
    }

    if (rawNode && !isDragging.value) {
      // 使用 JSON 序列化来解除响应式代理
      let dragNode;
      try {
        dragNode = JSON.parse(JSON.stringify(rawNode));
      } catch (e) {
        dragNode = { ...rawNode };
      }

      // 获取原始节点数据
      const originalNode = findNodeByKey(localTreeData.value, dragNode.key);

      const nodeData = {
        ...(originalNode || dragNode),
        key: String(dragNode.key || originalNode?.key || originalNode?.id || ''),
        id: String(dragNode.key || originalNode?.key || originalNode?.id || ''),
        filePath: originalNode?.filePath || dragNode.filePath || dragNode.label || '',
        fileName: originalNode?.fileName || dragNode.fileName || dragNode.label || '',
      };

      draggedNode.value = nodeData;
      isDragging.value = true;

      console.log('✅ drag enter set draggedNode:', nodeData);

      if (info.event && info.event.dataTransfer) {
        const jsonData = JSON.stringify(nodeData);
        info.event.dataTransfer.setData('application/json', jsonData);
        info.event.dataTransfer.setData('text/plain', jsonData);
      }

      setupGlobalDragListeners();
    }
  }

  function onDragLeave(info) {
    console.log('drag leave:', info);
    // 清除节点拖拽状态
    dragOverNode.value = null;
  }

  function onDragOver(info) {
    console.log('🔄 drag over:', info);
    const { event, node } = info;

    // 更新鼠标位置
    lastMousePosition.value = { x: event.clientX, y: event.clientY };

    // 只有文件夹可以作为目标
    if (node && node.isDirectory) {
      event.preventDefault();
      event.dataTransfer.dropEffect = 'move';
      dragOverNode.value = node;
      isDragOverRoot.value = false;
      console.log('📁 dragOver - folder target:', node.fileName);
    } else {
      // 如果不是文件夹，清除dragOverNode状态
      dragOverNode.value = null;
      console.log('📄 dragOver - file target, checking root area');

      // 检查是否在根目录区域
      checkRootAreaFromEvent(event);
    }
  }

  // 从拖拽事件中检查根目录区域
  function checkRootAreaFromEvent(event) {
    if (!isDragging.value) return;

    const { clientX, clientY } = event;
    console.log('🔍 checkRootAreaFromEvent - mouse:', clientX, clientY);

    // 检查是否在容器内
    if (explorerContainer.value) {
      const containerRect = explorerContainer.value.getBoundingClientRect();

      if (
        clientX >= containerRect.left &&
        clientX <= containerRect.right &&
        clientY >= containerRect.top &&
        clientY <= containerRect.bottom
      ) {
        // 检查鼠标位置的元素
        const elementAtPoint = document.elementFromPoint(clientX, clientY);
        const treeNode = elementAtPoint?.closest('.ant-tree-treenode');

        console.log(
          '🎯 checkRootAreaFromEvent - elementAtPoint:',
          elementAtPoint?.tagName,
          'treeNode:',
          !!treeNode
        );

        if (!treeNode) {
          // 在容器内但不在树节点上，说明在根目录区域
          console.log('✅ checkRootAreaFromEvent - ROOT AREA DETECTED!');
          isDragOverRoot.value = true;
          event.preventDefault();
          event.dataTransfer.dropEffect = 'move';
        } else {
          console.log('❌ checkRootAreaFromEvent - on tree node, not root');
          isDragOverRoot.value = false;
        }
      } else {
        console.log('❌ checkRootAreaFromEvent - outside container');
        isDragOverRoot.value = false;
      }
    }
  }

  function onDrop(info) {
    console.log('drop:', info);
    const { event, node, dragNode } = info;

    // 优先使用已保存的draggedNode，因为NTree的dragNode可能为undefined
    const rawDragNode = draggedNode.value || dragNode || info.node;

    // 使用 JSON 序列化来解除响应式代理
    let sourceNode = null;
    let targetNode = null;

    if (rawDragNode) {
      try {
        sourceNode = JSON.parse(JSON.stringify(rawDragNode));
      } catch (e) {
        sourceNode = { ...rawDragNode };
      }
    }

    if (node) {
      try {
        targetNode = JSON.parse(JSON.stringify(node));
      } catch (e) {
        targetNode = { ...node };
      }
    }

    console.log('📦 onDrop sourceNode:', sourceNode);
    console.log('📦 onDrop targetNode:', targetNode);

    // 检查是否是根目录拖拽
    if (isDragOverRoot.value || !targetNode) {
      console.log('Dropping to root directory');
      handleRootDrop(sourceNode);
      return;
    }

    if (!sourceNode || !targetNode.isDirectory || targetNode.isEditing) {
      console.log('Invalid drop target:', { sourceNode, targetNode });
      clearDragState();
      return;
    }

    const sourceId = sourceNode.key;
    const targetId = targetNode.key;

    // 不能移动到自己
    if (sourceId === targetId) {
      console.log('Cannot move to self');
      clearDragState();
      return;
    }

    // 不能移动到自己的子节点
    if (isDescendant(sourceNode, targetNode)) {
      console.log('Cannot move to descendant');
      clearDragState();
      return;
    }

    console.log('Moving from', sourceId, 'to', targetId);

    // 触发移动事件
    emit('move', {
      sourceId: String(sourceId),
      targetId: String(targetId),
      sourceNode,
      targetNode,
    });

    // 清理状态
    clearDragState();
  }

  // 处理根目录拖拽
  function handleRootDrop(dragNode) {
    if (!dragNode) return;

    // 确保节点有key
    const sourceId = dragNode.key || dragNode.id;
    if (!sourceId) {
      console.error('handleRootDrop: 节点没有key或id');
      message.error('节点数据不完整');
      clearDragState();
      return;
    }

    console.log('Moving to root directory:', dragNode.fileName || dragNode.label || sourceId);

    // 移动到根目录
    emit('move', {
      sourceId: String(sourceId),
      targetId: '0',
      sourceNode: dragNode,
      targetNode: { key: '0', isDirectory: true },
    });

    // 清理状态
    clearDragState();
  }

  // 根目录拖放区域事件处理
  function onRootZoneDragOver(event) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
    isDragOverRoot.value = true;
  }

  function onRootZoneDragLeave(event) {
    // 检查鼠标是否真的离开了根目录区域（避免子元素触发dragleave）
    const rect = event.currentTarget.getBoundingClientRect();
    const { clientX, clientY } = event;
    if (
      clientX < rect.left ||
      clientX > rect.right ||
      clientY < rect.top ||
      clientY > rect.bottom
    ) {
      isDragOverRoot.value = false;
    }
  }

  function onRootZoneDrop(event) {
    event.preventDefault();
    event.stopPropagation();

    console.log('Root zone drop triggered');
    console.log('draggedNode:', draggedNode.value);

    // 优先使用已有的draggedNode（从NTree拖拽时设置）
    let nodeToMove = draggedNode.value;

    // 如果draggedNode不存在，尝试从dataTransfer获取
    if (!nodeToMove) {
      try {
        const dragData = event.dataTransfer.getData('application/json');
        console.log('dataTransfer data:', dragData);
        if (dragData) {
          nodeToMove = JSON.parse(dragData);
        }
      } catch (e) {
        console.error('Failed to parse drag data:', e);
      }
    }

    // 如果仍然没有节点数据，尝试从dataTransfer的text/plain获取（备用方案）
    if (!nodeToMove) {
      try {
        const textData = event.dataTransfer.getData('text/plain');
        console.log('text/plain data:', textData);
        if (textData) {
          nodeToMove = JSON.parse(textData);
        }
      } catch (e) {
        console.error('Failed to parse text data:', e);
      }
    }

    console.log('nodeToMove:', nodeToMove);

    if (nodeToMove) {
      // 确保节点有key属性
      if (!nodeToMove.key && nodeToMove.id) {
        nodeToMove.key = nodeToMove.id;
      }

      // 检查节点是否有必要的属性
      if (!nodeToMove.key) {
        message.error('节点数据不完整，请重新选择文件');
        clearDragState();
        return;
      }

      // 如果已经在根目录，提示用户
      // 同时检查正斜杠和反斜杠（Windows 路径）
      const filePath = nodeToMove.filePath || nodeToMove.label || '';
      const isInRoot = filePath && !filePath.includes('/') && !filePath.includes('\\');
      if (isInRoot) {
        message.info('该文件已在根目录中');
        clearDragState();
        return;
      }

      handleRootDrop(nodeToMove);
    } else {
      message.warning('无法识别要移动的文件，请重试');
      clearDragState();
    }
  }

  // 容器级别的拖拽事件处理
  function onContainerDragOver(event) {
    // 只有在非树节点区域才处理，用于支持拖拽到根目录
    const elementAtPoint = document.elementFromPoint(event.clientX, event.clientY);
    const treeNode = elementAtPoint?.closest('.ant-tree-treenode');
    const rootZone = elementAtPoint?.closest('.root-drop-zone');

    // 如果不在树节点上，也不在根目录拖放区域上，但有拖拽的节点，则视为根目录区域
    if (!treeNode && !rootZone && draggedNode.value) {
      event.preventDefault();
      event.dataTransfer.dropEffect = 'move';
      isDragOverRoot.value = true;
    }
  }

  function onContainerDrop(event) {
    // 检查是否是释放在非树节点区域（即根目录区域）
    const elementAtPoint = document.elementFromPoint(event.clientX, event.clientY);
    const treeNode = elementAtPoint?.closest('.ant-tree-treenode');
    const rootZone = elementAtPoint?.closest('.root-drop-zone');

    // 如果不在树节点上，且不在根目录拖放区域上，且我们有正在拖拽的节点
    if (!treeNode && !rootZone && draggedNode.value) {
      event.preventDefault();
      event.stopPropagation();
      console.log('Container drop - treating as root drop, node:', draggedNode.value);
      handleRootDrop(draggedNode.value);
    }
  }

  // 测试函数 - 确认事件绑定工作
  function onContainerClick(event) {
    console.log('🖱️ Container clicked!', event.target);
  }

  function onContainerDragEnter(event) {
    console.log('🌊 Container dragenter triggered!', event.target);
  }

  function onTreeAreaContextMenu(event) {
    if (event.target.closest('.ant-tree')) return;
    event.preventDefault();
    event.stopPropagation();
    dropdownNode.value = null;
    dropdownOptions.value = [
      {
        label: '新增文件',
        key: 'addFile',
        icon: FileTrayFullOutline,
      },
      {
        label: '新增文件夹',
        key: 'addFolder',
        icon: Folder,
      },
      { type: 'divider', key: 'divider1' },
      {
        label: '上传ZIP包',
        key: 'uploadZip',
        icon: Folder,
      },
      {
        label: '上传文件',
        key: 'uploadCodeFile',
        icon: FileTrayFullOutline,
      },
    ];
    showDropdown.value = true;
    dropdownX.value = event.clientX;
    dropdownY.value = event.clientY;
  }
  function addFile() {
    showDropdown.value = false;
    addType.value = 'file';
    insertEditingNode('file');
  }
  function addFolder() {
    showDropdown.value = false;
    addType.value = 'folder';
    insertEditingNode('folder');
  }
  function insertEditingNode(type) {
    const newTreeData = JSON.parse(JSON.stringify(localTreeData.value));
    removeEditingNode(newTreeData);
    newName.value = '';
    const newKey = '__new__' + Date.now() + Math.random().toString(36).slice(2);
    let siblings = [];
    let parentNode = null;
    const parentId = dropdownNode.value
      ? String(dropdownNode.value.id || dropdownNode.value.key)
      : '0';
    if (parentId === '0') {
      siblings = newTreeData;
    } else {
      function findParent(list, pid) {
        for (const item of list) {
          if (String(item.id) === pid || String(item.key) === pid) return item;
          if (item.children) {
            const found = findParent(item.children, pid);
            if (found) return found;
          }
        }
        return null;
      }
      parentNode = findParent(newTreeData, parentId);
      siblings = parentNode && parentNode.children ? parentNode.children : [];
    }
    let maxSort = 0;
    siblings.forEach((n) => {
      const s = Number(n.sort);
      if (!isNaN(s) && s > maxSort) maxSort = s;
    });
    const newNode = {
      key: newKey,
      id: newKey,
      label: '',
      filePath: '',
      isEditing: true,
      isLeaf: type === 'file',
      isDirectory: type === 'folder' ? 1 : 0,
      parentId,
      sort: maxSort + 1,
      children: [],
      parentNode, // 保存父节点引用
    };
    editingNode.value = newNode;
    if (parentId === '0') {
      newTreeData.unshift(newNode);
    } else {
      insertToParent(newTreeData, parentId, newNode);
    }
    localTreeData.value = newTreeData;
  }
  function insertToParent(list, parentId, node) {
    for (const item of list) {
      if (String(item.id) === String(parentId)) {
        if (!item.children) item.children = [];
        item.children.unshift(node);
        return true;
      }
      if (item.children && insertToParent(item.children, parentId, node)) return true;
    }
    return false;
  }
  function removeEditingNode(list) {
    for (let i = list.length - 1; i >= 0; i--) {
      if (list[i].isEditing) {
        // 只处理新增状态，删除临时节点
        list.splice(i, 1);
      } else if (list[i].children) {
        removeEditingNode(list[i].children);
      }
    }
  }
  function confirmAddNode() {
    if (!newName.value) {
      message.warning('请输入名称');
      return;
    }

    // 验证文件名是否包含非法字符
    // Windows文件名不允许的字符：\ / : * ? " < > |
    // 但是允许正斜杠 / 用于路径分隔符
    const illegalChars = /[\\:*?"<>|]/;
    if (illegalChars.test(newName.value)) {
      message.error('文件名不能包含以下字符：\\ : * ? " < > |');
      return;
    }

    // 检查是否为空或只有空格/点
    const trimmed = newName.value.trim();
    if (trimmed === '' || trimmed === '.' || trimmed === '..') {
      message.error('文件名不能为空、点或双点');
      return;
    }

    if (renamingNode.value) {
      // 处理重命名
      emit('rename', {
        id: renamingNode.value.id || renamingNode.value.key,
        oldName: renamingNode.value.fileName || renamingNode.value.label,
        newName: newName.value,
        node: renamingNode.value,
      });
      // 清除重命名状态
      const newTreeData = JSON.parse(JSON.stringify(localTreeData.value));
      function clearEditingState(list) {
        for (const item of list) {
          if (
            String(item.id || item.key) === String(renamingNode.value.id || renamingNode.value.key)
          ) {
            item.isEditing = false;
            return true;
          }
          if (item.children && clearEditingState(item.children)) {
            return true;
          }
        }
        return false;
      }
      clearEditingState(newTreeData);
      localTreeData.value = newTreeData;
      renamingNode.value = null;
    } else {
      // 处理新增
      const parentNode = editingNode.value.parentNode || { filePath: '' };
      emit('reload', {
        name: newName.value,
        type: addType.value,
        parentId: editingNode.value.parentId,
        sort: editingNode.value.sort,
        node: parentNode, // 传递父节点信息，包含 filePath
      });
      editingNode.value = null;
      removeEditingNode(localTreeData.value);
    }
  }

  function cancelAddNode() {
    if (renamingNode.value) {
      // 取消重命名：恢复编辑状态，不清除节点
      const newTreeData = JSON.parse(JSON.stringify(localTreeData.value));
      function clearEditingState(list) {
        for (const item of list) {
          if (
            String(item.id || item.key) === String(renamingNode.value.id || renamingNode.value.key)
          ) {
            item.isEditing = false;
            return true;
          }
          if (item.children && clearEditingState(item.children)) {
            return true;
          }
        }
        return false;
      }
      clearEditingState(newTreeData);
      localTreeData.value = newTreeData;
      renamingNode.value = null;
    } else {
      // 取消新增：删除临时节点
      editingNode.value = null;
      removeEditingNode(localTreeData.value);
    }
  }
  function handleDropdownSelect(key) {
    showDropdown.value = false;
    if (key === 'addFile') addFile();
    else if (key === 'addFolder') addFolder();
    else if (key === 'deleteNode') deleteNode();
    else if (key === 'renameNode') renameNode();
    else if (key === 'uploadZip') uploadZip();
    else if (key === 'uploadCodeFile') uploadCodeFile();
    else if (key === 'setCondition') setCondition();
    else if (key === 'moveToRoot') moveToRoot();
  }

  // 移动到根目录
  function moveToRoot() {
    if (!dropdownNode.value) return;

    const node = dropdownNode.value;
    console.log('Moving to root via context menu:', node.fileName || node.label);

    emit('move', {
      sourceId: node.key,
      targetId: '0',
      sourceNode: node,
      targetNode: { key: '0', isDirectory: true },
    });
  }
  function handleDropdownClickoutside() {
    showDropdown.value = false;
  }
  function deleteNode() {
    if (!dropdownNode.value) return;
    const id = String(dropdownNode.value.id || dropdownNode.value.key);
    const filePath = dropdownNode.value.filePath || '';
    function recursiveDelete(list) {
      for (let i = list.length - 1; i >= 0; i--) {
        if (String(list[i].id || list[i].key) === id) {
          list.splice(i, 1);
        } else if (list[i].children) {
          recursiveDelete(list[i].children);
        }
      }
    }
    const newTree = JSON.parse(JSON.stringify(localTreeData.value));
    recursiveDelete(newTree);
    localTreeData.value = newTree;
    emit('reload', { type: 'delete', filePath });
  }

  function renameNode() {
    if (!dropdownNode.value) return;
    const node = dropdownNode.value;
    const oldName = node.fileName || node.label;
    newName.value = oldName;
    renamingNode.value = node;

    // 设置节点为编辑状态
    const newTreeData = JSON.parse(JSON.stringify(localTreeData.value));
    function setEditingState(list) {
      for (const item of list) {
        if (String(item.id || item.key) === String(node.id || node.key)) {
          item.isEditing = true;
          return true;
        }
        if (item.children && setEditingState(item.children)) {
          return true;
        }
      }
      return false;
    }
    setEditingState(newTreeData);
    localTreeData.value = newTreeData;
  }

  function uploadZip() {
    fileInput.value.click();
  }

  function handleFileSelect({ target }) {
    const file = target.files[0];
    if (!file) return;

    // 文件验证
    if (!file.name.endsWith('.zip')) {
      message.error('请选择ZIP格式的文件');
      target.value = '';
      return;
    }

    if (file.size > 1024 * 1024) {
      message.error('文件大小不能超过1MB');
      target.value = '';
      return;
    }

    // 发送上传事件给父组件
    emit('uploadZip', {
      file,
      parentId: dropdownNode.value ? String(dropdownNode.value.id || dropdownNode.value.key) : '0',
    });

    // 清空文件输入
    target.value = '';
  }

  function uploadCodeFile() {
    codeFileInput.value.value = ''; // reset
    codeFileInput.value.click();
  }

  function setCondition() {
    if (!dropdownNode.value) return;
    emit('setCondition', dropdownNode.value);
  }

  function handleCodeFileSelect({ target }) {
    const file = target.files[0];
    if (!file) return;
    const parentPath = dropdownNode.value ? dropdownNode.value.filePath : undefined;
    emit('uploadCodeFile', { file, parentPath });
    target.value = '';
  }


  // 处理导出模板
  function handleExport() {
    if (!templateId) {
      message.error('模板ID不存在');
      return;
    }

    const fileName = props.templateName
      ? `${props.templateName}_template`
      : `template_${templateId}`;

    try {
      exportTemplate(templateId, 'files', `${fileName}.zip`);
    } catch (error) {
      console.error('导出失败:', error);
      message.error('导出失败');
    }
  }
</script>

<style scoped>
  .template-explorer {
    min-width: 120px;
    max-width: 1000px;
    background: #ffffff;
    border-right: 1px solid var(--editor-border, #e2e8f0);
    padding: 0;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    position: relative;
    flex-shrink: 0;
  }
  .explorer-title {
    height: 48px;
    padding: 0 12px;
    margin-bottom: 0;
    border-bottom: 1px solid var(--editor-border, #e2e8f0);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--editor-muted, #64748b);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .title-text {
    flex: 1;
  }

  .title-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .action-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--editor-muted, #94a3b8);
    opacity: 0;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .explorer-title:hover .action-icon {
    opacity: 1;
  }

  .action-icon:hover {
    background: var(--editor-hover-bg, #f1f5f9);
    color: var(--editor-accent, #22c55e);
  }
  .explorer-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding: 8px 12px 12px 12px;
  }
  .explorer-container::-webkit-scrollbar {
    width: 6px;
  }
  .explorer-container::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 3px;
  }
  .explorer-container::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 3px;
  }
  .explorer-container::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
  }
  /* Firefox 滚动条样式 */
  .explorer-container {
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 #f1f1f1;
  }
  /* 自定义右键菜单样式 */
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
    background: #fff;
    border: 1px solid #e8e8e8;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    min-width: 160px;
    padding: 4px 0;
    z-index: 10000;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
    color: #333;
    transition: background 0.15s;
  }

  .context-menu-item:hover {
    background: #f5f5f5;
  }

  .context-menu-divider {
    height: 1px;
    background: #e8e8e8;
    margin: 4px 0;
  }

  /* VSCode 风格的文件树输入框样式 */
  :deep(.vscode-tree-input) {
    width: 100%;
    height: 22px;
    padding: 1px 4px;
    font-size: 13px;
    font-family: 'Segoe UI', 'Consolas', 'Monaco', monospace;
    background: #ffffff;
    border: 1px solid var(--editor-accent, #22c55e);
    border-radius: 0;
    outline: none;
    color: var(--editor-primary, #1e293b);
    line-height: 18px;
    box-shadow: 0 0 0 1px var(--editor-accent, #22c55e);
    margin: 0;
    display: block;
  }

  /* 编辑节点容器样式 */
  .edit-node-container {
    display: flex !important;
    align-items: center !important;
    gap: 4px !important;
    flex: 1 !important;
    width: 100%;
  }

  /* 编辑确认和取消按钮样式 */
  .edit-confirm-btn,
  .edit-cancel-btn {
    background: var(--editor-accent, #22c55e) !important;
    border: none !important;
    border-radius: 4px !important;
    color: white !important;
    cursor: pointer !important;
    padding: 2px 6px !important;
    font-size: 12px !important;
    display: flex !important;
    align-items: center !important;
    justify-content: center !important;
    min-width: 20px !important;
    height: 20px !important;
    line-height: 1 !important;
    transition: all 0.15s ease !important;
    font-weight: bold !important;
  }

  .edit-confirm-btn:hover {
    background: #16a34a !important;
  }

  .edit-cancel-btn {
    background: #ef4444 !important;
  }

  .edit-cancel-btn:hover {
    background: #dc2626 !important;
  }

  .edit-confirm-btn:active,
  .edit-cancel-btn:active {
    transform: scale(0.95);
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
    background: rgba(34, 197, 94, 0.08);
  }

  .resize-handle:hover:before {
    background: var(--editor-accent, #22c55e);
    height: 60px;
  }

  .resize-handle.is-resizing {
    background: rgba(34, 197, 94, 0.15);
  }

  .resize-handle.is-resizing:before {
    background: var(--editor-accent, #22c55e);
    height: 80px;
  }

  /* 拖拽时的全局样式 */
  .resize-handle:active,
  .resize-handle.is-resizing {
    background: rgba(24, 160, 88, 0.2);
  }

  /* 拖拽移动样式 */
  :deep(.ant-tree-treenode.dragging) {
    opacity: 0.5;
    background: rgba(34, 197, 94, 0.08);
  }

  :deep(.ant-tree-treenode.drag-over) {
    background: rgba(34, 197, 94, 0.12);
    border: 2px dashed var(--editor-accent, #22c55e);
    border-radius: 4px;
  }

  :deep(.ant-tree-treenode.drag-over .ant-tree-node-content-wrapper) {
    background: rgba(34, 197, 94, 0.08);
  }

  /* 拖拽时的全局样式 */
  .template-explorer.dragging {
    user-select: none;
  }

  :deep(.ant-tree-treenode[draggable='true']) {
    cursor: grab;
  }

  :deep(.ant-tree-treenode[draggable='true']:active) {
    cursor: grabbing;
  }

  /* 根目录拖拽样式 */
  .explorer-container.drag-over-root {
    background: rgba(34, 197, 94, 0.08);
    border: 2px dashed var(--editor-accent, #22c55e);
    border-radius: 4px;
    position: relative;
    animation: drag-over-pulse 1s infinite;
  }

  .explorer-container.drag-over-root::before {
    content: '拖拽到此处移动到根目录';
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    color: var(--editor-accent, #22c55e);
    font-size: 14px;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.95);
    padding: 8px 12px;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(34, 197, 94, 0.2);
    pointer-events: none;
    z-index: 1000;
    border: 1px solid var(--editor-accent, #22c55e);
  }

  /* 根目录拖放区域样式 */
  .root-drop-zone {
    margin-top: 12px;
    padding: 12px 8px;
    border: 2px dashed var(--editor-border, #e2e8f0);
    border-radius: 6px;
    background: var(--editor-panel-bg, #fafbfc);
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .root-drop-zone:hover {
    border-color: var(--editor-accent, #22c55e);
    background: rgba(34, 197, 94, 0.05);
  }

  .root-drop-zone.drag-over {
    border-color: var(--editor-accent, #22c55e);
    background: rgba(34, 197, 94, 0.12);
    box-shadow: 0 2px 12px rgba(34, 197, 94, 0.2);
  }

  .root-drop-zone-content {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--editor-muted, #94a3b8);
    font-size: 12px;
  }

  .root-drop-zone:hover .root-drop-zone-content {
    color: var(--editor-accent, #22c55e);
  }

  .root-drop-zone.drag-over .root-drop-zone-content {
    color: var(--editor-accent, #22c55e);
    font-weight: 500;
  }

  @keyframes drag-over-pulse {
    0% {
      border-color: var(--editor-accent, #22c55e);
    }
    50% {
      border-color: #4ade80;
    }
    100% {
      border-color: var(--editor-accent, #22c55e);
    }
  }
</style>
