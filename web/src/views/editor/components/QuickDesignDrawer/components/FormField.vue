<template>
  <n-form-item :required="fieldSchema.required">
    <template #label>
      <n-tooltip v-if="fieldSchema.description" placement="right" trigger="hover">
        <template #trigger>
          <span>{{ fieldSchema.title || fieldName }}</span>
        </template>
        <span>{{ fieldSchema.description }}</span>
      </n-tooltip>
      <span v-else>{{ fieldSchema.title || fieldName }}</span>
    </template>

    <!-- 字符串类型 -->
    <template v-if="fieldSchema.type === 'string'">
      <n-input
        :value="modelValue"
        :placeholder="
          fieldSchema.placeholder ||
          fieldSchema.description ||
          `请输入${fieldSchema.title || fieldName}`
        "
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 整数类型 -->
    <template v-else-if="fieldSchema.type === 'integer'">
      <n-input-number
        :value="modelValue"
        :min="fieldSchema.minimum"
        :max="fieldSchema.maximum"
        :placeholder="fieldSchema.placeholder || undefined"
        style="width: 100%"
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 浮点数类型 -->
    <template v-else-if="fieldSchema.type === 'number'">
      <n-input-number
        :value="modelValue"
        :min="fieldSchema.minimum"
        :max="fieldSchema.maximum"
        :precision="2"
        :placeholder="fieldSchema.placeholder || undefined"
        style="width: 100%"
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 布尔类型 -->
    <template v-else-if="fieldSchema.type === 'boolean'">
      <n-switch :value="modelValue" @update:value="$emit('update:modelValue', $event)" />
    </template>

    <!-- 枚举类型 -->
    <template v-else-if="fieldSchema.type === 'enum'">
      <n-select
        :value="modelValue"
        :options="enumOptions"
        :placeholder="fieldSchema.placeholder || `请选择${fieldSchema.title || fieldName}`"
        style="width: 100%"
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 密码类型 -->
    <template v-else-if="fieldSchema.type === 'secret'">
      <n-input
        :value="modelValue"
        type="password"
        show-password-on="click"
        :placeholder="fieldSchema.placeholder || `请输入${fieldSchema.title || fieldName}`"
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 简单数组类型 -->
    <template v-else-if="fieldSchema.type === 'array'">
      <n-dynamic-tags
        :value="modelValue || []"
        @update:value="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 对象类型 -->
    <div v-else-if="fieldSchema.type === 'object'" class="nested-object">
      <n-collapse v-if="hasProperties">
        <n-collapse-item :title="fieldSchema.title || fieldName" name="nested">
          <template #header-extra>
            <n-tag size="small" type="info" style="margin-left: 8px"> 对象 </n-tag>
          </template>
          <div class="nested-fields">
            <FormField
              v-for="(subFieldSchema, subFieldName) in fieldSchema.properties"
              :key="subFieldName"
              :field-name="subFieldName"
              :field-schema="subFieldSchema"
              :model-value="modelValue?.[subFieldName]"
              @update:model-value="handleNestedChange(subFieldName, $event)"
            />
          </div>
        </n-collapse-item>
      </n-collapse>
      <n-empty v-else description="暂无子字段" size="small" />
    </div>

    <!-- 对象数组类型 -->
    <div v-else-if="fieldSchema.type === 'object_arr'" class="object-array">
      <div v-if="!modelValue || modelValue.length === 0" class="empty-array">
        <n-empty description="暂无数据" size="small">
          <template #extra>
            <n-button size="small" @click="handleAddObject"> 添加对象 </n-button>
          </template>
        </n-empty>
      </div>
      <div v-else class="array-items">
        <div v-for="(item, index) in modelValue" :key="index" class="array-item">
          <div class="item-header">
            <n-text strong>项目 {{ index + 1 }}</n-text>
            <n-button size="tiny" type="error" quaternary @click="handleRemoveObject(index)">
              删除
            </n-button>
          </div>
          <n-collapse>
            <n-collapse-item name="item">
              <div class="nested-fields">
                <FormField
                  v-for="(subFieldSchema, subFieldName) in fieldSchema.items.properties"
                  :key="`${index}-${subFieldName}`"
                  :field-name="subFieldName"
                  :field-schema="subFieldSchema"
                  :model-value="item?.[subFieldName]"
                  @update:model-value="handleArrayItemChange(index, subFieldName, $event)"
                />
              </div>
            </n-collapse-item>
          </n-collapse>
        </div>
        <n-button size="small" dashed block @click="handleAddObject"> 添加对象 </n-button>
      </div>
    </div>
  </n-form-item>
