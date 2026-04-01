<template>
  <div class="design-canvas">
    <div class="canvas-header">
      <n-text strong>变量</n-text>
    </div>
    <div class="canvas-area" @drop="handleDrop" @dragover.prevent @dragenter.prevent>
      <!-- 空状态 -->
      <div v-if="components.length === 0" class="empty-canvas">
        <n-empty description="暂无组件" size="large">
          <template #icon>
            <n-icon size="64" color="#ccc">
              <svg viewBox="0 0 24 24" width="64" height="64">
                <path fill="currentColor" d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
            </n-icon>
          </template>
          <template #extra>
            <n-text depth="3"> 从左侧拖拽组件到画布，或点击组件添加 </n-text>
          </template>
        </n-empty>
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
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="contextMenuX"
      :y="contextMenuY"
      :options="contextMenuOptions"
      :show="showContextMenu"
      :show-arrow="true"
      @clickoutside="showContextMenu = false"
      @select="handleContextMenuSelect"
    />
  </div>
</template>

<script setup>
  import { ref, computed, h } from 'vue';
  import { NText, NEmpty, NIcon, NDropdown } from 'naive-ui';
  import { EnterOutline } from '@vicons/ionicons5';
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

  // 右键菜单选项
  const contextMenuOptions = computed(() => {
    if (!contextMenuComponent.value) return [];

    const schema = contextMenuComponent.value.schema;
    const isComplex = schema.type === 'object' || schema.type === 'object_arr';

    const options = [
      {
        label: '删除',
        key: 'delete',
        icon: () => h('span', '🗑️'),
      },
    ];

    if (isComplex) {
      options.unshift({
        label: '进入内部编辑',
        key: 'enter',
        icon: () => h(NIcon, null, { default: () => h(EnterOutline) }),
      });
    }

    return options;
  });

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
    background: #fff;
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
</style>
