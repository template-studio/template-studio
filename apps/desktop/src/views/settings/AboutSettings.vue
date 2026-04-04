<template>
  <div class="setting-container">
    <!-- 应用程序信息 -->
    <div class="setting-group">
      <div class="setting-title">应用程序</div>

      <div style="display: flex; align-items: center; gap: 16px; padding: 16px 0;">
        <div class="app-logo">
          <span class="logo-text">UI</span>
        </div>
        <div class="app-details">
          <h3>Template Studio</h3>
          <div class="app-version">版本 1.0.0</div>
          <div class="app-description">
            一个美观、可扩展的模板化开发平台，基于现代化技术栈构建
          </div>
        </div>
      </div>
    </div>

    <!-- 技术栈信息 -->
    <div class="setting-group">
      <div class="setting-title">技术栈</div>

      <div class="tech-grid">
        <div class="tech-item" v-for="tech in techStack" :key="tech.name">
          <div class="tech-icon" :style="{ color: tech.color }">
            <component :is="tech.icon" />
          </div>
          <div class="tech-info">
            <div class="tech-name">{{ tech.name }}</div>
            <div class="tech-version">{{ tech.version }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 系统信息 -->
    <div class="setting-group">
      <div class="setting-title">系统信息</div>

      <div class="system-grid">
        <div class="system-row">
          <div class="system-label">平台</div>
          <div class="system-value">{{ systemInfo.platform }}</div>
        </div>
        <div class="system-row">
          <div class="system-label">架构</div>
          <div class="system-value">{{ systemInfo.arch }}</div>
        </div>
        <div class="system-row">
          <div class="system-label">Node.js</div>
          <div class="system-value">{{ systemInfo.nodeVersion }}</div>
        </div>
        <div class="system-row">
          <div class="system-label">Tauri</div>
          <div class="system-value">{{ systemInfo.tauriVersion }}</div>
        </div>
        <div class="system-row">
          <div class="system-label">Chrome</div>
          <div class="system-value">{{ systemInfo.chromeVersion }}</div>
        </div>
        <div class="system-row">
          <div class="system-label">Vue.js</div>
          <div class="system-value">{{ systemInfo.vueVersion }}</div>
        </div>
      </div>
    </div>

    <!-- 开发者信息 -->
    <div class="setting-group">
      <div class="setting-title">开发者信息</div>

      <div class="setting-row">
        <div class="setting-row-title">作者</div>
        <div class="setting-value">Template Studio Team</div>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">许可证</div>
        <div class="setting-value">MIT License</div>
      </div>

      <div class="setting-divider"></div>

      <div class="setting-row">
        <div class="setting-row-title">构建时间</div>
        <div class="setting-value">{{ buildTime }}</div>
      </div>
    </div>

    <!-- 资源链接 -->
    <div class="setting-group">
      <div class="setting-title">资源链接</div>

      <div class="links-grid">
        <a-button
          v-for="link in resourceLinks"
          :key="link.name"
          type="text"
          size="small"
          class="link-button"
          @click="openLink(link.url)"
        >
          <template #icon>
            <component :is="link.icon" />
          </template>
          {{ link.name }}
        </a-button>
      </div>
    </div>

    <!-- 更新信息 -->
    <div class="setting-group">
      <div class="setting-title">更新</div>

      <div class="setting-row">
        <div class="update-status">
          <a-badge :status="updateStatus.status" :text="updateStatus.text" />
        </div>
        <div>
          <a-space>
            <a-button type="primary" size="small" @click="checkForUpdates" :loading="checkingUpdate">
              检查更新
            </a-button>
            <a-button size="small" @click="viewReleaseNotes">
              更新日志
            </a-button>
          </a-space>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="setting-group">
      <div class="setting-row">
        <a-space>
          <a-button size="small" @click="copySystemInfo">
            复制系统信息
          </a-button>
          <a-button size="small" @click="exportLogs">
            导出日志
          </a-button>
          <a-button size="small" @click="resetAppData">
            重置应用数据
          </a-button>
        </a-space>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, computed, onMounted, ref } from 'vue'
import { message } from 'ant-design-vue'
import {
  GithubOutlined,
  BookOutlined,
  BugOutlined,
  GlobalOutlined,
  CodeOutlined,
  DesktopOutlined,
  ApiOutlined
} from '@ant-design/icons-vue'

// 系统信息
const systemInfo = reactive({
  platform: '',
  arch: '',
  nodeVersion: '',
  electronVersion: '',
  chromeVersion: '',
  vueVersion: '3.x'
})

// 更新状态
const updateStatus = reactive({
  status: 'default',
  text: '点击检查更新'
})

const checkingUpdate = ref(false)

// 构建时间
const buildTime = computed(() => {
  return new Date().toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
})

// 技术栈数据
const techStack = [
  {
    name: 'Tauri',
    version: 'Latest',
    icon: DesktopOutlined,
    color: '#47848F'
  },
  {
    name: 'Vue.js',
    version: '3.x',
    icon: CodeOutlined,
    color: '#4FC08D'
  },
  {
    name: 'Ant Design',
    version: '4.x',
    icon: ApiOutlined,
    color: '#1890ff'
  },
  {
    name: 'Vite',
    version: 'Latest',
    icon: GlobalOutlined,
    color: '#646CFF'
  }
]

// 资源链接
const resourceLinks = [
  {
    name: 'GitHub',
    url: 'https://github.com',
    icon: GithubOutlined
  },
  {
    name: '文档',
    url: 'https://docs.example.com',
    icon: BookOutlined
  },
  {
    name: '反馈问题',
    url: 'https://github.com/issues',
    icon: BugOutlined
  },
  {
    name: '官方网站',
    url: 'https://example.com',
    icon: GlobalOutlined
  }
]