</template>

<script setup>
  import { computed } from 'vue';
  import {
    NFormItem,
    NInput,
    NInputNumber,
    NSwitch,
    NSelect,
    NDynamicTags,
    NCollapse,
    NCollapseItem,
    NButton,
    NEmpty,
    NText,
    NTag,
    NTooltip,
  } from 'naive-ui';

  const props = defineProps({
    fieldName: {
      type: String,
      required: true,
    },
    fieldSchema: {
      type: Object,
      required: true,
    },
    modelValue: {
      type: [String, Number, Boolean, Array, Object],
      default: undefined,
    },
  });

  const emit = defineEmits(['update:modelValue']);

  // 枚举选项
  const enumOptions = computed(() => {
    if (props.fieldSchema.type !== 'enum') return [];

    const enums = props.fieldSchema.enum || [];
    const enumNames = props.fieldSchema.enumNames || enums;

    return enums.map((value, index) => ({
      label: enumNames[index] || value,
      value,
    }));
  });

  // 对象是否有属性
  const hasProperties = computed(() => {
    return props.fieldSchema.properties && Object.keys(props.fieldSchema.properties).length > 0;
  });

  // 处理嵌套对象字段变化
  const handleNestedChange = (subFieldName, value) => {
    const newValue = { ...(props.modelValue || {}) };
    newValue[subFieldName] = value;
    emit('update:modelValue', newValue);
  };

  // 处理对象数组项字段变化
  const handleArrayItemChange = (index, subFieldName, value) => {
    const newArray = [...(props.modelValue || [])];
    if (!newArray[index]) {
      newArray[index] = {};
    }
    newArray[index][subFieldName] = value;
    emit('update:modelValue', newArray);
  };

  // 添加对象到数组
  const handleAddObject = () => {
    const newArray = [...(props.modelValue || [])];

    // 创建空对象
    const newObject = {};
    if (props.fieldSchema.items?.properties) {
      Object.entries(props.fieldSchema.items.properties).forEach(
        ([subFieldName, subFieldSchema]) => {
          if (subFieldSchema.default !== undefined) {
            newObject[subFieldName] = JSON.parse(JSON.stringify(subFieldSchema.default));
          } else {
            // 根据类型设置默认值
            switch (subFieldSchema.type) {
              case 'string':
              case 'enum':
              case 'secret':
                newObject[subFieldName] = '';
                break;
              case 'integer':
                newObject[subFieldName] = 0;
                break;
              case 'number':
                newObject[subFieldName] = 0.0;
                break;
              case 'boolean':
                newObject[subFieldName] = false;
                break;
              default:
                newObject[subFieldName] = null;
            }
          }
        }
      );
    }

    newArray.push(newObject);
    emit('update:modelValue', newArray);
  };

  // 从数组移除对象
  const handleRemoveObject = (index) => {
    const newArray = [...(props.modelValue || [])];
    newArray.splice(index, 1);
    emit('update:modelValue', newArray);
  };
</script>

<style scoped>
  .nested-object {
    width: 100%;
  }

  .nested-fields {
    padding: 8px 0;
    background: #fafafa;
    border-radius: 4px;
  }

  .nested-fields :deep(.n-form-item) {
    padding: 0 16px;
  }

  .object-array {
    width: 100%;
  }

  .empty-array {
    padding: 16px;
    background: #fafafa;
    border-radius: 4px;
    text-align: center;
  }

  .array-items {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .array-item {
    padding: 12px;
    background: #fafafa;
    border: 1px solid #e8e8e8;
    border-radius: 4px;
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
</style>
