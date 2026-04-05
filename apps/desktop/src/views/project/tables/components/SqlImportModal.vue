<template>
  <a-modal :open="open" title="从SQL导入表结构" width="800px" ok-text="导入" cancel-text="取消"
    @update:open="$emit('update:open', $event)" @ok="importFromSql" @cancel="closeSqlImportDialog">
    <a-form :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }">
      <a-form-item label="SQL类型" required>
        <a-select v-model:value="sqlImportForm.dialect" placeholder="请选择SQL类型">
          <a-select-option value="mysql">MySQL</a-select-option>
          <a-select-option value="postgresql">PostgreSQL</a-select-option>
          <a-select-option value="sqlite">SQLite</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="上传文件">
        <a-upload :before-upload="handleSqlFileUpload" :show-upload-list="false" accept=".sql">
          <a-button><UploadOutlined /> 选择SQL文件</a-button>
        </a-upload>
        <div v-if="sqlImportForm.fileName" style="margin-top: 8px; color: var(--color-text-secondary)">已选择: {{ sqlImportForm.fileName }}</div>
      </a-form-item>
      <a-form-item label="或输入SQL">
        <a-textarea v-model:value="sqlImportForm.sqlContent" :rows="12" placeholder="请输入CREATE TABLE语句，支持CREATE TABLE语法..." />
      </a-form-item>
      <a-form-item label="示例">
        <div style="background: var(--color-surface); padding: 12px; border-radius: 4px; font-size: 12px; font-family: monospace;">
CREATE TABLE users (<br>&nbsp;&nbsp;id INT PRIMARY KEY,<br>&nbsp;&nbsp;username VARCHAR(50) NOT NULL,<br>&nbsp;&nbsp;email VARCHAR(100) UNIQUE,<br>&nbsp;&nbsp;created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP<br>);
        </div>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { reactive, watch } from 'vue'
import { useRoute } from 'vue-router'
import { UploadOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import * as projectsApi from '@/api/projects'

const props = defineProps({
  open: { type: Boolean, default: false },
  project: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'imported'])

const route = useRoute()
const sqlImportForm = reactive({ dialect: 'mysql', sqlContent: '', fileName: '' })

watch(() => props.open, (val) => {
  if (val && props.project) {
    sqlImportForm.dialect = props.project?.datasource?.type_ || 'mysql'
    sqlImportForm.sqlContent = ''
    sqlImportForm.fileName = ''
  }
})

const handleSqlFileUpload = (file) => {
  const reader = new FileReader()
  reader.onload = (e) => { sqlImportForm.sqlContent = e.target.result; sqlImportForm.fileName = file.name }
  reader.readAsText(file)
  return false
}

const importFromSql = async () => {
  if (!sqlImportForm.sqlContent.trim()) { message.warning('请输入或上传SQL内容'); return }
  try {
    const projectId = parseInt(route.params.id)
    const result = await projectsApi.parseSqlAndCreate(projectId, sqlImportForm.sqlContent, sqlImportForm.dialect)
    message.success(result)
    closeSqlImportDialog()
    emit('imported')
  } catch (error) { message.error('SQL导入失败: ' + error) }
}

const closeSqlImportDialog = () => {
  emit('update:open', false)
  sqlImportForm.dialect = 'mysql'
  sqlImportForm.sqlContent = ''
  sqlImportForm.fileName = ''
}
</script>
