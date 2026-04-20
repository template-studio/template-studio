import request from '@/utils/request';

// 获取首页数据
export function getIndexData(params = {}) {
  return request({
    url: '/api/v1/studio/index',
    method: 'get',
    params: {
      categoryLimit: params.categoryLimit || 6,
      featuredLimit: params.featuredLimit || 8,
    },
  });
}
