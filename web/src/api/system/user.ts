import { Alova } from '@/utils/http/alova/index';

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
