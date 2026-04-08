<template>
  <div class="step-panel">
    <a-form ref="configFormRef" :model="formModel" layout="vertical">
      <a-alert message="配置项目路径" description="请设置项目名称和输出目录，项目将在指定目录下创建" type="info" show-icon style="margin-bottom:24px;" />
      <a-form-item label="项目名称" name="projectName" :rules="projectNameRules">
        <a-input :value="projectName" @update:value="$emit('update:projectName', $event)" placeholder="请输入项目名称（如：my-awesome-project）" size="large" allow-clear />
        <div class="form-hint">只能包含字母、数字、下划线和连字符，长度 2-50 个字符</div>
      </a-form-item>
      <a-form-item label="输出目录" name="outputDir" :rules="[{ required: true, message: '请选择输出目录' }]">
        <div style="display:flex;gap:12px;align-items:center;">
          <a-input :value="outputDir" placeholder="请选择项目输出目录" disabled size="large" style="flex:1" />
          <a-button type="primary" size="large" @click="$emit('select-output-dir')" style="width:120px;flex-shrink:0;">浏览...</a-button>
        </div>
      </a-form-item>
      <div class="section-divider"><FolderOpenOutlined /><span>路径预览</span></div>
      <a-form-item>
        <a-alert v-if="finalOutputPath && !outputPathExists" :message="finalOutputPath" type="success" show-icon><template #icon><CheckCircleOutlined /></template><template #description>项目将在以上路径创建</template></a-alert>
        <a-alert v-else-if="finalOutputPath && outputPathExists" :message="finalOutputPath" type="error" show-icon><template #icon><WarningOutlined /></template><template #description>警告：该路径已存在，创建项目将覆盖原有内容</template></a-alert>
        <a-alert v-else message="路径未配置" description="请输入项目名称并选择输出目录" type="warning" show-icon><template #icon><WarningOutlined /></template></a-alert>
      </a-form-item>
    </a-form>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { FolderOpenOutlined, WarningOutlined, CheckCircleOutlined } from '@ant-design/icons-vue'

const props = defineProps({
  projectName: String,
  outputDir: String,
  finalOutputPath: String,
  outputPathExists: Boolean,
  projectNameRules: Array
})

defineEmits(['update:projectName', 'select-output-dir'])

const configFormRef = ref(null)
const formModel = computed(() => ({ projectName: props.projectName, outputDir: props.outputDir }))

defineExpose({ validate: () => configFormRef.value?.validate() })
</script>

<style scoped>
.step-panel { padding: 8px; display: flex; flex-direction: column; min-height: 0; }
.form-hint { font-size: 12px; color: var(--color-text-secondary); margin-top: 4px; }
.section-divider { display: flex; align-items: center; gap: 8px; margin: 24px 0; padding-bottom: 12px; border-bottom: 1px solid var(--color-border); font-size: 16px; font-weight: 600; color: var(--color-text); }
</style>
