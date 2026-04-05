<template>
  <a-modal
    v-model:open="dialogVisible"
    :title="mode === 'edit' ? '编辑数据源' : '新建数据源'"
    width="580px"
    :confirm-loading="submitting"
    @ok="handleSubmit"
  >
    <a-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      layout="vertical"
      @finish="handleSubmit"
    >
      <a-form-item label="数据源名称" name="name">
        <a-input
          v-model:value="formData.name"
          placeholder="请输入数据源名称"
          size="large"
        />
      </a-form-item>

      <a-form-item label="数据库类型" name="type">
        <a-select
          v-model:value="formData.type"
          placeholder="请选择数据库类型"
          size="large"
          @change="handleTypeChange"
        >
          <a-select-option value="mysql">MySQL</a-select-option>
          <a-select-option value="postgresql">PostgreSQL</a-select-option>
          <a-select-option value="sqlite">SQLite</a-select-option>
        </a-select>
      </a-form-item>

      <!-- MySQL/PostgreSQL 网络配置 -->
      <template v-if="formData.type !== 'sqlite'">
        <a-row :gutter="16">
          <a-col :span="16">
            <a-form-item label="主机地址" name="host">
              <a-input
                v-model:value="formData.host"
                placeholder="localhost 或 IP 地址"
                size="large"
              />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item label="端口" name="port">
              <a-input-number
                v-model:value="formData.port"
                :min="1"
                :max="65535"
                placeholder="端口"
                size="large"
                style="width: 100%"
              />
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item label="用户名" name="username">
          <a-input
            v-model:value="formData.username"
            placeholder="数据库用户名"
            size="large"
          />
        </a-form-item>

        <a-form-item label="密码" name="password">
          <a-input-password
            v-model:value="formData.password"
            placeholder="数据库密码"
            size="large"
          />
        </a-form-item>

        <!-- PostgreSQL 初始数据库 -->
        <a-form-item v-if="formData.type === 'postgresql'" label="初始数据库" name="database">
          <a-input
            v-model:value="formData.database"
            placeholder="默认: postgres"
            size="large"
          />
        </a-form-item>
      </template>

      <!-- SQLite 文件选择 -->
      <template v-else>
        <a-form-item label="数据库文件" name="sqliteFile">
          <a-input-search
            v-model:value="formData.sqliteFile"
            placeholder="请选择 SQLite 数据库文件"
            size="large"
            enter-button="选择文件"
            readonly
            @search="selectSqliteFile"
          />
        </a-form-item>
      </template>
    </a-form>

    <!-- 对话框底部按钮 -->
    <template #footer>
      <a-button @click="dialogVisible = false">取消</a-button>
      <a-button
        type="default"
        :loading="testing"
        @click="handleTestConnection"
      >
        测试连接
      </a-button>
      <a-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ mode === 'edit' ? '保存' : '创建' }}
      </a-button>
    </template>
  </a-modal>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import { open } from '@tauri-apps/plugin-dialog'
import * as datasourcesApi from '@/api/datasources'

const props = defineProps({
  open: { type: Boolean, default: false },
  mode: { type: String, default: 'create' },
  datasource: { type: Object, default: null }
})

const emit = defineEmits(['update:open', 'saved'])

const dialogVisible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

const submitting = ref(false)
const testing = ref(false)
const formRef = ref()

const formData = reactive({
  name: '',
  type: 'mysql',
  host: 'localhost',
  port: 3306,
  username: '',
  password: '',
  database: '',
  sqliteFile: ''
})

