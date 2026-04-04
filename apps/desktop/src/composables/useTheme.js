import { computed, watch, onMounted } from 'vue'
import { useThemeStore } from '@/stores/theme'

export function useTheme() {
  const themeStore = useThemeStore()

  // Computed properties
  const currentTheme = computed(() => themeStore.currentTheme)
  const isDark = computed(() => themeStore.isDark)
  const isLight = computed(() => themeStore.isLight)
  const systemTheme = computed(() => themeStore.systemTheme)
  const themeClass = computed(() => themeStore.themeClass)

  // Theme utilities
  const setTheme = (theme) => {
    themeStore.setTheme(theme)
  }

  const toggleTheme = () => {
    themeStore.toggleTheme()
  }

  const setSystemTheme = () => {
    themeStore.setTheme(themeStore.systemTheme)
  }

  // System theme detection
  const detectSystemTheme = async () => {
    await themeStore.detectSystemTheme()
  }

  // Theme matching utility
  const getContrastColor = (bgColor) => {
    // Simple contrast calculation - could be enhanced
    const colors = {
      light: {
        text: '#212121',
        textSecondary: '#757575',
        border: '#e0e0e0'
      },
      dark: {
        text: '#ffffff',
        textSecondary: '#a0a0a0',
        border: '#333333'
      }
    }
    return colors[currentTheme.value]
  }

  // Theme-aware styling helper
  const getThemeColor = (colorType) => {
    const colorMap = {
      primary: isDark.value ? '#90caf9' : '#1976d2',
      secondary: isDark.value ? '#b0b0b0' : '#424242',
      background: isDark.value ? '#121212' : '#ffffff',
      surface: isDark.value ? '#1e1e1e' : '#f5f5f5',
      text: isDark.value ? '#ffffff' : '#212121',
      textSecondary: isDark.value ? '#a0a0a0' : '#757575',
      border: isDark.value ? '#333333' : '#e0e0e0',
      hover: isDark.value ? 'rgba(144, 202, 249, 0.08)' : 'rgba(25, 118, 210, 0.08)',
      active: isDark.value ? 'rgba(144, 202, 249, 0.12)' : 'rgba(25, 118, 210, 0.12)'
    }
    return colorMap[colorType] || colorMap.text
  }

  // CSS variable helper
  const updateCSSVariable = (name, value) => {
    document.documentElement.style.setProperty(`--${name}`, value)
  }

  // CSS variables are now handled by the global theme system in variables.css
  // No need to manually apply hardcoded values

  // Theme persistence and loading
  const loadThemePreference = () => {
    themeStore.loadTheme()
  }

  const saveThemePreference = (theme) => {
    themeStore.saveTheme(theme)
  }

  // Auto theme based on system preference
  const enableAutoTheme = () => {
    watch(systemTheme, (newSystemTheme) => {
      const savedTheme = localStorage.getItem('app-theme')
      if (!savedTheme) {
        setTheme(newSystemTheme)
      }
    })
  }

  // Initialize on mount
  onMounted(async () => {
    await themeStore.initializeTheme()
    enableAutoTheme()
  })

  return {
    // State
    currentTheme,
    isDark,
    isLight,
    systemTheme,
    themeClass,

    // Methods
    setTheme,
    toggleTheme,
    setSystemTheme,
    detectSystemTheme,

    // Utilities
    getContrastColor,
    getThemeColor,
    updateCSSVariable,
    loadThemePreference,
    saveThemePreference,
    enableAutoTheme
  }
}