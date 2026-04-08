<template>
  <div class="step-panel">
    <div v-if="loadingVariables" class="loading-ct"><a-spin size="large"><template #description>正在加载模板变量...</template></a-spin></div>
    <div v-else-if="!variableDefinitions?.length" class="loading-ct"><a-empty description="该模板没有配置变量"><template #image><SettingOutlined style="font-size:64px;color:var(--color-text-muted);" /></template></a-empty></div>
    <div v-else class="variables-form">
      <div class="variables-layout">
        <div class="variables-sidebar">
          <div class="sidebar-header"><h3 class="sidebar-title">编辑模式</h3></div>
          <div class="mode-tabs-vertical">
            <div class="mode-tab" :class="{ active: variableMode === 'normal' }" @click="$emit('update:variableMode', 'normal')"><EditOutlined class="mode-icon" /><span class="mode-label">普通模式</span></div>
            <div class="mode-tab" :class="{ active: variableMode === 'advanced' }" @click="$emit('update:variableMode', 'advanced')"><CodeOutlined class="mode-icon" /><span class="mode-label">高级模式</span></div>
          </div>
        </div>
        <div class="variables-content">
          <div v-show="variableMode === 'normal'" class="normal-mode">
            <a-form :model="variables" layout="horizontal" :label-col="{ style: 'width:120px' }">
              <div v-for="v in variableDefinitions" :key="v.name" class="form-field-item">
                <a-form-item :label="v.title || v.name" :required="v.required" :class="{ 'boolean-form-item': v.type === 'boolean' }">
                  <a-switch v-if="v.type === 'boolean'" :checked="variables[v.name]" @update:checked="$emit('update:variable', { name: v.name, value: $event })" :checked-children="v.trueText || '是'" :un-checked-children="v.falseText || '否'" />
                  <a-input-number v-else-if="v.type === 'number'" :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :min="v.min" :max="v.max" style="width:100%;" />
                  <a-select v-else-if="v.type === 'select' || v.type === 'enum'" :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :placeholder="`请选择${v.title||v.name}`" :options="v.options" allow-clear />
                  <a-select v-else-if="v.type === 'multi-select'" :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :placeholder="`请选择${v.title||v.name}`" :options="v.options" mode="multiple" allow-clear />
                  <a-textarea v-else-if="v.type === 'object' || v.type === 'array'" :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :placeholder="v.description || '请输入 JSON'" :rows="6" />
                  <a-textarea v-else-if="v.type === 'text'" :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :placeholder="v.description || `请输入${v.title||v.name}`" :rows="4" :maxlength="v.maxLength" show-count />
                  <a-input v-else :value="variables[v.name]" @update:value="$emit('update:variable', { name: v.name, value: $event })" :placeholder="v.description || `请输入${v.title||v.name}`" allow-clear><template v-if="v.name==='author'" #prefix><UserOutlined style="color:rgba(0,0,0,0.25);" /></template></a-input>
                </a-form-item>
              </div>
            </a-form>
          </div>
          <div v-show="variableMode === 'advanced'" class="advanced-mode">
            <div class="editor-wrap">
              <div class="editor-header"><span>JSON 编辑器</span><div class="actions"><a-button size="small" @click="$emit('format-json')">格式化</a-button><a-button size="small" type="primary" @click="$emit('sync-normal-mode')">同步普通模式</a-button></div></div>
              <div ref="jsonEditorContainer" class="json-editor"></div>
              <div class="editor-footer"><span v-if="jsonValid" class="ok">✅ JSON 格式正确</span><span v-else class="err">❌ {{ jsonError }}</span></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { SettingOutlined, EditOutlined, CodeOutlined, UserOutlined } from '@ant-design/icons-vue'

defineProps({
  loadingVariables: Boolean,
  variableDefinitions: Array,
  variableMode: String,
  variables: Object,
  jsonValid: Boolean,
  jsonError: String
})

defineEmits(['update:variableMode', 'update:variable', 'format-json', 'sync-normal-mode'])

const jsonEditorContainer = ref(null)

defineExpose({ jsonEditorContainer })
</script>

<style scoped>
.step-panel { padding: 8px; display: flex; flex-direction: column; min-height: 0; }
.loading-ct { display: flex; justify-content: center; align-items: center; min-height: 300px; }
.variables-form { padding: 0; height: 100%; display: flex; flex-direction: column; }
.variables-layout { display: flex; height: 100%; min-height: 500px; }
.variables-sidebar { width: 200px; flex-shrink: 0; background: var(--color-bg-elevated); border-right: 1px solid var(--color-border); display: flex; flex-direction: column; }
.sidebar-header { padding: 20px 16px 12px; border-bottom: 1px solid var(--color-border); font-size: 14px; font-weight: 600; color: var(--color-text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
.mode-tabs-vertical { padding: 12px 8px; display: flex; flex-direction: column; gap: 4px; }
.mode-tab { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 6px; cursor: pointer; transition: all 0.2s ease; color: var(--color-text-secondary); font-size: 14px; font-weight: 500; }
.mode-tab:hover { background: var(--color-bg-spotlight); color: var(--color-text); }
.mode-tab.active { background: var(--color-primary-bg); color: var(--color-primary); }
.mode-icon { font-size: 18px; color: var(--color-text-secondary); }
.variables-content { flex: 1; overflow-y: auto; padding: 24px 32px; background: var(--color-bg-container); }
.normal-mode { width: 100%; }
.normal-mode :deep(.ant-form-item) { margin-bottom: 24px; }
.normal-mode :deep(.ant-form-item-label) { width: 120px; padding-right: 12px; }
.normal-mode :deep(.ant-form-item-control) { flex: 1; max-width: none; }
.normal-mode :deep(.boolean-form-item .ant-form-item-control) { flex: 0 0 auto; }
.normal-mode :deep(.ant-form-item-label > label) { font-weight: 600; font-size: 14px; }
.form-field-item { width: 100%; }
.normal-mode :deep(.ant-input), .normal-mode :deep(.ant-input-number), .normal-mode :deep(.ant-select), .normal-mode :deep(.ant-switch), .normal-mode :deep(.ant-select-selector), .normal-mode :deep(.ant-input-number-input) { width: 100%; }
.advanced-mode { width: 100%; height: 100%; display: flex; flex-direction: column; }
.editor-wrap { border: 1px solid var(--color-border); border-radius: 6px; overflow: hidden; background: var(--color-bg-container); flex: 1; display: flex; flex-direction: column; min-height: 0; }
.editor-header { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: var(--color-bg-elevated); border-bottom: 1px solid var(--color-border); font-size: 14px; font-weight: 500; flex-shrink: 0; }
.json-editor { flex: 1; min-height: 0; overflow: auto; }
.editor-footer { padding: 8px 16px; background: var(--color-bg-elevated); font-size: 12px; display: flex; align-items: center; gap: 6px; border-top: 1px solid var(--color-border); }
.editor-footer .ok { color: var(--color-success); } .editor-footer .err { color: var(--color-error); }
</style>
