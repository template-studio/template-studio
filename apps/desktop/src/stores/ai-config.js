/**
 * AI 配置 Store
 * 管理 AI 提供商和模型的配置
 */

import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'

export const useAIConfigStore = defineStore('ai-config', {
  state: () => ({
    // AI 提供商列表
    providers: [],
    // 当前选中的提供商
    selectedProvider: null,
    // 加载状态
    loading: false,
    // 默认提供商
    defaultProvider: null,
    // 全局设置
    globalSettings: {
      defaultProvider: '',
      autoRetry: true,
      maxRetries: 3
    }
  }),

  getters: {
    // 获取启用的提供商
    enabledProviders: (state) => {
      return state.providers.filter(p => p.isEnabled)
    },

    // 根据 providerName 获取提供商
    getProviderByName: (state) => (providerName) => {
      return state.providers.find(p => p.providerName === providerName)
    },

    // 获取默认提供商
    getDefaultProvider: (state) => {
      if (state.defaultProvider) {
        return state.providers.find(p => p.providerName === state.defaultProvider)
      }
      // 如果没有设置默认提供商，返回第一个启用的提供商
      return state.enabledProviders[0] || null
    }
  },

  actions: {
    // 初始化配置
    async initialize() {
      await this.loadAllProviders()
    },

    // 加载所有提供商
    async loadAllProviders() {
      this.loading = true
      try {
        const result = await invoke('ai_get_all_providers')
        this.providers = JSON.parse(result)

        // 设置默认提供商
        const defaultProvider = this.providers.find(p => p.isDefault)
        if (defaultProvider) {
          this.defaultProvider = defaultProvider.providerName
        }
      } catch (error) {
        console.error('加载 AI 提供商失败:', error)
        message.error('加载 AI 提供商失败: ' + error)
      } finally {
        this.loading = false
      }
    },

    // 获取单个提供商
    async loadProvider(providerName) {
      try {
        const result = await invoke('ai_get_provider', { providerName })
        const provider = JSON.parse(result)
        this.selectedProvider = provider
        return provider
      } catch (error) {
        console.error('获取 AI 提供商失败:', error)
        message.error('获取 AI 提供商失败: ' + error)
        return null
      }
    },

    // 保存提供商配置
    async saveProviderConfig(providerConfig) {
      try {
        const result = await invoke('ai_save_provider', {
          params: providerConfig
        })

        // 更新本地状态
        const index = this.providers.findIndex(
          p => p.providerName === providerConfig.providerName
        )
        if (index !== -1) {
          this.providers[index] = {
            ...this.providers[index],
            ...providerConfig
          }
        } else {
          this.providers.push(providerConfig)
        }

        message.success(result || '配置已保存')
        return true
      } catch (error) {
        message.error('保存失败: ' + error)
        return false
      }
    },

    // 切换启用状态
    async toggleProvider(providerName, enabled) {
      try {
        await invoke('ai_toggle_provider', {
          providerName,
          enabled
        })

        const provider = this.getProviderByName(providerName)
        if (provider) {
          provider.isEnabled = enabled
        }

        message.success(`提供商已${enabled ? '启用' : '禁用'}`)
        return true
      } catch (error) {
        message.error('切换失败: ' + error)
        return false
      }
    },

    // 删除提供商
    async deleteProvider(providerName) {
      try {
        await invoke('ai_delete_provider', { providerName })

        this.providers = this.providers.filter(p => p.providerName !== providerName)
        message.success('提供商已删除')
        return true
      } catch (error) {
        message.error('删除失败: ' + error)
        return false
      }
    },

    // 获取提供商的模型分组
    async getProviderModelsGrouped(providerName) {
      try {
        const result = await invoke('ai_get_provider_models_grouped', { providerName })
        return JSON.parse(result)
      } catch (error) {
        console.error('获取模型分组失败:', error)
        message.error('获取模型分组失败: ' + error)
        return []
      }
    },

    // 添加模型
    async addModel(modelData) {
      try {
        const modelId = await invoke('ai_add_model', {
          params: modelData
        })
        message.success('模型已添加')
        return modelId
      } catch (error) {
        message.error('添加模型失败: ' + error)
        return null
      }
    },

    // 删除模型
    async deleteModel(modelId) {
      try {
        await invoke('ai_delete_model', { modelId })
        message.success('模型已删除')
        return true
      } catch (error) {
        message.error('删除模型失败: ' + error)
        return false
      }
    },

    // 更新模型
    async updateModel(modelId, modelData) {
      try {
        await invoke('ai_update_model', {
          modelId,
          params: modelData
        })
        message.success('模型已更新')
        return true
      } catch (error) {
        message.error('更新模型失败: ' + error)
        return false
      }
    },

    // 从提供商 API 获取可用模型列表
    async fetchProviderModels(providerName) {
      try {
        const result = await invoke('ai_fetch_models', { providerName })
        return JSON.parse(result)
      } catch (error) {
        message.error('获取模型列表失败: ' + error)
        return []
      }
    },

    // 批量添加模型
    async batchAddModels(providerName, models) {
      try {
        const count = await invoke('ai_batch_add_models', {
          providerName,
          models
        })
        if (count > 0) {
          message.success(`成功添加 ${count} 个模型`)
        } else {
          message.info('没有新模型需要添加')
        }
        return count
      } catch (error) {
        message.error('批量添加模型失败: ' + error)
        return 0
      }
    },

    // 设置默认提供商
    setDefaultProvider(providerName) {
      this.defaultProvider = providerName
      this.globalSettings.defaultProvider = providerName
    }
  }
})