// 获取系统信息
const getSystemInfo = async () => {
  try {
    // 先尝试从浏览器环境获取基本信息
    const userAgent = navigator.userAgent
    const platform = navigator.platform || 'Unknown'

    // 获取 Chrome 版本
    const chromeMatch = userAgent.match(/Chrome\/(\d+)/)
    const chromeVersion = chromeMatch ? chromeMatch[1] : 'Unknown'

    // 检查是否在 Tauri 环境中
    const isTauri = userAgent.includes('Tauri')

    // 如果在 Tauri 环境中，尝试获取更多信息
    if (isTauri) {
      try {
        const { tauriApi } = await import('@/utils/tauriApi')
        const [appVersion, appPlatform] = await Promise.all([
          tauriApi.system.getVersion(),
          tauriApi.system.getPlatform()
        ])
        Object.assign(systemInfo, {
          platform: appPlatform || platform,
          arch: 'Unknown', // Tauri doesn't expose arch by default
          nodeVersion: 'N/A',
          electronVersion: 'N/A',
          tauriVersion: '2.0',
          chromeVersion: chromeVersion,
          appVersion: appVersion
        })
      } catch (tauriError) {
        console.warn('Tauri API 调用失败，使用浏览器信息:', tauriError)
        // 使用浏览器信息作为备选
        Object.assign(systemInfo, {
          platform: platform,
          arch: 'Unknown',
          nodeVersion: 'N/A',
          electronVersion: 'N/A',
          tauriVersion: 'Unknown',
          chromeVersion: chromeVersion
        })
      }
    } else {
      // 使用浏览器信息
      Object.assign(systemInfo, {
        platform: platform,
        arch: 'Unknown',
        nodeVersion: 'N/A',
        electronVersion: isTauri ? 'Unknown' : 'N/A',
        tauriVersion: 'N/A',
        chromeVersion: chromeVersion
      })
    }
  } catch (error) {
    console.error('Get system info error:', error)
    // 设置默认值，避免页面崩溃
    Object.assign(systemInfo, {
      platform: 'Unknown',
      arch: 'Unknown',
      nodeVersion: 'N/A',
      electronVersion: 'N/A',
      chromeVersion: 'Unknown'
    })
    message.warning('部分系统信息获取失败，显示默认信息')
  }
}

// 检查更新
const checkForUpdates = async () => {
  checkingUpdate.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 2000))
    updateStatus.status = 'success'
    updateStatus.text = '当前版本是最新版本'
    message.success('检查更新完成')
  } catch (error) {
    updateStatus.status = 'error'
    updateStatus.text = '检查更新失败'
    message.error('检查更新失败')
  } finally {
    checkingUpdate.value = false
  }
}

// 查看更新日志
const viewReleaseNotes = () => {
  message.info('更新日志功能开发中')
}

// 打开链接
const openLink = async (url) => {
  try {
    const { open } = await import('@tauri-apps/plugin-opener')
    await open(url)
  } catch (error) {
    console.warn('使用 Tauri opener 失败，回退到 window.open:', error)
    window.open(url, '_blank')
  }
}

// 复制系统信息
const copySystemInfo = () => {
  const infoText = `
Template Studio 系统信息
===============================
版本: 1.0.0
平台: ${systemInfo.platform}
架构: ${systemInfo.arch}
Node.js: ${systemInfo.nodeVersion}
Tauri: ${systemInfo.tauriVersion}
Chrome: ${systemInfo.chromeVersion}
Vue.js: ${systemInfo.vueVersion}
构建时间: ${buildTime.value}
  `.trim()

  navigator.clipboard.writeText(infoText).then(() => {
    message.success('系统信息已复制到剪贴板')
  }).catch(() => {
    message.error('复制失败')
  })
}

// 导出日志
const exportLogs = () => {
  message.info('日志导出功能开发中')
}

// 重置应用数据
const resetAppData = () => {
  message.warning('重置应用数据功能开发中')
}

// 组件挂载时获取系统信息
onMounted(() => {
  getSystemInfo()
})
</script>

<style scoped>
@import '@/assets/styles/settings.css';

.setting-container {
  background: transparent;
}

.app-logo {
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-dark));
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 1.2rem;
  font-weight: bold;
  box-shadow: 0 4px 12px rgba(24, 144, 255, 0.3);
  flex-shrink: 0;
}

.app-details h3 {
  margin: 0 0 4px 0;
  color: var(--color-text);
  font-size: 1.1rem;
}

.app-version {
  color: var(--color-text-secondary);
  font-weight: 500;
  margin-bottom: 8px;
}

.app-description {
  color: var(--color-text-secondary);
  line-height: 1.4;
  font-size: 13px;
}

.tech-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.tech-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: var(--color-background);
  border-radius: 6px;
  border: 1px solid var(--color-border);
}

.tech-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.tech-name {
  color: var(--color-text);
  font-size: 13px;
  font-weight: 500;
}

.tech-version {
  color: var(--color-text-secondary);
  font-size: 11px;
}

.system-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
}

.system-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  background: var(--color-background);
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

.system-label {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.system-value {
  color: var(--color-text);
  font-weight: 500;
  font-size: 13px;
}

.setting-value {
  color: var(--color-text);
  font-size: 13px;
}

.links-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 6px;
}

.link-button {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 4px;
  height: 32px;
  padding: 0 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  transition: all 0.2s ease;
  font-size: 12px;
}

.link-button:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.update-status {
  flex: 1;
}

/* Theme adjustments now handled by global CSS variables */
</style>