<template>
  <a-modal
    :open="open"
    :title="mapping ? `配置映射 - ${mapping.dbType}` : '添加映射'"
    :confirm-loading="saving"
    width="520px"
    @ok="handleSave"
    @cancel="handleClose"
    @update:open="(val) => !val && handleClose()"
  >
    <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
      <a-form-item label="数据库类型">
        <a-input v-model:value="form.pattern" disabled />
        <div class="form-hint">数据库字段类型（只读）</div>
      </a-form-item>

      <a-form-item label="目标类型" required>
        <a-auto-complete
          v-model:value="form.targetType"
          :options="targetTypeOptions"
          placeholder="如: String, Long, BigDecimal"
          :filter-option="filterTargetType"
          allow-clear
        >
          <template #option="{ value, label, description }">
            <div>
              <div style="font-weight: 500">{{ label }}</div>
              <div v-if="description" style="font-size: 12px; color: var(--color-text-muted)">{{ description }}</div>
            </div>
          </template>
        </a-auto-complete>
        <div class="form-hint">
          从 <a @click="openLanguageFieldTypes">{{ currentLanguageName }} 类型字段</a> 中选择，或输入自定义类型
        </div>
      </a-form-item>

      <a-form-item label="优先级">
        <a-input-number v-model:value="form.priority" :min="0" :max="100" style="width: 100%" />
        <div class="form-hint">数值越大优先级越高，精确匹配优先于通配匹配</div>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { reactive, watch } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'

const props = defineProps({
  open: { type: Boolean, default: false },
  mapping: { type: Object, default: null },
  targetTypeOptions: { type: Array, default: () => [] },
  currentLanguageName: { type: String, default: '' },
  saving: { type: Boolean, default: false }
})

const emit = defineEmits(['update:open', 'saved'])

const form = reactive({ pattern: '', targetType: '', priority: 10 })

watch(() => props.mapping, (record) => {
  if (record) {
    form.pattern = record.pattern
    form.targetType = record.targetType || ''
    form.priority = record.priority || 10
  }
})

const router = useRouter()
const openLanguageFieldTypes = () => { router.push('/languages') }

const filterTargetType = (input, option) => {
  const search = input.toLowerCase()
  return option.label.toLowerCase().includes(search) || (option.description && option.description.toLowerCase().includes(search))
}

const handleClose = () => { emit('update:open', false) }

const handleSave = () => {
  if (!form.targetType.trim()) { message.warning('请输入目标类型'); return }
  emit('saved', form.targetType.trim(), form.priority)
}
</script>

<style scoped>
.form-hint { font-size: 12px; color: var(--color-text-secondary); margin-top: 4px; }
.form-hint a { color: var(--color-primary); cursor: pointer; }
.form-hint a:hover { text-decoration: underline; }
</style>
