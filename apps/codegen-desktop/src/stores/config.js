import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useConfigStore = defineStore('config', {
  state: () => ({
    apiURL: 'http://127.0.0.1:8080',
    apiKey: null,
    templatePath: '',
    configLoaded: false
  }),

  getters: {
    // 获取完整的 API 基础 URL
    baseURL: (state) => state.apiURL,

    // 是否已配置 API Key
    hasApiKey: (state) => !!state.apiKey
  },

  actions: {
    // 从后端加载配置
    async loadConfig() {
      try {
        const configJson = await invoke('get_config')
        const data = JSON.parse(configJson)

        this.apiURL = data.web_server?.api_url || 'http://127.0.0.1:8080'
        this.apiKey = data.web_server?.api_key || null
        this.templatePath = data.storage?.template_path || ''
        this.configLoaded = true

        console.log('配置已加载:', {
          apiURL: this.apiURL,
          hasApiKey: this.hasApiKey,
          templatePath: this.templatePath
        })

        return true
      } catch (error) {
        console.error('加载配置失败:', error)
        // 使用默认配置
        this.apiURL = 'http://127.0.0.1:8080'
        this.configLoaded = true
        return false
      }
    },

    // 更新 API URL
    updateAPIURL(url) {
      this.apiURL = url
      // 重新创建 axios 实例需要刷新页面或重新初始化
      window.location.reload()
    },

    // 更新 API Key
    updateAPIKey(key) {
      this.apiKey = key || null
    }
  }
})
