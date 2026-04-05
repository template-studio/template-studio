<template>
  <a-drawer :open="open" title="AI 智能建表" placement="right" :width="aiDrawerWidth" :closable="true" :maskClosable="false"
    @update:open="$emit('update:open', $event)">
    <a-steps :current="aiCreateStep - 1" size="small" style="margin-bottom: 24px">
      <a-step title="输入描述" /><a-step title="生成 SQL" /><a-step title="预览字段" /><a-step title="完成" />
    </a-steps>

    <!-- 步骤1：输入描述 -->
    <div v-if="aiCreateStep === 1">
      <a-form layout="vertical">
        <a-form-item label="SQL 类型" required>
          <a-select v-model:value="aiCreateForm.sqlType" placeholder="请选择数据库类型">
            <a-select-option value="mysql">MySQL</a-select-option>
            <a-select-option value="postgresql">PostgreSQL</a-select-option>
            <a-select-option value="sqlite">SQLite</a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="表描述" required>
          <a-textarea v-model:value="aiCreateForm.description" :rows="8"
            placeholder="请描述需要创建的表，例如：&#10;&#10;用户表：&#10;- id：主键，自增&#10;- username：用户名，唯一，不为空&#10;- email：邮箱，唯一&#10;- password：密码，加密存储&#10;- created_at：创建时间，默认当前时间&#10;- updated_at：更新时间，自动更新&#10;&#10;可以一次创建多个表，每表一行描述。"
            show-count :maxlength="2000" />
        </a-form-item>
        <a-form-item>
          <a-checkbox v-model:checked="aiCreateForm.followPreferences">遵循项目规范</a-checkbox>
          <div class="preference-hint">勾选后，AI 将按照项目配置的表规范生成 SQL（主键规范、审计字段、软删除等）</div>
        </a-form-item>
        <a-alert message="提示" description="AI 将根据您的描述自动生成建表 SQL 语句。支持一次创建多个表。" type="info" show-icon style="margin-bottom: 16px;" />
        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <a-button @click="$emit('update:open', false)">取消</a-button>
          <a-button type="primary" @click="generateAISQL" :loading="aiGenerating">生成 SQL</a-button>
        </div>
      </a-form>
    </div>

    <!-- 步骤2：SQL 预览和编辑 -->
    <div v-if="aiCreateStep === 2">
      <div style="margin-bottom: 12px">
        <a-alert :message="aiGeneratedMessage" :type="aiGeneratedMessage.includes('成功') ? 'success' : 'info'" show-icon closable />
      </div>
      <div v-if="aiShowDiff && aiOriginalSQL" :style="{ height: 'calc(100vh - 280px)', display: 'flex', flexDirection: 'column' }">
        <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <div style="font-weight: 500; color: var(--color-text);">SQL 差异对比</div>
          <a-button @click="aiShowDiff = false" size="small">返回编辑</a-button>
        </div>
        <div style="flex: 1;"><SQLDiffEditor :original="aiOriginalSQL" :modified="aiCreateForm.generatedSQL" theme="light" /></div>
      </div>
      <div v-else :style="{ display: 'flex', gap: '16px', height: aiShowConversation ? 'calc(100vh - 280px)' : 'auto' }">
        <div :style="{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }">
          <div style="margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
            <div style="font-weight: 500; color: var(--color-text);">SQL 编辑器</div>
            <a-button v-if="aiOriginalSQL" @click="aiShowDiff = true" size="small"><template #icon><FileTextOutlined /></template>查看差异</a-button>
          </div>
          <div :style="{ height: aiShowConversation ? 'calc(100vh - 340px)' : '400px' }"><SQLEditor v-model="aiCreateForm.generatedSQL" theme="light" /></div>
        </div>
        <div v-if="aiShowConversation" :style="{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column' }">
          <div style="margin-bottom: 8px; font-weight: 500; color: var(--color-text);">AI 对话助手</div>
          <div style="flex: 1; overflow-y: auto; background: var(--color-bg-secondary); padding: 12px; border-radius: 6px; margin-bottom: 12px; max-height: 450px;">
            <div v-if="aiConversationHistory.length === 0" style="text-align: center; color: var(--color-text-secondary); padding: 40px 0;">
              <div style="font-size: 48px; margin-bottom: 16px;">💬</div>
              <div>开始与 AI 对话优化 SQL</div>
            </div>
            <div v-else>
              <div v-for="(msg, index) in aiConversationHistory" :key="index" style="margin-bottom: 12px; padding: 10px; background: var(--color-bg-container); border-radius: 6px;">
                <div style="font-size: 12px; color: var(--color-text-secondary); margin-bottom: 6px; font-weight: 500;">{{ msg.role === 'user' ? '👤 您' : '🤖 AI 助手' }}</div>
                <div style="font-size: 13px; color: var(--color-text); white-space: pre-wrap; line-height: 1.6;">{{ msg.content }}</div>
              </div>
            </div>
          </div>
          <a-input-search v-model:value="aiUserMessage" placeholder="输入修改建议，例如：给 username 字段添加索引..." enter-button="发送" size="large" @search="continueAIConversation" :disabled="aiGenerating" />
          <div style="color: var(--color-text-secondary); font-size: 12px; margin-top: 8px;">提示：描述您想要的修改，AI 会基于当前 SQL 进行优化</div>
        </div>
      </div>
      <div style="display: flex; justify-content: space-between; margin-top: 16px;">
        <a-button @click="aiCreateStep = 1" :disabled="aiExecuting">上一步</a-button>
        <div style="display: flex; gap: 8px;">
          <a-button v-if="!aiShowConversation" @click="aiShowConversation = true"><template #icon><MessageOutlined /></template>继续优化</a-button>
          <a-button type="primary" @click="parseAISQL" :loading="aiParsing">下一步</a-button>
        </div>
      </div>
    </div>

    <!-- 步骤3：字段预览 -->
    <div v-if="aiCreateStep === 3">
      <a-spin :spinning="aiParsing">
        <div v-if="aiParsedTables.length > 0">
          <a-collapse v-model:activeKey="aiCollapseActiveKey" accordion>
            <a-collapse-panel v-for="table in aiParsedTables" :key="table.name">
              <template #header>
                <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                  <span style="font-weight: 500"><TableOutlined style="margin-right: 8px" />{{ table.name }}<a-tag v-if="table.comment" color="blue" style="margin-left: 8px">{{ table.comment }}</a-tag></span>
                  <a-tag color="green">{{ table.columns?.length || 0 }} 个字段</a-tag>
                </div>
              </template>
              <a-table :columns="aiColumnColumns" :data-source="table.columns || []" :pagination="false" size="small" :scroll="{ y: 240 }">
                <template #bodyCell="{ column, record }">
                  <template v-if="column.key === 'name'">
                    <div v-if="record.isPrimaryKey" style="display: flex; align-items: center; gap: 4px;"><KeyOutlined style="color: var(--color-warning); font-size: 12px;" /><span style="font-weight: 500;">{{ record.name }}</span></div>
                    <span v-else>{{ record.name }}</span>
                  </template>
                  <template v-else-if="column.key === 'dataType'"><a-tag color="blue" size="small">{{ record.dataType }}</a-tag></template>
                  <template v-else-if="column.key === 'isNullable'"><a-tag :color="record.isNullable ? 'green' : 'red'" size="small">{{ record.isNullable ? '可空' : '必填' }}</a-tag></template>
                  <template v-else-if="column.key === 'isPrimaryKey'"><a-tag v-if="record.isPrimaryKey" color="orange" size="small">主键</a-tag><span v-else>-</span></template>
                </template>
              </a-table>
            </a-collapse-panel>
          </a-collapse>
          <div style="margin-top: 16px"><a-alert :message="`即将创建 ${aiParsedTables.length} 张表，${aiParsedTables.reduce((sum, t) => sum + (t.columns?.length || 0), 0)} 个字段`" type="info" show-icon /></div>
        </div>
        <a-empty v-else description="未解析到表结构" />
      </a-spin>
      <div style="display: flex; justify-content: space-between; margin-top: 24px;">
        <a-button @click="aiCreateStep = 2" :disabled="aiExecuting">上一步</a-button>
        <a-button type="primary" @click="executeAISQL" :loading="aiExecuting">完成</a-button>
      </div>
    </div>

    <!-- 步骤4：执行结果 -->
    <div v-if="aiCreateStep === 4">
      <a-result :status="aiExecuteError ? 'error' : 'success'" :title="aiExecuteError ? '执行失败' : '执行成功'" :sub-title="aiExecuteResult">
        <template #extra>
          <div v-if="aiExecuteError" style="margin-top: 16px; text-align: left;">
            <a-alert message="错误信息" :description="aiExecuteError" type="error" show-icon style="margin-bottom: 16px;" />
          </div>
          <div style="display: flex; justify-content: center; gap: 8px;">
            <a-button @click="$emit('update:open', false)">关闭</a-button>
            <a-button v-if="aiExecuteError" type="primary" @click="fixAISQL" :loading="aiFixing">AI 修复</a-button>
          </div>
        </template>
      </a-result>
    </div>
  </a-drawer>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import SQLEditor from '@/components/SQLEditor.vue'
