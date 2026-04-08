<template>
  <a-modal
    :open="open"
    title="添加类型映射"
    ok-text="添加"
    cancel-text="取消"
    @ok="emit('submit')"
    @cancel="emit('cancel')"
    @update:open="emit('update:open', $event)"
  >
    <a-form layout="vertical">
      <a-form-item label="数据库字段类型">
        <a-input :value="form.sourceType" @input="form.sourceType = $event.target.value" placeholder="请输入数据库字段类型（如：varchar）" />
      </a-form-item>
      <a-form-item label="语言字段类型">
        <a-input :value="form.targetType" @input="form.targetType = $event.target.value" placeholder="请输入语言字段类型" />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
defineProps({
  open: { type: Boolean, default: false },
  form: { type: Object, default: () => ({}) },
  languageOptions: { type: Array, default: () => [] }
})

const emit = defineEmits(['update:open', 'submit', 'cancel'])
</script>
