import request from '@/utils/apiRequest';

// ===== 用户模板 API（我的模板）=====

// 我的模板列表
export function listMyTemplates(params) {
  return request({
    url: '/api/v1/admin/my/templates/list',
    method: 'get',
    params,
  });
}

// 创建模板（返回 data.id）
export function createUserTemplate(data) {
  return request({
    url: '/api/v1/admin/my/templates/add',
    method: 'post',
    data,
  });
}

// 更新模板元数据
export function updateUserTemplate(id, data) {
  return request({
    url: `/api/v1/admin/my/templates/${id}`,
    method: 'put',
    data,
  });
}

// 删除模板
export function deleteUserTemplate(id) {
  return request({
    url: `/api/v1/admin/my/templates/${id}`,
    method: 'delete',
  });
}

// 提交审核
export function submitForReview(id) {
  return request({
    url: `/api/v1/admin/my/templates/${id}/submit-review`,
    method: 'post',
  });
}
