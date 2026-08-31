import request from '@/utils/apiRequest';

// 获取内置函数列表
export function getBuiltinFunctions() {
  return request({
    url: '/api/v1/builtin-functions',
    method: 'get',
  });
}
