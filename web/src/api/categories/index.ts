import request from '@/utils/request';

// 分类-新增
export function addCategory(data) {
  return request({
    url: '/api/v1/admin/categories/add',
    method: 'post',
    data,
  });
}

// 分类-批量删除
export function batchDeleteCategory(params) {
  return request({
    url: '/api/v1/admin/categories/batchdel',
    method: 'delete',
    params,
  });
}

// 分类-删除
export function deleteCategory(params) {
  return request({
    url: '/api/v1/admin/categories/del',
    method: 'delete',
    params,
  });
}

// 分类-详情
export function getCategoryDetail(params) {
  return request({
    url: '/api/v1/admin/categories/detail',
    method: 'get',
    params,
  });
}

// 分类-修改
export function editCategory(data) {
  return request({
    url: '/api/v1/admin/categories/edit',
    method: 'put',
    data,
  });
}

// 分类-列表
export function listCategories(params) {
  return request({
    url: '/api/v1/admin/categories/list',
    method: 'get',
    params,
  });
}
