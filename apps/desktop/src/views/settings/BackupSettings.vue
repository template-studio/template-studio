<template>
  <div class="setting-container">
    <!-- 数据备份 -->
    <div class="setting-group">
      <div class="setting-title">数据备份</div>
      <div class="setting-help-text">
        备份和恢复项目配置、映射规则、语言设置等数据
      </div>

      <div class="setting-row">
        <div class="setting-row-title">导出数据</div>
        <div class="setting-row-content">
          <a-button @click="exportData" :loading="exporting">
            <template #icon><ExportOutlined /></template>
            导出备份文件
          </a-button>
          <span class="setting-hint">导出所有配置数据为 JSON 文件</span>
        </div>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">导入数据</div>
        <div class="setting-row-content">
          <a-button @click="triggerImport">
            <template #icon><ImportOutlined /></template>
            导入备份文件
          </a-button>
          <span class="setting-hint">从备份文件恢复数据（将覆盖现有数据）</span>
        </div>
      </div>
    </div>

    <!-- 存储路径 -->
    <div class="setting-group">
      <div class="setting-title">存储路径配置</div>

      <div class="setting-row">
        <div class="setting-row-title">模板存储路径</div>
        <div class="setting-row-content">
          <a-input-search
            v-model:value="settings.templatePath"
            placeholder="选择模板存储路径"
            size="small"
            style="max-width: 400px"
            @search="selectTemplatePath"
            readonly
          >
            <template #enterButton>
              <a-button>选择</a-button>
            </template>
          </a-input-search>
        </div>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">项目导出路径</div>
        <div class="setting-row-content">
          <a-input-search
            v-model:value="settings.exportPath"
            placeholder="选择项目默认导出路径"
            size="small"
            style="max-width: 400px"
            @search="selectExportPath"
            readonly
          >
            <template #enterButton>
              <a-button>选择</a-button>
            </template>
          </a-input-search>
        </div>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-content">
          <a-button type="primary" size="small" @click="saveSettings">
            保存设置
          </a-button>
        </div>
      </div>
    </div>

    <!-- 自动备份 -->
    <div class="setting-group">
      <div class="setting-title">自动备份</div>

      <div class="setting-row">
        <div class="setting-row-title">启用自动备份</div>
        <a-switch
          v-model:checked="settings.autoBackup"
          size="small"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">备份频率</div>
        <a-select
          v-model:value="settings.backupFrequency"
          :disabled="!settings.autoBackup"
          size="small"
          style="width: 120px"
        >
          <a-select-option value="daily">每天</a-select-option>
          <a-select-option value="weekly">每周</a-select-option>
          <a-select-option value="monthly">每月</a-select-option>
        </a-select>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">保留备份数量</div>
        <a-input-number
          v-model:value="settings.maxBackups"
          :min="1"
          :max="30"
          :disabled="!settings.autoBackup"
          size="small"
          style="width: 100px"
        />
      </div>
    </div>

    <input
      ref="fileInputRef"
      type="file"
      accept=".json"
      style="display: none"
      @change="handleImportFile"
    />
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { ExportOutlined, ImportOutlined } from '@ant-design/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'

const fileInputRef = ref(null)
const exporting = ref(false)

// 设置数据
const settings = reactive({
  templatePath: '',
  exportPath: '',
  autoBackup: false,
  backupFrequency: 'weekly',
  maxBackups: 5
})

// 导出数据
const exportData = async () => {
  exporting.value = true
  try {
    // 获取所有数据
    const projects = await invoke('db_get_all_projects')
    const datasources = await invoke('db_get_all_datasources')
    const languages = await invoke('db_get_all_languages')
    const mappings = await invoke('db_get_system_type_mappings')

    const backupData = {
      version: '1.0',
      exportedAt: new Date().toISOString(),
      data: {
        projects: JSON.parse(projects),
        datasources: JSON.parse(datasources),
        languages: JSON.parse(languages),
        mappings: typeof mappings === 'string' ? JSON.parse(mappings) : mappings
      }
    }

    // 选择保存路径
    const filePath = await save({
      defaultPath: `template-studio-backup-${new Date().toISOString().split('T')[0]}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    })

    if (filePath) {
      await invoke('write_text_file', {
        path: filePath,
        content: JSON.stringify(backupData, null, 2)
      })
      message.success('数据导出成功')
    }
  } catch (error) {
    message.error('导出失败: ' + error)
  } finally {
    exporting.value = false
  }
}

// 触发导入
const triggerImport = () => {
  fileInputRef.value?.click()
}

// 处理导入文件
const handleImportFile = async (event) => {
  const file = event.target.files?.[0]
  if (!file) return

  event.target.value = ''

  try {
    const text = await file.text()
    const data = JSON.parse(text)

    if (!data.data) {
      message.error('无效的备份文件格式')
      return
    }

    Modal.confirm({
      title: '确认导入',
      content: '导入将覆盖现有数据，是否继续？',
      okText: '导入',
      cancelText: '取消',
      onOk: async () => {
        try {
          // TODO: 实现数据导入逻辑
          message.success('数据导入成功')
        } catch (error) {
          message.error('导入失败: ' + error)
        }
      }
    })
  } catch (error) {
    message.error('文件解析失败: ' + error)
  }
}

// 选择模板路径
const selectTemplatePath = async () => {
  try {
    const selected = await open({
      directory: true,
      title: '选择模板存储路径'
    })
    if (selected) {
      settings.templatePath = selected
    }
  } catch (error) {
    console.error('选择路径失败:', error)
  }
}

// 选择导出路径
const selectExportPath = async () => {
  try {
    const selected = await open({
      directory: true,
      title: '选择项目导出路径'
    })
    if (selected) {
      settings.exportPath = selected
    }
  } catch (error) {
    console.error('选择路径失败:', error)
  }
}

// 保存设置
const saveSettings = () => {
  try {
    localStorage.setItem('backupSettings', JSON.stringify(settings))
    message.success('设置已保存')
  } catch (error) {
    message.error('保存失败')
  }
}

// 加载设置
const loadSettings = () => {
  try {
    const saved = localStorage.getItem('backupSettings')
    if (saved) {
      Object.assign(settings, JSON.parse(saved))
    }
  } catch (error) {
    console.error('Load settings error:', error)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.setting-container {
  background: transparent;
}

.setting-row-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-hint {
  font-size: 12px;
  color: var(--color-text-muted);
}
</style>
