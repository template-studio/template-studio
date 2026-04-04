import { ref, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'

export function useSettingsNavigation() {
  const router = useRouter()
  const route = useRoute()

  // 菜单状态管理
  const currentMainTab = ref('general')
  const currentSubTab = ref('basic')
  const currentThirdTab = ref('debug')

  // 计算属性
  const showSubSidebar = computed(() => {
    return currentMainTab.value === 'general' ||
           currentMainTab.value === 'advanced' ||
           currentMainTab.value === 'ai-services'
  })

  const showThirdSidebar = computed(() => {
    return currentSubTab.value === 'developer'
  })

  // 切换函数
  const switchMainTab = (tab) => {
    if (currentMainTab.value !== tab) {
      currentMainTab.value = tab

      // 重置子菜单
      if (tab === 'general') {
        currentSubTab.value = 'basic'
        currentThirdTab.value = 'debug'
      } else if (tab === 'web-server') {
        currentSubTab.value = ''
        currentThirdTab.value = ''
      } else if (tab === 'advanced') {
        currentSubTab.value = 'security'
        currentThirdTab.value = 'debug'
      } else if (tab === 'ai-services') {
        currentSubTab.value = 'default-service'
        currentThirdTab.value = ''
      } else if (tab === 'about') {
        currentSubTab.value = ''
        currentThirdTab.value = ''
      }

      updateURL(false)
    }
  }

  const switchSubTab = (tab) => {
    if (currentSubTab.value !== tab) {
      currentSubTab.value = tab

      // 重置三级菜单
      if (tab !== 'developer') {
        currentThirdTab.value = 'debug'
      }

      updateURL(false)
    }
  }

  const switchThirdTab = (tab) => {
    if (currentThirdTab.value !== tab) {
      currentThirdTab.value = tab
      updateURL(false)
    }
  }

  // 更新URL但不触发路由监听
  const updateURL = (notifyRouter = true) => {
    let url = '/settings'

    if (currentMainTab.value === 'general') {
      url += '/general'
      if (currentSubTab.value) {
        url += '/' + currentSubTab.value
      }
    } else if (currentMainTab.value === 'web-server') {
      url += '/web-server'
    } else if (currentMainTab.value === 'advanced') {
      url += '/advanced'
      if (currentSubTab.value) {
        url += '/' + currentSubTab.value
        if (currentThirdTab.value && currentSubTab.value === 'developer') {
          url += '/' + currentThirdTab.value
        }
      }
    } else if (currentMainTab.value === 'ai-services') {
      url += '/ai-services'
      if (currentSubTab.value) {
        url += '/' + currentSubTab.value
      }
    } else if (currentMainTab.value === 'about') {
      url += '/about'
    }

    if (notifyRouter) {
      router.replace(url)
    } else {
      window.history.replaceState({}, '', url)
    }
  }

  // 监听路由变化，同步状态（用于浏览器后退/前进）
  watch(() => route.path, (newPath) => {
    if (newPath.startsWith('/settings')) {
      const pathParts = newPath.replace('/settings', '').split('/').filter(Boolean)

      if (pathParts.length === 0) {
        switchMainTab('general')
      } else {
        currentMainTab.value = pathParts[0] || 'general'

        if (pathParts[1]) {
          currentSubTab.value = pathParts[1]

          if (pathParts[2] && pathParts[1] === 'developer') {
            currentThirdTab.value = pathParts[2]
          }
        } else {
          currentSubTab.value = ''
          currentThirdTab.value = 'debug'
        }
      }
    }
  }, { immediate: true })

  // 初始化导航状态
  const initializeNavigation = () => {
    const pathParts = route.path.replace('/settings', '').split('/').filter(Boolean)

    if (pathParts.length === 0) {
      switchMainTab('general')
    } else {
      currentMainTab.value = pathParts[0] || 'general'
      currentSubTab.value = pathParts[1] || ''
      currentThirdTab.value = pathParts[2] || 'debug'
    }
  }

  return {
    // 状态
    currentMainTab,
    currentSubTab,
    currentThirdTab,
    showSubSidebar,
    showThirdSidebar,

    // 方法
    switchMainTab,
    switchSubTab,
    switchThirdTab,
    updateURL,
    initializeNavigation
  }
}