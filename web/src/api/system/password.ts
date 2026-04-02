import { Alova } from '@/utils/http/alova/index';

// 忘记密码 - 发送重置邮件
export function forgotPassword(email: string) {
  return Alova.Post('/v1/auth/forgot-password', { email }, { meta: { isTransformResponse: false } });
}

// 重置密码
export function resetPassword(token: string, password: string) {
  return Alova.Post('/v1/auth/reset-password', { token, password }, { meta: { isTransformResponse: false } });
}
