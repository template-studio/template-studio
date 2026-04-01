import { createPinia } from 'pinia'

const pinia = createPinia()

export default pinia

// Export store modules individually
export { useLayoutStore } from './layout'
export { useThemeStore } from './theme'
export { useNavigationStore } from './navigation'