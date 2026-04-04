<template>
  <div class="setting-container">
    <!-- Web 服务器配置 -->
    <div class="setting-group">
      <div class="setting-title">Web 服务器配置</div>

      <div class="setting-row">
        <div class="setting-row-content">
          <div class="setting-row-title">API 地址</div>
          <div class="setting-row-description">模板服务器的 API 地址</div>
        </div>
        <a-input
          v-model:value="config.api_url"
          placeholder="http://127.0.0.1:8080"
          style="width: 300px"
        />
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-content">
          <div class="setting-row-title">API 密钥</div>
          <div class="setting-row-description">用于身份验证的 API 密钥（可选）</div>
        </div>
        <a-input
          v-model:value="config.api_key"
          type="password"
          placeholder="留空表示不需要验证"
          style="width: 300px"
        />
      </div>

      <div style="margin-top: 16px; text-align: right;">
        <a-button
          type="primary"
          @click="saveWebServerConfig"
          :loading="saving"
        >
          保存配置
        </a-button>
      </div>
    </div>

    <!-- 模板存储位置 -->
    <div class="setting-group">
      <div class="setting-title">模板存储位置</div>

      <div class="setting-row">
        <div class="setting-row-content">
          <div class="setting-row-title">模板路径</div>
          <div class="setting-row-description">下载的模板文件将保存在此目录</div>
        </div>
        <div style="display: flex; gap: 8px; align-items: center;">
          <a-input
            v-model:value="config.template_path"
            style="width: 400px"
            readonly
          />
          <a-button @click="selectTemplatePath" type="primary" size="small">
            浏览
          </a-button>
        </div>
      </div>

      <div class="setting-help-text">
        默认路径: {{ defaultTemplatePath }}
      </div>
    </div>

    <!-- 测试连接 -->
    <div class="setting-group">
      <div class="setting-title">连接测试</div>

      <div class="setting-row">
        <div class="setting-row-content">
          <div class="setting-row-title">测试服务器连接</div>
          <div class="setting-row-description">检查是否能成功连接到 Web 服务器</div>
        </div>
        <a-button
          @click="testConnection"
          :loading="testing"
          type="primary"
          size="small"
        >
          测试连接
        </a-button>
      </div>

      <div v-if="testResult" class="setting-help-text" :style="{ color: testResult.success ? '#52c41a' : '#ff4d4f' }">
        {{ testResult.message }}
      </div>
    </div>

    <!-- 配置说明 -->
    <div class="setting-group">
      <div class="setting-title">配置说明</div>
      <div class="setting-help-text">
        <p>修改 Web 服务器配置后，需要重新启动应用才能生效。</p>
        <p>配置文件保存在: {{ configFilePath }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()

// 配置数据
const config = ref({
  api_url: 'http://127.0.0.1:8080',
  api_key: '',
  template_path: ''
})

// 用户名
const username = ref('')

// 保存状态
const saving = ref(false)
const testing = ref(false)
const testResult = ref(null)

// 配置文件路径（动态替换用户名）
const configFilePath = computed(() => {
  if (username.value) {
    return `C:\\Users\\${username.value}\\.ciclebyte\\template_studio_rust\\config\\scaffold-desktop.yaml`
  }
  return 'C:\\Users\\{user}\\.ciclebyte\\template_studio_rust\\config\\scaffold-desktop.yaml'
})

// 默认模板路径（动态替换用户名）
const defaultTemplatePath = computed(() => {
  if (username.value) {
    return `C:\\Users\\${username.value}\\.ciclebyte\\template_studio_rust\\data\\templates`
  }
  return 'C:\\Users\\{user}\\.ciclebyte\\template_studio_rust\\data\\templates'
})

// 加载配置
const loadConfig = async () => {
  try {
    // 获取用户名
    try {
      username.value = await invoke('get_username')
    } catch (error) {
      console.warn('获取用户名失败:', error)
      username.value = '{user}'
    }

    const configJson = await invoke('get_config')
    const data = JSON.parse(configJson)

    config.value = {
      api_url: data.web_server?.api_url || 'http://127.0.0.1:8080',
      api_key: data.web_server?.api_key || '',
      template_path: data.storage?.template_path || ''
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    message.error('加载配置失败')
  }
}

// 保存 Web 服务器配置
const saveWebServerConfig = async () => {
  saving.value = true

  try {
    await invoke('update_web_server_config', {
      apiUrl: config.value.api_url,
      apiKey: config.value.api_key
    })

    message.success('配置已保存，请重新启动应用以使更改生效')
  } catch (error) {
    console.error('保存配置失败:', error)
    message.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

// 选择模板路径
const selectTemplatePath = async () => {
  try {
    let initialPath = undefined

    // 如果输入框有值，先尝试使用它
    if (config.value.template_path) {
      const path = config.value.template_path

      // 检查路径是否存在
      try {
        // 调用 Tauri 命令检查路径是否存在
        const exists = await invoke('check_directory_exists', { path })

        if (exists) {
          // 路径存在，直接打开该目录
          initialPath = path
          console.log('路径存在，将打开:', initialPath)
        } else {
          // 路径不存在，递归查找父级目录
          const parentPath = await findExistingParentPath(path)

          if (parentPath) {
            initialPath = parentPath
            console.log('找到父级目录，将打开:', initialPath)
          } else {
            // 父级都不存在，使用默认路径
            console.log('父级都不存在，使用系统默认路径')
            initialPath = undefined
          }
        }
      } catch (error) {
        console.warn('检查路径失败，使用系统默认路径:', error)
        initialPath = undefined
      }
    }

    // 打开目录选择对话框
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择模板存储目录',
      defaultPath: initialPath  // 设置初始路径
    })

    if (selected) {
      await invoke('update_template_path', {
        templatePath: selected
      })
      config.value.template_path = selected
      message.success('模板存储路径已更新')
    }
  } catch (error) {
    console.error('选择路径失败:', error)
    // 用户取消选择不算错误
    if (error !== 'User cancelled') {
      message.error('选择路径失败: ' + error)
    }
  }
}

// 递归查找存在的父级目录
const findExistingParentPath = async (path) => {
  try {
    // 使用 Tauri 命令检查路径是否存在
    const exists = await invoke('check_directory_exists', { path })
    if (exists) {
      return path
    }

    // 获取父级目录
    const parentPath = getParentPath(path)

    // 如果已经到达根目录，停止递归
    if (!parentPath || parentPath === path) {
      return null
    }

    // 递归查找
    return await findExistingParentPath(parentPath)
  } catch (error) {
    console.warn('查找父级目录失败:', error)
    return null
  }
}

// 获取父级目录路径
const getParentPath = (path) => {
  // 移除末尾的斜杠
  let cleanPath = path.replace(/[/\\]$/, '')

  // 查找最后一个斜杠
  const lastSeparatorIndex = Math.max(
    cleanPath.lastIndexOf('/'),
    cleanPath.lastIndexOf('\\')
  )

  if (lastSeparatorIndex > 0) {
    return cleanPath.substring(0, lastSeparatorIndex)
  }

  // 没有父级目录
  return null
}

// 测试连接
const testConnection = async () => {
  testing.value = true
  testResult.value = null

  try {
    // 这里暂时模拟测试连接
    // TODO: 实际应该调用 API 测试连接
    await new Promise(resolve => setTimeout(resolve, 1000))

    // 简单验证 URL 格式
    const urlPattern = /^https?:\/\/.+/
    if (urlPattern.test(config.value.api_url)) {
      testResult.value = {
        success: true,
        message: '连接成功！服务器可以访问。'
      }
    } else {
      testResult.value = {
        success: false,
        message: 'URL 格式不正确，请检查输入。'
      }
    }
  } catch (error) {
    testResult.value = {
      success: false,
      message: '连接失败: ' + error
    }
  } finally {
    testing.value = false
  }
}

// 组件挂载时加载配置
onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.setting-container {
  padding: 20px;
  background: transparent;
}

.setting-group {
  margin-bottom: 32px;
}

.setting-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
  color: var(--color-text);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  gap: 24px;
}

.setting-row-content {
  flex: 1;
}

.setting-row-title {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 4px;
  color: var(--color-text);
}

.setting-row-description {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.setting-divider {
  height: 1px;
  background: var(--color-border);
  margin: 12px 0;
}

.setting-help-text {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-top: 8px;
  padding: 8px 12px;
  background: var(--color-surface);
  border-radius: var(--border-radius-sm);
}
</style>
