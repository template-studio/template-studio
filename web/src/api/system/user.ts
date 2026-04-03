import { Alova } from '@/utils/http/alova/index';
import request from '@/utils/request';

/**
 * @description: 用户登录
 */
export function login(params) {
  return Alova.Post('/v1/auth/login', params, {
    meta: {
      isReturnNativeResponse: true,
    },
  });
}

/**
 * @description: 用户注册
 */
export function register(params) {
  return Alova.Post('/v1/auth/register', params, {
    meta: {
      isReturnNativeResponse: true,
    },
  });
}

/**
 * @description: 获取用户信息（需认证）
 */
export function getUserInfo() {
  return Alova.Get('/v1/admin/auth/info', {
    meta: {
      isReturnNativeResponse: true,
    },
  });
}

/**
 * @description: 修改密码
 */
export function changePassword(data) {
  return Alova.Put('/v1/admin/auth/password', data);
}

/**
 * @description: 创建 PAT 令牌
 */
export function createPat(data: { name: string; expires_in_days?: number | null }) {
  return Alova.Post('/v1/admin/auth/tokens', data);
}

/**
 * @description: 获取 PAT 令牌列表
 */
export function listPats() {
  return Alova.Get('/v1/admin/auth/tokens', {
    cacheFor: 0,
  });
}

/**
 * @description: 删除 PAT 令牌
 */
export function deletePat(id: number) {
  return Alova.Delete(`/v1/admin/auth/tokens/${id}`);
}

/**
 * @description: 更新个人资料（bio）
 */
export function updateProfile(data: { bio?: string; avatar?: string }) {
  return request.put('/api/v1/admin/auth/profile', data);
}

/**
 * @description: 上传头像
 */
export function uploadAvatar(file: File) {
  const formData = new FormData();
  formData.append('file', file);
  return request.post('/api/v1/admin/auth/avatar', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
}

/**
 * @description: 获取公开用户主页
 */
export function getPublicProfile(username: string) {
  return request.get(`/api/v1/auth/users/${username}`);
}
