import axios from 'axios'
import qs from 'qs'
import { message } from 'ant-design-vue'
import { useConfigStore } from '@/stores/config'

// 编辑器 API 客户端：语义与 web 端 utils/request.ts 对齐（token 头 + code!==0 拦截），
// 供移植自 web 编辑器的模块使用。旧 utils/request.js（信封透传）继续服务存量模块，
// 两套契约并存直至桌面端整体迁移（见 dev-docs/desktop-editor-plan.md 决策 1）。
const apiRequest = axios.create({
  baseURL: 'http://127.0.0.1:8080',
  timeout: 50000,
  headers: { 'Content-Type': 'application/json' },
  paramsSerializer: {
    serialize(params) {
      return qs.stringify(params, { allowDots: true, arrayFormat: 'brackets' })
    },
  },
})

// 请求拦截器：动态 baseURL + token 头（后端只认 token 头，非 Bearer）
apiRequest.interceptors.request.use(
  (config) => {
    const configStore = useConfigStore()
    if (configStore.configLoaded) {
      config.baseURL = configStore.baseURL
    }
    if (configStore.apiKey) {
      config.headers['token'] = configStore.apiKey
    }
    return config
  },
  (error) => Promise.reject(error)
)

apiRequest.interceptors.response.use(
  (response) => {
    if (response.config.responseType === 'blob') {
      return response
    }

    const contentType = response.headers['content-type']
    if (contentType && contentType.includes('text/html')) {
      message.error('API请求失败，请检查Web服务器配置')
      return Promise.reject(new Error('Invalid response: received HTML instead of JSON'))
    }

    // 统一信封：仅 code:0 视为成功
    const res = response.data
    if (res.code !== 0) {
      message.error(res.message || '请求失败')
      return Promise.reject(new Error(res.message || '未知错误'))
    }

    return response
  },
  (error) => {
    if (error.response) {
      const { status, data } = error.response
      if (status >= 500) {
        message.error('服务器错误，请稍后重试')
      } else if (status === 404) {
        message.error('API接口不存在，请检查Web服务地址')
      } else if (status === 401) {
        // 桌面端无登录页：提示到设置页配置 API Key，不做跳转
        message.error('未授权：请在「设置 → Web服务器」填写有效的 API Token')
      } else if (status >= 400) {
        message.error(data?.message || '请求错误')
      }
    } else if (error.request) {
      message.error('网络错误：无法连接服务器，请检查Web服务器配置')
    }
    return Promise.reject(error)
  }
)

export default apiRequest
