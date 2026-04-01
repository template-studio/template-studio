import request from '@/utils/request';

// 模板-新增
export function addTemplate(data) {
  return request({
    url: '/api/v1/template/templates/add',
    method: 'post',
    data,
  });
}

// 模板-批量删除
export function batchDeleteTemplate(params) {
  return request({
    url: '/api/v1/template/templates/batchdel',
    method: 'delete',
    params,
  });
}

// 模板-删除
export function deleteTemplate(params) {
  return request({
    url: '/api/v1/template/templates/del',
    method: 'delete',
    params,
  });
}

// 模板-详情
export function getTemplateDetail(params) {
  return request({
    url: '/api/v1/template/templates/detail',
    method: 'get',
    params,
  });
}

// 模板-修改
export function editTemplate(data) {
  return request({
    url: '/api/v1/template/templates/edit',
    method: 'put',
    data,
  });
}

// 模板-列表
export function listTemplates(params) {
  return request({
    url: '/api/v1/template/templateList',
    method: 'get',
    params,
  });
}

// 模板-变量列表
export function getTemplateVariables(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/variables`,
    method: 'get',
  });
}

// 模板-分析变量
export function analyzeTemplateVariables(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/analyze-variables`,
    method: 'post',
  });
}

// 获取模板类型列表
export function getTemplateTypes() {
  return request({
    url: '/api/v1/template/templates/types',
    method: 'get',
  });
}

// Fork模板
export function forkTemplate(data) {
  return request({
    url: '/api/v1/template/templates/fork',
    method: 'post',
    data,
  });
}

// 切换推荐状态
export function toggleTemplateFeatured(data) {
  return request({
    url: '/api/v1/template/templates/toggle-featured',
    method: 'put',
    data,
  });
}

// 导出模板 - 直接下载文件
export function exportTemplate(templateId, format = 'files', fileName = null) {
  // 创建一个临时链接来触发文件下载
  const exportUrl = `/api/v1/template/templates/${templateId}/export?format=${format}`;
  const link = document.createElement('a');
  link.href = exportUrl;
  link.download = fileName || `template_${templateId}.${format === 'json' ? 'json' : 'zip'}`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
