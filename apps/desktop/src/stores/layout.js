import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useLayoutStore = defineStore('layout', () => {
  // State
  const sidebarCollapsed = ref(false)
  const windowSize = ref({
    width: window.innerWidth,
    height: window.innerHeight
  })

  // 全局分页 footer 状态
  const footerPagination = ref({
    visible: false,
    current: 1,
    pageSize: 12,
    total: 0
  })

  // Computed
  const isMobile = computed(() => windowSize.value.width < 768)
  const isTablet = computed(() => windowSize.value.width >= 768 && windowSize.value.width < 1024)
  const isDesktop = computed(() => windowSize.value.width >= 1024)

  const sidebarWidth = computed(() => {
    if (isMobile.value) return 0
    return sidebarCollapsed.value ? 'var(--sidebar-width)' : 'var(--sidebar-width-expanded)'
  })

  // Actions
  const toggleSidebar = () => {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  const setSidebarCollapsed = (collapsed) => {
    sidebarCollapsed.value = collapsed
  }

  const updateWindowSize = (size) => {
    windowSize.value = size

    // Auto-collapse sidebar on mobile
    if (isMobile.value) {
      sidebarCollapsed.value = true
    }
  }

  // 需要 footer 的路由
  const footerRoutes = ['/projects', '/datasource', '/languages']

  // 分页 footer 操作
  const showFooterPagination = (total, current = 1, pageSize = 12) => {
    footerPagination.value = { visible: true, current, pageSize, total }
  }

  const hideFooterPagination = () => {
    footerPagination.value.visible = false
  }

  const updateFooterPagination = (data) => {
    Object.assign(footerPagination.value, data)
  }

  // 路由切换时判断是否显示 footer
  const onRouteChange = (path) => {
    if (!footerRoutes.some(r => path.startsWith(r))) {
      hideFooterPagination()
    }
  }

  return {
    // State
    sidebarCollapsed,
    windowSize,
    footerPagination,

    // Computed
    isMobile,
    isTablet,
    isDesktop,
    sidebarWidth,

    // Actions
    toggleSidebar,
    setSidebarCollapsed,
    updateWindowSize,
    showFooterPagination,
    hideFooterPagination,
    updateFooterPagination,
    onRouteChange
  }
})