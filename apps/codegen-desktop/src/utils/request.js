import axios from 'axios'
import { useConfigStore } from '@/stores/config'

// 创建 axios 实例
const request = axios.create({
  baseURL: 'http://127.0.0.1:8080', // 默认地址，会在拦截器中动态更新
  timeout: 10000,
})

// 请求拦截器
request.interceptors.request.use(
  async (config) => {
    // 动态获取最新的 API URL
    const configStore = useConfigStore()

    // 如果配置已加载，使用配置的 URL
    if (configStore.configLoaded) {
      config.baseURL = configStore.baseURL
    }

    // 如果有 API Key，添加到请求头
    if (configStore.apiKey) {
      config.headers['Authorization'] = `Bearer ${configStore.apiKey}`
      config.headers['X-API-Key'] = configStore.apiKey
    }

    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器
request.interceptors.response.use(
  (response) => {
    return response.data
  },
  (error) => {
    console.error('API 请求失败:', error)

    // 如果是网络错误，可能是 API 地址配置错误
    if (error.code === 'ERR_NETWORK' || error.code === 'ECONNREFUSED') {
      console.error('无法连接到服务器，请检查 Web 服务器配置')
    }

    return Promise.reject(error)
  }
)

export default request
