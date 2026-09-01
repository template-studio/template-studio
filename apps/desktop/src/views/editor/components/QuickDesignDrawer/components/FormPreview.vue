<template>
  <div class="form-preview">
    <a-form
      ref="formRef"
      :model="formData"
      layout="horizontal"
      :label-col="{ style: { width: '120px' } }"
      size="middle"
    >
      <template v-if="isEmptySchema">
        <a-empty description="暂无表单字段" />
      </template>

      <template v-else>
        <FormField
          v-for="(fieldSchema, fieldName) in schema"
          :key="fieldName"
          :field-name="fieldName"
          :field-schema="fieldSchema"
          :model-value="formData[fieldName]"
          @update:model-value="handleFieldChange(fieldName, $event)"
        />
      </template>
    </a-form>

    <a-divider v-if="!isEmptySchema" style="margin: 16px 0" />

    <div v-if="!isEmptySchema" class="preview-actions">
      <a-space>
        <a-button size="small" @click="handleReset"> 重置 </a-button>
        <a-button size="small" type="primary" @click="handleShowData"> 查看数据 </a-button>
      </a-space>
    </div>

    <!-- 数据预览对话框 -->
    <a-modal v-model:open="showDataModal" title="表单数据" :width="600" :footer="null">
      <pre style="background: #f5f5f5; padding: 12px; border-radius: 4px; overflow: auto; max-height: 400px"><code>{{ JSON.stringify(formData, null, 2) }}</code></pre>
      <template #footer>
        <a-space style="justify-content: flex-end">
          <a-button @click="handleCopyData">复制</a-button>
          <a-button type="primary" @click="showDataModal = false">关闭</a-button>
        </a-space>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
  import { ref, computed, watch } from 'vue';
  import { message } from 'ant-design-vue';
  import FormField from './FormField.vue';

  const props = defineProps({
    schema: {
      type: Object,
      default: () => ({}),
    },
  });

  const emit = defineEmits(['change']);

  // 表单数据
  const formData = ref({});

  const showDataModal = ref(false);

  // 是否为空 Schema
  const isEmptySchema = computed(() => {
    return !props.schema || Object.keys(props.schema).length === 0;
  });

  // 初始化表单数据（根据 default 值）
  const initFormData = () => {
    const data = {};

    const initField = (fieldName, fieldSchema) => {
      if (fieldSchema.default !== undefined) {
        // 深拷贝 default 值
        data[fieldName] = JSON.parse(JSON.stringify(fieldSchema.default));
      } else {
        // 根据 type 设置默认值
        switch (fieldSchema.type) {
          case 'string':
          case 'enum':
          case 'secret':
            data[fieldName] = '';
            break;
          case 'integer':
            data[fieldName] = 0;
            break;
          case 'number':
            data[fieldName] = 0.0;
            break;
          case 'boolean':
            data[fieldName] = false;
            break;
          case 'array':
            data[fieldName] = [];
            break;
          case 'object':
          case 'object_arr':
            data[fieldName] = {};
            break;
          default:
            data[fieldName] = null;
        }
      }
    };

    Object.entries(props.schema).forEach(([fieldName, fieldSchema]) => {
      initField(fieldName, fieldSchema);
    });

    formData.value = data;
  };

  // 处理字段值变化
  const handleFieldChange = (fieldName, value) => {
    formData.value[fieldName] = value;
    emit('change', formData.value);
  };

  // 重置表单
  const handleReset = () => {
    initFormData();
    message.success('已重置表单');
    emit('change', formData.value);
  };

  // 显示数据
  const handleShowData = () => {
    showDataModal.value = true;
  };

  // 复制数据
  const handleCopyData = () => {
    const json = JSON.stringify(formData.value, null, 2);
    navigator.clipboard
      .writeText(json)
      .then(() => {
        message.success('已复制到剪贴板');
      })
      .catch(() => {
        message.error('复制失败');
      });
  };

  // 监听 Schema 变化，重新初始化表单
  watch(
    () => props.schema,
    () => {
      initFormData();
    },
    { immediate: true, deep: true }
  );
</script>

<style scoped>
  .form-preview {
    padding: 16px;
    background: var(--editor-panel-bg, #fff);
    border-radius: 4px;
  }

  .preview-actions {
    display: flex;
    justify-content: flex-end;
    padding: 0 16px 16px;
  }
</style>
