import request from '@/utils/request';

// 变量预设-新增
export function addVarPreset(data) {
  return request({
    url: '/api/v1/admin/var-preset/add',
    method: 'post',
    data,
  });
}

// 变量预设-批量删除
export function batchDeleteVarPresets(data) {
  return request({
    url: '/api/v1/admin/var-preset/batch-del',
    method: 'delete',
    data,
  });
}

// 变量预设-删除
export function deleteVarPreset(params) {
  return request({
    url: '/api/v1/admin/var-preset/del',
    method: 'delete',
    params,
  });
}

// 变量预设-详情
export function getVarPresetDetail(params) {
  return request({
    url: '/api/v1/admin/var-preset/detail',
    method: 'get',
    params,
  });
}

// 变量预设-修改
export function editVarPreset(data) {
  return request({
    url: '/api/v1/admin/var-preset/edit',
    method: 'put',
    data,
  });
}

// 变量预设-列表（分页）
export function listVarPresets(params) {
  return request({
    url: '/api/v1/admin/var-preset/list',
    method: 'get',
    params,
  });
}

// 变量预设-全部（不分页）
export function getAllVarPresets(params) {
  return request({
    url: '/api/v1/admin/var-preset/all',
    method: 'get',
    params,
  });
}

// 变量预设-启用/禁用
export function toggleVarPreset(data) {
  return request({
    url: '/api/v1/admin/var-preset/toggle',
    method: 'put',
    data,
  });
}

// 模板变量预设-获取关联的预设列表
export function getTemplateVarPresets(templateId) {
  return request({
    url: `/api/v1/templates/${templateId}/var-presets`,
    method: 'get',
  });
}

// 模板变量预设-设置关联（覆盖）
export function setTemplateVarPresets(templateId, data) {
  return request({
    url: `/api/v1/templates/${templateId}/var-presets`,
    method: 'put',
    data,
  });
}

// 模板变量预设-添加关联
export function addTemplateVarPresets(templateId, data) {
  return request({
    url: `/api/v1/templates/${templateId}/var-presets/add`,
    method: 'post',
    data,
  });
}

// 模板变量预设-移除关联
export function removeTemplateVarPresets(templateId, data) {
  return request({
    url: `/api/v1/templates/${templateId}/var-presets/remove`,
    method: 'delete',
    data,
  });
}
