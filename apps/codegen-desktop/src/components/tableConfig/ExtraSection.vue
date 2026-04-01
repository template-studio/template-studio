<template>
  <div class="section extra-section">
    <div class="section-header">
      <h4><AppstoreAddOutlined /> {{ section.title }}</h4>
      <span class="hint">根据模板组动态显示</span>
    </div>
    <div class="section-content">
      <a-row :gutter="16">
        <a-col
          v-for="item in section.items"
          :key="item.key"
          :span="8"
        >
          <a-form-item :label="item.label" class="form-item">
            <!-- 文本输入 -->
            <template v-if="item.type === 'text'">
              <a-input
                :value="config.extra[item.key]?.value"
                @change="(e) => updateExtra(item.key, e.target.value)"
              />
            </template>

            <!-- 数字输入 -->
            <template v-else-if="item.type === 'number'">
              <a-input-number
                :value="config.extra[item.key]?.value"
                @change="(val) => updateExtra(item.key, val)"
                style="width: 100%"
              />
            </template>

            <!-- 下拉选择 -->
            <template v-else-if="item.type === 'select'">
              <a-select
                :value="config.extra[item.key]?.value"
                @change="(val) => updateExtra(item.key, val)"
              >
                <a-select-option
                  v-for="opt in item.options"
                  :key="opt.value"
                  :value="opt.value"
                >
                  {{ opt.label }}
                </a-select-option>
              </a-select>
            </template>

            <!-- 开关 -->
            <template v-else-if="item.type === 'switch'">
              <a-switch
                :checked="config.extra[item.key]?.value"
                @change="(val) => updateExtra(item.key, val)"
              />
            </template>

            <!-- 默认文本 -->
            <template v-else>
              <a-input
                :value="config.extra[item.key]?.value || config.extra[item.key]"
                @change="(e) => updateExtra(item.key, e.target.value)"
              />
            </template>
          </a-form-item>
        </a-col>
      </a-row>
    </div>
  </div>
</template>

<script setup>
import { AppstoreAddOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  section: { type: Object, required: true },
  config: { type: Object, required: true }
})

const emit = defineEmits(['update'])

// 更新 extra 配置
const updateExtra = (key, value) => {
  if (props.config.extra[key] && typeof props.config.extra[key] === 'object') {
    props.config.extra[key].value = value
  } else {
    props.config.extra[key] = value
  }
  emit('update', 'extra', { ...props.config.extra })
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
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.hint {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 400;
}

.section-content {
  padding: 16px;
}

.form-item {
  margin-bottom: 12px;
}
</style>
