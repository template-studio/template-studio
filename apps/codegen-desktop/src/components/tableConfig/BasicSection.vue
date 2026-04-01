<template>
  <div class="section basic-section">
    <div class="section-header">
      <h4><SettingOutlined /> {{ section.title }}</h4>
    </div>
    <div class="section-content">
      <a-row :gutter="16">
        <a-col
          v-for="field in section.fields"
          :key="field.key"
          :span="getFieldSpan(field)"
        >
          <a-form-item :label="field.label" class="form-item">
            <!-- 只读字段 -->
            <template v-if="field.readonly">
              <a-input :value="field.value" disabled class="mono" />
            </template>

            <!-- 开关 -->
            <template v-else-if="field.type === 'switch'">
              <a-switch
                :checked="config[field.key]"
                @change="(val) => $emit('update', field.key, val)"
              />
            </template>

            <!-- 下拉选择 -->
            <template v-else-if="field.type === 'select' && field.options">
              <a-select
                :value="config[field.key]"
                @change="(val) => $emit('update', field.key, val)"
              >
                <a-select-option
                  v-for="opt in field.options"
                  :key="opt.value"
                  :value="opt.value"
                >
                  {{ opt.label }}
                </a-select-option>
              </a-select>
            </template>

            <!-- 数字 -->
            <template v-else-if="field.type === 'number'">
              <a-input-number
                :value="config[field.key]"
                @change="(val) => $emit('update', field.key, val)"
                style="width: 100%"
              />
            </template>

            <!-- 文本 -->
            <template v-else>
              <a-input
                :value="config[field.key]"
                @change="(e) => $emit('update', field.key, e.target.value)"
                :placeholder="'请输入' + field.label"
              />
            </template>
          </a-form-item>
        </a-col>
      </a-row>
    </div>
  </div>
</template>

<script setup>
import { SettingOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  section: { type: Object, required: true },
  config: { type: Object, required: true }
})

defineEmits(['update'])

// 获取字段占用的列数
const getFieldSpan = (field) => {
  // 表名、表注释等只读字段占满一行
  if (field.readonly) return 12
  // 其他字段
  return 8
}
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

.form-item {
  margin-bottom: 12px;
}

.mono {
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
