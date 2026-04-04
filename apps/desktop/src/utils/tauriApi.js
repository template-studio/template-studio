import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'

export const tauriApi = {
  window: {
    minimize: () => invoke('window_minimize'),
    maximize: () => invoke('window_maximize'),
    close: () => invoke('window_close')
  },

  system: {
    getPlatform: () => invoke('get_platform'),
    getVersion: getVersion
  },

  theme: {
    getSystemTheme: () => invoke('get_system_theme')
  }
}
