import { useThemeStore } from '@/stores/theme'

export const ThemePlugin = {
  install(app) {
    // Provide theme store globally
    const themeStore = useThemeStore()

    // Add global properties
    app.config.globalProperties.$theme = themeStore

    // Provide theme store for injection
    app.provide('themeStore', themeStore)

    // Initialize theme system
    themeStore.initializeTheme()
  }
}

// Export a composable for easy access
export const useGlobalTheme = () => {
  return useThemeStore()
}