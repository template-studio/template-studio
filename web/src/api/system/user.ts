import { Alova } from '@/utils/http/alova/index';

/**
 * @description: 用户登录
 */
export function login(params) {
  return Alova.Post('/auth/login', params, {
    meta: {
      isReturnNativeResponse: true,
    },
  });
}

/**
 * @description: 获取用户信息
 */
export function getUserInfo() {
  return Alova.Get('/auth/info', {
    meta: {
      isReturnNativeResponse: true,
    },
  });
}

/**
 * @description: 修改密码
 */
export function changePassword(data) {
  return Alova.Put('/auth/password', data);
}
