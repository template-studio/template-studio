import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useNavigationStore = defineStore('navigation', () => {
  // State
  const currentRoute = ref('/')
  const navigationHistory = ref([])

  // Computed
  const canGoBack = computed(() => navigationHistory.value.length > 1)
  const canGoForward = computed(() => false) // Simple implementation

  // Actions
  const navigateTo = (route) => {
    if (route !== currentRoute.value) {
      navigationHistory.value.push(currentRoute.value)
      currentRoute.value = route
    }
  }

  const goBack = () => {
    if (canGoBack.value) {
      const previousRoute = navigationHistory.value.pop()
      currentRoute.value = previousRoute
    }
  }

  const clearHistory = () => {
    navigationHistory.value = []
  }

  const setCurrentRoute = (route) => {
    currentRoute.value = route
  }

  return {
    // State
    currentRoute,
    navigationHistory,

    // Computed
    canGoBack,
    canGoForward,

    // Actions
    navigateTo,
    goBack,
    clearHistory,
    setCurrentRoute
  }
})