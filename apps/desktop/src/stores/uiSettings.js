import { defineStore } from 'pinia'

// 设置页 UI 偏好（安全/网络/行为/调试/实验性）——localStorage 持久化，与主题 store 同模式。
// 注意：部分开关（如沙盒/CSP/代理）仅存储偏好值，生效逻辑待对应功能实现时接线。
const STORAGE_KEY = 'ui-settings-v1'

const DEFAULTS = {
  security: {
    enableCSP: true,
    blockExternalLinks: false,
    enableSandbox: true,
  },
  network: {
    proxyMode: 'system',
    requestTimeout: '30',
    maxConnections: 10,
  },
  behavior: {
    doubleClickToClose: false,
    dragToSort: true,
    autoHideToolbar: false,
  },
  debug: {
    debugMode: false,
    showPerformanceMonitor: false,
    logLevel: 'info',
    debugPort: '9222',
  },
  experimental: {
    betaFeatures: false,
    newUIPreview: false,
    performanceMode: false,
  },
}

function loadSaved() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    const saved = JSON.parse(raw)
    return saved && typeof saved === 'object' ? saved : {}
  } catch {
    return {}
  }
}

// 深合并默认值与已存值（新增字段自动补默认，不丢用户已有设置）
function mergeDefaults(defaults, saved) {
  const out = {}
  for (const key of Object.keys(defaults)) {
    const dv = defaults[key]
    const sv = saved[key]
    if (dv !== null && typeof dv === 'object') {
      out[key] = mergeDefaults(dv, sv && typeof sv === 'object' ? sv : {})
    } else {
      out[key] = sv !== undefined ? sv : dv
    }
  }
  return out
}

export const useUiSettingsStore = defineStore('uiSettings', {
  state: () => mergeDefaults(DEFAULTS, loadSaved()),

  actions: {
    // 任意变更后统一落盘（由 main.js 的 pinia 插件 $subscribe 触发，页面无需手动调用）
    persist() {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.$state))
      } catch (error) {
        console.error('UI 设置保存失败:', error)
      }
    },
    resetAll() {
      this.$patch(structuredClone(DEFAULTS))
    },
  },
})

// pinia 插件：该 store 任意状态变更自动落盘（在 main.js 经 pinia.use 注册）
export function uiSettingsPersistPlugin({ store }) {
  if (store.$id === 'uiSettings') {
    store.$subscribe(() => store.persist(), { detached: true })
  }
}
