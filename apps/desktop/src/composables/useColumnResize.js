import { ref } from 'vue'

/**
 * 列宽拖拽 composable（设计器各栏统一，#200 补20）
 *
 * 统一此前三份手写 mousedown/mousemove/mouseup 拷贝：方向语义、硬边界、
 * 动态上限（容器可用宽）、localStorage 持久化、body 光标/选区抑制全部收敛于此。
 *
 * @param {Object} opts
 * @param {string}  opts.key            localStorage 持久化键（空串则不持久化）
 * @param {number}  opts.initial        初始宽（px）
 * @param {number}  opts.min            硬下限
 * @param {number}  opts.max            硬上限
 * @param {'left'|'right'} opts.side    手柄所在边：left=左拖变宽，right=右拖变宽
 * @param {() => number} [opts.getDynamicMax] 动态上限（容器可用宽等），与 max 取小
 *
 * @returns {{ width: import('vue').Ref<number>, start: (e: MouseEvent) => void }}
 */
export function useColumnResize(opts) {
  const { key = '', initial, min, max, side = 'left', getDynamicMax = null } = opts

  const clamp = (w) => {
    let hi = max
    if (getDynamicMax) hi = Math.min(hi, Math.max(min, getDynamicMax()))
    return Math.min(hi, Math.max(min, w))
  }

  // 初始化只用硬边界:容器可能处于 display:none(v-show 工作室层)导致动态上限为 0,
  // 会把初始宽误压到 min;动态上限在拖拽时(容器必然可见)才参与
  const hardClamp = (w) => Math.min(max, Math.max(min, w))

  const stored = key ? parseFloat(localStorage.getItem(key)) : NaN
  const width = ref(Number.isFinite(stored) ? hardClamp(stored) : hardClamp(initial))

  const start = (e) => {
    e.preventDefault()
    const startX = e.clientX
    const startW = width.value

    const onMove = (ev) => {
      const delta = ev.clientX - startX
      width.value = clamp(startW + (side === 'left' ? -delta : delta))
    }
    const onUp = () => {
      document.removeEventListener('mousemove', onMove)
      document.removeEventListener('mouseup', onUp)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
      if (key) localStorage.setItem(key, String(width.value))
    }

    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    document.addEventListener('mousemove', onMove)
    document.addEventListener('mouseup', onUp)
  }

  return { width, start }
}
