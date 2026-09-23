import { useUiSettingsStore } from '@/stores/uiSettings'

/**
 * 显示偏好应用器(#716)
 *
 * 把 uiSettings.display 的值落到文档级 CSS 变量/类名,主题颜色映射到
 * --color-brand 及其衍生槽位;启动时与设置页变更时各调一次 applyDisplay()。
 *
 * 主题色衍生规则:以基色的 0.08/0.14 透明度作 hover/active 底,保证任意
 * 基色下都有可用的交互层次,不依赖逐色调色板。
 */

function hexToRgb(hex) {
  const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(String(hex).trim())
  if (!m) return null
  return { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) }
}

function rgba(hex, alpha) {
  const c = hexToRgb(hex)
  if (!c) return ''
  return `rgba(${c.r}, ${c.g}, ${c.b}, ${alpha})`
}

const SPEED_FACTOR = { slow: 1.6, normal: 1, fast: 0.55 }

export function applyDisplay(display) {
  const d = display || useUiSettingsStore().display
  const root = document.documentElement

  // 主题色:基色 + hover/active 透明度衍生(全局 --color-brand 槽位)
  if (d.primaryColor && hexToRgb(d.primaryColor)) {
    root.style.setProperty('--color-brand', d.primaryColor)
    root.style.setProperty('--color-brand-hover', rgba(d.primaryColor, 0.85))
    root.style.setProperty('--color-brand-active', rgba(d.primaryColor, 0.75))
    root.style.setProperty('--color-brand-bg', rgba(d.primaryColor, 0.08))
    root.style.setProperty('--color-brand-bg-strong', rgba(d.primaryColor, 0.14))
    root.style.setProperty('--editor-accent', d.primaryColor)
  }

  // 基础字号(rem 基准)
  root.style.setProperty('--base-font-size', `${d.fontSize || 14}px`)

  // 紧凑模式:类名开关,间距变量在 variables.css 里按类切换
  root.classList.toggle('compact-mode', !!d.compactMode)

  // 动画:禁用→全局 off;速度→统一时长变量(现有 transition 多为 0.2s 档)
  const factor = SPEED_FACTOR[d.animationSpeed] ?? 1
  root.style.setProperty('--ui-speed-factor', String(factor))
  root.classList.toggle('no-animations', d.enableAnimations === false)
  root.style.setProperty('--ui-transition-base', `${0.2 * factor}s`)
}

/** 侧边栏显隐由 layout store 消费(uiSettings.display.showSidebar 为持久值) */
export function sidebarVisibleFromSettings() {
  return useUiSettingsStore().display.showSidebar !== false
}
