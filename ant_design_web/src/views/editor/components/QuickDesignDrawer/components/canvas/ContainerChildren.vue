<template>
  <div class="container-children">
    <div class="children-header">
      <span style="color: #999; font-size: 11px">
        {{ isNested ? '子字段' : '子字段（右键复杂组件可进入内部编辑）' }}
      </span>
    </div>

    <!-- 子字段列表 -->
    <div v-if="hasChildren" class="children-list">
      <div
        v-for="child in children"
        :key="child.id"
        class="child-component"
        @click="$emit('select-child', child)"
        @contextmenu="(event) => handleContextMenu(event, child)"
      >
        <div class="child-header">
          <div class="child-title">
            <span :style="{ fontSize: '14px', color: getComponentDisplayInfo(child.type || child.schema?.type).color }">
              <TextOutline />
            </span>
            <span>{{ child.schema.title || child.fieldName }}</span>
            <a-tag :color="getTagColor(child.type)" style="margin-left: 4px">
              {{ getComponentDisplayInfo(child.type || child.schema?.type).label }}
            </a-tag>
            <!-- 如果是嵌套的复杂组件，显示标识 -->
            <a-tag
              v-if="isNested && isComplexChild(child)"
              color="orange"
              style="margin-left: 4px"
            >
              含嵌套
            </a-tag>
          </div>
          <a-button
            size="small"
            danger
            @click.stop="$emit('remove-child', child.id)"
          >
            <template #icon><TrashOutline /></template>
          </a-button>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="children-empty">
      <a-empty
        :description="isNested ? '暂无子字段，从左侧拖拽添加' : '暂无子字段，右键父组件进入添加'"
      />
    </div>
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { TrashOutline, TextOutline } from '@/icons/ionicons5';
  import { getComponentDisplayInfo } from '../../utils/componentTemplates';

  /**
   * ContainerChildren 组件
   * 负责显示容器组件的子字段列表
   */

  // Props
  const props = defineProps({
    component: {
      type: Object,
      required: true,
    },
    isNested: {
      type: Boolean,
      default: false,
    },
  });

  // Emits
  const emit = defineEmits(['select-child', 'remove-child', 'enter-child', 'contextmenu']);

  // 计算属性 - 获取子字段列表
  const children = computed(() => {
    const schema = props.component.schema;

    // object 类型：从 schema.properties 获取
    if (schema.type === 'object' && schema.properties) {
      return Object.entries(schema.properties).map(([fieldName, fieldSchema]) => ({
        id: `${props.component.id}_${fieldName}`,
        fieldName,
        type: fieldSchema.type || 'string',
        schema: fieldSchema,
      }));
    }

    // object_arr 类型：从 schema.items.properties 获取
    if (schema.type === 'object_arr' && schema.items?.properties) {
      return Object.entries(schema.items.properties).map(([fieldName, fieldSchema]) => ({
        id: `${props.component.id}_items_${fieldName}`,
        fieldName,
        type: fieldSchema.type || 'string',
        schema: fieldSchema,
      }));
    }

    return [];
  });

  // 计算属性
  const hasChildren = computed(() => {
    return children.value && children.value.length > 0;
  });

  // 检查子组件是否是复杂类型
  const isComplexChild = (child) => {
    const type = child.type || child.schema?.type;
    return type === 'object' || type === 'object_arr';
  };

  // 获取标签颜色
  const getTagColor = (type) => {
    const colorMap = {
      string: 'blue',
      integer: 'green',
      number: 'cyan',
      boolean: 'orange',
      enum: 'purple',
      secret: 'red',
      object: 'magenta',
      array: 'default',
      object_arr: 'geekblue',
    };
    return colorMap[type] || 'default';
  };

  // 右键菜单处理
  const handleContextMenu = (event, child) => {
    event.preventDefault();
    emit('contextmenu', {
      event,
      child,
    });
  };
</script>

<style scoped>
  .container-children {
    background: #f9f9f9;
    border: 2px solid #e0e0e0;
    border-top: none;
    border-radius: 0 0 8px 8px;
    padding: 12px;
  }

  .children-header {
    margin-bottom: 12px;
  }

  .children-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .child-component {
    padding: 8px 12px;
    background: #fff;
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .child-component:hover {
    border-color: #18a058;
    box-shadow: 0 1px 4px rgba(24, 160, 88, 0.1);
  }

  .child-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .child-title {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
  }

  .children-empty {
    padding: 16px;
    text-align: center;
  }
</style>
