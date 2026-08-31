import { createPinia } from 'pinia'
import { uiSettingsPersistPlugin } from './uiSettings'

const pinia = createPinia()

// uiSettings store 变更自动持久化到 localStorage
pinia.use(uiSettingsPersistPlugin)

export default pinia

// Export store modules individually
export { useLayoutStore } from './layout'
export { useThemeStore } from './theme'
export { useNavigationStore } from './navigation'