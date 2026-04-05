<template>
  <a-modal :open="open" :title="mode === 'add' ? '新增表' : '编辑表'" width="600px" ok-text="确定" cancel-text="取消"
    @update:open="$emit('update:open', $event)" @ok="handleSave" @cancel="handleClose">
    <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
      <a-form-item label="表名" required>
        <a-input v-model:value="form.name" placeholder="请输入表名" />
      </a-form-item>
      <a-form-item label="表类型" required>
        <a-radio-group v-model:value="form.tableType">
          <a-radio value="table">表</a-radio>
          <a-radio value="view">视图</a-radio>
        </a-radio-group>
      </a-form-item>
      <a-form-item label="引擎">
        <a-input v-model:value="form.engine" placeholder="如：InnoDB" />
      </a-form-item>
      <a-form-item label="说明">
        <a-textarea v-model:value="form.comment" :rows="3" placeholder="请输入表说明" />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { reactive, watch } from 'vue'
import { useRoute } from 'vue-router'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import * as projectsApi from '@/api/projects'

const props = defineProps({
  open: { type: Boolean, default: false },
  mode: { type: String, default: 'add' },
  table: { type: Object, default: null },
  project: { type: Object, default: null }
})
const emit = defineEmits(['update:open', 'saved'])

const route = useRoute()
const form = reactive({ id: null, name: '', comment: '', engine: '', tableType: 'table' })

watch(() => props.open, (val) => {
  if (val) {
    if (props.mode === 'edit' && props.table) {
      form.id = props.table.id
      form.name = props.table.name
      form.comment = props.table.comment || ''
      form.engine = props.table.engine || ''
      form.tableType = props.table.table_type
    } else {
      form.id = null
      form.name = ''
      form.comment = ''
      form.engine = ''
      form.tableType = 'table'
    }
  }
})

const handleSave = async () => {
  if (!form.name) { message.warning('请输入表名'); return }
  try {
    if (props.mode === 'add') {
      const projectId = parseInt(route.params.id)
      await invoke('db_create_table', { projectId, name: form.name, comment: form.comment || null, engine: form.engine || null, tableType: form.tableType })
      message.success('表添加成功')
    } else {
      await projectsApi.updateTable(form.id, { name: form.name, comment: form.comment, engine: form.engine, tableType: form.tableType })
      message.success('表信息更新成功')
    }
    emit('saved')
    handleClose()
  } catch (error) { message.error(props.mode === 'add' ? '添加表失败: ' + error : '更新表失败: ' + error) }
}

const handleClose = () => {
  emit('update:open', false)
  form.id = null
  form.name = ''
  form.comment = ''
  form.engine = ''
  form.tableType = 'table'
}
</script>
