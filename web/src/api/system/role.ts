import request from '@/utils/request';

// 角色列表
export function getRoleList() {
  return request({
    url: '/api/v1/admin/roles/list',
    method: 'get',
  });
}

// 创建角色
export function createRole(data) {
  return request({
    url: '/api/v1/admin/roles/add',
    method: 'post',
    data,
  });
}

// 更新角色
export function updateRole(data) {
  return request({
    url: '/api/v1/admin/roles/edit',
    method: 'put',
    data,
  });
}

// 删除角色
export function deleteRole(id) {
  return request({
    url: `/api/v1/admin/roles/del/${id}`,
    method: 'delete',
  });
}

// 获取角色权限
export function getRolePermissions(roleId) {
  return request({
    url: `/api/v1/admin/roles/${roleId}/permissions`,
    method: 'get',
  });
}

// 分配权限
export function assignPermissions(roleId, permissionIds) {
  return request({
    url: `/api/v1/admin/roles/${roleId}/permissions`,
    method: 'put',
    data: { permission_ids: permissionIds },
  });
}
