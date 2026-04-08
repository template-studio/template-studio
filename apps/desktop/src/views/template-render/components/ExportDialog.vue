<template>
  <a-modal
    :open="open"
    title="导出渲染结果"
    @ok="$emit('export')"
    @update:open="$emit('update:open', $event)"
    :confirm-loading="exporting"
    ok-text="导出"
    cancel-text="取消"
  >
    <div class="export-form">
      <div class="export-field">
        <span class="export-label">输出目录</span>
        <a-input
          :value="exportDir"
          placeholder="请输入输出目录路径"
          @update:value="$emit('update:exportDir', $event)"
        />
      </div>
      <div v-if="fileCount > 0" class="export-stats">
        <a-tag color="success">{{ successCount }} 个文件</a-tag>
        <a-tag v-if="errorCount > 0" color="error">
          {{ errorCount }} 个错误
        </a-tag>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
defineProps({
  open: Boolean,
  exporting: Boolean,
  exportDir: String,
  fileCount: Number,
  successCount: Number,
  errorCount: Number,
})

defineEmits(['update:open', 'update:exportDir', 'export'])
</script>

<style scoped>
.export-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.export-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.export-label {
  font-size: 13px;
  font-weight: 500;
}

.export-stats {
  display: flex;
  gap: 8px;
}
</style>
