<template>
  <a-form-item :required="fieldSchema.required">
    <template #label>
      <a-tooltip v-if="fieldSchema.description" placement="right">
        <template #title>
          <span>{{ fieldSchema.description }}</span>
        </template>
        <span>{{ fieldSchema.title || fieldName }}</span>
      </a-tooltip>
      <span v-else>{{ fieldSchema.title || fieldName }}</span>
    </template>

    <!-- 字符串类型 -->
    <template v-if="fieldSchema.type === 'string'">
      <a-input
        :value="modelValue"
        :placeholder="
          fieldSchema.placeholder ||
          fieldSchema.description ||
          `请输入${fieldSchema.title || fieldName}`
        "
        @change="$emit('update:modelValue', $event.target.value)"
      />
    </template>

    <!-- 整数类型 -->
    <template v-else-if="fieldSchema.type === 'integer'">
      <a-input-number
        :value="modelValue"
        :min="fieldSchema.minimum"
        :max="fieldSchema.maximum"
        :placeholder="fieldSchema.placeholder || undefined"
        style="width: 100%"
        @change="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 浮点数类型 -->
    <template v-else-if="fieldSchema.type === 'number'">
      <a-input-number
        :value="modelValue"
        :min="fieldSchema.minimum"
        :max="fieldSchema.maximum"
        :precision="2"
        :placeholder="fieldSchema.placeholder || undefined"
        style="width: 100%"
        @change="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 布尔类型 -->
    <template v-else-if="fieldSchema.type === 'boolean'">
      <a-switch :checked="modelValue" @change="$emit('update:modelValue', $event)" />
    </template>

    <!-- 枚举类型 -->
    <template v-else-if="fieldSchema.type === 'enum'">
      <a-select
        :value="modelValue"
        :options="enumOptions"
        :placeholder="fieldSchema.placeholder || `请选择${fieldSchema.title || fieldName}`"
        style="width: 100%"
        @change="$emit('update:modelValue', $event)"
      />
    </template>

    <!-- 密码类型 -->
    <template v-else-if="fieldSchema.type === 'secret'">
      <a-input-password
        :value="modelValue"
        :placeholder="fieldSchema.placeholder || `请输入${fieldSchema.title || fieldName}`"
        @change="$emit('update:modelValue', $event.target.value)"
      />
    </template>

    <!-- 简单数组类型 -->
    <template v-else-if="fieldSchema.type === 'array'">
      <div class="dynamic-tags">
        <a-tag
          v-for="(tag, index) in (modelValue || [])"
          :key="index"
          closable
          @close="handleRemoveTag(index)"
        >
          {{ tag }}
        </a-tag>
        <a-input
          v-if="showTagInput"
          ref="tagInputRef"
          size="small"
          style="width: 100px"
          @blur="handleTagInputConfirm"
          @keyup.enter="handleTagInputConfirm"
          v-model:value="newTagValue"
        />
        <a-tag v-else @click="showTagInput = true" style="border-style: dashed; cursor: pointer">
          + 添加
        </a-tag>
      </div>
    </template>

    <!-- 对象类型 -->
    <div v-else-if="fieldSchema.type === 'object'" class="nested-object">
      <a-collapse v-if="hasProperties">
        <a-collapse-panel :header="fieldSchema.title || fieldName" key="nested">
          <template #extra>
            <a-tag color="blue" style="margin-left: 8px"> 对象 </a-tag>
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
        </a-collapse-panel>
      </a-collapse>
      <a-empty v-else description="暂无子字段" />
    </div>

    <!-- 对象数组类型 -->
    <div v-else-if="fieldSchema.type === 'object_arr'" class="object-array">
      <div v-if="!modelValue || modelValue.length === 0" class="empty-array">
        <a-empty description="暂无数据">
          <a-button size="small" @click="handleAddObject"> 添加对象 </a-button>
        </a-empty>
      </div>
      <div v-else class="array-items">
        <div v-for="(item, index) in modelValue" :key="index" class="array-item">
          <div class="item-header">
            <strong>项目 {{ index + 1 }}</strong>
            <a-button size="small" danger @click="handleRemoveObject(index)">
              删除
            </a-button>
          </div>
          <a-collapse>
            <a-collapse-panel key="item">
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
            </a-collapse-panel>
          </a-collapse>
        </div>
        <a-button size="small" block @click="handleAddObject" style="border-style: dashed">
          添加对象
        </a-button>
      </div>
    </div>
  </a-form-item>
</template>

<script setup>
  import { computed, ref, nextTick } from 'vue';

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

  // 动态标签相关
  const showTagInput = ref(false);
  const newTagValue = ref('');
  const tagInputRef = ref(null);

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

  // 动态标签操作
  const handleRemoveTag = (index) => {
    const newTags = [...(props.modelValue || [])];
    newTags.splice(index, 1);
    emit('update:modelValue', newTags);
  };

  const handleTagInputConfirm = () => {
    if (newTagValue.value) {
      const newTags = [...(props.modelValue || []), newTagValue.value];
      emit('update:modelValue', newTags);
    }
    showTagInput.value = false;
    newTagValue.value = '';
  };
</script>

<style scoped>
  .nested-object {
    width: 100%;
  }

  .nested-fields {
    padding: 8px 0;
    background: var(--editor-inset-bg, #fafafa);
    border-radius: 4px;
  }

  .nested-fields :deep(.ant-form-item) {
    padding: 0 16px;
  }

  .object-array {
    width: 100%;
  }

  .empty-array {
    padding: 16px;
    background: var(--editor-inset-bg, #fafafa);
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
    background: var(--editor-inset-bg, #fafafa);
    border: 1px solid var(--editor-border, #e8e8e8);
    border-radius: 4px;
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .dynamic-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
  }
</style>
