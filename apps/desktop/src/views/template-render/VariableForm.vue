<template>
  <div class="variable-form">
    <template v-if="fields.length > 0">
      <template v-for="group in groupedFields" :key="group.name">
        <div v-if="group.name" class="group-title">
          <a-divider orientation="left" orientation-margin="0">
            <span class="group-label">{{ group.name }}</span>
          </a-divider>
        </div>
        <div class="group-fields">
          <template v-for="field in group.fields" :key="field.name">
            <div
              v-if="isFieldVisible(field)"
              class="form-field"
            >
              <div class="field-label">
                <span class="field-name">{{ field.label || field.name }}</span>
                <span v-if="field.required" class="required-mark">*</span>
                <span v-if="field.description" class="field-desc">{{ field.description }}</span>
              </div>

              <!-- string -->
              <a-input
                v-if="!field.type || field.type === 'string'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || `请输入${field.label || field.name}`"
                :defaultValue="field.default"
                @update:value="updateField(field.name, $event)"
              />

              <!-- text -->
              <a-textarea
                v-else-if="field.type === 'text'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || `请输入${field.label || field.name}`"
                :defaultValue="field.default"
                :auto-size="{ minRows: 3, maxRows: 10 }"
                @update:value="updateField(field.name, $event)"
              />

              <!-- number -->
              <a-input-number
                v-else-if="field.type === 'number'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder"
                :defaultValue="field.default"
                style="width: 100%"
                @update:value="updateField(field.name, $event)"
              />

              <!-- boolean -->
              <a-switch
                v-else-if="field.type === 'boolean'"
                :checked="modelValue[field.name]"
                :defaultChecked="field.default"
                @change="updateField(field.name, $event)"
              />

              <!-- select -->
              <a-select
                v-else-if="field.type === 'select'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || '请选择'"
                :defaultValue="field.default"
                :options="(field.options || []).map(o => typeof o === 'string' ? { label: o, value: o } : o)"
                @update:value="updateField(field.name, $event)"
              />

              <!-- multi-select -->
              <a-select
                v-else-if="field.type === 'multi-select'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || '请选择'"
                :defaultValue="field.default"
                :options="(field.options || []).map(o => typeof o === 'string' ? { label: o, value: o } : o)"
                mode="multiple"
                @update:value="updateField(field.name, $event)"
              />

              <!-- date -->
              <a-date-picker
                v-else-if="field.type === 'date'"
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || '选择日期'"
                style="width: 100%"
                @update:value="updateField(field.name, $event)"
              />

              <!-- json (嵌套对象) -->
              <div v-else-if="field.type === 'json'" ref="jsonEditorRefs" class="json-field-editor" />

              <!-- fallback: string -->
              <a-input
                v-else
                :value="modelValue[field.name]"
                :placeholder="field.placeholder || `请输入${field.label || field.name}`"
                :defaultValue="field.default"
                @update:value="updateField(field.name, $event)"
              />
            </div>
          </template>
        </div>
      </template>
    </template>

    <a-empty v-else description="此模板无可配置变量" :image-style="{ height: '60px' }">
      <template #description>
        <span style="font-size: 13px">可切换到高级模式直接编辑 JSON</span>
      </template>
    </a-empty>
  </div>
</template>

<script setup>
import { computed, watch, ref, onBeforeUnmount, nextTick } from 'vue'
import { EditorView, basicSetup } from 'codemirror'
import { json } from '@codemirror/lang-json'

const props = defineProps({
  schema: { type: Object, default: null },
  modelValue: { type: Object, default: () => ({}) },
})

const emit = defineEmits(['update:modelValue'])

const fields = computed(() => {
  if (!props.schema?.fields) return []
  return props.schema.fields
})

// 按 group 分组
const groupedFields = computed(() => {
  const groups = new Map()
  for (const field of fields.value) {
    const group = field.group || ''
    if (!groups.has(group)) {
      groups.set(group, { name: group, fields: [] })
    }
    groups.get(group).fields.push(field)
  }
  return Array.from(groups.values())
})

// 条件字段显隐
const isFieldVisible = (field) => {
  if (!field.condition) return true
  const { field: condField, operator, value } = field.condition
  const currentVal = props.modelValue[condField]
  switch (operator) {
    case 'eq': return currentVal === value
    case 'ne': return currentVal !== value
    case 'in': return Array.isArray(value) && value.includes(currentVal)
    case 'not_in': return Array.isArray(value) && !value.includes(currentVal)
    case 'gt': return currentVal > value
    case 'lt': return currentVal < value
    case 'gte': return currentVal >= value
    case 'lte': return currentVal <= value
    default: return true
  }
}

const updateField = (name, value) => {
  emit('update:modelValue', { ...props.modelValue, [name]: value })
}

// JSON 类型字段的编辑器
const jsonEditorRefs = ref([])
const jsonEditors = []

watch(() => props.modelValue, async () => {
  await nextTick()
  // 为 json 类型字段初始化编辑器
  const jsonFields = fields.value.filter(f => f.type === 'json')
  jsonFields.forEach((field, idx) => {
    const container = jsonEditorRefs.value?.[idx]
    if (!container || container._hasEditor) return
    container._hasEditor = true

    const val = props.modelValue[field.name]
    const doc = typeof val === 'string' ? val : JSON.stringify(val, null, 2)

    const view = new EditorView({
      doc,
      extensions: [
        basicSetup,
        json(),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            try {
              const parsed = JSON.parse(update.state.doc.toString())
              updateField(field.name, parsed)
            } catch {
              // 保持原文
              updateField(field.name, update.state.doc.toString())
            }
          }
        }),
      ],
      parent: container,
    })
    jsonEditors.push(view)
  })
}, { immediate: true })

onBeforeUnmount(() => {
  jsonEditors.forEach(v => v.destroy())
})
</script>

<style scoped>
.variable-form {
  padding: 0 4px;
}

.group-title {
  margin-top: 8px;
}

.group-title:first-child {
  margin-top: 0;
}

.group-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.group-fields {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 0 4px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  display: flex;
  align-items: baseline;
  gap: 4px;
  flex-wrap: wrap;
}

.field-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
}

.required-mark {
  color: #ff4d4f;
  font-size: 13px;
}

.field-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.json-field-editor {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
  min-height: 120px;
}

.json-field-editor :deep(.cm-editor) {
  min-height: 120px;
}
</style>