import SQLDiffEditor from '@/components/SQLDiffEditor.vue'
import { TableOutlined, KeyOutlined, FileTextOutlined, MessageOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  open: { type: Boolean, default: false },
  project: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'tables-created'])

const route = useRoute()
const aiCreateStep = ref(1)
const aiGenerating = ref(false)
const aiParsing = ref(false)
const aiExecuting = ref(false)
const aiFixing = ref(false)
const aiGeneratedMessage = ref('')
const aiExecuteResult = ref('')
const aiExecuteError = ref('')
const aiCollapseActiveKey = ref([])
const aiShowConversation = ref(false)
const aiShowDiff = ref(false)
const aiOriginalSQL = ref('')
const aiConversationHistory = ref([])
const aiUserMessage = ref('')
const aiParsedTables = ref([])

const aiCreateForm = reactive({ sqlType: 'mysql', description: '', generatedSQL: '', followPreferences: true })
const aiDrawerWidth = computed(() => aiShowConversation.value ? 1400 : 1000)

const aiColumnColumns = [
  { title: '字段名', dataIndex: 'name', key: 'name', width: 150 },
  { title: '类型', dataIndex: 'dataType', key: 'dataType', width: 120 },
  { title: '可空', dataIndex: 'isNullable', key: 'isNullable', width: 80 },
  { title: '主键', dataIndex: 'isPrimaryKey', key: 'isPrimaryKey', width: 80 },
  { title: '说明', dataIndex: 'comment', key: 'comment', ellipsis: true }
]

