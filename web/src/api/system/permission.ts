import request from '@/utils/request';

// 权限列表
export function getPermissionList() {
  return request({
    url: '/api/v1/admin/permissions/list',
    method: 'get',
  });
}

// 权限树
export function getPermissionTree() {
  return request({
    url: '/api/v1/admin/permissions/tree',
    method: 'get',
  });
}
