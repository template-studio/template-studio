<template>
  <div class="property-panel" :class="{ 'property-panel-tree': mode === 'tree' }">
    <div class="panel-header">
      <n-text strong>属性</n-text>
    </div>
    <div class="property-content">
      <!-- 未选中状态 -->
      <div v-if="!component" class="no-selection">
        <n-empty description="请选择一个组件" size="small" />
      </div>

      <!-- 选中状态：显示属性编辑表单 -->
      <n-form v-else ref="formRef" :model="component.schema" label-placement="top" size="small">
        <n-form-item label="字段类型" path="type">
          <n-select
            :key="`type-${component?.id || 'none'}-${component?.type || 'unknown'}`"
            :value="component.type"
            :options="typeOptions"
            @update:value="handleTypeChange"
          />
        </n-form-item>

        <n-form-item
          label="字段名"
          path="fieldName"
          :rule="{ required: true, message: '请输入字段名' }"
        >
          <n-input
            :value="component.fieldName"
            placeholder="字段名（英文，如：username）"
            @update:value="(value) => handleFieldNameUpdate(value)"
          />
        </n-form-item>

        <n-form-item label="标题" path="title" :rule="{ required: true, message: '请输入标题' }">
          <n-input
            :value="component.schema.title"
            placeholder="用户友好的显示名称"
            @update:value="(value) => handleSchemaUpdate('title', value)"
          />
        </n-form-item>

        <n-form-item label="描述" path="description">
          <n-input
            :value="component.schema.description"
            type="textarea"
            placeholder="字段说明"
            :rows="2"
            @update:value="(value) => handleSchemaUpdate('description', value)"
          />
        </n-form-item>

        <n-form-item label="占位符" path="placeholder">
          <n-input
            :value="component.schema.placeholder"
            placeholder="输入框占位符文本"
            @update:value="(value) => handleSchemaUpdate('placeholder', value)"
          />
        </n-form-item>

        <n-form-item label="必填" path="required">
          <n-switch
            :value="component.schema.required"
            @update:value="(value) => handleSchemaUpdate('required', value)"
          />
        </n-form-item>

        <n-form-item label="默认值" path="default">
          <n-input
            :value="component.schema.default"
            placeholder="默认值"
            @update:value="(value) => handleSchemaUpdate('default', value)"
          />
        </n-form-item>

        <!-- 枚举类型特殊处理 -->
        <template v-if="component.type === 'enum'">
          <n-form-item label="枚举值" path="enum">
            <n-dynamic-tags
              :value="component.schema.enum"
              @update:value="(value) => handleSchemaUpdate('enum', value)"
            />
          </n-form-item>
        </template>

        <!-- 数字类型特殊处理 -->
        <template v-if="component.type === 'integer' || component.type === 'number'">
          <n-form-item label="最小值" path="minimum">
            <n-input-number
              :value="component.schema.minimum"
              placeholder="无限制"
              clearable
              @update:value="(value) => handleSchemaUpdate('minimum', value)"
            />
          </n-form-item>
          <n-form-item label="最大值" path="maximum">
            <n-input-number
              :value="component.schema.maximum"
              placeholder="无限制"
              clearable
              @update:value="(value) => handleSchemaUpdate('maximum', value)"
            />
          </n-form-item>
        </template>

        <!-- 数组类型特殊处理 -->
        <template v-if="component.type === 'array'">
          <n-form-item label="元素类型" path="itemType">
            <n-select
              :value="component.schema.items?.type || 'string'"
              :options="elementTypeOptions"
              @update:value="handleItemTypeUpdate"
            />
          </n-form-item>
        </template>
      </n-form>
    </div>
  </div>
</template>

<script setup>
  import { ref, nextTick } from 'vue';
  import {
    NText,
    NEmpty,
    NForm,
    NFormItem,
    NInput,
    NSwitch,
    NInputNumber,
    NDynamicTags,
    NSelect,
    NButton,
  } from 'naive-ui';

  /**
   * PropertyPanel 组件
   * 负责显示选中组件的属性编辑表单
   */

  // Props
  const props = defineProps({
    component: {
      type: Object,
      default: null,
    },
    mode: {
      type: String,
      default: 'design',
      validator: (value) => ['design', 'tree'].includes(value),
    },
  });

  // Emits
  const emit = defineEmits(['update:component']);

  // Refs
  const formRef = ref(null);
  let isUpdating = false; // 非响应式标志，防止重复更新
  let lastUpdateTime = 0; // 记录上次更新时间

  // 类型选项
  const typeOptions = [
    { label: '字符串 (string)', value: 'string' },
    { label: '整数 (integer)', value: 'integer' },
    { label: '数字 (number)', value: 'number' },
    { label: '布尔 (boolean)', value: 'boolean' },
    { label: '枚举 (enum)', value: 'enum' },
    { label: '密码 (secret)', value: 'secret' },
    { label: '对象 (object)', value: 'object' },
    { label: '数组 (array)', value: 'array' },
    { label: '对象数组 (object_arr)', value: 'object_arr' },
  ];

  // 常量
  const elementTypeOptions = [
    { label: '字符串', value: 'string' },
    { label: '整数', value: 'integer' },
    { label: '数字', value: 'number' },
    { label: '布尔', value: 'boolean' },
  ];

  // 事件处理
  const handleTypeChange = (newType) => {
    // 防止在冷却期内重复触发
    const timeSinceLastUpdate = Date.now() - lastUpdateTime;
    if (timeSinceLastUpdate < 1000) {
      return;
    }

    // 防止并发更新
    if (isUpdating) {
      return;
    }

    // 防止类型未改变时触发更新
    if (!props.component || props.component.type === newType) {
      return;
    }

    // 设置标志，阻止后续更新
    isUpdating = true;

    // 记录更新时间
    lastUpdateTime = Date.now();

    // 发出更新事件
    emit('update:component', {
      field: 'type',
      value: newType,
    });

    // 800ms 后重置标志，确保父组件完成所有更新
    setTimeout(() => {
      isUpdating = false;
    }, 800);
  };

  const handleFieldNameUpdate = (value) => {
    emit('update:component', {
      field: 'fieldName',
      value,
    });
  };

  const handleSchemaUpdate = (field, value) => {
    emit('update:component', {
      field: 'schema',
      value: { [field]: value },
    });
  };

  const handleItemTypeUpdate = (value) => {
    if (!props.component.schema.items) {
      props.component.schema.items = {};
    }
    emit('update:component', {
      field: 'schema',
      value: { items: { type: value } },
    });
  };
</script>

<style scoped>
  .property-panel {
    width: 320px;
    border-left: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    background: #fafafa;
    min-height: 0;
    overflow: hidden;
  }

  .property-panel-tree {
    flex: 1;
    width: auto;
    min-width: 0;
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

  .property-content {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
  }

  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
  }
</style>
