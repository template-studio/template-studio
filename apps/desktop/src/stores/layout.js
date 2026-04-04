import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useLayoutStore = defineStore('layout', () => {
  // State
  const sidebarCollapsed = ref(false)
  const windowSize = ref({
    width: window.innerWidth,
    height: window.innerHeight
  })

  // 全局 footer 状态
  const footerType = ref(null) // null | 'pagination' | 'overview'

  const footerPagination = ref({
    current: 1,
    pageSize: 12,
    total: 0
  })

  const footerOverview = ref({
    items: [] // [{ label, value, icon? }]
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
    if (isMobile.value) {
      sidebarCollapsed.value = true
    }
  }

  // 需要 footer 的路由
  const footerRoutes = ['/projects', '/datasource', '/languages', '/templates', '/mappings']

  // 分页 footer
  const showFooterPagination = (total, current = 1, pageSize = 12) => {
    footerType.value = 'pagination'
    footerPagination.value = { current, pageSize, total }
  }

  const updateFooterPagination = (data) => {
    Object.assign(footerPagination.value, data)
  }

  // 概览 footer
  const showFooterOverview = (items) => {
    footerType.value = 'overview'
    footerOverview.value = { items }
  }

  // 隐藏 footer
  const hideFooter = () => {
    footerType.value = null
  }

  // 路由切换时判断是否显示 footer
  const onRouteChange = (path) => {
    if (!footerRoutes.some(r => path.startsWith(r))) {
      hideFooter()
    }
  }

  return {
    // State
    sidebarCollapsed,
    windowSize,
    footerType,
    footerPagination,
    footerOverview,

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
    updateFooterPagination,
    showFooterOverview,
    hideFooter,
    onRouteChange
  }
})