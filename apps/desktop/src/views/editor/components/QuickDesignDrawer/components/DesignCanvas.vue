<template>
  <div class="design-canvas">
    <div class="canvas-header">
      <strong>变量</strong>
    </div>
    <div class="canvas-area" @drop="handleDrop" @dragover.prevent @dragenter.prevent>
      <!-- 空状态 -->
      <div v-if="components.length === 0" class="empty-canvas">
        <a-empty description="暂无组件">
          <template #image>
            <svg viewBox="0 0 24 24" width="64" height="64" style="color: #ccc">
              <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
            </svg>
          </template>
          <span style="color: #999"> 从左侧拖拽组件到画布，或点击组件添加 </span>
        </a-empty>
      </div>

      <!-- 组件列表 -->
      <div v-else class="canvas-components">
        <CanvasComponent
          v-for="(component, index) in components"
          :key="component.id"
          :component="component"
          :is-selected="selectedComponentId === component.id"
          :is-expanded="isComponentExpanded(component.id)"
          :is-container-component="isContainerComponent"
          :get-children-count="getChildrenCount"
          @select="handleSelectComponent"
          @remove="handleRemoveComponent"
          @toggle-expand="handleToggleExpand"
          @select-child="handleSelectChild"
          @remove-child="handleRemoveChild"
          @contextmenu="handleComponentContextMenu"
          @contextmenu-child="handleChildContextMenu"
        />
      </div>
    </div>

    <!-- 右键菜单 -->
    <div
      v-if="showContextMenu"
      class="context-menu-overlay"
      @click="showContextMenu = false"
      @contextmenu.prevent="showContextMenu = false"
    >
      <div
        class="context-menu"
        :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
        @click.stop
      >
        <div
          v-if="contextMenuComponent?.schema?.type === 'object' || contextMenuComponent?.schema?.type === 'object_arr'"
          class="context-menu-item"
          @click="handleContextMenuSelect('enter')"
        >
          <EnterOutline style="font-size: 14px; margin-right: 8px" />
          进入内部编辑
        </div>
        <div class="context-menu-item context-menu-item-danger" @click="handleContextMenuSelect('delete')">
          🗑️ 删除
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, computed } from 'vue';
  import { EnterOutline } from '@/icons/ionicons5';
  import CanvasComponent from './canvas/CanvasComponent.vue';

  /**
   * DesignCanvas 组件
   * 负责显示设计画布区域和组件列表
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
    expandedComponents: {
      type: Set,
      default: () => new Set(),
    },
    isContainerComponent: {
      type: Function,
      default: () => false,
    },
    getChildrenCount: {
      type: Function,
      default: () => 0,
    },
  });

  // Emits
  const emit = defineEmits([
    'drop',
    'select-component',
    'remove-component',
    'toggle-expand',
    'select-child',
    'remove-child',
    'enter-component',
  ]);

  // 右键菜单状态
  const showContextMenu = ref(false);
  const contextMenuX = ref(0);
  const contextMenuY = ref(0);
  const contextMenuComponent = ref(null);

  // 方法
  const isComponentExpanded = (componentId) => {
    return props.expandedComponents.has(componentId);
  };

  const handleDrop = (event) => {
    emit('drop', event);
  };

  const handleSelectComponent = (component) => {
    emit('select-component', component);
  };

  const handleRemoveComponent = (component) => {
    const index = props.components.findIndex((c) => c.id === component.id);
    if (index > -1) {
      emit('remove-component', index);
    }
  };

  const handleToggleExpand = (componentId) => {
    emit('toggle-expand', componentId);
  };

  const handleSelectChild = (child) => {
    emit('select-child', child);
  };

  const handleRemoveChild = ({ container, childId }) => {
    emit('remove-child', { container, childId });
  };

  const handleComponentContextMenu = ({ event, component }) => {
    contextMenuComponent.value = component;
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenu.value = true;
  };

  const handleChildContextMenu = ({ event, child }) => {
    contextMenuComponent.value = child;
    contextMenuX.value = event.clientX;
    contextMenuY.value = event.clientY;
    showContextMenu.value = true;
  };

  const handleContextMenuSelect = (key) => {
    showContextMenu.value = false;

    if (key === 'enter' && contextMenuComponent.value) {
      emit('enter-component', contextMenuComponent.value);
    } else if (key === 'delete' && contextMenuComponent.value) {
      handleRemoveComponent(contextMenuComponent.value);
    }

    contextMenuComponent.value = null;
  };
</script>

<style scoped>
  .design-canvas {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .canvas-header {
    padding: 16px;
    border-bottom: 1px solid #e0e0e0;
    background: var(--editor-panel-bg, #fff);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .canvas-area {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    background: #f5f5f5;
    min-height: 0;
  }

  .empty-canvas {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
  }

  .canvas-components {
    display: flex;
    flex-direction: column;
  }

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
    border: 1px solid #e8e8e8;
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
    color: #333;
    transition: background 0.2s;
  }

  .context-menu-item:hover {
    background: #f5f5f5;
  }

  .context-menu-item-danger {
    color: #ff4d4f;
  }

  .context-menu-item-danger:hover {
    background: #fff1f0;
  }
</style>
