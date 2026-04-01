<template>
  <div class="tree-node" :style="{ paddingLeft: level * 20 + 'px' }">
    <div class="node-content">
      <div class="node-expand" @click="toggleExpand" v-if="hasChildren">
        <n-icon size="14" :class="{ expanded: expanded }">
          <ChevronForwardOutline />
        </n-icon>
      </div>
      <div class="node-info">
        <div class="node-main">
          <span class="node-name">{{ node.label }}</span>
          <n-tag size="small" :type="getTypeColor(node.nodeType)" style="margin-left: 8px">
            {{ getTypeLabel(node.nodeType) }}
          </n-tag>
          <span v-if="node.description" class="node-desc">- {{ node.description }}</span>
        </div>
        <div class="node-value" v-if="!hasChildren">
          <span class="value-text">{{ formatValue(node.value) }}</span>
        </div>
      </div>
      <div class="node-actions" v-if="!readonly">
        <n-button size="tiny" type="primary" secondary @click.stop="$emit('add-child', node)">
          <template #icon>
            <n-icon size="12"><AddOutline /></n-icon>
          </template>
        </n-button>
        <n-button
          size="tiny"
          type="info"
          secondary
          @click.stop="$emit('edit-node', node)"
          style="margin-left: 4px"
        >
          <template #icon>
            <n-icon size="12"><CreateOutline /></n-icon>
          </template>
        </n-button>
        <n-button
          size="tiny"
          type="error"
          secondary
          @click.stop="$emit('delete-node', node)"
          style="margin-left: 4px"
        >
          <template #icon>
            <n-icon size="12"><TrashOutline /></n-icon>
          </template>
        </n-button>
      </div>
    </div>

    <!-- 子节点 -->
    <div v-if="hasChildren && expanded" class="node-children">
      <TreeNode
        v-for="child in node.children"
        :key="child.key"
        :node="child"
        :level="level + 1"
        :readonly="readonly"
        @add-child="$emit('add-child', $event)"
        @edit-node="$emit('edit-node', $event)"
        @delete-node="$emit('delete-node', $event)"
      />
    </div>
  </div>
</template>

<script setup>
  import { ref, computed } from 'vue';
  import { NButton, NIcon, NTag } from 'naive-ui';
  import {
    AddOutline,
    CreateOutline,
    TrashOutline,
    ChevronForwardOutline,
  } from '@vicons/ionicons5';

  const props = defineProps({
    node: {
      type: Object,
      required: true,
    },
    level: {
      type: Number,
      default: 0,
    },
    readonly: {
      type: Boolean,
      default: false,
    },
  });

  defineEmits(['add-child', 'edit-node', 'delete-node']);

  const expanded = ref(true);

  const hasChildren = computed(() => {
    return props.node.children && props.node.children.length > 0;
  });

  const toggleExpand = () => {
    expanded.value = !expanded.value;
  };

  const getTypeColor = (type) => {
    const colors = {
      string: 'success',
      number: 'info',
      boolean: 'warning',
      object: 'default',
      array: 'error',
    };
    return colors[type] || 'default';
  };

  const getTypeLabel = (type) => {
    const labels = {
      string: 'String',
      number: 'Number',
      boolean: 'Boolean',
      object: 'Object',
      array: 'Array',
    };
    return labels[type] || type;
  };

  const formatValue = (value) => {
    if (value === null || value === undefined) {
      return 'null';
    }
    if (typeof value === 'string') {
      return `"${value}"`;
    }
    if (typeof value === 'boolean') {
      return value ? 'true' : 'false';
    }
    return String(value);
  };
</script>

<style scoped>
  .tree-node {
    user-select: none;
  }

  .node-content {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background-color 0.2s;
    min-height: 32px;
  }

  .node-content:hover {
    background-color: #f5f5f5;
  }

  .node-content:hover .node-actions {
    opacity: 1;
  }

  .node-expand {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    margin-right: 4px;
  }

  .node-expand .n-icon {
    transition: transform 0.2s;
  }

  .node-expand .n-icon.expanded {
    transform: rotate(90deg);
  }

  .node-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .node-main {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .node-name {
    font-weight: 500;
    color: #333;
    white-space: nowrap;
  }

  .node-desc {
    font-size: 12px;
    color: #666;
    margin-left: 8px;
    font-style: italic;
  }

  .node-value {
    margin-top: 2px;
  }

  .value-text {
    font-size: 12px;
    color: #999;
    font-family: 'Courier New', monospace;
  }

  .node-actions {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.2s;
    margin-left: 8px;
  }

  .node-children {
    border-left: 1px solid #e8e8e8;
    margin-left: 8px;
  }
</style>