watch(() => props.open, (val) => {
  if (val) resetState()
})

const resetState = () => {
  Object.assign(aiCreateForm, { sqlType: props.project?.datasource?.type_ || 'mysql', description: '', generatedSQL: '', followPreferences: true })
  Object.assign({ aiCreateStep: 1, aiGeneratedMessage: '', aiParsedTables: [], aiExecuteResult: '', aiExecuteError: '', aiCollapseActiveKey: [], aiShowConversation: false, aiShowDiff: false, aiOriginalSQL: '', aiConversationHistory: [], aiUserMessage: '' })
  aiCreateStep.value = 1
  aiGeneratedMessage.value = ''
  aiParsedTables.value = []
  aiExecuteResult.value = ''
  aiExecuteError.value = ''
  aiCollapseActiveKey.value = []
  aiShowConversation.value = false
  aiShowDiff.value = false
  aiOriginalSQL.value = ''
  aiConversationHistory.value = []
  aiUserMessage.value = ''
}

const buildPreferencesPrompt = (prefs) => {
  const rules = []
  if (prefs.pkEnabled) rules.push(`【主键规范】必须包含主键字段：${prefs.pkFieldName || 'id'}，类型 ${prefs.pkFieldType || 'BIGINT'}${prefs.pkAutoIncrement ? '，自增' : ''}，注释"${prefs.pkComment || '主键ID'}"`)
  if (prefs.auditEnabled && prefs.auditFields) {
    const auditFields = []
    if (prefs.auditFields.createdAt?.enabled) auditFields.push(`${prefs.auditFields.createdAt.fieldName || 'created_at'}(${prefs.auditFields.createdAt.fieldType || 'TIMESTAMP'})`)
    if (prefs.auditFields.updatedAt?.enabled) auditFields.push(`${prefs.auditFields.updatedAt.fieldName || 'updated_at'}(${prefs.auditFields.updatedAt.fieldType || 'TIMESTAMP'})`)
    if (prefs.auditFields.createdBy?.enabled) auditFields.push(`${prefs.auditFields.createdBy.fieldName || 'created_by'}(${prefs.auditFields.createdBy.fieldType || 'BIGINT'})`)
    if (prefs.auditFields.updatedBy?.enabled) auditFields.push(`${prefs.auditFields.updatedBy.fieldName || 'updated_by'}(${prefs.auditFields.updatedBy.fieldType || 'BIGINT'})`)
    if (auditFields.length > 0) rules.push(`【审计字段】每个表必须包含：${auditFields.join('、')}`)
  }
  if (prefs.softDeleteEnabled) rules.push(`【软删除】每个表必须包含软删除字段：${prefs.softDeleteField || 'deleted_at'}，类型 ${prefs.softDeleteFieldType || 'TIMESTAMP'}，允许 NULL，默认 NULL`)
  if (prefs.booleanPrefix) rules.push(`【命名规范】布尔字段使用前缀 "${prefs.booleanPrefix}"，如 ${prefs.booleanPrefix}active`)
  if (prefs.datetimeSuffix) rules.push(`【命名规范】时间字段使用后缀 "${prefs.datetimeSuffix}"，如 created${prefs.datetimeSuffix}`)
  if (prefs.engineType) rules.push(`【存储引擎】使用 ${prefs.engineType}`)
  if (prefs.charset) rules.push(`【字符集】使用 ${prefs.charset}`)
  if (prefs.collation) rules.push(`【排序规则】使用 ${prefs.collation}`)
  return rules.length === 0 ? '' : `\n\n项目表规范（必须严格遵守）：\n${rules.map((r, i) => `${i + 9}. ${r}`).join('\n')}`
}

