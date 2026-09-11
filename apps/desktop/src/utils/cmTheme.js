import { watch } from 'vue'
import { dracula } from '@uiw/codemirror-theme-dracula'
import { useThemeStore } from '@/stores/theme'

/**
 * CodeMirror 明暗主题共享工具
 *
 * 用法:
 *   extensions: [basicSetup, ...cmThemeExtensions(), ...]
 *   watchCmTheme(() => initEditor())   // 主题切换时重建编辑器
 */

/** 当前主题下的编辑器扩展:暗色挂 dracula(与主编辑器一致),亮色默认 */
export function cmThemeExtensions() {
  const themeStore = useThemeStore()
  return themeStore.isDark ? [dracula] : []
}

/** 监听明暗切换,触发编辑器重建;返回取消函数 */
export function watchCmTheme(reinit) {
  const themeStore = useThemeStore()
  return watch(() => themeStore.isDark, () => reinit())
}