const formRules = computed(() => {
  const rules = {
    name: [{ required: true, message: '请输入数据源名称', trigger: 'blur' }],
    type: [{ required: true, message: '请选择数据库类型', trigger: 'change' }]
  }

  if (formData.type !== 'sqlite') {
    rules.host = [{ required: true, message: '请输入主机地址', trigger: 'blur' }]
    rules.port = [{ required: true, message: '请输入端口号', trigger: 'blur' }]
    rules.username = [{ required: true, message: '请输入用户名', trigger: 'blur' }]
    rules.password = [{ required: true, message: '请输入密码', trigger: 'blur' }]
    if (formData.type === 'postgresql') {
      rules.database = [{ required: true, message: '请输入初始数据库', trigger: 'blur' }]
    }
  } else {
    rules.sqliteFile = [{ required: true, message: '请选择数据库文件', trigger: 'change' }]
  }

  return rules
})

watch(() => props.open, (val) => {
  if (val) {
    if (props.mode === 'edit' && props.datasource) {
      Object.assign(formData, {
        name: props.datasource.name,
        type: props.datasource.type_,
        host: props.datasource.host || 'localhost',
        port: props.datasource.port || 3306,
        username: props.datasource.username || '',
        password: props.datasource.password || '',
        database: props.datasource.database || '',
        sqliteFile: props.datasource.sqlite_file || ''
      })
    } else {
      Object.assign(formData, {
        name: '',
        type: 'mysql',
        host: 'localhost',
        port: 3306,
        username: '',
        password: '',
        database: '',
        sqliteFile: ''
      })
    }
  }
})

const handleTypeChange = (type) => {
  if (type === 'mysql') {
    formData.port = 3306
  } else if (type === 'postgresql') {
    formData.port = 5432
  }

  if (type === 'sqlite') {
    formData.host = ''
    formData.port = null
    formData.username = ''
    formData.password = ''
    formData.database = ''
  } else {
    formData.sqliteFile = ''
    if (type === 'postgresql' && !formData.database) {
      formData.database = 'postgres'
    }
  }
}

const selectSqliteFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'SQLite Database',
        extensions: ['db', 'sqlite', 'sqlite3', 'db3']
      }]
    })

    if (selected) {
      formData.sqliteFile = selected
    }
  } catch (error) {
    message.error('选择文件失败: ' + error)
  }
}

const handleTestConnection = async () => {
  try {
    await formRef.value.validate()
  } catch (error) {
    message.warning('请先填写完整的连接信息')
    return
  }

  testing.value = true
  try {
    const connectionParams = {
      type: formData.type,
      host: formData.host,
      port: formData.port,
      username: formData.username,
      password: formData.password
    }

    if (formData.type === 'postgresql') {
      connectionParams.database = formData.database || 'postgres'
    }

    if (formData.type === 'sqlite') {
      connectionParams.sqliteFile = formData.sqliteFile
    }

    const result = await datasourcesApi.testConnection(connectionParams)
    message.success(result)
  } catch (error) {
    message.error('连接测试失败: ' + error)
  } finally {
    testing.value = false
  }
}

const handleSubmit = async () => {
  try {
    await formRef.value.validate()
  } catch (error) {
    return
  }

  submitting.value = true
  try {
    const data = {
      name: formData.name,
      type: formData.type,
      host: formData.host,
      port: formData.port,
      username: formData.username,
      password: formData.password,
      database: formData.database,
      sqliteFile: formData.sqliteFile
    }

    if (props.mode === 'edit') {
      await datasourcesApi.updateDatasource(props.datasource.id, data)
      message.success('数据源更新成功')
    } else {
      await datasourcesApi.createDatasource(data)
      message.success('数据源创建成功')
    }
    dialogVisible.value = false
    emit('saved')
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
:deep(.ant-modal-content) {
  border-radius: var(--border-radius-lg);
}

:deep(.ant-modal-header) {
  border-bottom: 1px solid var(--color-border);
}

:deep(.ant-form-item-label > label) {
  font-size: 14px;
  font-weight: 500;
}

:deep(.ant-input),
:deep(.ant-input-number),
:deep(.ant-select-selector),
:deep(.ant-input-password) {
  font-size: 14px;
}

:deep(.ant-input-number-input) {
  width: 100%;
}
</style>
