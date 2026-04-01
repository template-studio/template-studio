<template>
  <div class="status-bar">
    <n-space size="small">
      <!-- 左侧：模板信息 -->
      <n-text depth="3">
        <n-icon size="14"><Folder /></n-icon>
        {{ status.template }}
      </n-text>

      <n-divider vertical />

      <!-- 中间：变量统计 -->
      <n-text depth="3">
        <n-icon size="14"><Settings /></n-icon>
        变量: {{ status.variables }} 个
      </n-text>

      <!-- 右侧：预览状态 -->
      <n-text depth="3" style="margin-left: auto;">
        {{ status.preview }}
      </n-text>
    </n-space>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { NText, NSpace, NDivider, NIcon } from 'naive-ui';
import { Folder, Settings } from '@vicons/ionicons5';

const props = defineProps({
  status: {
    type: Object,
    required: true
  }
});
</script>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 16px;
  background: linear-gradient(90deg, #f8f9fa 0%, #ffffff 100%);
}

:deep(.n-space) {
  width: 100%;
}

:deep(.n-text) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: #666;
  transition: all 0.2s ease;
}

:deep(.n-text:hover) {
  color: #18a058;
}

:deep(.n-icon) {
  color: #18a058;
}

:deep(.n-divider) {
  background: #e5e7eb;
  margin: 0 12px;
}
</style>
