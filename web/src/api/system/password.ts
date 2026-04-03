import request from '@/utils/request';

// 忘记密码 - 发送重置邮件
export function forgotPassword(email: string) {
  return request.post('/api/v1/auth/forgot-password', { email });
}

// 重置密码
export function resetPassword(token: string, password: string) {
  return request.post('/api/v1/auth/reset-password', { token, password });
}
