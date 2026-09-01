<template>
  <div class="property-panel" :class="{ 'property-panel-tree': mode === 'tree' }" :style="{ width: panelWidth + 'px' }">
    <div class="col-resize-handle" @mousedown="startResize"></div>
    <div class="panel-header">
      <strong>属性</strong>
    </div>
    <div class="property-content">
      <!-- 未选中状态 -->
      <div v-if="!component" class="no-selection">
        <a-empty description="请选择一个组件" :image-style="{ height: '40px' }" />
      </div>

      <!-- 选中状态：显示属性编辑表单 -->
      <a-form v-else ref="formRef" :model="component.schema" layout="vertical" size="small">
        <a-form-item label="字段类型" name="type">
          <a-select
            :key="`type-${component?.id || 'none'}-${component?.type || 'unknown'}`"
            :value="component.type"
            :options="typeOptions"
            @change="handleTypeChange"
          />
        </a-form-item>

        <a-form-item
          label="字段名"
          name="fieldName"
          :rules="[{ required: true, message: '请输入字段名' }]"
        >
          <a-input
            :value="component.fieldName"
            placeholder="字段名（英文，如：username）"
            @change="(e) => handleFieldNameUpdate(e.target.value)"
          />
        </a-form-item>

        <a-form-item label="标题" name="title" :rules="[{ required: true, message: '请输入标题' }]">
          <a-input
            :value="component.schema.title"
            placeholder="用户友好的显示名称"
            @change="(e) => handleSchemaUpdate('title', e.target.value)"
          />
        </a-form-item>

        <a-form-item label="描述" name="description">
          <a-textarea
            :value="component.schema.description"
            placeholder="字段说明"
            :rows="2"
            @change="(e) => handleSchemaUpdate('description', e.target.value)"
          />
        </a-form-item>

        <a-form-item label="占位符" name="placeholder">
          <a-input
            :value="component.schema.placeholder"
            placeholder="输入框占位符文本"
            @change="(e) => handleSchemaUpdate('placeholder', e.target.value)"
          />
        </a-form-item>

        <a-form-item label="必填" name="required">
          <a-switch
            :checked="component.schema.required"
            @change="(checked) => handleSchemaUpdate('required', checked)"
          />
        </a-form-item>

        <a-form-item label="默认值" name="default">
          <a-input
            :value="component.schema.default"
            placeholder="默认值"
            @change="(e) => handleSchemaUpdate('default', e.target.value)"
          />
        </a-form-item>

        <!-- 枚举类型特殊处理 -->
        <template v-if="component.type === 'enum'">
          <a-form-item label="枚举值" name="enum">
            <a-select
              mode="tags"
              :value="component.schema.enum || []"
              placeholder="输入枚举值后回车"
              @change="(value) => handleSchemaUpdate('enum', value)"
            />
          </a-form-item>
        </template>

        <!-- 数字类型特殊处理 -->
        <template v-if="component.type === 'integer' || component.type === 'number'">
          <a-form-item label="最小值" name="minimum">
            <a-input-number
              :value="component.schema.minimum"
              placeholder="无限制"
              style="width: 100%"
              @change="(value) => handleSchemaUpdate('minimum', value)"
            />
          </a-form-item>
          <a-form-item label="最大值" name="maximum">
            <a-input-number
              :value="component.schema.maximum"
              placeholder="无限制"
              style="width: 100%"
              @change="(value) => handleSchemaUpdate('maximum', value)"
            />
          </a-form-item>
        </template>

        <!-- 数组类型特殊处理 -->
        <template v-if="component.type === 'array'">
          <a-form-item label="元素类型" name="itemType">
            <a-select
              :value="component.schema.items?.type || 'string'"
              :options="elementTypeOptions"
              @change="handleItemTypeUpdate"
            />
          </a-form-item>
        </template>
      </a-form>
    </div>
  </div>
</template>

<script setup>
// 面板宽度（左缘拖拽调节，范围 200–420）
const panelWidth = ref(300)
const startResize = (e) => {
  e.preventDefault()
  const startX = e.clientX
  const startW = panelWidth.value
  const onMove = (ev) => {
    panelWidth.value = Math.min(440, Math.max(180, startW - (ev.clientX - startX)))
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

  import { ref, nextTick } from 'vue';

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
  .col-resize-handle {
    position: absolute;
    left: 0;
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

  .property-panel {
    flex-shrink: 0;
    position: relative;
    border-left: 1px solid var(--editor-border, #e0e0e0);
    border-left: 1px solid var(--editor-border, #e0e0e0);
    display: flex;
    flex-direction: column;
    background: var(--editor-inset-bg, #fafafa);
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
    border-bottom: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-panel-bg, #fff);
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
