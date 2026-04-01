<template>
  <div class="canvas-component-wrapper">
    <!-- 组件卡片 -->
    <div
      class="canvas-component"
      :data-component-id="component.id"
      :class="{
        'is-selected': isSelected,
        'is-container': isContainer,
        'is-expanded': isExpanded,
      }"
      @click="$emit('select', component)"
      @contextmenu="handleContextMenu"
    >
      <div class="component-header">
        <div class="component-title">
          <!-- 展开/折叠按钮（所有容器组件都显示） -->
          <n-icon
            v-if="isContainer"
            size="14"
            style="margin-right: 4px; cursor: pointer; transition: transform 0.2s"
            :style="{ transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
            @click.stop="$emit('toggle-expand', component.id)"
          >
            <ChevronForwardOutline />
          </n-icon>

          <n-icon size="16" :color="displayInfo.color">
            <TextOutline />
          </n-icon>
          <span>{{ component.schema.title || component.fieldName }}</span>
          <n-tag size="small" :type="tagType" style="margin-left: 8px">
            {{ displayInfo.label }}
          </n-tag>
        </div>
        <div class="component-actions">
          <n-button size="tiny" quaternary type="error" @click.stop="$emit('remove', component)">
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
          </n-button>
        </div>
      </div>

      <div class="component-preview">
        <n-text depth="3" style="font-size: 12px"> 字段名: {{ component.fieldName }} </n-text>
        <n-text v-if="component.schema.required" type="error" style="font-size: 12px">
          必填
        </n-text>
        <!-- 容器组件：显示子字段数量和标识 -->
        <n-tag v-if="isContainer" size="small" type="info" style="margin-left: 8px">
          {{ childrenCount }} 个字段
        </n-tag>
        <!-- 如果是复杂组件且有子字段，添加特殊标识 -->
        <n-tag v-if="hasNestedFields" size="small" type="warning" style="margin-left: 4px">
          含嵌套
        </n-tag>
      </div>
    </div>

    <!-- 展开的容器子字段 -->
    <ContainerChildren
      v-if="isContainer && isExpanded"
      :component="component"
      :is-nested="component.isNested"
      @select-child="$emit('select-child', $event)"
      @remove-child="$emit('remove-child', { container: component, childId: $event })"
      @contextmenu="$emit('contextmenu-child', $event)"
    />
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { NIcon, NButton, NTag, NText } from 'naive-ui';
  import { ChevronForwardOutline, TrashOutline, TextOutline } from '@vicons/ionicons5';
  import { getComponentDisplayInfo, getTypeTagType } from '../../utils/componentTemplates';
  import ContainerChildren from './ContainerChildren.vue';

  /**
   * CanvasComponent 组件
   * 负责显示设计画布中的单个组件卡片
   */

  // Props
  const props = defineProps({
    component: {
      type: Object,
      required: true,
    },
    isSelected: {
      type: Boolean,
      default: false,
    },
    isExpanded: {
      type: Boolean,
      default: false,
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
    'select',
    'remove',
    'toggle-expand',
    'select-child',
    'remove-child',
    'contextmenu',
    'contextmenu-child',
  ]);

  // 右键菜单处理
  const handleContextMenu = (event) => {
    event.preventDefault();
    emit('contextmenu', {
      event,
      component: props.component,
    });
  };

  // 计算属性
  const isContainer = computed(() => {
    return props.isContainerComponent(props.component);
  });

  const displayInfo = computed(() => {
    return getComponentDisplayInfo(props.component.type);
  });

  const tagType = computed(() => {
    return getTypeTagType(props.component.type);
  });

  const childrenCount = computed(() => {
    return props.getChildrenCount(props.component);
  });

  // 检查是否含有嵌套字段（复杂组件内部的复杂组件）
  const hasNestedFields = computed(() => {
    const schema = props.component.schema;
    if (!schema) return false;

    // 检查 object 类型的 properties 是否含有复杂类型
    if (schema.type === 'object' && schema.properties) {
      return Object.values(schema.properties).some((field) => {
        const fieldType = field.type || field.schema?.type;
        return fieldType === 'object' || fieldType === 'object_arr';
      });
    }

    // 检查 object_arr 类型的 items.properties 是否含有复杂类型
    if (schema.type === 'object_arr' && schema.items?.properties) {
      return Object.values(schema.items.properties).some((field) => {
        const fieldType = field.type || field.schema?.type;
        return fieldType === 'object' || fieldType === 'object_arr';
      });
    }

    return false;
  });
</script>

<style scoped>
  .canvas-component-wrapper {
    margin-bottom: 12px;
  }

  .canvas-component {
    padding: 12px;
    background: #fff;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .canvas-component:hover {
    border-color: #18a058;
    box-shadow: 0 2px 8px rgba(24, 160, 88, 0.1);
  }

  .canvas-component.is-selected {
    border-color: #18a058;
    background: #f0f9f4;
  }

  .canvas-component.is-container {
    border-radius: 8px 8px 0 0;
  }

  .component-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .component-title {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    font-size: 14px;
    font-weight: 500;
  }

  .component-actions {
    flex-shrink: 0;
  }

  .component-preview {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 20px;
  }
</style>
