import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  // State
  const currentTheme = ref('light')
  const systemTheme = ref('light')

  // Computed
  const isDark = computed(() => currentTheme.value === 'dark')
  const isLight = computed(() => currentTheme.value === 'light')
  const themeClass = computed(() => `theme-${currentTheme.value}`)

  // Actions
  const setTheme = (theme) => {
    if (['light', 'dark'].includes(theme)) {
      currentTheme.value = theme
      applyTheme(theme)
      saveTheme(theme)
      // Note: Tauri doesn't need IPC notification for theme changes
    }
  }

  const toggleTheme = () => {
    const newTheme = currentTheme.value === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
  }

  const setSystemTheme = (theme) => {
    systemTheme.value = theme
  }

  const applyTheme = (theme) => {
    document.documentElement.setAttribute('data-theme', theme)
  }

  const saveTheme = (theme) => {
    try {
      localStorage.setItem('app-theme', theme)
    } catch (error) {
      console.warn('Failed to save theme preference:', error)
    }
  }

  const loadTheme = () => {
    try {
      const saved = localStorage.getItem('app-theme')
      if (saved && ['light', 'dark'].includes(saved)) {
        currentTheme.value = saved
        applyTheme(saved)
      }
    } catch (error) {
      console.warn('Failed to load theme preference:', error)
    }
  }

  const detectSystemTheme = async () => {
    try {
      const { tauriApi } = await import('@/utils/tauriApi')
      const theme = await tauriApi.theme.getSystemTheme()
      setSystemTheme(theme)
      return theme
    } catch (error) {
      console.warn('Failed to detect system theme:', error)
      return 'light'
    }
  }

  // Initialize theme on store creation
  const initializeTheme = async () => {
    await detectSystemTheme()
    loadTheme()
  }

  // Watch for system theme changes
  watch(systemTheme, (newSystemTheme) => {
    // If user hasn't explicitly set a theme, follow system
    const savedTheme = localStorage.getItem('app-theme')
    if (!savedTheme) {
      setTheme(newSystemTheme)
    }
  })

  return {
    // State
    currentTheme,
    systemTheme,

    // Computed
    isDark,
    isLight,
    themeClass,

    // Actions
    setTheme,
    toggleTheme,
    setSystemTheme,
    applyTheme,
    loadTheme,
    detectSystemTheme,
    initializeTheme
  }
})