import request from '@/utils/request';

// 获取设置列表
export function getSettings(params?) {
  return request({
    url: '/api/v1/admin/settings/list',
    method: 'get',
    params,
  });
}

// 更新单个设置
export function updateSetting(data) {
  return request({
    url: '/api/v1/admin/settings/edit',
    method: 'put',
    data,
  });
}

// 批量更新设置
export function batchUpdateSettings(data) {
  return request({
    url: '/api/v1/admin/settings/batch-edit',
    method: 'post',
    data,
  });
}

// 公开获取设置（Footer 等）
export function getPublicSettings(group: string) {
  return request({
    url: `/api/v1/settings/${group}`,
    method: 'get',
  });
}
