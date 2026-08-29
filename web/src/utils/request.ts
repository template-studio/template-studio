import axios from 'axios';
import qs from 'qs';
import { message } from 'ant-design-vue';
import { storage } from '@/utils/Storage';
import { ACCESS_TOKEN } from '@/store/mutation-types';

function getApiBaseUrl() {
  // 1. 优先使用显式配置的环境变量
  if (import.meta.env.VITE_API_URL) {
    return import.meta.env.VITE_API_URL;
  }

  // 开发环境使用后端端口
  if (import.meta.env.DEV) {
    return 'http://localhost:8001';
  }

  // 生产环境使用当前域名
  const { protocol, hostname, port } = window.location;
  let basePort = port ? `:${port}` : '';

  // 处理非标准端口
  if (
    (protocol === 'http:' && port === '80') ||
    (protocol === 'https:' && port === '443') ||
    port === '5173'
  ) {
    // 前端开发端口
    basePort = '';
  }

  return `${protocol}//${hostname}${basePort}`;
}

// 配置新建一个 axios 实例
const service = axios.create({
  baseURL: getApiBaseUrl(), // 修改为根路径，避免重复的/api
  timeout: 50000,
  headers: { 'Content-Type': 'application/json' },
  paramsSerializer: {
    serialize(params) {
      return qs.stringify(params, { allowDots: true, arrayFormat: 'brackets' });
    },
  },
});

// 添加请求拦截器 - 注入 token
service.interceptors.request.use((config) => {
  const token = storage.get(ACCESS_TOKEN);
  if (token) {
    config.headers['token'] = token;
  }
  return config;
});

// 添加响应拦截器 - 处理通用错误
service.interceptors.response.use(
  (response) => {
    // 如果是下载文件，直接返回response
    if (response.config.responseType === 'blob') {
      return response;
    }

    // 检查响应数据类型
    const contentType = response.headers['content-type'];
    if (contentType && contentType.includes('text/html')) {
      console.error('API返回了HTML页面，可能是404或服务器错误');
      message.error('API请求失败，请检查后端服务是否正常运行');
      return Promise.reject(new Error('Invalid response: received HTML instead of JSON'));
    }

    const res = response.data;
    // 信封过渡期兼容（详见 dev-docs/api-envelope-analysis.md）：
    // code:0（阵营 A 主流）与 code:200（阵营 B，待后端收敛）均视为成功
    if (res.code !== 0 && res.code !== 200) {
      console.error('API错误:', res.message);
      message.error(res.message || '请求失败');
      return Promise.reject(new Error(res.message || '未知错误'));
    }

    return response;
  },
  (error) => {
    // 请求已发出，但服务器响应状态码不在 2xx 范围内
    if (error.response) {
      const { status, data } = error.response;

      if (status >= 500) {
        message.error('服务器错误，请稍后重试');
      } else if (status === 404) {
        message.error('API接口不存在，请检查后端服务');
      } else if (status === 401) {
        storage.clear();
        window.location.href = '/login';
        return Promise.reject(error);
      } else if (status >= 400) {
        message.error(data?.message || '请求错误');
      }

      console.error('请求错误:', error.response);
    } else if (error.request) {
      // 请求已发出但没有收到响应
      console.error('网络错误，请检查后端服务是否启动:', error.message);
      message.error('网络错误，请检查后端服务是否启动');
    } else {
      // 发生了触发请求错误的问题
      console.error('请求配置错误:', error.message);
    }

    return Promise.reject(error);
  }
);

// 导出 axios 实例
export default service;