const cleanSQL = (sql) => sql.replace(/```sql\n?/g, '').replace(/```\n?/g, '').trim()

const generateAISQL = async () => {
  if (!aiCreateForm.description.trim()) { message.warning('请输入表描述'); return }
  aiShowConversation.value = false; aiShowDiff.value = false; aiOriginalSQL.value = ''
  aiConversationHistory.value = []; aiUserMessage.value = ''
  aiGenerating.value = true
  try {
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')
    if (!defaultService.provider || !defaultService.model) { message.warning('请先在"设置 → AI 服务 → 默认服务"中配置默认提供商和模型'); return }
    let preferencesText = ''
    if (aiCreateForm.followPreferences) {
      try {
        const projectId = parseInt(route.params.id)
        const prefsResult = await invoke('db_get_table_preferences', { projectId })
        if (prefsResult) preferencesText = buildPreferencesPrompt(typeof prefsResult === 'string' ? JSON.parse(prefsResult) : prefsResult)
      } catch (error) { console.warn('获取表规范配置失败:', error) }
    }
    const userPrompt = `请根据以下描述生成 ${aiCreateForm.sqlType.toUpperCase()} 建表 SQL 语句：\n\n${aiCreateForm.description}\n\n要求：\n1. 生成标准的 ${aiCreateForm.sqlType.toUpperCase()} CREATE TABLE 语句\n2. 包含所有字段的类型、约束、默认值、注释\n3. 主键使用 AUTO_INCREMENT (MySQL) 或 SERIAL (PostgreSQL) 或 INTEGER PRIMARY KEY AUTOINCREMENT (SQLite)\n4. 时间字段使用 TIMESTAMP 或 DATETIME\n5. 创建时间字段默认值使用 CURRENT_TIMESTAMP\n6. 每个表添加 ENGINE=InnoDB (仅 MySQL)\n7. 支持一次创建多个表，每条 CREATE 语句用分号分隔\n8. 只返回 SQL 语句，不要其他解释文字\n${preferencesText}\n请直接输出 SQL 语句：`
    aiConversationHistory.value = [{ role: 'user', content: userPrompt }]
    const result = await invoke('ai_generate_sql', { provider: defaultService.provider, model: defaultService.model, messages: aiConversationHistory.value })
    const cleanedSQL = cleanSQL(result)
    aiCreateForm.generatedSQL = cleanedSQL
    const createCount = cleanedSQL.split(';').filter(s => s.trim().toUpperCase().includes('CREATE')).length
    aiGeneratedMessage.value = `SQL 生成成功，共 ${createCount} 条 CREATE 语句`
    aiConversationHistory.value.push({ role: 'assistant', content: `已生成 SQL，包含 ${createCount} 个表。` })
    aiCreateStep.value = 2
  } catch (error) { message.error('生成 SQL 失败: ' + error) } finally { aiGenerating.value = false }
}

