/**
 * useSchemaStore - Schema 数据共享机制
 *
 * 功能：
 * 1. 管理快速模式和高级模式的共享 Schema 数据
 * 2. 独立草稿系统（localStorage）
 * 3. 数据同步和版本控制
 * 4. 统计数据收集
 */

import { ref, computed, watch } from 'vue'
import { getTemplateExpose, setTemplateExpose } from '@/api/editor/templateExpose'

/**
 * 存储键常量
 */
const STORAGE_KEYS = {
  QUICK_DRAFT: 'template_studio_quick_draft_',     // 快速模式草稿
  ADVANCED_DRAFT: 'template_studio_advanced_draft_', // 高级模式草稿
  ANALYTICS: 'template_studio_analytics_'          // 统计数据
}

/**
 * 全局单例 Store
 */
let globalStore = null

/**
 * 获取 Schema Store 实例（单例模式）
 */
export function useSchemaStore(templateId) {
  if (!globalStore) {
    globalStore = createSchemaStore(templateId)
  }
  return globalStore
}

/**
 * 创建 Schema Store
 */
function createSchemaStore(initialTemplateId) {
  // ========== 状态 ==========

  // 当前模板 ID（使用 ref 以支持动态更新）
  const currentTemplateId = ref(initialTemplateId)

  // 唯一数据源：服务器 Schema（权威版本）
  const serverSchema = ref({})

  // 快速模式草稿
  const quickDraft = ref({})

  // 高级模式草稿
  const advancedDraft = ref({})

  // 当前活动模式
  const activeMode = ref(null) // 'quick' | 'advanced' | null

  // 加载状态
  const loading = ref(false)
  const saving = ref(false)

  // 统计数据
  const analytics = ref({
    // 模式使用次数
    quickModeOpens: 0,
    advancedModeOpens: 0,

    // 保存成功率
    quickModeSaveSuccess: 0,
    quickModeSaveFail: 0,
    advancedModeSaveSuccess: 0,
    advancedModeSaveFail: 0,

    // 平均保存时间（毫秒）
    quickModeAvgTime: 0,
    advancedModeAvgTime: 0,

    // 切换行为
    quickToAdvancedCount: 0,
    advancedToQuickCount: 0,

    // 最后编辑时间
    quickModeLastEdited: null,
    advancedModeLastEdited: null
  })

  // ========== 计算属性 ==========

  /**
   * 当前模式应该使用的 Schema
   */
  const currentSchema = computed(() => {
    if (activeMode.value === 'quick') {
      // 优先使用快速模式草稿，如果没有则使用服务器 Schema
      return Object.keys(quickDraft.value).length > 0
        ? quickDraft.value
        : serverSchema.value
    } else if (activeMode.value === 'advanced') {
      // 优先使用高级模式草稿，如果没有则使用服务器 Schema
      return Object.keys(advancedDraft.value).length > 0
        ? advancedDraft.value
        : serverSchema.value
    }
    return serverSchema.value
  })

  /**
   * 是否有未保存的草稿
   */
  const hasUnsavedDraft = computed(() => {
    if (activeMode.value === 'quick') {
      return Object.keys(quickDraft.value).length > 0 &&
             JSON.stringify(quickDraft.value) !== JSON.stringify(serverSchema.value)
    } else if (activeMode.value === 'advanced') {
      return Object.keys(advancedDraft.value).length > 0 &&
             JSON.stringify(advancedDraft.value) !== JSON.stringify(serverSchema.value)
    }
    return false
  })

  /**
   * 快速模式使用率
   */
  const quickModeUsageRate = computed(() => {
    const total = analytics.value.quickModeOpens + analytics.value.advancedModeOpens
    return total > 0
      ? (analytics.value.quickModeOpens / total * 100).toFixed(2)
      : '0.00'
  })

  /**
   * 快速模式保存成功率
   */
  const quickModeSuccessRate = computed(() => {
    const total = analytics.value.quickModeSaveSuccess + analytics.value.quickModeSaveFail
    return total > 0
      ? (analytics.value.quickModeSaveSuccess / total * 100).toFixed(2)
      : '0.00'
  })

  // ========== 方法 ==========

  /**
   * 从服务器加载 Schema
   */
  const loadServerSchema = async () => {
    if (!currentTemplateId.value) {
      console.log('⚠️ templateId 为空，跳过加载')
      return
    }

    loading.value = true
    try {
      console.log('🔄 开始加载服务器 Schema，templateId:', currentTemplateId.value)
      const response = await getTemplateExpose({ templateId: currentTemplateId.value })

      // axios response.data 是后端返回的完整数据: { code: 0, data: { templateExpose: {...} }, message: "OK" }
      // 我们需要 response.data.data.templateExpose
      const backendData = response.data

      if (!backendData || backendData.code !== 0) {
        console.error('API 返回错误:', backendData?.message)
        serverSchema.value = {}
        return
      }

      const templateExpose = backendData.data?.templateExpose
      if (!templateExpose) {
        console.log('⚠️ 未找到 templateExpose 数据')
        serverSchema.value = {}
        return
      }

      // 尝试多种字段名（兼容新旧格式）
      const schemaJson = templateExpose.vars_schema || templateExpose.fieldSchemaJson

      if (schemaJson) {
        try {
          serverSchema.value = JSON.parse(schemaJson)
          console.log('✓ 成功加载服务器 Schema，字段数量:', Object.keys(serverSchema.value).length)
        } catch (parseError) {
          console.error('解析 Schema JSON 失败:', parseError)
          serverSchema.value = {}
        }
      } else {
        serverSchema.value = {}
        console.log('⚠️ templateExpose 中没有 Schema 数据')
      }
    } catch (error) {
      console.error('加载服务器 Schema 失败:', error)
      serverSchema.value = {}
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * 从 localStorage 加载草稿
   */
  const loadDrafts = () => {
    if (!currentTemplateId.value) return

    // 加载快速模式草稿
    const quickDraftStr = localStorage.getItem(STORAGE_KEYS.QUICK_DRAFT + currentTemplateId.value)
    if (quickDraftStr) {
      try {
        quickDraft.value = JSON.parse(quickDraftStr)
      } catch (error) {
        console.error('加载快速模式草稿失败:', error)
        quickDraft.value = {}
      }
    }

    // 加载高级模式草稿
    const advancedDraftStr = localStorage.getItem(STORAGE_KEYS.ADVANCED_DRAFT + currentTemplateId.value)
    if (advancedDraftStr) {
      try {
        advancedDraft.value = JSON.parse(advancedDraftStr)
      } catch (error) {
        console.error('加载高级模式草稿失败:', error)
        advancedDraft.value = {}
      }
    }

    // 加载统计数据
    const analyticsStr = localStorage.getItem(STORAGE_KEYS.ANALYTICS + currentTemplateId.value)
    if (analyticsStr) {
      try {
        analytics.value = JSON.parse(analyticsStr)
      } catch (error) {
        console.error('加载统计数据失败:', error)
      }
    }
  }

  /**
   * 保存草稿到 localStorage
   */
  const saveDraft = (mode, schema) => {
    if (!currentTemplateId.value) return

    const draftKey = mode === 'quick'
      ? STORAGE_KEYS.QUICK_DRAFT + currentTemplateId.value
      : STORAGE_KEYS.ADVANCED_DRAFT + currentTemplateId.value

    try {
      localStorage.setItem(draftKey, JSON.stringify(schema))

      if (mode === 'quick') {
        quickDraft.value = schema
        analytics.value.quickModeLastEdited = Date.now()
      } else {
        advancedDraft.value = schema
        analytics.value.advancedModeLastEdited = Date.now()
      }
    } catch (error) {
      console.error(`保存${mode}模式草稿失败:`, error)
    }
  }

  /**
   * 保存 Schema 到服务器
   */
  const saveToServer = async (mode, schema) => {
    if (!currentTemplateId.value) return

    saving.value = true
    const startTime = Date.now()

    try {
      // 调用 API 保存到服务器
      await setTemplateExpose({
        templateId: parseInt(currentTemplateId.value),
        varsSchema: schema
      })

      // 更新服务器 Schema
      serverSchema.value = { ...schema }

      // 清除当前模式的草稿
      const draftKey = mode === 'quick'
        ? STORAGE_KEYS.QUICK_DRAFT + currentTemplateId.value
        : STORAGE_KEYS.ADVANCED_DRAFT + currentTemplateId.value
      localStorage.removeItem(draftKey)

      if (mode === 'quick') {
        quickDraft.value = {}
        analytics.value.quickModeSaveSuccess++
      } else {
        advancedDraft.value = {}
        analytics.value.advancedModeSaveSuccess++
      }

      // 更新平均保存时间
      const elapsed = Date.now() - startTime
      if (mode === 'quick') {
        const totalTime = analytics.value.quickModeAvgTime * (analytics.value.quickModeSaveSuccess - 1) + elapsed
        analytics.value.quickModeAvgTime = Math.round(totalTime / analytics.value.quickModeSaveSuccess)
      } else {
        const totalTime = analytics.value.advancedModeAvgTime * (analytics.value.advancedModeSaveSuccess - 1) + elapsed
        analytics.value.advancedModeAvgTime = Math.round(totalTime / analytics.value.advancedModeSaveSuccess)
      }

      // 保存统计数据
      saveAnalytics()

      return { success: true }
    } catch (error) {
      console.error('保存到服务器失败:', error)

      if (mode === 'quick') {
        analytics.value.quickModeSaveFail++
      } else {
        analytics.value.advancedModeSaveFail++
      }

      saveAnalytics()

      return { success: false, error }
    } finally {
      saving.value = false
    }
  }

  /**
   * 保存统计数据到 localStorage
   */
  const saveAnalytics = () => {
    if (!currentTemplateId.value) return
    try {
      localStorage.setItem(STORAGE_KEYS.ANALYTICS + currentTemplateId.value, JSON.stringify(analytics.value))
    } catch (error) {
      console.error('保存统计数据失败:', error)
    }
  }

  /**
   * 切换模式
   */
  const switchMode = (newMode) => {
    if (activeMode.value && activeMode.value !== newMode) {
      // 记录切换行为
      if (activeMode.value === 'quick' && newMode === 'advanced') {
        analytics.value.quickToAdvancedCount++
      } else if (activeMode.value === 'advanced' && newMode === 'quick') {
        analytics.value.advancedToQuickCount++
      }
      saveAnalytics()
    }

    activeMode.value = newMode

    // 记录打开次数
    if (newMode === 'quick') {
      analytics.value.quickModeOpens++
    } else if (newMode === 'advanced') {
      analytics.value.advancedModeOpens++
    }
    saveAnalytics()
  }

  /**
   * 清除草稿
   */
  const clearDrafts = () => {
    if (!currentTemplateId.value) return

    localStorage.removeItem(STORAGE_KEYS.QUICK_DRAFT + currentTemplateId.value)
    localStorage.removeItem(STORAGE_KEYS.ADVANCED_DRAFT + currentTemplateId.value)
    quickDraft.value = {}
    advancedDraft.value = {}
  }

  /**
   * 重置统计数据
   */
  const resetAnalytics = () => {
    analytics.value = {
      quickModeOpens: 0,
      advancedModeOpens: 0,
      quickModeSaveSuccess: 0,
      quickModeSaveFail: 0,
      advancedModeSaveSuccess: 0,
      advancedModeSaveFail: 0,
      quickModeAvgTime: 0,
      advancedModeAvgTime: 0,
      quickToAdvancedCount: 0,
      advancedToQuickCount: 0,
      quickModeLastEdited: null,
      advancedModeLastEdited: null
    }
    saveAnalytics()
  }

  /**
   * 获取统计数据报告
   */
  const getAnalyticsReport = () => {
    return {
      ...analytics.value,
      quickModeUsageRate: quickModeUsageRate.value,
      quickModeSuccessRate: quickModeSuccessRate.value,
      hasQuickDraft: Object.keys(quickDraft.value).length > 0,
      hasAdvancedDraft: Object.keys(advancedDraft.value).length > 0
    }
  }

  /**
   * 设置模板 ID（支持动态切换模板）
   */
  const setTemplateId = (newId) => {
    if (newId !== currentTemplateId.value) {
      console.log('🔄 切换模板 ID:', currentTemplateId.value, '→', newId)
      currentTemplateId.value = newId
      // 清空旧数据
      serverSchema.value = {}
      quickDraft.value = {}
      advancedDraft.value = {}
    }
  }

  // ========== 初始化 ==========

  // 监听 currentTemplateId 变化，重新加载数据
  watch(currentTemplateId, async (newId) => {
    if (newId) {
      await loadServerSchema()
      loadDrafts()
    }
  }, { immediate: true })

  // 返回接口
  return {
    // 状态
    serverSchema,
    quickDraft,
    advancedDraft,
    activeMode,
    loading,
    saving,
    analytics,

    // 计算属性
    currentSchema,
    hasUnsavedDraft,
    quickModeUsageRate,
    quickModeSuccessRate,

    // 方法
    loadServerSchema,
    loadDrafts,
    saveDraft,
    saveToServer,
    switchMode,
    clearDrafts,
    resetAnalytics,
    getAnalyticsReport,
    setTemplateId  // 新增：支持动态切换模板
  }
}
