import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useLayoutStore } from '@/stores/layout'

export function useLayout() {
  const layoutStore = useLayoutStore()
  const isResizing = ref(false)

  // Computed properties
  const isMobile = computed(() => layoutStore.isMobile)
  const isTablet = computed(() => layoutStore.isTablet)
  const isDesktop = computed(() => layoutStore.isDesktop)
  const sidebarWidth = computed(() => layoutStore.sidebarWidth)
  const sidebarCollapsed = computed(() => layoutStore.sidebarCollapsed)

  // Layout utilities
  const toggleSidebar = () => {
    layoutStore.toggleSidebar()
  }

  const setSidebarCollapsed = (collapsed) => {
    layoutStore.setSidebarCollapsed(collapsed)
  }

  // Window resize handling with debouncing
  let resizeTimeout = null

  const handleResize = () => {
    isResizing.value = true

    // Clear existing timeout
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }

    // Debounce resize handling
    resizeTimeout = setTimeout(() => {
      layoutStore.updateWindowSize({
        width: window.innerWidth,
        height: window.innerHeight
      })
      isResizing.value = false
    }, 200)
  }

  // Responsive breakpoints
  const getBreakpoint = (width) => {
    if (width < 768) return 'mobile'
    if (width < 1024) return 'tablet'
    return 'desktop'
  }

  const currentBreakpoint = computed(() =>
    getBreakpoint(layoutStore.windowSize.width)
  )

  // Content area calculations
  const contentWidth = computed(() => {
    const sidebar = sidebarCollapsed.value ? 60 : 240
    return layoutStore.windowSize.width - sidebar
  })

  const contentHeight = computed(() => {
    return layoutStore.windowSize.height - 48 // Subtract navbar height
  })

  // Layout helpers
  const shouldCollapseSidebar = computed(() => {
    return isMobile.value || layoutStore.windowSize.width < 900
  })

  // Grid system helpers
  const getGridColumns = (columns = 12) => {
    if (isMobile.value) return Math.min(columns, 1)
    if (isTablet.value) return Math.min(columns, 2)
    return columns
  }

  const getGridGutter = () => {
    if (isMobile.value) return 8
    if (isTablet.value) return 16
    return 24
  }

  // Lifecycle
  onMounted(() => {
    window.addEventListener('resize', handleResize)
    // Initialize window size
    handleResize()
  })

  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
  })

  return {
    // State
    isResizing,

    // Computed
    isMobile,
    isTablet,
    isDesktop,
    sidebarWidth,
    sidebarCollapsed,
    currentBreakpoint,
    contentWidth,
    contentHeight,
    shouldCollapseSidebar,

    // Methods
    toggleSidebar,
    setSidebarCollapsed,
    handleResize,
    getBreakpoint,
    getGridColumns,
    getGridGutter
  }
}