const continueAIConversation = async () => {
  if (!aiUserMessage.value.trim()) { message.warning('请输入修改建议'); return }
  const userInput = aiUserMessage.value.trim(); aiUserMessage.value = ''
  aiGenerating.value = true
  try {
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')
    if (!defaultService.provider || !defaultService.model) { message.warning('请先配置默认服务'); return }
    aiConversationHistory.value.push({ role: 'user', content: userInput + `\n\n当前 SQL：\n${aiCreateForm.generatedSQL}` })
    const result = await invoke('ai_generate_sql', { provider: defaultService.provider, model: defaultService.model, messages: aiConversationHistory.value })
    const cleanedSQL = cleanSQL(result)
    if (cleanedSQL && cleanedSQL.toUpperCase().includes('CREATE')) {
      if (!aiOriginalSQL.value) aiOriginalSQL.value = aiCreateForm.generatedSQL
      aiCreateForm.generatedSQL = cleanedSQL
      const createCount = cleanedSQL.split(';').filter(s => s.trim().toUpperCase().includes('CREATE')).length
      aiGeneratedMessage.value = `SQL 已更新，共 ${createCount} 条 CREATE 语句`
      aiConversationHistory.value.push({ role: 'assistant', content: `已根据您的建议"${userInput}"更新 SQL。` })
      message.success('SQL 已更新，可点击"查看差异"查看修改前后对比')
    } else {
      aiGeneratedMessage.value = result
      aiConversationHistory.value.push({ role: 'assistant', content: result })
      message.info('AI 已回复，请查看上方对话历史')
    }
  } catch (error) { message.error('AI 生成错误: ' + error) } finally { aiGenerating.value = false }
}

const parseAISQL = async () => {
  if (!aiCreateForm.generatedSQL.trim()) { message.warning('没有可解析的 SQL'); return }
  aiParsing.value = true
  try {
    const projectId = parseInt(route.params.id)
    const result = await invoke('parse_ai_sql', { projectId, sql: aiCreateForm.generatedSQL, dialect: aiCreateForm.sqlType })
    const parsed = JSON.parse(result)
    aiParsedTables.value = parsed.tables || []
    if (aiParsedTables.value.length === 0) { message.warning('未解析到任何表，请检查 SQL 格式'); return }
    aiCreateStep.value = 3
  } catch (error) { message.error('解析 SQL 失败: ' + error) } finally { aiParsing.value = false }
}

const executeAISQL = async () => {
  aiExecuting.value = true; aiExecuteError.value = ''; aiExecuteResult.value = ''
  try {
    const projectId = parseInt(route.params.id)
    const result = await invoke('execute_ai_sql', { projectId, sql: aiCreateForm.generatedSQL, dialect: aiCreateForm.sqlType })
    aiExecuteResult.value = result || '所有表创建成功'
    aiCreateStep.value = 4
    emit('tables-created')
  } catch (error) { aiExecuteError.value = error; aiExecuteResult.value = '执行失败，请查看错误信息'; aiCreateStep.value = 4 } finally { aiExecuting.value = false }
}

const fixAISQL = async () => {
  if (!aiExecuteError.value) return
  aiFixing.value = true
  try {
    const defaultService = JSON.parse(localStorage.getItem('ai-default-service') || '{}')
    const result = await invoke('ai_fix_sql', { provider: defaultService.provider, model: defaultService.model, sql: aiCreateForm.generatedSQL, error: aiExecuteError.value, dialect: aiCreateForm.sqlType })
    aiCreateForm.generatedSQL = cleanSQL(result)
    aiGeneratedMessage.value = 'SQL 已修复，请检查后继续执行'
    message.success('SQL 已修复')
    await parseAISQL()
    aiCreateStep.value = 3
  } catch (error) { message.error('修复 SQL 失败: ' + error) } finally { aiFixing.value = false }
}
</script>

<style scoped>
.preference-hint { margin-top: 4px; font-size: 12px; color: var(--color-text-secondary); }
</style>
