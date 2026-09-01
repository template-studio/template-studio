<template>
  <div class="component-library-panel">
    <div class="panel-header">
      <strong>组件</strong>
    </div>
    <div class="component-list">
      <div v-for="category in categories" :key="category.id" class="component-category">
        <div class="category-title">{{ category.name }}</div>
        <div class="category-items">
          <div
            v-for="component in category.components"
            :key="component.type"
            class="component-item"
            draggable="true"
            @dragstart="handleDragStart(component, $event)"
            @click="handleClick(component)"
          >
            <div class="component-icon" :style="{ color: component.color }">
              <component :is="component.icon" style="font-size: 24px" />
            </div>
            <div class="component-info">
              <div class="component-name">{{ component.name }}</div>
              <div class="component-desc">{{ component.description }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  /**
   * ComponentLibrary 组件
   * 负责显示可拖拽的组件列表
   */

  // Props
  const props = defineProps({
    categories: {
      type: Array,
      required: true,
      default: () => [],
    },
  });

  // Emits
  const emit = defineEmits(['add-component', 'drag-start']);

  // 事件处理
  const handleClick = (component) => {
    emit('add-component', component);
  };

  const handleDragStart = (component, event) => {
    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData('component', JSON.stringify(component));
    emit('drag-start', component, event);
  };
</script>

<style scoped>
  .component-library-panel {
    width: 240px;
    height: 100%;
    border-right: 1px solid var(--editor-border, #e0e0e0);
    display: flex;
    flex-direction: column;
    background: var(--editor-inset-bg, #fafafa);
    min-height: 0;
    overflow: hidden;
  }

  .panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-panel-bg, #fff);
    flex-shrink: 0;
  }

  .component-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 12px;
    min-height: 0;
  }

  .component-category {
    margin-bottom: 16px;
  }

  .component-category:last-child {
    margin-bottom: 0;
  }

  .category-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--editor-muted, #666);
    margin-bottom: 8px;
    padding: 0 4px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .category-items {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .component-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--editor-panel-bg, #fff);
    border: 1px solid var(--editor-border, #e0e0e0);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .component-item:hover {
    border-color: var(--editor-accent, #18a058);
    box-shadow: 0 2px 8px rgba(24, 160, 88, 0.15);
    transform: translateY(-1px);
  }

  .component-item:active {
    transform: translateY(0);
  }

  .component-icon {
    flex-shrink: 0;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--editor-inset-bg, #f5f5f5);
    border-radius: 8px;
  }

  .component-info {
    flex: 1;
    min-width: 0;
  }

  .component-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--editor-primary, #333);
    margin-bottom: 2px;
  }

  .component-desc {
    font-size: 12px;
    color: var(--editor-muted, #999);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
