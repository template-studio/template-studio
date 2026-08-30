/**
 * 简单防抖函数（项目内共享实现，替代为单个函数引入 lodash-es 依赖）
 */
// eslint-disable-next-line no-unused-vars
export function debounce<T extends (...args: any[]) => void>(func: T, wait: number) {
  let timeout: ReturnType<typeof setTimeout> | undefined
  return function executedFunction(this: unknown, ...args: Parameters<T>) {
    const later = () => {
      clearTimeout(timeout)
      func.apply(this, args)
    }
    clearTimeout(timeout)
    timeout = setTimeout(later, wait)
  }
}
