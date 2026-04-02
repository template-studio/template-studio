import request from '@/utils/request';

// 用户列表
export function getUserList() {
  return request({
    url: '/api/v1/admin/users/list',
    method: 'get',
  });
}

// 创建用户
export function createUser(data) {
  return request({
    url: '/api/v1/admin/users/add',
    method: 'post',
    data,
  });
}

// 更新用户
export function updateUser(data) {
  return request({
    url: '/api/v1/admin/users/edit',
    method: 'put',
    data,
  });
}

// 删除用户
export function deleteUser(id) {
  return request({
    url: `/api/v1/admin/users/del/${id}`,
    method: 'delete',
  });
}

// 分配角色
export function assignRoles(userId, roleIds) {
  return request({
    url: `/api/v1/admin/users/${userId}/roles`,
    method: 'put',
    data: { role_ids: roleIds },
  });
}
