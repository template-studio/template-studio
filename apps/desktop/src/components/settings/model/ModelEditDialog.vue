<template>
  <a-modal
    :open="open"
    :title="editingModel ? '编辑模型' : '添加模型'"
    width="500px"
    ok-text="确定"
    cancel-text="取消"
    @ok="emit('submit')"
    @cancel="emit('cancel')"
    @update:open="(val) => emit('update:open', val)"
  >
    <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
      <a-form-item label="模型 ID" required>
        <a-input
          v-model:value="modelForm.modelId"
          placeholder="如: gpt-4"
        />
      </a-form-item>
      <a-form-item label="模型名称" required>
        <a-input
          v-model:value="modelForm.modelName"
          placeholder="如: GPT-4"
        />
      </a-form-item>
      <a-form-item label="分组">
        <a-select v-model:value="modelForm.groupId">
          <a-select-option value="chat">对话模型</a-select-option>
          <a-select-option value="code">代码模型</a-select-option>
          <a-select-option value="image">图像模型</a-select-option>
          <a-select-option value="embedding">嵌入模型</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="描述">
        <a-textarea
          v-model:value="modelForm.description"
          :rows="3"
          placeholder="模型描述（可选）"
        />
      </a-form-item>
      <a-form-item label="最大 Tokens">
        <a-input-number
          v-model:value="modelForm.maxTokens"
          :min="1"
          :max="128000"
          :step="1000"
          style="width: 100%"
        />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
defineProps({
  open: {
    type: Boolean,
    default: false
  },
  modelForm: {
    type: Object,
    required: true
  },
  editingModel: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:open', 'submit', 'cancel'])
</script>
