<template>
  <div class="section options-section">
    <div class="section-header">
      <h4><CheckCircleOutlined /> {{ section.title }}</h4>
    </div>
    <div class="section-content">
      <a-row :gutter="[16, 8]">
        <a-col
          v-for="item in section.items"
          :key="item.key"
          :span="6"
        >
          <div class="option-item">
            <a-switch
              :checked="config.options[item.key]"
              size="small"
              @change="(val) => $emit('update', 'options.' + item.key, val)"
            />
            <span class="option-label">{{ item.label }}</span>
          </div>
        </a-col>
      </a-row>
    </div>
  </div>
</template>

<script setup>
import { CheckCircleOutlined } from '@ant-design/icons-vue'

defineProps({
  section: { type: Object, required: true },
  config: { type: Object, required: true }
})

defineEmits(['update'])
</script>

<style scoped>
.section {
  background: var(--color-bg-container);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
}

.section-header {
  padding: 12px 16px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-content {
  padding: 16px;
}

.option-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.option-label {
  font-size: 13px;
  color: var(--color-text);
}
</style>
