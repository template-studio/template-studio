import request from '@/utils/request';

// 语言-新增
export function addLanguage(data) {
  return request({
    url: '/api/v1/admin/languages/add',
    method: 'post',
    data,
  });
}

// 语言-批量删除
export function batchDeleteLanguage(params) {
  return request({
    url: '/api/v1/admin/languages/batchdel',
    method: 'delete',
    params,
  });
}

// 语言-删除
export function deleteLanguage(params) {
  return request({
    url: '/api/v1/admin/languages/del',
    method: 'delete',
    params,
  });
}

// 语言-详情
export function getLanguageDetail(params) {
  return request({
    url: '/api/v1/admin/languages/detail',
    method: 'get',
    params,
  });
}

// 语言-修改
export function editLanguage(data) {
  return request({
    url: '/api/v1/admin/languages/edit',
    method: 'put',
    data,
  });
}

// 语言-列表
export function listLanguages(params) {
  return request({
    url: '/api/v1/admin/languages/list',
    method: 'get',
    params,
  });
}
